//! Manga deserialization tests
//!
//! Tests for MangaList, Manga, and Chapter deserialization.
//!
//! NOTE: JSON fixtures are duplicated across test files rather than shared.
//! This is intentional: Rust integration tests compile each file as a separate
//! binary. Sharing fixtures via a common module causes dead_code warnings
//! because each binary only uses a subset of the shared fixtures.
//! Duplication is cleaner than suppressing warnings or using #[allow(dead_code)].
use mangayomi_server::sync::manga::model::MangaList;

/// Timestamp value from 2026 (DateTime.now().millisecondsSinceEpoch)
const TIMESTAMP_2026: i64 = 1_775_156_240_310_i64;

/// i64 ID that exceeds i32::MAX (Isar-generated ID example)
const ID_I64_LARGE: i64 = 3_000_000_000_i64;
const ID_I64_CHAPTER: i64 = 5_000_000_000_i64;

const EMPTY_MANGA_LIST: &str = r#"{
    "categories": [],
    "deleted_categories": [],
    "manga": [],
    "deleted_manga": [],
    "chapters": [],
    "deleted_chapters": [],
    "tracks": [],
    "deleted_tracks": []
}"#;

const MANGA_LIST_WITH_RESET_ALL: &str = r#"{
    "categories": [],
    "deleted_categories": [],
    "manga": [],
    "deleted_manga": [],
    "chapters": [],
    "deleted_chapters": [],
    "tracks": [],
    "deleted_tracks": [],
    "resetAll": true
}"#;

const MANGA_WITH_I64_ID: &str = r#"{
    "categories": [],
    "deleted_categories": [],
    "manga": [{
        "id": 3000000000,
        "name": "One Piece",
        "link": "https://example.com/one-piece",
        "imageUrl": "https://example.com/cover.jpg",
        "description": "A pirate adventure",
        "author": "Eiichiro Oda",
        "artist": "Eiichiro Oda",
        "status": 0,
        "favorite": true,
        "source": "MangaDex",
        "sourceId": 1234567890123,
        "lang": "en",
        "dateAdded": 1775156240310,
        "lastUpdate": 1775156240310,
        "lastRead": 1775156240310,
        "isLocalArchive": false,
        "customCoverFromTracker": null,
        "itemType": 0,
        "genre": ["Action", "Adventure"],
        "categories": [1, 2],
        "updatedAt": 1775156240310,
        "customCoverImage": null,
        "smartUpdateDays": null
    }],
    "deleted_manga": [],
    "chapters": [],
    "deleted_chapters": [],
    "tracks": [],
    "deleted_tracks": []
}"#;

const MANGA_WITH_NULL_OPTIONALS: &str = r#"{
    "categories": [],
    "deleted_categories": [],
    "manga": [{
        "id": 1,
        "name": "Test Manga",
        "link": "https://example.com",
        "imageUrl": "https://example.com/img.jpg",
        "description": null,
        "author": null,
        "artist": null,
        "status": 3,
        "favorite": false,
        "source": "TestSource",
        "sourceId": null,
        "lang": "en",
        "dateAdded": null,
        "lastUpdate": null,
        "lastRead": null,
        "isLocalArchive": null,
        "customCoverFromTracker": null,
        "itemType": 0,
        "genre": null,
        "categories": null,
        "updatedAt": 0
    }],
    "deleted_manga": [],
    "chapters": [],
    "deleted_chapters": [],
    "tracks": [],
    "deleted_tracks": []
}"#;

const CHAPTER_WITH_NULLABLE_BOOLEANS: &str = r#"{
    "categories": [],
    "deleted_categories": [],
    "manga": [],
    "deleted_manga": [],
    "chapters": [{
        "id": 5000000000,
        "name": "Chapter 1",
        "url": "https://example.com/ch1",
        "dateUpload": "2026-01-01",
        "scanlator": "TestGroup",
        "isBookmarked": null,
        "isRead": null,
        "lastPageRead": "5",
        "archivePath": null,
        "mangaId": 3000000000,
        "updatedAt": 1775156240310,
        "isFiller": false,
        "thumbnailUrl": null,
        "description": null,
        "downloadSize": null,
        "duration": null
    }],
    "deleted_chapters": [],
    "tracks": [],
    "deleted_tracks": []
}"#;

const CHAPTER_WITH_EXPLICIT_BOOLEANS: &str = r#"{
    "categories": [],
    "deleted_categories": [],
    "manga": [],
    "deleted_manga": [],
    "chapters": [{
        "id": 1,
        "name": "Chapter 1",
        "url": "",
        "dateUpload": "",
        "scanlator": "",
        "isBookmarked": true,
        "isRead": false,
        "lastPageRead": "",
        "archivePath": "",
        "mangaId": 1,
        "updatedAt": 0
    }],
    "deleted_chapters": [],
    "tracks": [],
    "deleted_tracks": []
}"#;

#[test]
fn test_manga_list_empty_library_first_sync() {
    let result: Result<MangaList, _> = serde_json::from_str(EMPTY_MANGA_LIST);
    assert!(
        result.is_ok(),
        "Empty library payload must deserialize: {:?}",
        result.err()
    );

    let list = result.unwrap();
    assert!(list.categories.is_empty());
    assert!(list.manga.is_empty());
    assert!(list.chapters.is_empty());
    assert!(list.tracks.is_empty());
    assert!(list.deleted_categories.is_empty());
    assert!(
        list.reset_all.is_none(),
        "resetAll should be None when not sent"
    );
}

#[test]
fn test_manga_list_upload_with_reset_all() {
    let result: Result<MangaList, _> = serde_json::from_str(MANGA_LIST_WITH_RESET_ALL);
    assert!(
        result.is_ok(),
        "Upload payload with resetAll must deserialize: {:?}",
        result.err()
    );
    assert_eq!(result.unwrap().reset_all, Some(true));
}

#[test]
fn test_manga_single_item_with_i64_id_and_large_timestamp() {
    let result: Result<MangaList, _> = serde_json::from_str(MANGA_WITH_I64_ID);
    assert!(
        result.is_ok(),
        "Manga with i64 ID must deserialize: {:?}",
        result.err()
    );

    let list = result.unwrap();
    assert_eq!(list.manga.len(), 1);
    assert_eq!(list.manga[0].id, ID_I64_LARGE);
    assert_eq!(list.manga[0].updated_at, TIMESTAMP_2026);
    assert_eq!(list.manga[0].name, "One Piece");
}

#[test]
fn test_manga_with_all_optional_fields_null() {
    let result: Result<MangaList, _> = serde_json::from_str(MANGA_WITH_NULL_OPTIONALS);
    assert!(
        result.is_ok(),
        "Manga with null optionals must deserialize: {:?}",
        result.err()
    );

    let manga = &result.unwrap().manga[0];
    assert!(manga.description.is_none());
    assert!(manga.author.is_none());
    assert!(manga.artist.is_none());
    assert!(manga.source_id.is_none());
}

#[test]
fn test_chapter_with_nullable_booleans_and_i64_ids() {
    let result: Result<MangaList, _> = serde_json::from_str(CHAPTER_WITH_NULLABLE_BOOLEANS);
    assert!(
        result.is_ok(),
        "Chapter with null booleans must deserialize: {:?}",
        result.err()
    );

    let chapter = &result.unwrap().chapters[0];
    assert_eq!(chapter.id, ID_I64_CHAPTER);
    assert_eq!(chapter.manga_id, ID_I64_LARGE);
    assert!(chapter.is_bookmarked.is_none());
    assert!(chapter.is_read.is_none());
}

#[test]
fn test_chapter_with_explicit_boolean_values() {
    let result: Result<MangaList, _> = serde_json::from_str(CHAPTER_WITH_EXPLICIT_BOOLEANS);
    assert!(
        result.is_ok(),
        "Chapter with explicit booleans must deserialize: {:?}",
        result.err()
    );

    let chapter = &result.unwrap().chapters[0];
    assert_eq!(chapter.is_bookmarked, Some(true));
    assert_eq!(chapter.is_read, Some(false));
}
