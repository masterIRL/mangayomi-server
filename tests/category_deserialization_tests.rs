//! Category deserialization tests
//!
//! Tests for Category entity and deleted ID arrays.
//!
//! NOTE: JSON fixtures are duplicated across test files rather than shared.
//! This is intentional: Rust integration tests compile each file as a separate
//! binary. Sharing fixtures via a common module causes dead_code warnings
//! because each binary only uses a subset of the shared fixtures.
//! Duplication is cleaner than suppressing warnings or using #[allow(dead_code)].
use mangayomi_server::sync::manga::model::MangaList;

/// i64 ID that exceeds i32::MAX (Isar-generated ID example)
const ID_I64_LARGE: i64 = 3_000_000_000_i64;

const CATEGORY_WITH_I64_ID: &str = r#"{
    "categories": [{
        "id": 3000000000,
        "name": "Reading",
        "forItemType": 0,
        "pos": 0,
        "hide": false,
        "shouldUpdate": true,
        "updatedAt": 1775156240310
    }],
    "deleted_categories": [],
    "manga": [],
    "deleted_manga": [],
    "chapters": [],
    "deleted_chapters": [],
    "tracks": [],
    "deleted_tracks": []
}"#;

const DELETED_IDS_AS_I64: &str = r#"{
    "categories": [],
    "deleted_categories": [3000000000, 3000000001],
    "manga": [],
    "deleted_manga": [4000000000],
    "chapters": [],
    "deleted_chapters": [5000000000, 5000000001, 5000000002],
    "tracks": [],
    "deleted_tracks": [6000000000]
}"#;

#[test]
fn test_category_with_i64_id() {
    let result: Result<MangaList, _> = serde_json::from_str(CATEGORY_WITH_I64_ID);
    assert!(
        result.is_ok(),
        "Category with i64 ID must deserialize: {:?}",
        result.err()
    );
    assert_eq!(result.unwrap().categories[0].id, ID_I64_LARGE);
}

#[test]
fn test_deleted_ids_as_i64() {
    // deleted_* arrays contain Isar IDs which are 64-bit on the Dart side.
    // A deletion list with IDs > i32::MAX must deserialize.
    let result: Result<MangaList, _> = serde_json::from_str(DELETED_IDS_AS_I64);
    assert!(
        result.is_ok(),
        "Deletion IDs as i64 must deserialize: {:?}",
        result.err()
    );

    let list = result.unwrap();
    assert_eq!(
        list.deleted_categories,
        vec![3_000_000_000_i64, 3_000_000_001_i64]
    );
    assert_eq!(list.deleted_manga, vec![4_000_000_000_i64]);
    assert_eq!(list.deleted_chapters.len(), 3);
    assert_eq!(list.deleted_tracks, vec![6_000_000_000_i64]);
}
