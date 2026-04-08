//! Update deserialization tests
//!
//! Tests for Update and UpdateList deserialization.
//!
//! NOTE: JSON fixtures are duplicated across test files rather than shared.
//! This is intentional: Rust integration tests compile each file as a separate
//! binary. Sharing fixtures via a common module causes dead_code warnings
//! because each binary only uses a subset of the shared fixtures.
//! Duplication is cleaner than suppressing warnings or using #[allow(dead_code)].
use mangayomi_server::sync::update::model::UpdateList;

/// i64 ID that exceeds i32::MAX (Isar-generated ID example)
const ID_I64_LARGE: i64 = 3_000_000_000_i64;

const EMPTY_UPDATE_LIST: &str = r#"{
    "updates": [],
    "deleted_updates": []
}"#;

const UPDATE_WITH_I64_IDS: &str = r#"{
    "updates": [{
        "id": 3000000000,
        "mangaId": 4000000000,
        "chapterName": "Chapter 100",
        "date": "2026-01-01",
        "updatedAt": 1775156240310
    }],
    "deleted_updates": []
}"#;

#[test]
fn test_update_list_empty_first_sync() {
    let result: Result<UpdateList, _> = serde_json::from_str(EMPTY_UPDATE_LIST);
    assert!(
        result.is_ok(),
        "Empty update list must deserialize: {:?}",
        result.err()
    );
    assert!(result.unwrap().updates.is_empty());
}

#[test]
fn test_update_with_i64_ids() {
    let result: Result<UpdateList, _> = serde_json::from_str(UPDATE_WITH_I64_IDS);
    assert!(
        result.is_ok(),
        "Update with i64 IDs must deserialize: {:?}",
        result.err()
    );

    let update = &result.unwrap().updates[0];
    assert_eq!(update.id, ID_I64_LARGE);
    assert_eq!(update.manga_id, 4_000_000_000_i64);
}
