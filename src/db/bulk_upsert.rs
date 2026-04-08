use mongodb::bson::{doc, to_document};
use mongodb::options::UpdateOptions;
use mongodb::Collection;
use serde::Serialize;
use std::sync::OnceLock;
use mongodb::bson::oid::ObjectId;

use crate::config::bulk_upsert;
use crate::config::logging;
use crate::config::env_or_default;

fn get_chunk_size() -> usize {
    env_or_default(
        bulk_upsert::ENV_CHUNK_SIZE,
        bulk_upsert::DEFAULT_CHUNK_SIZE,
        bulk_upsert::MIN_CHUNK_SIZE,
        bulk_upsert::MAX_CHUNK_SIZE,
    )
}

fn get_concurrency() -> usize {
    env_or_default(
        bulk_upsert::ENV_CONCURRENCY,
        bulk_upsert::DEFAULT_CONCURRENCY,
        bulk_upsert::MIN_CONCURRENCY,
        bulk_upsert::MAX_CONCURRENCY,
    )
}

/// Result of a batch upsert operation
#[derive(Debug, Clone)]
pub struct UpsertBatchResult {
    pub modified_count: usize,
    pub matched_count: usize,
    pub failed_items: Vec<FailedItem>,
}

#[derive(Debug, Clone)]
pub struct FailedItem {
    pub id: i64,
    pub error: String,
}

/// Bulk upserter that automatically selects the best strategy based on MongoDB version
#[derive(Clone, Copy, Debug)]
pub enum BulkUpserter {
    Legacy,
    V8,
}

impl BulkUpserter {
    pub async fn detect(client: &mongodb::Client) -> Self {
        match Self::fetch_server_version(client).await {
            Ok(version) if version.major >= 8 => {
                log::info!(
                    target: logging::SYNC_API,
                    "MongoDB {} detected - using parallel update mode",
                    version
                );
                BulkUpserter::V8
            }
            Ok(version) => {
                log::info!(
                    target: logging::SYNC_API,
                    "MongoDB {} detected - using sequential update mode",
                    version
                );
                BulkUpserter::Legacy
            }
            Err(e) => {
                log::warn!(
                    target: logging::SYNC_API,
                    "Failed to detect MongoDB version ({}), using legacy mode",
                    e
                );
                BulkUpserter::Legacy
            }
        }
    }
    
    async fn fetch_server_version(client: &mongodb::Client) -> mongodb::error::Result<semver::Version> {
        let db = client.database("admin");
        
        let doc = db.run_command(doc! { "buildInfo": 1 }).await?;
        let version_str = doc.get_str("version")
            .map_err(|e| mongodb::error::Error::custom(format!("Invalid version: {}", e)))?;
        
        semver::Version::parse(version_str)
            .map_err(|e| mongodb::error::Error::custom(format!("Parse error: {}", e)))
    }

    pub async fn upsert_batch<T>(
        self,
        collection: &Collection<T>,
        items: &[T],
        user_id: ObjectId,
        key_fn: impl Fn(&T) -> (i64, i64) + Send + Sync,
    ) -> mongodb::error::Result<UpsertBatchResult>
    where
        T: Serialize + Send + Sync + Clone,
    {
        if items.is_empty() {
            return Ok(UpsertBatchResult {
                modified_count: 0,
                matched_count: 0,
                failed_items: vec![],
            });
        }
        
        match self {
            BulkUpserter::Legacy => {
                self.legacy_upsert(collection, items, user_id, key_fn).await
            }
            BulkUpserter::V8 => {
                self.v8_parallel_upsert(collection, items, user_id, key_fn).await
            }
        }
    }
    
    fn build_update_doc<T: Serialize>(
        item: &T,
        user_id: ObjectId,
        id: i64,
        updated_at: i64,
    ) -> Result<(mongodb::bson::Document, mongodb::bson::Document), FailedItem> {
        let mut doc = to_document(item)
            .map_err(|e| FailedItem { 
                id, 
                error: format!("Serialization: {}", e) 
            })?;
        doc.insert("user", user_id);
        
        let filter = doc! {
            "id": id,
            "user": user_id,
            "updatedAt": { "$lt": updated_at }
        };
        
        Ok((doc, filter))
    }
    
    async fn legacy_upsert<T>(
        self,
        collection: &Collection<T>,
        items: &[T],
        user_id: ObjectId,
        key_fn: impl Fn(&T) -> (i64, i64) + Send + Sync,
    ) -> mongodb::error::Result<UpsertBatchResult>
    where
        T: Serialize + Send + Sync + Clone,
    {
        let mut modified_count = 0usize;
        let mut matched_count = 0usize;
        let mut failed_items = Vec::new();
        
        let chunk_size = get_chunk_size();
        let options = UpdateOptions::builder().upsert(true).build();
        
        for chunk in items.chunks(chunk_size) {
            for item in chunk {
                let (id, updated_at) = key_fn(item);
                
                let (doc, filter) = match Self::build_update_doc(item, user_id, id, updated_at) {
                    Ok(d) => d,
                    Err(e) => {
                        failed_items.push(e);
                        continue;
                    }
                };
                
                match collection.update_one(filter, doc! { "$set": doc }).with_options(options.clone()).await {
                    Ok(result) => {
                        modified_count += result.modified_count as usize;
                        matched_count += result.matched_count as usize;
                    }
                    Err(e) => {
                        failed_items.push(FailedItem { 
                            id, 
                            error: e.to_string() 
                        });
                    }
                }
            }
        }
        
        if !failed_items.is_empty() {
            log::warn!(
                target: logging::SYNC_API,
                "Legacy upsert: {} failures out of {}", 
                failed_items.len(),
                items.len()
            );
        }
        
        Ok(UpsertBatchResult {
            modified_count,
            matched_count,
            failed_items,
        })
    }
    
    async fn v8_parallel_upsert<T>(
        self,
        collection: &Collection<T>,
        items: &[T],
        user_id: ObjectId,
        key_fn: impl Fn(&T) -> (i64, i64) + Send + Sync,
    ) -> mongodb::error::Result<UpsertBatchResult>
    where
        T: Serialize + Send + Sync + Clone,
    {
        use futures::stream::{self, StreamExt};
        
        let concurrency = get_concurrency();
        let collection = collection.clone();
        let options = UpdateOptions::builder().upsert(true).build();
        
        let results: Vec<_> = stream::iter(items.iter().cloned())
            .map(|item| {
                let coll = collection.clone();
                let opts = options.clone();
                let (id, updated_at) = key_fn(&item);
                
                async move {
                    let (doc, filter) = Self::build_update_doc(&item, user_id, id, updated_at)?;
                    
                    match coll.update_one(filter, doc! { "$set": doc }).with_options(opts).await {
                        Ok(result) => Ok((result.modified_count, result.matched_count)),
                        Err(e) => Err(FailedItem { id, error: e.to_string() }),
                    }
                }
            })
            .buffer_unordered(concurrency)
            .collect()
            .await;
        
        let mut modified_count = 0usize;
        let mut matched_count = 0usize;
        let mut failed_items = Vec::new();
        
        for result in results {
            match result {
                Ok((mod_count, mat_count)) => {
                    modified_count += mod_count as usize;
                    matched_count += mat_count as usize;
                }
                Err(failed) => {
                    failed_items.push(failed);
                }
            }
        }
        
        if !failed_items.is_empty() {
            log::warn!(
                target: logging::SYNC_API,
                "Parallel upsert: {} failures out of {}", 
                failed_items.len(),
                items.len()
            );
        } else {
            log::info!(
                target: logging::SYNC_API,
                "Parallel upsert: {} items processed",
                items.len()
            );
        }
        
        Ok(UpsertBatchResult {
            modified_count,
            matched_count,
            failed_items,
        })
    }
}

static GLOBAL_UPSERTER: OnceLock<BulkUpserter> = OnceLock::new();

pub async fn initialize_global_upserter(client: &mongodb::Client) {
    let upserter = BulkUpserter::detect(client).await;
    
    log::info!(
        target: logging::SYNC_API,
        "Global BulkUpserter initialized: {:?}",
        upserter
    );
    
    let _ = GLOBAL_UPSERTER.set(upserter);
}

/// Get the global upserter instance
///
/// # Returns
/// * `Some(BulkUpserter)` - The global upserter if initialized
/// * `None` - If not yet initialized
pub async fn get_global_upserter() -> Option<BulkUpserter> {
    GLOBAL_UPSERTER.get().copied()
}
