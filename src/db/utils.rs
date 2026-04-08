use mongodb::bson::oid::ObjectId;
use mongodb::options::FindOptions;
use mongodb::{Collection, Database};
use serde::de::DeserializeOwned;

use crate::config::{db as db_config, logging};

/// Error type for database utility operations
#[derive(Debug, Clone)]
pub enum DbUtilError {
    DatabaseError(String),
    Timeout,
    CollectionError(String),
}

impl std::fmt::Display for DbUtilError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DbUtilError::DatabaseError(e) => write!(f, "Database error: {}", e),
            DbUtilError::Timeout => write!(f, "Operation timed out"),
            DbUtilError::CollectionError(e) => write!(f, "Collection error: {}", e),
        }
    }
}

impl std::error::Error for DbUtilError {}

impl From<mongodb::error::Error> for DbUtilError {
    fn from(e: mongodb::error::Error) -> Self {
        DbUtilError::DatabaseError(e.to_string())
    }
}

/// Delete documents by IDs for a specific user
///
/// # Arguments
/// * `collection` - MongoDB collection
/// * `user_id` - User ID to filter by
/// * `ids` - Document IDs to delete
///
/// # Errors
/// Returns error if `ids` is empty or database operation fails
pub async fn delete_many_by_ids<T: Send + Sync>(
    collection: &Collection<T>,
    user_id: ObjectId,
    ids: &[i64],
) -> Result<usize, DbUtilError> {
    if ids.is_empty() {
        return Err(DbUtilError::CollectionError(
            "ids cannot be empty - use delete_all_by_user to delete all documents".to_string()
        ));
    }

    let filter = mongodb::bson::doc! {
        "id": { "$in": ids },
        "user": user_id,
    };

    match collection.delete_many(filter).await {
        Ok(result) => {
            log::info!(
                target: logging::SYNC_API,
                "[DB] Deleted {} documents from {} (by IDs)",
                result.deleted_count,
                collection.name()
            );
            Ok(result.deleted_count as usize)
        }
        Err(e) => {
            log::error!(
                target: logging::SYNC_API,
                "[DB] Failed to delete from {}: {}",
                collection.name(),
                e
            );
            Err(DbUtilError::from(e))
        }
    }
}

/// Delete ALL documents for a specific user
///
/// # Arguments
/// * `collection` - MongoDB collection
/// * `user_id` - User ID to filter by
///
/// # Errors
/// Returns error if database operation fails
pub async fn delete_all_by_user<T: Send + Sync>(
    collection: &Collection<T>,
    user_id: ObjectId,
) -> Result<usize, DbUtilError> {
    let filter = mongodb::bson::doc! { "user": user_id };

    match collection.delete_many(filter).await {
        Ok(result) => {
            log::info!(
                target: logging::SYNC_API,
                "[DB] Deleted {} documents from {} (all for user)",
                result.deleted_count,
                collection.name()
            );
            Ok(result.deleted_count as usize)
        }
        Err(e) => {
            log::error!(
                target: logging::SYNC_API,
                "[DB] Failed to delete all from {}: {}",
                collection.name(),
                e
            );
            Err(DbUtilError::from(e))
        }
    }
}

/// Find all documents for a user with timeout
///
/// # Arguments
/// * `collection` - MongoDB collection
/// * `user_id` - User ID to filter by
///
/// # Errors
/// Returns error if database operation fails or times out
pub async fn find_all_by_user<T: DeserializeOwned + Unpin + Send + Sync>(
    collection: &Collection<T>,
    user_id: ObjectId,
) -> Result<Vec<T>, DbUtilError> {
    let options = FindOptions::builder()
        .max_time(db_config::get_timeout())
        .build();

    match collection.find(mongodb::bson::doc! { "user": user_id }).with_options(options).await {
        Ok(cursor) => {
            use futures::TryStreamExt;
            match cursor.try_collect().await {
                Ok(items) => Ok(items),
                Err(e) => {
                    log::error!(
                        target: logging::SYNC_API,
                        "[DB] Failed to collect from {}: {}",
                        collection.name(),
                        e
                    );
                    Err(DbUtilError::from(e))
                }
            }
        }
        Err(e) => {
            log::error!(
                target: logging::SYNC_API,
                "[DB] Failed to query {}: {}",
                collection.name(),
                e
            );
            Err(DbUtilError::from(e))
        }
    }
}

/// Find all documents for a user, returning empty vector on error (legacy behavior)
#[deprecated(
    since = "0.1.3",
    note = "Use find_all_by_user which returns Result for proper error handling"
)]
pub async fn find_all_by_user_or_default<T: DeserializeOwned + Unpin + Send + Sync>(
    collection: &Collection<T>,
    user_id: ObjectId,
) -> Vec<T> {
    match find_all_by_user(collection, user_id).await {
        Ok(items) => items,
        Err(_) => vec![],
    }
}

/// Get database reference
pub fn get_database(client: &mongodb::Client) -> Database {
    client.database(db_config::DB_NAME)
}