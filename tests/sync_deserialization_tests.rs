/// Integration-level deserialization tests that simulate REAL Dart client payloads.
///
/// These tests construct the exact JSON the Mangayomi Dart client sends to the
/// Rust backend and verify that deserialization succeeds. They cover:
/// - i64 timestamps and IDs (Dart `int` is 64-bit)
/// - Empty library first-sync payloads
/// - Nullable fields the Dart client may send as null
/// - Unknown fields the Rust model doesn't define (serde should ignore them)
/// - Edge cases around `resetAll`, deletion arrays, and optional nested objects
use mangayomi_server::sync::settings::model::SettingsObj;
use mangayomi_server::sync::manga::model::MangaList;
use mangayomi_server::sync::history::model::HistoryList;
use mangayomi_server::sync::update::model::UpdateList;

// =============================================================================
// SETTINGS DESERIALIZATION TESTS
// =============================================================================

#[test]
fn test_settings_full_dart_client_payload() {
    // This is an exact simulation of what the Dart client sends via _getSettingsData()
    // when download=false (normal sync). Settings always has id=227.
    // updatedAt is DateTime.now().millisecondsSinceEpoch which in 2026 is ~1.77 trillion.
    let json = r##"{
        "settings": {
            "id": 227,
            "updatedAt": 1775156240310,
            "displayType": 0,
            "defaultReaderMode": 0,
            "animeDisplayType": 0,
            "scaleType": 0,
            "backgroundColor": 0,
            "colorFilterBlendMode": 0,
            "mangaHomeDisplayType": 1,
            "disableSectionType": 0,
            "novelDisplayType": 1,
            "novelTextAlign": 0,
            "flexColorSchemeBlendLevel": 10.0,
            "dateFormat": "M/d/y",
            "relativeTimesTamps": 2,
            "flexSchemeColorIndex": 2,
            "themeIsDark": false,
            "followSystemTheme": false,
            "incognitoMode": false,
            "showPagesNumber": true,
            "userAgent": "Mozilla/5.0",
            "animatePageTransitions": true,
            "doubleTapAnimationSpeed": 1,
            "onlyIncludePinnedSources": false,
            "pureBlackDarkMode": false,
            "downloadOnlyOnWifi": false,
            "saveAsCBZArchive": false,
            "concurrentDownloads": 2,
            "downloadLocation": "",
            "cropBorders": false,
            "autoExtensionsUpdates": false,
            "enableLogs": false,
            "checkForAppUpdates": true,
            "checkForExtensionUpdates": true,
            "usePageTapZones": true,
            "markEpisodeAsSeenType": 85,
            "defaultSkipIntroLength": 85,
            "defaultDoubleTapToSkipLength": 10,
            "defaultPlayBackSpeed": 1.0,
            "fullScreenPlayer": false,
            "forceLandscapePlayer": false,
            "updateProgressAfterReading": true,
            "customDns": "",
            "doHEnabled": false,
            "doHProviderId": 0,
            "btServerAddress": "127.0.0.1",
            "fullScreenReader": true,
            "enableCustomColorFilter": false,
            "appFontFamily": null,
            "mangaGridSize": null,
            "animeGridSize": null,
            "novelGridSize": null,
            "downloadedOnlyMode": false,
            "pagePreloadAmount": 6,
            "useLibass": true,
            "hwdecMode": "auto",
            "enableHardwareAcceleration": null,
            "novelFontSize": 14,
            "novelReaderTheme": "#292832",
            "novelReaderTextColor": "#CCCCCC",
            "novelReaderPadding": 16,
            "novelReaderLineHeight": 1.5,
            "novelShowScrollPercentage": true,
            "novelRemoveExtraParagraphSpacing": false,
            "novelTapToScroll": false,
            "clearChapterCacheOnAppLaunch": false,
            "mergeLibraryNavMobile": false,
            "enableDiscordRpc": true,
            "hideDiscordRpcInIncognito": true,
            "rpcShowReadingWatchingProgress": true,
            "rpcShowTitle": true,
            "rpcShowCoverImage": true,
            "useMpvConfig": true,
            "enableGpuNext": false,
            "useYUV420P": false,
            "enableAudioPitchCorrection": null,
            "cookiesList": [],
            "deleteDownloadAfterReading": false,
            "appLockEnabled": false,
            "libraryFilterMangasCompletedType": 0,
            "libraryFilterAnimeCompletedType": 0,
            "libraryFilterNovelCompletedType": 0,
            "keepScreenOnReader": true,
            "webtoonSidePadding": 0,
            "showPageGaps": true,
            "invertColors": false,
            "grayscale": false,
            "readerBrightness": 0.0,
            "readerContrast": 1.0,
            "readerSaturation": 1.0,
            "readerNavigationLayout": 0
        }
    }"##;

    let result: Result<SettingsObj, _> = serde_json::from_str(json);
    assert!(result.is_ok(), "Failed to deserialize full Dart settings payload: {:?}", result.err());

    let obj = result.unwrap();
    assert!(obj.settings.is_some());
    let settings = obj.settings.unwrap();
    assert_eq!(settings.id, 227);
    assert_eq!(settings.updated_at, 1775156240310_i64);
    assert_eq!(settings.display_type, 0);
}

#[test]
fn test_settings_download_only_empty_payload() {
    // When download=true, the Dart client sends {} (no "settings" key at all).
    // This must deserialize to SettingsObj { settings: None } — NOT panic.
    let json = r##"{}"##;

    let result: Result<SettingsObj, _> = serde_json::from_str(json);
    assert!(result.is_ok(), "Empty payload must deserialize: {:?}", result.err());

    let obj = result.unwrap();
    assert!(obj.settings.is_none(), "settings should be None for download-only payload");
}

#[test]
fn test_settings_with_explicit_null() {
    // Edge case: some clients might send {"settings": null} explicitly.
    let json = r#"{"settings": null}"#;

    let result: Result<SettingsObj, _> = serde_json::from_str(json);
    assert!(result.is_ok(), "Explicit null settings must deserialize: {:?}", result.err());
    assert!(result.unwrap().settings.is_none());
}

#[test]
fn test_settings_timestamp_at_i32_boundary() {
    // Exact i32::MAX boundary — this is the value that JUST fits in i32.
    // Values above this cause the original bug.
    let json = format!(r#"{{
        "settings": {{
            "id": 227,
            "updatedAt": {},
            "displayType": 0,
            "defaultReaderMode": 0,
            "animeDisplayType": 0,
            "scaleType": 0,
            "backgroundColor": 0,
            "colorFilterBlendMode": 0,
            "mangaHomeDisplayType": 1,
            "disableSectionType": 0,
            "novelDisplayType": 1,
            "novelTextAlign": 0
        }}
    }}"#, i32::MAX as i64 + 1);

    let result: Result<SettingsObj, _> = serde_json::from_str(&json);
    assert!(result.is_ok(), "Timestamp just above i32::MAX must work: {:?}", result.err());
    assert_eq!(result.unwrap().settings.unwrap().updated_at, i32::MAX as i64 + 1);
}

// =============================================================================
// MANGA LIST DESERIALIZATION TESTS
// =============================================================================

#[test]
fn test_manga_list_empty_library_first_sync() {
    // First sync with completely empty library — all arrays empty.
    // This is what the Dart client sends when the user has no manga/anime/novels.
    let json = r#"{
        "categories": [],
        "deleted_categories": [],
        "manga": [],
        "deleted_manga": [],
        "chapters": [],
        "deleted_chapters": [],
        "tracks": [],
        "deleted_tracks": []
    }"#;

    let result: Result<MangaList, _> = serde_json::from_str(json);
    assert!(result.is_ok(), "Empty library payload must deserialize: {:?}", result.err());

    let list = result.unwrap();
    assert!(list.categories.is_empty());
    assert!(list.manga.is_empty());
    assert!(list.chapters.is_empty());
    assert!(list.tracks.is_empty());
    assert!(list.deleted_categories.is_empty());
    assert!(list.reset_all.is_none(), "resetAll should be None when not sent");
}

#[test]
fn test_manga_list_upload_with_reset_all() {
    // Upload-only sync adds resetAll: true. This triggers server to delete all
    // existing data before upserting.
    let json = r#"{
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

    let result: Result<MangaList, _> = serde_json::from_str(json);
    assert!(result.is_ok(), "Upload payload with resetAll must deserialize: {:?}", result.err());
    assert_eq!(result.unwrap().reset_all, Some(true));
}

#[test]
fn test_manga_single_item_with_i64_id_and_large_timestamp() {
    // A single manga with an Isar-generated ID that exceeds i32::MAX,
    // plus a 2026 timestamp. This is the EXACT scenario causing the 400 error.
    let json = r#"{
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

    let result: Result<MangaList, _> = serde_json::from_str(json);
    assert!(result.is_ok(), "Manga with i64 ID must deserialize: {:?}", result.err());

    let list = result.unwrap();
    assert_eq!(list.manga.len(), 1);
    assert_eq!(list.manga[0].id, 3_000_000_000_i64);
    assert_eq!(list.manga[0].updated_at, 1775156240310_i64);
    assert_eq!(list.manga[0].name, "One Piece");
}

#[test]
fn test_manga_with_all_optional_fields_null() {
    // Dart sends nullable fields for description, author, artist etc.
    // The server must handle nulls gracefully.
    let json = r#"{
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

    let result: Result<MangaList, _> = serde_json::from_str(json);
    assert!(result.is_ok(), "Manga with null optionals must deserialize: {:?}", result.err());

    let manga = &result.unwrap().manga[0];
    assert!(manga.description.is_none());
    assert!(manga.author.is_none());
    assert!(manga.artist.is_none());
    assert!(manga.source_id.is_none());
}

#[test]
fn test_chapter_with_nullable_booleans_and_i64_ids() {
    // Dart Chapter has bool? isBookmarked and bool? isRead.
    // The server must NOT reject these when null.
    // Also tests i64 IDs for both chapter and manga_id.
    let json = r#"{
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

    let result: Result<MangaList, _> = serde_json::from_str(json);
    assert!(result.is_ok(), "Chapter with null booleans must deserialize: {:?}", result.err());

    let chapter = &result.unwrap().chapters[0];
    assert_eq!(chapter.id, 5_000_000_000_i64);
    assert_eq!(chapter.manga_id, 3_000_000_000_i64);
    // null booleans should be None (not crash)
    assert!(chapter.is_bookmarked.is_none());
    assert!(chapter.is_read.is_none());
}

#[test]
fn test_chapter_with_explicit_boolean_values() {
    // Normal case: booleans are present and set.
    let json = r#"{
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

    let result: Result<MangaList, _> = serde_json::from_str(json);
    assert!(result.is_ok(), "Chapter with explicit booleans must deserialize: {:?}", result.err());

    let chapter = &result.unwrap().chapters[0];
    assert_eq!(chapter.is_bookmarked, Some(true));
    assert_eq!(chapter.is_read, Some(false));
}

#[test]
fn test_track_with_all_nullable_id_fields() {
    // Dart Track has int? for libraryId, mediaId, mangaId, syncId, and String? for title/trackingUrl.
    // A brand-new track might have some of these as null.
    let json = r#"{
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
            "status": 0,
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

    let result: Result<MangaList, _> = serde_json::from_str(json);
    assert!(result.is_ok(), "Track with null IDs must deserialize: {:?}", result.err());

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
    let json = r#"{
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
            "lastChapterRead": 50,
            "status": 0,
            "syncId": 4000000002,
            "title": "One Piece",
            "totalChapter": 1100,
            "trackingUrl": "https://myanimelist.net/manga/13",
            "isManga": true,
            "itemType": 0,
            "updatedAt": 1775156240310
        }],
        "deleted_tracks": []
    }"#;

    let result: Result<MangaList, _> = serde_json::from_str(json);
    assert!(result.is_ok(), "Track with complete i64 data must deserialize: {:?}", result.err());

    let track = &result.unwrap().tracks[0];
    assert_eq!(track.id, 4_000_000_000_i64);
    assert_eq!(track.media_id, Some(4_000_000_001_i64));
    assert_eq!(track.manga_id, Some(3_000_000_000_i64));
    assert_eq!(track.sync_id, Some(4_000_000_002_i64));
    assert_eq!(track.title.as_deref(), Some("One Piece"));
    assert_eq!(track.tracking_url.as_deref(), Some("https://myanimelist.net/manga/13"));
}

#[test]
fn test_category_with_i64_id() {
    let json = r#"{
        "categories": [{
            "id": 3000000000,
            "name": "Favorites",
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

    let result: Result<MangaList, _> = serde_json::from_str(json);
    assert!(result.is_ok(), "Category with i64 ID must deserialize: {:?}", result.err());
    assert_eq!(result.unwrap().categories[0].id, 3_000_000_000_i64);
}

#[test]
fn test_deleted_ids_as_i64() {
    // deleted_* arrays contain Isar IDs which are 64-bit on the Dart side.
    // A deletion list with IDs > i32::MAX must deserialize.
    let json = r#"{
        "categories": [],
        "deleted_categories": [3000000000, 3000000001],
        "manga": [],
        "deleted_manga": [4000000000],
        "chapters": [],
        "deleted_chapters": [5000000000, 5000000001, 5000000002],
        "tracks": [],
        "deleted_tracks": [6000000000]
    }"#;

    let result: Result<MangaList, _> = serde_json::from_str(json);
    assert!(result.is_ok(), "Deletion IDs as i64 must deserialize: {:?}", result.err());

    let list = result.unwrap();
    assert_eq!(list.deleted_categories, vec![3_000_000_000_i64, 3_000_000_001_i64]);
    assert_eq!(list.deleted_manga, vec![4_000_000_000_i64]);
    assert_eq!(list.deleted_chapters.len(), 3);
    assert_eq!(list.deleted_tracks, vec![6_000_000_000_i64]);
}

// =============================================================================
// HISTORY DESERIALIZATION TESTS
// =============================================================================

#[test]
fn test_history_list_empty_first_sync() {
    let json = r#"{
        "histories": [],
        "deleted_histories": [],
        "resetAll": null
    }"#;

    let result: Result<HistoryList, _> = serde_json::from_str(json);
    assert!(result.is_ok(), "Empty history list must deserialize: {:?}", result.err());
    assert!(result.unwrap().histories.is_empty());
}

#[test]
fn test_history_without_reset_all_key() {
    // The Dart client doesn't include resetAll in the JSON unless doing an upload.
    // The Rust model uses Option<bool> — missing key should be None.
    let json = r#"{
        "histories": [],
        "deleted_histories": []
    }"#;

    let result: Result<HistoryList, _> = serde_json::from_str(json);
    assert!(result.is_ok(), "History without resetAll key must deserialize: {:?}", result.err());
    assert!(result.unwrap().reset_all.is_none());
}

#[test]
fn test_history_with_i64_ids_and_large_timestamp() {
    let json = r#"{
        "histories": [{
            "id": 3000000000,
            "date": "2026-04-01",
            "mangaId": 4000000000,
            "chapterId": 5000000000,
            "itemType": 0,
            "updatedAt": 1775156240310,
            "readingTimeSeconds": 3600,
            "isManga": true
        }],
        "deleted_histories": [6000000000]
    }"#;

    let result: Result<HistoryList, _> = serde_json::from_str(json);
    assert!(result.is_ok(), "History with i64 IDs must deserialize: {:?}", result.err());

    let history = &result.unwrap().histories[0];
    assert_eq!(history.id, 3_000_000_000_i64);
    assert_eq!(history.manga_id, 4_000_000_000_i64);
    assert_eq!(history.chapter_id, 5_000_000_000_i64);
    assert_eq!(history.updated_at, 1775156240310_i64);
}

// =============================================================================
// UPDATE DESERIALIZATION TESTS
// =============================================================================

#[test]
fn test_update_list_empty_first_sync() {
    let json = r#"{
        "updates": [],
        "deleted_updates": []
    }"#;

    let result: Result<UpdateList, _> = serde_json::from_str(json);
    assert!(result.is_ok(), "Empty update list must deserialize: {:?}", result.err());
    assert!(result.unwrap().updates.is_empty());
}

#[test]
fn test_update_with_i64_ids() {
    let json = r#"{
        "updates": [{
            "id": 3000000000,
            "mangaId": 4000000000,
            "chapterName": "Chapter 100",
            "date": "2026-04-01",
            "updatedAt": 1775156240310
        }],
        "deleted_updates": [5000000000]
    }"#;

    let result: Result<UpdateList, _> = serde_json::from_str(json);
    assert!(result.is_ok(), "Update with i64 IDs must deserialize: {:?}", result.err());

    let update = &result.unwrap().updates[0];
    assert_eq!(update.id, 3_000_000_000_i64);
    assert_eq!(update.manga_id, 4_000_000_000_i64);
}

// =============================================================================
// COMPLEX MULTI-ITEM SCENARIO: typical real sync with mixed data
// =============================================================================

#[test]
fn test_full_manga_sync_payload_with_multiple_items() {
    // Simulates a real sync where user has 2 mangas, 3 chapters, 1 track,
    // 1 category, and some deleted items. All with i64 IDs and 2026 timestamps.
    let json = r#"{
        "categories": [
            {"id": 1, "name": "Reading", "forItemType": 0, "pos": 0, "hide": false, "shouldUpdate": true, "updatedAt": 1775156240310}
        ],
        "deleted_categories": [],
        "manga": [
            {"id": 100, "name": "Naruto", "link": "https://ex.com/naruto", "imageUrl": "https://ex.com/n.jpg", "description": "Ninja manga", "author": "Kishimoto", "artist": "Kishimoto", "status": 1, "favorite": true, "source": "MangaDex", "sourceId": null, "lang": "en", "itemType": 0, "genre": ["Action"], "categories": [1], "updatedAt": 1775156240310},
            {"id": 3000000000, "name": "Bleach", "link": "https://ex.com/bleach", "imageUrl": "https://ex.com/b.jpg", "description": null, "author": null, "artist": null, "status": 1, "favorite": false, "source": "MangaDex", "sourceId": 9876543210123, "lang": "en", "itemType": 0, "genre": null, "categories": null, "updatedAt": 1775156240310}
        ],
        "deleted_manga": [2000000000],
        "chapters": [
            {"id": 1000, "name": "Ch 1", "url": "", "dateUpload": "", "scanlator": "", "isBookmarked": false, "isRead": true, "lastPageRead": "20", "archivePath": null, "mangaId": 100, "updatedAt": 1775156240310},
            {"id": 1001, "name": "Ch 2", "url": "", "dateUpload": "", "scanlator": null, "isBookmarked": null, "isRead": null, "lastPageRead": null, "archivePath": null, "mangaId": 100, "updatedAt": 1775156240310},
            {"id": 5000000000, "name": "Ch 1", "url": "", "dateUpload": "", "scanlator": "", "isBookmarked": true, "isRead": false, "lastPageRead": "0", "archivePath": null, "mangaId": 3000000000, "updatedAt": 1775156240310}
        ],
        "deleted_chapters": [9000000000],
        "tracks": [
            {"id": 500, "libraryId": 1, "mediaId": 100, "mangaId": 100, "score": 90, "startedReadingDate": 1775156240310, "finishedReadingDate": null, "lastChapterRead": 2, "status": 0, "syncId": 1, "title": "Naruto", "totalChapter": 700, "trackingUrl": "https://mal.net/1", "isManga": true, "itemType": 0, "updatedAt": 1775156240310}
        ],
        "deleted_tracks": []
    }"#;

    let result: Result<MangaList, _> = serde_json::from_str(json);
    assert!(result.is_ok(), "Full multi-item sync payload must deserialize: {:?}", result.err());

    let list = result.unwrap();
    assert_eq!(list.categories.len(), 1);
    assert_eq!(list.manga.len(), 2);
    assert_eq!(list.chapters.len(), 3);
    assert_eq!(list.tracks.len(), 1);
    assert_eq!(list.deleted_manga, vec![2_000_000_000_i64]);
    assert_eq!(list.deleted_chapters, vec![9_000_000_000_i64]);

    // Verify the manga with i64 ID
    let bleach = &list.manga[1];
    assert_eq!(bleach.id, 3_000_000_000_i64);
    assert!(bleach.description.is_none());
    assert!(bleach.author.is_none());

    // Verify chapter with null booleans
    let ch2 = &list.chapters[1];
    assert!(ch2.is_bookmarked.is_none());
    assert!(ch2.is_read.is_none());

    // Verify chapter with i64 mangaId
    let ch_bleach = &list.chapters[2];
    assert_eq!(ch_bleach.manga_id, 3_000_000_000_i64);
}

// =============================================================================
// SERIALIZATION ROUND-TRIP: verify server response can be parsed by client
// =============================================================================

#[test]
fn test_settings_obj_none_serializes_to_null() {
    // When the server has no settings for a user, it returns {"settings": null}.
    // The Dart client receives this JSON. Verify it serializes correctly.
    let obj = SettingsObj { settings: None };
    let json = serde_json::to_string(&obj).unwrap();

    // The JSON must contain "settings":null
    assert!(json.contains(r#""settings":null"#), "SettingsObj with None must serialize settings as null, got: {}", json);
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
