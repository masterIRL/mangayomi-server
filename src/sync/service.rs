use crate::config::logging;
use crate::db::bulk_upsert::get_global_upserter;
use crate::db::utils::{delete_all_by_user, delete_many_by_ids};
use crate::sync::error::SyncError;
use crate::sync::model::Syncable;
use mongodb::bson::oid::ObjectId;
use mongodb::Collection;
use serde::{de::DeserializeOwned, Serialize};

/// Trait for sync list types that can be processed generically
///
/// This trait abstracts over HistoryList, UpdateList, and similar types
/// to eliminate code duplication in sync services.
///
/// # IMPORTANT ARCHITECTURAL NOTE FOR FUTURE MAINTAINERS
///
/// This trait is designed for **HOMOGENEOUS entity lists only** - meaning lists
/// that contain a SINGLE type of entity stored in a SINGLE MongoDB collection.
///
/// ## Why MangaList does NOT implement this trait:
///
/// `MangaList` contains **4 different entity types** (Category, Manga, Chapter, Track)
/// stored in **4 separate MongoDB collections**. The `SyncList` trait defines a single
/// associated type `Item`, which makes it inherently unsuitable for heterogeneous
/// aggregates like `MangaList`.
///
/// ### Technical constraint:
/// The SyncList trait defines: `type Item: Syncable + Serialize + ...;`
/// This SINGLE associated type cannot represent multiple entity types.
///
/// ### Current implementations:
/// - ✅ `HistoryList` - 1 entity type (History) → implements `SyncList`
/// - ✅ `UpdateList` - 1 entity type (Update) → implements `SyncList`
/// - ❌ `MangaList` - 4 entity types → CANNOT implement `SyncList` (by design)
/// - ❌ `SettingsObj` - Singleton pattern → CANNOT implement `SyncList`
///
/// ### Do NOT attempt to "unify" these patterns:
/// Making `MangaList` implement `SyncList` would require either:
/// 1. Making `Item` a union/enum type (extremely complex, breaks type safety)
/// 2. Using dynamic dispatch (runtime overhead, loses zero-cost abstraction)
/// 3. Adding 4 associated types to the trait (over-engineering for one use case)
///
/// The current architecture is INTENTIONAL and CORRECT. `MangaList` uses its own
/// specialized service function (`sync_manga_list`) with parallel fetching via
/// `tokio::join!` for performance, which would be impossible with the generic
/// `sync_entity_list` anyway.
///
/// If you need to add a new sync endpoint:
/// - Single entity type? → Implement `SyncList`, use `sync_entity_list()`
/// - Multiple entity types? → Create specialized service, do NOT try to fit into `SyncList`
pub trait SyncList: Send + Sync + Clone {
    /// The entity type contained in the list
    ///
    /// ## Constraint
    /// This must be a single, homogeneous entity type. This trait cannot represent
    /// heterogeneous collections like `MangaList` which contains multiple entity types.
    type Item: Syncable + Serialize + DeserializeOwned + Unpin + Clone + Send + Sync;

    /// Returns true if reset_all flag is set
    fn should_reset_all(&self) -> bool;

    /// Returns the items to upsert
    fn items(&self) -> &[Self::Item];

    /// Returns the IDs to delete
    fn deleted_ids(&self) -> &[i64];

    /// Creates a result list from processed items
    fn into_result(self, items: Vec<Self::Item>) -> Self;
}

/// Generic sync function for simple entity lists (history, update, etc.)
///
/// # Arguments
/// * `user_id` - User ID
/// * `list` - The sync list containing items and metadata
/// * `db` - MongoDB client
/// * `collection_name` - Name of the collection
///
/// # Returns
/// * `Ok(List)` - Synced list with server state
/// * `Err(SyncError)` - Sync operation failed
///
/// # Idempotency
/// This operation is idempotent - calling it multiple times with the same
/// data will produce the same result.
///
/// # IMPORTANT
/// This function works with **single entity type lists only**. See [`SyncList`]
/// documentation for why `MangaList` cannot use this function.
pub async fn sync_entity_list<List>(
    user_id: ObjectId,
    list: &List,
    db: &mongodb::Client,
    collection_name: &'static str,
) -> Result<List, SyncError>
where
    List: SyncList,
{
    let database = crate::db::utils::get_database(db);
    let col: Collection<List::Item> = database.collection(collection_name);
    let reset_all = list.should_reset_all();

    if reset_all {
        reset_all_entities(&col, user_id, collection_name).await?;
    }

    upsert_batch(&col, user_id, list.items(), collection_name).await?;

    if !reset_all {
        delete_by_ids(&col, user_id, list.deleted_ids(), collection_name).await?;
    }

    let items = crate::db::utils::find_all_by_user(&col, user_id)
        .await
        .map_err(|e| SyncError::DatabaseError(e.to_string()))?;

    Ok(list.clone().into_result(items))
}

/// Upsert a batch of entities
///
/// # Arguments
/// * `collection` - MongoDB collection
/// * `user_id` - User ID for filtering
/// * `items` - Items to upsert
/// * `entity_name` - Name for logging (e.g., "categories")
///
/// # Returns
/// * `Ok(())` - All items processed
/// * `Err(SyncError)` - Upsert failed
///
/// # Idempotency
/// This operation is idempotent - calling it multiple times with the same
/// data will produce the same result (upserts are idempotent by nature).
pub async fn upsert_batch<T>(
    collection: &Collection<T>,
    user_id: ObjectId,
    items: &[T],
    entity_name: &str,
) -> Result<(), SyncError>
where
    T: Syncable + Serialize + Send + Sync,
{
    if items.is_empty() {
        return Ok(());
    }

    let upserter = get_global_upserter().await.ok_or(SyncError::NotInitialized)?;

    match upserter
        .upsert_batch(
            collection,
            items,
            user_id,
            |item: &T| (item.get_id(), item.get_updated_at()),
        )
        .await
    {
        Ok(result) => {
            if !result.failed_items.is_empty() {
                log::warn!(
                    target: logging::SYNC_API,
                    "[SYNC] {} upsert had {} failures",
                    entity_name,
                    result.failed_items.len()
                );
            }
            Ok(())
        }
        Err(e) => {
            log::error!(
                target: logging::SYNC_API,
                "[SYNC] Failed to upsert {}: {}",
                entity_name,
                e
            );
            Err(SyncError::UpsertFailed(e.to_string()))
        }
    }
}

/// Delete entities by IDs with proper error handling
///
/// # Arguments
/// * `collection` - MongoDB collection
/// * `user_id` - User ID for filtering
/// * `ids` - IDs to delete
/// * `entity_name` - Name for logging
///
/// # Returns
/// * `Ok(usize)` - Number of documents deleted
/// * `Err(SyncError)` - Delete operation failed
///
/// # Idempotency
/// This operation is idempotent - deleting the same IDs multiple times
/// will have the same end state (documents are gone).
pub async fn delete_by_ids<T>(
    collection: &Collection<T>,
    user_id: ObjectId,
    ids: &[i64],
    entity_name: &str,
) -> Result<usize, SyncError>
where
    T: Send + Sync,
{
    if ids.is_empty() {
        return Ok(0);
    }

    match delete_many_by_ids(collection, user_id, ids).await {
        Ok(deleted_count) => {
            log::debug!(
                target: logging::SYNC_API,
                "[SYNC] Deleted {} {}(s)",
                deleted_count,
                entity_name
            );
            Ok(deleted_count)
        }
        Err(e) => {
            log::error!(
                target: logging::SYNC_API,
                "[SYNC] Failed to delete {} {}(s): {}",
                ids.len(),
                entity_name,
                e
            );
            Err(SyncError::DatabaseError(e.to_string()))
        }
    }
}

/// Reset all user data (delete all entities for user)
///
/// # Arguments
/// * `collection` - MongoDB collection
/// * `user_id` - User ID
/// * `entity_name` - Name for logging
///
/// # Returns
/// * `Ok(usize)` - Number of documents deleted
/// * `Err(SyncError)` - Reset failed
///
/// # Warning
/// This operation is destructive and cannot be undone. Use with caution.
///
/// # Idempotency
/// This operation is idempotent - resetting when there's nothing to delete
/// produces the same end state.
pub async fn reset_all_entities<T>(
    collection: &Collection<T>,
    user_id: ObjectId,
    entity_name: &str,
) -> Result<usize, SyncError>
where
    T: Send + Sync,
{
    match delete_all_by_user(collection, user_id).await {
        Ok(deleted_count) => {
            log::info!(
                target: logging::SYNC_API,
                "[SYNC] Reset all {} for user ({} documents deleted)",
                entity_name,
                deleted_count
            );
            Ok(deleted_count)
        }
        Err(e) => {
            log::error!(
                target: logging::SYNC_API,
                "[SYNC] Failed to reset {}: {}",
                entity_name,
                e
            );
            Err(SyncError::DatabaseError(e.to_string()))
        }
    }
}
