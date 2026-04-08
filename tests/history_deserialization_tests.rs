//! History deserialization tests
//!
//! Tests for History and HistoryList deserialization.
//!
//! NOTE: JSON fixtures are duplicated across test files rather than shared.
//! This is intentional: Rust integration tests compile each file as a separate
//! binary. Sharing fixtures via a common module causes dead_code warnings
//! because each binary only uses a subset of the shared fixtures.
//! Duplication is cleaner than suppressing warnings or using #[allow(dead_code)].
use mangayomi_server::sync::history::model::HistoryList;

/// Timestamp value from 2026 (DateTime.now().millisecondsSinceEpoch)
const TIMESTAMP_2026: i64 = 1_775_156_240_310_i64;

/// i64 ID that exceeds i32::MAX (Isar-generated ID example)
const ID_I64_CHAPTER: i64 = 5_000_000_000_i64;
const ID_I64_HISTORY: i64 = 3_000_000_000_i64;

const EMPTY_HISTORY_LIST: &str = r#"{
    "histories": [],
    "deleted_histories": []
}"#;

const HISTORY_WITHOUT_RESET_ALL: &str = r#"{
    "histories": [{
        "id": 1,
        "date": "2026-01-01",
        "mangaId": 100,
        "chapterId": 200,
        "itemType": 0,
        "updatedAt": 1775156240310
    }],
    "deleted_histories": []
}"#;

const HISTORY_WITH_I64_IDS: &str = r#"{
    "histories": [{
        "id": 3000000000,
        "date": "2026-01-01",
        "mangaId": 4000000000,
        "chapterId": 5000000000,
        "itemType": 0,
        "updatedAt": 1775156240310
    }],
    "deleted_histories": []
}"#;

#[test]
fn test_history_list_empty_first_sync() {
    let result: Result<HistoryList, _> = serde_json::from_str(EMPTY_HISTORY_LIST);
    assert!(
        result.is_ok(),
        "Empty history list must deserialize: {:?}",
        result.err()
    );
    assert!(result.unwrap().histories.is_empty());
}

#[test]
fn test_history_without_reset_all_key() {
    // The Dart client doesn't include resetAll in the JSON unless doing an upload.
    // The Rust model uses Option<bool> — missing key should be None.
    let result: Result<HistoryList, _> = serde_json::from_str(HISTORY_WITHOUT_RESET_ALL);
    assert!(
        result.is_ok(),
        "History without resetAll key must deserialize: {:?}",
        result.err()
    );
    assert!(result.unwrap().reset_all.is_none());
}

#[test]
fn test_history_with_i64_ids_and_large_timestamp() {
    let result: Result<HistoryList, _> = serde_json::from_str(HISTORY_WITH_I64_IDS);
    assert!(
        result.is_ok(),
        "History with i64 IDs must deserialize: {:?}",
        result.err()
    );

    let history = &result.unwrap().histories[0];
    assert_eq!(history.id, ID_I64_HISTORY);
    assert_eq!(history.manga_id, 4_000_000_000_i64);
    assert_eq!(history.chapter_id, ID_I64_CHAPTER);
    assert_eq!(history.updated_at, TIMESTAMP_2026);
}
