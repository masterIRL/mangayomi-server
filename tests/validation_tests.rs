//! Tests for the validation helper module
//!
//! These tests verify the centralized validation logic that eliminates
//! code duplication across entity types.

use mangayomi_server::config::sync;
use mangayomi_server::sync::validation::{
    validate_deleted_count, validate_deleted_ids, validate_id, validate_ids,
};

/// Test that valid IDs within the acceptable range pass validation
///
/// Business rule: IDs must be positive (>= MIN_ID_VALUE) and not exceed MAX_ID_VALUE.
#[test]
fn test_validate_id_accepts_valid_range() {
    // Minimum valid ID
    assert!(validate_id(sync::MIN_ID_VALUE, "test").is_ok());

    // Typical valid IDs
    assert!(validate_id(1, "category").is_ok());
    assert!(validate_id(100, "manga").is_ok());
    assert!(validate_id(999999, "chapter").is_ok());

    // Maximum valid ID
    assert!(validate_id(sync::MAX_ID_VALUE, "track").is_ok());
}

/// Test that ID of 0 is rejected
///
/// Business rule: 0 is not a valid ID (likely indicates uninitialized data).
#[test]
fn test_validate_id_rejects_zero() {
    let result = validate_id(0, "test");

    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(error.contains("Invalid test ID"));
    assert!(error.contains("0"));
}

/// Test that negative IDs are rejected
///
/// Security: Negative IDs could indicate integer underflow or malicious input.
#[test]
fn test_validate_id_rejects_negative() {
    let test_cases = vec![
        (-1, "single negative"),
        (-100, "large negative"),
        (i64::MIN, "minimum i64"),
    ];

    for (id, description) in test_cases {
        let result = validate_id(id, "entity");
        assert!(result.is_err(), "{} should be rejected", description);
    }
}

/// Test error message includes entity type context
///
/// DX: Error messages should help developers identify which entity failed validation.
#[test]
fn test_validate_id_error_includes_entity_type() {
    let result = validate_id(0, "Category").unwrap_err();

    assert!(
        result.contains("Invalid Category ID"),
        "Error should include entity type 'Category'"
    );
}

/// Test error message includes ID value and valid range
///
/// DX: Error messages should help users understand what values are acceptable.
#[test]
fn test_validate_id_error_includes_range_info() {
    let result = validate_id(0, "test").unwrap_err();

    assert!(
        result.contains(&sync::MIN_ID_VALUE.to_string()),
        "Error should include minimum valid value"
    );
    assert!(
        result.contains(&sync::MAX_ID_VALUE.to_string()),
        "Error should include maximum valid value"
    );
}

/// Test batch validation with all valid IDs
///
/// Performance: Batch validation should be efficient for large collections.
#[test]
fn test_validate_ids_all_valid() {
    let ids: Vec<i64> = (1..=1000).collect();

    let result = validate_ids(&ids, "history", |&id| id);

    assert!(result.is_ok(), "All valid IDs should pass batch validation");
}

/// Test batch validation fails on first invalid ID
///
/// Business rule: Fail fast to avoid unnecessary processing.
#[test]
fn test_validate_ids_fails_fast_on_first_invalid() {
    // First ID is invalid
    let ids = vec![0, 1, 2, 3];

    let result = validate_ids(&ids, "test", |&id| id);

    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(error.contains("0"), "Should identify the first invalid ID");
}

/// Test batch validation fails on last invalid ID
#[test]
fn test_validate_ids_detects_invalid_at_end() {
    let ids = vec![1, 2, 3, -1];

    let result = validate_ids(&ids, "update", |&id| id);

    assert!(result.is_err());
    assert!(result.unwrap_err().contains("-1"));
}

/// Test batch validation with empty collection
///
/// Edge case: Empty collections should be valid (nothing to validate).
#[test]
fn test_validate_ids_empty_collection() {
    let empty: Vec<i64> = vec![];

    let result = validate_ids(&empty, "test", |&id| id);

    assert!(result.is_ok(), "Empty collection should be valid");
}

/// Test batch validation with single valid ID
#[test]
fn test_validate_ids_single_valid() {
    let ids = vec![42];

    assert!(validate_ids(&ids, "manga", |&id| id).is_ok());
}

/// Test batch validation extracts ID correctly via closure
///
/// This tests the flexibility of the validation function with custom types.
#[test]
fn test_validate_ids_with_complex_type() {
    struct Item {
        id: i64,
        #[allow(dead_code)]
        name: String,
    }

    let items = vec![
        Item {
            id: 1,
            name: "First".to_string(),
        },
        Item {
            id: 2,
            name: "Second".to_string(),
        },
    ];

    let result = validate_ids(&items, "item", |item| item.id);

    assert!(result.is_ok());
}

/// Test deleted count validation within limits
///
/// Business rule: Number of deleted items per sync must be bounded
/// to prevent abuse and ensure reasonable payload sizes.
#[test]
fn test_validate_deleted_count_within_limits() {
    assert!(
        validate_deleted_count(0, "test").is_ok(),
        "Zero should be valid"
    );
    assert!(
        validate_deleted_count(1, "manga").is_ok(),
        "Single deletion should be valid"
    );
    assert!(
        validate_deleted_count(100, "chapter").is_ok(),
        "Moderate count should be valid"
    );
    assert!(
        validate_deleted_count(sync::MAX_DELETED_ITEMS, "track").is_ok(),
        "Maximum allowed should be valid"
    );
}

/// Test deleted count validation rejects excess
///
/// Security: Excessive deletions could indicate malicious behavior or bugs.
#[test]
fn test_validate_deleted_count_rejects_excess() {
    let result = validate_deleted_count(sync::MAX_DELETED_ITEMS + 1, "category");

    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(error.contains("Too many deleted category"));
    assert!(error.contains(&(sync::MAX_DELETED_ITEMS + 1).to_string()));
    assert!(error.contains(&sync::MAX_DELETED_ITEMS.to_string()));
}

/// Test deleted IDs validation combines count and ID validation
///
/// This is the convenience function that validates both the count
/// of deleted items and that each ID is valid.
#[test]
fn test_validate_deleted_ids_all_valid() {
    let ids = vec![1, 2, 3, 4, 5];

    let result = validate_deleted_ids(&ids, "history");

    assert!(result.is_ok());
}

/// Test deleted IDs validation rejects excessive count
#[test]
fn test_validate_deleted_ids_rejects_too_many() {
    // Create more IDs than allowed
    let ids: Vec<i64> = (1..=sync::MAX_DELETED_ITEMS + 1)
        .map(|i| i as i64)
        .collect();

    let result = validate_deleted_ids(&ids, "update");

    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Too many deleted update"));
}

/// Test deleted IDs validation rejects invalid ID in list
#[test]
fn test_validate_deleted_ids_rejects_invalid_id() {
    let ids = vec![1, 2, 0, 4]; // 0 is invalid

    let result = validate_deleted_ids(&ids, "manga");

    assert!(result.is_err());
    // Should mention "deleted manga" to indicate context
    assert!(result.unwrap_err().contains("deleted manga"));
}

/// Test deleted IDs validation with empty list
#[test]
fn test_validate_deleted_ids_empty_list() {
    let empty: Vec<i64> = vec![];

    let result = validate_deleted_ids(&empty, "test");

    assert!(result.is_ok(), "Empty deleted list should be valid");
}

/// Test boundary values around i64::MAX
///
/// Edge case: Ensure we handle large values correctly without overflow.
#[test]
fn test_validate_id_boundary_at_max_i64() {
    // i64::MAX should be rejected as it exceeds MAX_ID_VALUE
    if i64::MAX > sync::MAX_ID_VALUE {
        let result = validate_id(i64::MAX, "test");
        assert!(
            result.is_err(),
            "i64::MAX should be rejected when > MAX_ID_VALUE"
        );
    }
}

/// Test error messages are descriptive for debugging
#[test]
fn test_error_messages_are_descriptive() {
    let id_error = validate_id(-5, "Manga").unwrap_err();
    assert!(
        id_error.contains("Invalid"),
        "Should indicate invalid input"
    );
    assert!(id_error.contains("Manga"), "Should identify entity type");
    assert!(id_error.contains("-5"), "Should show actual value");
    assert!(id_error.contains("between"), "Should indicate constraint");

    let count_error = validate_deleted_count(sync::MAX_DELETED_ITEMS + 100, "Chapter").unwrap_err();
    assert!(count_error.contains("Too many"), "Should indicate excess");
    assert!(
        count_error.contains("Chapter"),
        "Should identify entity type"
    );
}
