//! Track deserialization tests
//!
//! Tests for Track entity deserialization within MangaList.
//!
//! NOTE: JSON fixtures are duplicated across test files rather than shared.
//! This is intentional: Rust integration tests compile each file as a separate
//! binary. Sharing fixtures via a common module causes dead_code warnings
//! because each binary only uses a subset of the shared fixtures.
//! Duplication is cleaner than suppressing warnings or using #[allow(dead_code)].
use mangayomi_server::sync::manga::model::MangaList;

/// i64 ID that exceeds i32::MAX (Isar-generated ID example)
const ID_I64_LARGE: i64 = 3_000_000_000_i64;
const ID_I64_TRACK: i64 = 4_000_000_000_i64;

const TRACK_WITH_NULLABLE_IDS: &str = r#"{
    "categories": [],
    "deleted_categories": [],
    "manga": [],
    "deleted_manga": [],
    "chapters": [],
    "deleted_chapters": [],
    "tracks": [{
        "id": 1,
        "libraryId": null,
        "mediaId": null,
        "mangaId": null,
        "score": null,
        "startedReadingDate": null,
        "finishedReadingDate": null,
        "lastChapterRead": null,
        "status": null,
        "syncId": null,
        "title": null,
        "totalChapter": null,
        "trackingUrl": null,
        "isManga": null,
        "itemType": 0,
        "updatedAt": 0
    }],
    "deleted_tracks": []
}"#;

const TRACK_WITH_COMPLETE_DATA: &str = r#"{
    "categories": [],
    "deleted_categories": [],
    "manga": [],
    "deleted_manga": [],
    "chapters": [],
    "deleted_chapters": [],
    "tracks": [{
        "id": 4000000000,
        "libraryId": 100,
        "mediaId": 4000000001,
        "mangaId": 3000000000,
        "score": 85,
        "startedReadingDate": 1775156240310,
        "finishedReadingDate": null,
        "lastChapterRead": 1000,
        "status": 1,
        "syncId": 4000000002,
        "title": "One Piece",
        "totalChapter": null,
        "trackingUrl": "https://myanimelist.net/manga/13",
        "isManga": true,
        "itemType": 0,
        "updatedAt": 1775156240310
    }],
    "deleted_tracks": []
}"#;

#[test]
fn test_track_with_all_nullable_id_fields() {
    // Dart Track has int? for libraryId, mediaId, mangaId, syncId, and String? for title/trackingUrl.
    // A brand-new track might have some of these as null.
    let result: Result<MangaList, _> = serde_json::from_str(TRACK_WITH_NULLABLE_IDS);
    assert!(
        result.is_ok(),
        "Track with null IDs must deserialize: {:?}",
        result.err()
    );

    let track = &result.unwrap().tracks[0];
    assert!(track.media_id.is_none());
    assert!(track.manga_id.is_none());
    assert!(track.sync_id.is_none());
    assert!(track.title.is_none());
    assert!(track.tracking_url.is_none());
}

#[test]
fn test_track_with_complete_data_and_i64_ids() {
    // Fully populated track with i64 IDs.
    let result: Result<MangaList, _> = serde_json::from_str(TRACK_WITH_COMPLETE_DATA);
    assert!(
        result.is_ok(),
        "Track with complete i64 data must deserialize: {:?}",
        result.err()
    );

    let track = &result.unwrap().tracks[0];
    assert_eq!(track.id, ID_I64_TRACK);
    assert_eq!(track.media_id, Some(4_000_000_001_i64));
    assert_eq!(track.manga_id, Some(ID_I64_LARGE));
    assert_eq!(track.sync_id, Some(4_000_000_002_i64));
    assert_eq!(track.title.as_deref(), Some("One Piece"));
    assert_eq!(
        track.tracking_url.as_deref(),
        Some("https://myanimelist.net/manga/13")
    );
}
