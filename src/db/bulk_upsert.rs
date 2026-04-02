use mongodb::bson::{doc, to_document};
use mongodb::Collection;
use serde::Serialize;
use std::sync::OnceLock;
use tokio::sync::RwLock;
use mongodb::bson::oid::ObjectId;

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

/// Bulk upserter that automatically selects the best strategy
/// based on MongoDB server version
/// 
/// Note: MongoDB Rust driver 3.x doesn't support bulk_write yet,
/// so both Legacy and V8 modes use individual updates.
/// V8 mode uses parallel execution for better performance.
#[derive(Clone, Copy, Debug)]
pub enum BulkUpserter {
    /// Legacy mode: sequential individual update_one calls (MongoDB < 8.0 or driver limitation)
    Legacy,
    /// Modern mode: parallel individual update_one calls (MongoDB >= 8.0)
    /// Note: True bulk_write not available in driver 3.x, so we simulate with parallel updates
    V8,
}

impl BulkUpserter {
    /// Detect MongoDB version and create appropriate upserter
    pub async fn detect(client: &mongodb::Client) -> Self {
        match Self::fetch_server_version(client).await {
            Ok(version) if version.major >= 8 => {
                log::info!("MongoDB {} detected - using parallel update mode", version);
                BulkUpserter::V8
            }
            Ok(version) => {
                log::info!("MongoDB {} detected - using sequential update mode", version);
                BulkUpserter::Legacy
            }
            Err(e) => {
                log::warn!("Failed to detect MongoDB version ({}), using legacy mode", e);
                BulkUpserter::Legacy
            }
        }
    }
    
    async fn fetch_server_version(client: &mongodb::Client) -> mongodb::error::Result<semver::Version> {
        let db = client.database("admin");
        
        match db.run_command(doc! { "buildInfo": 1 }).await {
            Ok(doc) => {
                let version_str = doc.get_str("version")
                    .map_err(|e| mongodb::error::Error::custom(format!("Invalid version: {}", e)))?;
                
                semver::Version::parse(version_str)
                    .map_err(|e| mongodb::error::Error::custom(format!("Parse error: {}", e)))
            }
            Err(e) => Err(e)
        }
    }

    /// Upsert a batch of items
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
                // Sequential processing with chunking
                self.legacy_upsert(collection, items, user_id, key_fn).await
            }
            BulkUpserter::V8 => {
                // Parallel processing for better throughput
                self.v8_parallel_upsert(collection, items, user_id, key_fn).await
            }
        }
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
        
        // Process in chunks to avoid overwhelming the connection pool
        const CHUNK_SIZE: usize = 50;
        
        for chunk in items.chunks(CHUNK_SIZE) {
            for item in chunk {
                let (id, updated_at) = key_fn(item);
                
                let doc = match to_document(item) {
                    Ok(mut d) => {
                        d.insert("user", user_id);
                        d
                    }
                    Err(e) => {
                        failed_items.push(FailedItem { 
                            id, 
                            error: format!("Serialization: {}", e) 
                        });
                        continue;
                    }
                };
                
                let filter = doc! {
                    "id": id,
                    "user": user_id,
                    "updatedAt": { "$lt": updated_at }
                };
                
                let update = doc! { "$set": doc };
                
                match collection.update_one(filter, update).upsert(true).await {
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
            log::warn!("Legacy upsert: {} failures out of {}", 
                failed_items.len(), items.len());
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
        
        // Process with limited concurrency for better throughput
        const CONCURRENCY: usize = 20;
        
        // Clone collection for sharing across tasks (Collection is thread-safe + Clone)
        let collection = collection.clone();
        
        let results: Vec<_> = stream::iter(items.iter().cloned())
            .map(|item| {
                let coll = collection.clone();
                let (id, updated_at) = key_fn(&item);
                
                async move {
                    let doc = match to_document(&item) {
                        Ok(mut d) => {
                            d.insert("user", user_id);
                            d
                        }
                        Err(e) => {
                            return Err(FailedItem { 
                                id, 
                                error: format!("Serialization: {}", e) 
                            });
                        }
                    };
                    
                    let filter = doc! {
                        "id": id,
                        "user": user_id,
                        "updatedAt": { "$lt": updated_at }
                    };
                    
                    let update = doc! { "$set": doc };
                    
                    match coll.update_one(filter, update).upsert(true).await {
                        Ok(result) => Ok((result.modified_count, result.matched_count)),
                        Err(e) => Err(FailedItem { id, error: e.to_string() }),
                    }
                }
            })
            .buffer_unordered(CONCURRENCY)
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
            log::warn!("Parallel upsert: {} failures out of {}", 
                failed_items.len(), items.len());
        } else {
            log::info!("Parallel upsert: {} items processed", items.len());
        }
        
        Ok(UpsertBatchResult {
            modified_count,
            matched_count,
            failed_items,
        })
    }
}

/// Global upserter instance (initialized once at startup)
static GLOBAL_UPSERTER: OnceLock<RwLock<BulkUpserter>> = OnceLock::new();

/// Initialize the global upserter with MongoDB version detection
pub async fn initialize_global_upserter(client: &mongodb::Client) {
    let upserter = BulkUpserter::detect(client).await;
    
    let _ = GLOBAL_UPSERTER.set(RwLock::new(upserter));
    log::info!("Global BulkUpserter initialized: {:?}", upserter);
}

/// Get the global upserter instance
pub fn get_global_upserter() -> Option<BulkUpserter> {
    GLOBAL_UPSERTER.get().map(|lock| {
        // For Copy types, we can just read and copy
        match lock.try_read() {
            Ok(guard) => *guard,
            Err(_) => {
                log::warn!("Could not acquire upserter lock, using default Legacy");
                BulkUpserter::Legacy
            }
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bulk_upserter_is_copy() {
        // Ensure BulkUpserter is Copy for easy sharing
        let upserter = BulkUpserter::Legacy;
        let _copy = upserter;
        let _another = upserter;
    }
}