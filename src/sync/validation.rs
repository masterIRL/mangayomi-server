//! Validation helper functions for sync operations

use crate::config::sync;

/// Validates that an ID is within the acceptable range
///
/// # Arguments
/// * `id` - The ID to validate
/// * `entity_type` - Entity type for error messages
///
/// # Errors
/// Returns error string if ID is out of range
pub fn validate_id(id: i64, entity_type: &str) -> Result<(), String> {
    if id < sync::MIN_ID_VALUE || id > sync::MAX_ID_VALUE {
        Err(format!(
            "Invalid {} ID: {} (must be between {} and {})",
            entity_type,
            id,
            sync::MIN_ID_VALUE,
            sync::MAX_ID_VALUE
        ))
    } else {
        Ok(())
    }
}

/// Validates a collection of items that have an ID
///
/// # Arguments
/// * `items` - Items to validate
/// * `entity_type` - Entity type for error messages
/// * `get_id` - Function to extract ID from item
///
/// # Errors
/// Returns error string if any ID is invalid
pub fn validate_ids<T>(
    items: &[T],
    entity_type: &str,
    get_id: impl Fn(&T) -> i64,
) -> Result<(), String> {
    for item in items {
        let id = get_id(item);
        validate_id(id, entity_type)?;
    }
    Ok(())
}

/// Validates that a deleted items count is within acceptable limits
///
/// # Arguments
/// * `count` - Number of deleted items
/// * `entity_type` - Entity type for error messages
///
/// # Errors
/// Returns error string if count exceeds maximum
pub fn validate_deleted_count(count: usize, entity_type: &str) -> Result<(), String> {
    if count > sync::MAX_DELETED_ITEMS {
        Err(format!(
            "Too many deleted {}: {} (max: {})",
            entity_type,
            count,
            sync::MAX_DELETED_ITEMS
        ))
    } else {
        Ok(())
    }
}

/// Validates a collection of deleted IDs
///
/// # Arguments
/// * `ids` - Deleted IDs to validate
/// * `entity_type` - Entity type for error messages
///
/// # Errors
/// Returns error string if count or any ID is invalid
pub fn validate_deleted_ids(ids: &[i64], entity_type: &str) -> Result<(), String> {
    validate_deleted_count(ids.len(), entity_type)?;

    for &id in ids {
        validate_id(id, &format!("deleted {}", entity_type))?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_id_valid_range() {
        assert!(validate_id(sync::MIN_ID_VALUE, "test").is_ok());
        assert!(validate_id(sync::MAX_ID_VALUE, "test").is_ok());
        assert!(validate_id(1, "category").is_ok());
        assert!(validate_id(1000000, "manga").is_ok());
    }

    #[test]
    fn test_validate_id_below_minimum() {
        let result = validate_id(0, "test");
        assert!(result.is_err());
        let err_msg = result.unwrap_err();
        assert!(err_msg.contains("0"));
        assert!(err_msg.contains("test"));
    }

    #[test]
    fn test_validate_id_negative() {
        let result = validate_id(-1, "manga");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("-1"));
    }

    #[test]
    fn test_validate_id_above_maximum() {
        assert!(validate_id(i64::MAX, "chapter").is_ok());
    }

    #[test]
    fn test_validate_ids_empty_collection() {
        let empty: Vec<i64> = vec![];
        assert!(validate_ids(&empty, "test", |&id| id).is_ok());
    }

    #[test]
    fn test_validate_ids_all_valid() {
        let ids = vec![1, 10, 100, 1000];
        assert!(validate_ids(&ids, "history", |&id| id).is_ok());
    }

    #[test]
    fn test_validate_ids_first_invalid() {
        let ids = vec![0, 1, 2];
        let result = validate_ids(&ids, "update", |&id| id);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("0"));
    }

    #[test]
    fn test_validate_ids_last_invalid() {
        let ids = vec![1, 2, -1];
        let result = validate_ids(&ids, "category", |&id| id);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("-1"));
    }

    #[test]
    fn test_validate_deleted_count_within_limit() {
        assert!(validate_deleted_count(0, "test").is_ok());
        assert!(validate_deleted_count(100, "manga").is_ok());
        assert!(validate_deleted_count(sync::MAX_DELETED_ITEMS, "chapter").is_ok());
    }

    #[test]
    fn test_validate_deleted_count_exceeds_limit() {
        let result = validate_deleted_count(sync::MAX_DELETED_ITEMS + 1, "track");
        assert!(result.is_err());
        let err_msg = result.unwrap_err();
        assert!(err_msg.contains("Too many deleted track"));
        assert!(err_msg.contains(&sync::MAX_DELETED_ITEMS.to_string()));
    }

    #[test]
    fn test_validate_deleted_ids_all_valid() {
        let ids = vec![1, 2, 3];
        assert!(validate_deleted_ids(&ids, "manga").is_ok());
    }

    #[test]
    fn test_validate_deleted_ids_count_exceeded() {
        let ids: Vec<i64> = (1..=sync::MAX_DELETED_ITEMS + 1)
            .map(|i| i as i64)
            .collect();
        let result = validate_deleted_ids(&ids, "category");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Too many deleted category"));
    }

    #[test]
    fn test_validate_deleted_ids_invalid_id() {
        let ids = vec![1, 0, 3];
        let result = validate_deleted_ids(&ids, "history");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("deleted history"));
    }
}
