use serde::Serialize;

/// Common trait for all sync models
///
/// This trait ensures all models have a unique identifier and timestamp
/// for conflict resolution during sync operations.
pub trait Model {
    /// Get the unique identifier for this model
    fn get_id(&self) -> i64;

    /// Get the last update timestamp for conflict resolution
    ///
    /// The server will only overwrite existing data if the incoming
    /// updated_at is greater than the stored value.
    fn get_updated_at(&self) -> i64;
}

/// Trait for types that can be synchronized via the sync API
///
/// Combines the `Model` trait with serialization and thread-safety
/// requirements needed for database operations.
pub trait Syncable: Model + Serialize + Clone + Send + Sync + 'static {
    /// The collection name in MongoDB where this entity is stored
    const COLLECTION_NAME: &'static str;
}
