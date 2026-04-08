use std::fmt;

/// Errors that can occur during sync operations
#[derive(Debug, Clone)]
pub enum SyncError {
    UpsertFailed(String),
    NotInitialized,
    DatabaseError(String),
    SerializationError(String),
    Timeout,
}

impl fmt::Display for SyncError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SyncError::UpsertFailed(msg) => write!(f, "Upsert failed: {}", msg),
            SyncError::NotInitialized => write!(f, "BulkUpserter not initialized"),
            SyncError::DatabaseError(msg) => write!(f, "Database error: {}", msg),
            SyncError::SerializationError(msg) => write!(f, "Serialization error: {}", msg),
            SyncError::Timeout => write!(f, "Operation timed out"),
        }
    }
}

impl std::error::Error for SyncError {}

impl From<mongodb::error::Error> for SyncError {
    fn from(e: mongodb::error::Error) -> Self {
        SyncError::DatabaseError(e.to_string())
    }
}

impl From<serde_json::Error> for SyncError {
    fn from(e: serde_json::Error) -> Self {
        SyncError::SerializationError(e.to_string())
    }
}
