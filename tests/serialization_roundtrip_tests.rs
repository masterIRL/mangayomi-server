//! Serialization round-trip tests
//!
//! Tests that verify server responses can be parsed by the Dart client.
//!
//! NOTE: JSON fixtures are duplicated across test files rather than shared.
//! This is intentional: Rust integration tests compile each file as a separate
//! binary. Sharing fixtures via a common module causes dead_code warnings
//! because each binary only uses a subset of the shared fixtures.
//! Duplication is cleaner than suppressing warnings or using #[allow(dead_code)].
use mangayomi_server::sync::manga::model::MangaList;
use mangayomi_server::sync::settings::model::SettingsObj;

/// i64 ID that exceeds i32::MAX (Isar-generated ID example)
const ID_I64_LARGE: i64 = 3_000_000_000_i64;

const FULL_MANGA_SYNC_PAYLOAD: &str = r#"{
    "categories": [{
        "id": 1,
        "name": "Reading",
        "forItemType": 0,
        "pos": 0,
        "hide": false,
        "shouldUpdate": true,
        "updatedAt": 1775156240310
    }],
    "deleted_categories": [],
    "manga": [{
        "id": 1,
        "name": "Naruto",
        "link": "https://example.com/naruto",
        "imageUrl": "https://example.com/naruto.jpg",
        "description": "A ninja story",
        "author": "Masashi Kishimoto",
        "artist": "Masashi Kishimoto",
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
        "genre": ["Action", "Shonen"],
        "categories": [1],
        "updatedAt": 1775156240310
    }, {
        "id": 3000000000,
        "name": "Bleach",
        "link": "https://example.com/bleach",
        "imageUrl": "https://example.com/bleach.jpg",
        "description": null,
        "author": null,
        "artist": null,
        "status": 0,
        "favorite": true,
        "source": "MangaDex",
        "sourceId": 1234567890124,
        "lang": "en",
        "dateAdded": 1775156240310,
        "lastUpdate": 1775156240310,
        "lastRead": 1775156240310,
        "isLocalArchive": false,
        "customCoverFromTracker": null,
        "itemType": 0,
        "genre": null,
        "categories": [1],
        "updatedAt": 1775156240310
    }],
    "deleted_manga": [2000000000],
    "chapters": [{
        "id": 1,
        "name": "Chapter 1",
        "url": "https://example.com/ch1",
        "dateUpload": "2026-01-01",
        "scanlator": "Group A",
        "isBookmarked": true,
        "isRead": false,
        "lastPageRead": "10",
        "archivePath": null,
        "mangaId": 1,
        "updatedAt": 1775156240310
    }, {
        "id": 2,
        "name": "Chapter 2",
        "url": "https://example.com/ch2",
        "dateUpload": "2026-01-02",
        "scanlator": "Group A",
        "isBookmarked": null,
        "isRead": null,
        "lastPageRead": "5",
        "archivePath": null,
        "mangaId": 1,
        "updatedAt": 1775156240310
    }, {
        "id": 3,
        "name": "Chapter 1",
        "url": "https://example.com/bleach-ch1",
        "dateUpload": "2026-01-01",
        "scanlator": "Group B",
        "isBookmarked": false,
        "isRead": true,
        "lastPageRead": "20",
        "archivePath": null,
        "mangaId": 3000000000,
        "updatedAt": 1775156240310
    }],
    "deleted_chapters": [9000000000],
    "tracks": [{
        "id": 1,
        "libraryId": 100,
        "mediaId": 200,
        "mangaId": 1,
        "score": 90,
        "startedReadingDate": 1775156240310,
        "finishedReadingDate": null,
        "lastChapterRead": 100,
        "status": 2,
        "syncId": 1,
        "title": "Naruto",
        "totalChapter": null,
        "trackingUrl": "https://myanimelist.net/manga/20",
        "isManga": true,
        "itemType": 0,
        "updatedAt": 1775156240310
    }],
    "deleted_tracks": []
}"#;

#[test]
fn test_full_manga_sync_payload_with_multiple_items() {
    // Simulates a real sync where user has 2 mangas, 3 chapters, 1 track,
    // 1 category, and some deleted items. All with i64 IDs and 2026 timestamps.
    let result: Result<MangaList, _> = serde_json::from_str(FULL_MANGA_SYNC_PAYLOAD);
    assert!(
        result.is_ok(),
        "Full multi-item sync payload must deserialize: {:?}",
        result.err()
    );

    let list = result.unwrap();
    assert_eq!(list.categories.len(), 1);
    assert_eq!(list.manga.len(), 2);
    assert_eq!(list.chapters.len(), 3);
    assert_eq!(list.tracks.len(), 1);
    assert_eq!(list.deleted_manga, vec![2_000_000_000_i64]);
    assert_eq!(list.deleted_chapters, vec![9_000_000_000_i64]);

    // Verify the manga with i64 ID
    let bleach = &list.manga[1];
    assert_eq!(bleach.id, ID_I64_LARGE);
    assert!(bleach.description.is_none());
    assert!(bleach.author.is_none());

    // Verify chapter with null booleans
    let ch2 = &list.chapters[1];
    assert!(ch2.is_bookmarked.is_none());
    assert!(ch2.is_read.is_none());

    // Verify chapter with i64 mangaId
    let ch_bleach = &list.chapters[2];
    assert_eq!(ch_bleach.manga_id, ID_I64_LARGE);
}

#[test]
fn test_settings_obj_none_serializes_to_null() {
    // When the server has no settings for a user, it returns {"settings": null}.
    // The Dart client receives this JSON. Verify it serializes correctly.
    let obj = SettingsObj { settings: None };
    let json = serde_json::to_string(&obj).unwrap();

    // The JSON must contain "settings":null
    assert!(
        json.contains(r#""settings":null"#),
        "SettingsObj with None must serialize settings as null, got: {}",
        json
    );
}

#[test]
fn test_manga_list_empty_response_serialization() {
    // Server returns empty lists for a first-sync user. Verify it serializes
    // to the format the Dart client expects.
    let list = MangaList {
        categories: vec![],
        deleted_categories: vec![],
        manga: vec![],
        deleted_manga: vec![],
        chapters: vec![],
        deleted_chapters: vec![],
        tracks: vec![],
        deleted_tracks: vec![],
        reset_all: None,
    };
    let json = serde_json::to_string(&list).unwrap();

    // Dart client reads jsonData["categories"], jsonData["manga"], etc.
    // Verify these keys exist in the serialized JSON.
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert!(parsed["categories"].is_array());
    assert!(parsed["manga"].is_array());
    assert!(parsed["chapters"].is_array());
    assert!(parsed["tracks"].is_array());
    assert_eq!(parsed["categories"].as_array().unwrap().len(), 0);
}
