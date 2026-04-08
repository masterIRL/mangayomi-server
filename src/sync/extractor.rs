use actix_web::{web, HttpResponse};
use serde::de::DeserializeOwned;

use crate::config::sync;
use crate::config::logging;
use crate::config::env_or_default;

/// Maximum sync payload size in bytes (default: 250MB)
pub fn get_max_payload_size() -> usize {
    let size_mb = env_or_default(
        sync::ENV_PAYLOAD_SIZE,
        sync::DEFAULT_PAYLOAD_SIZE_MB,
        sync::MIN_PAYLOAD_SIZE_MB,
        sync::MAX_PAYLOAD_SIZE_MB,
    );
    size_mb << 20
}

/// Validates that a sync batch size is within acceptable limits
fn validate_batch_size(item_count: usize, entity_type: &str) -> Result<(), HttpResponse> {
    if item_count > sync::MAX_SYNC_ITEMS {
        log::error!(
            target: logging::SYNC_API,
            "[SYNC] {} batch too large: {} items (max: {})",
            entity_type, item_count, sync::MAX_SYNC_ITEMS
        );
        return Err(HttpResponse::PayloadTooLarge()
            .body(format!("Batch too large: {} items (max: {})",
                item_count, sync::MAX_SYNC_ITEMS)));
    }
    Ok(())
}

/// Extract and parse sync payload from request body with validation
///
/// # Arguments
/// * `body` - Raw request body bytes
/// * `entity_type` - Entity type name for logging
///
/// # Returns
/// * `Ok(T)` - Parsed and validated payload
/// * `Err(HttpResponse)` - Error response (400 or 413)
pub async fn extract_sync_payload<T>(body: web::Bytes, entity_type: &str) -> Result<T, HttpResponse>
where
    T: DeserializeOwned + ValidatePayload,
{
    let max_size = get_max_payload_size();
    if body.len() > max_size {
        log::error!(
            target: logging::SYNC_API,
            "[SYNC] {} payload too large: {} bytes (max: {} bytes)",
            sanitize_for_log(entity_type),
            body.len(),
            max_size
        );
        return Err(HttpResponse::PayloadTooLarge().body("Payload too large"));
    }

    log::debug!(
        target: logging::SYNC_PAYLOAD,
        "[SYNC] {} payload received: {} bytes",
        sanitize_for_log(entity_type),
        body.len()
    );

    let data: T = match serde_json::from_slice(&body) {
        Ok(d) => d,
        Err(e) => {
            log::warn!(
                target: logging::SYNC_PAYLOAD,
                "[SYNC] Failed to parse {} payload: {}",
                sanitize_for_log(entity_type),
                sanitize_for_log(&e.to_string())
            );
            return Err(HttpResponse::BadRequest()
                .body(format!("Invalid payload: {}", e)));
        }
    };

    if let Err(e) = data.validate() {
        log::warn!(
            target: logging::SYNC_API,
            "[SYNC] {} payload validation failed: {}",
            sanitize_for_log(entity_type),
            sanitize_for_log(&e)
        );
        return Err(HttpResponse::BadRequest().body(e));
    }

    let item_count = data.item_count();
    validate_batch_size(item_count, entity_type)?;

    log::debug!(
        target: logging::SYNC_PAYLOAD,
        "[SYNC] {} payload parsed and validated successfully ({} items)",
        sanitize_for_log(entity_type),
        item_count
    );

    Ok(data)
}

/// Trait for payloads that can be validated
pub trait ValidatePayload {
    fn item_count(&self) -> usize;
    fn validate(&self) -> Result<(), String>;
}

/// Sanitizes a string for safe logging
fn sanitize_for_log(input: &str) -> String {
    input
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
        .replace('\0', "")
        .chars()
        .take(1000)
        .collect()
}

/// Log sync completion with structured metrics
///
/// # Arguments
/// * `entity_type` - Type of entity synced
/// * `success_count` - Number of items successfully processed
/// * `failed_count` - Number of items that failed
/// * `duration_ms` - Duration in milliseconds
pub fn log_sync_complete(entity_type: &str, success_count: usize, failed_count: usize, duration_ms: u128) {
    if failed_count > 0 {
        log::warn!(
            target: logging::SYNC_API,
            "[SYNC] {} sync completed: {} items processed, {} failed in {}ms",
            entity_type, success_count, failed_count, duration_ms
        );
    } else {
        log::info!(
            target: logging::SYNC_API,
            "[SYNC] {} sync completed: {} items processed in {}ms",
            entity_type, success_count, duration_ms
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_removes_injection_characters() {
        assert_eq!(sanitize_for_log("hello\nworld"), "hello\\nworld");
        assert_eq!(sanitize_for_log("test%format"), "test%format");
        assert_eq!(sanitize_for_log("tab\there"), "tab\\there");
        assert_eq!(sanitize_for_log("null\0byte"), "nullbyte");
        assert_eq!(sanitize_for_log("cr\rreturn"), "cr\\rreturn");
    }

    #[test]
    fn test_sanitize_truncates_long_strings() {
        let long = "a".repeat(2000);
        let sanitized = sanitize_for_log(&long);
        assert_eq!(sanitized.len(), 1000);
    }

    #[test]
    fn test_validate_batch_size_at_boundary() {
        assert!(validate_batch_size(sync::MAX_SYNC_ITEMS, "test").is_ok());
        assert!(validate_batch_size(sync::MAX_SYNC_ITEMS + 1, "test").is_err());
    }
}