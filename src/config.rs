//! Centralized configuration constants and environment variable handling

use std::str::FromStr;

/// Reads an environment variable with a default value and clamps it to valid bounds.
pub fn env_or_default<T>(var_name: &str, default: T, min: T, max: T) -> T
where
    T: FromStr + PartialOrd + Copy,
{
    std::env::var(var_name)
        .ok()
        .and_then(|s| s.parse().ok())
        .map(|v: T| {
            if v < min {
                min
            } else if v > max {
                max
            } else {
                v
            }
        })
        .unwrap_or(default)
}

/// Database configuration
pub mod db {
    use super::env_or_default;
    use std::time::Duration;

    /// Database name
    pub const DB_NAME: &str = "mangayomi";

    /// Default timeout for database operations (seconds)
    pub const DEFAULT_TIMEOUT_SECS: u64 = 30;

    /// Maximum timeout allowed (5 minutes)
    pub const MAX_TIMEOUT_SECS: u64 = 300;

    /// Minimum timeout allowed (1 second)
    pub const MIN_TIMEOUT_SECS: u64 = 1;

    /// Get database operation timeout from environment or use default
    pub fn get_timeout() -> Duration {
        Duration::from_secs(env_or_default(
            "DB_OPERATION_TIMEOUT_SECS",
            DEFAULT_TIMEOUT_SECS,
            MIN_TIMEOUT_SECS,
            MAX_TIMEOUT_SECS,
        ))
    }
}

/// Sync endpoint configuration
pub mod sync {
    /// Minimum allowed payload size (1MB)
    pub const MIN_PAYLOAD_SIZE_MB: usize = 1;

    /// Maximum allowed payload size (1GB)
    pub const MAX_PAYLOAD_SIZE_MB: usize = 1024;

    /// Default payload size (250MB)
    pub const DEFAULT_PAYLOAD_SIZE_MB: usize = 250;

    /// Environment variable for payload size
    pub const ENV_PAYLOAD_SIZE: &str = "MAX_SYNC_PAYLOAD_SIZE_MB";

    /// Maximum number of items allowed in a sync batch
    pub const MAX_SYNC_ITEMS: usize = 100_000;

    /// Maximum number of deleted items allowed
    pub const MAX_DELETED_ITEMS: usize = 10_000;

    /// Maximum ID value allowed (prevents overflow)
    pub const MAX_ID_VALUE: i64 = i64::MAX;

    /// Minimum ID value allowed
    pub const MIN_ID_VALUE: i64 = 1;
}

/// Bulk upsert configuration
pub mod bulk_upsert {
    /// Environment variable for chunk size
    pub const ENV_CHUNK_SIZE: &str = "BULK_UPSERT_CHUNK_SIZE";

    /// Environment variable for concurrency
    pub const ENV_CONCURRENCY: &str = "BULK_UPSERT_CONCURRENCY";

    /// Default chunk size
    pub const DEFAULT_CHUNK_SIZE: usize = 50;

    /// Default concurrency
    pub const DEFAULT_CONCURRENCY: usize = 20;

    /// Maximum chunk size allowed
    pub const MAX_CHUNK_SIZE: usize = 1000;

    /// Minimum chunk size allowed
    pub const MIN_CHUNK_SIZE: usize = 1;

    /// Maximum concurrency allowed
    pub const MAX_CONCURRENCY: usize = 100;

    /// Minimum concurrency allowed
    pub const MIN_CONCURRENCY: usize = 1;
}

/// Logging configuration
pub mod logging {
    /// Log target for sync API operations
    pub const SYNC_API: &str = "sync_api";

    /// Log target for sync payload debugging
    pub const SYNC_PAYLOAD: &str = "sync_payload";
}

/// MongoDB collection name constants (SSOT)
pub mod collections {
    pub const USERS: &str = "users";
    pub const CATEGORIES: &str = "categories";
    pub const MANGA: &str = "manga";
    pub const CHAPTERS: &str = "chapters";
    pub const TRACKS: &str = "tracks";
    pub const HISTORIES: &str = "histories";
    pub const UPDATES: &str = "updates";
    pub const SETTINGS: &str = "settings";
}
