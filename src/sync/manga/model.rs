use crate::config::collections;
use crate::sync::extractor::ValidatePayload;
use crate::sync::model::{Model, Syncable};
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct Category {
    #[serde(rename = "_id", skip_serializing)]
    pub oid: Option<ObjectId>,
    pub id: i64,
    pub name: String,
    #[serde(rename = "forItemType")]
    pub for_item_type: i64,
    pub pos: Option<i64>,
    pub hide: Option<bool>,
    #[serde(rename = "shouldUpdate")]
    pub should_update: Option<bool>,
    #[serde(skip_serializing)]
    pub user: Option<ObjectId>,
    #[serde(rename = "updatedAt")]
    pub updated_at: i64,
}

impl Model for Category {
    fn get_id(&self) -> i64 {
        self.id
    }
    fn get_updated_at(&self) -> i64 {
        self.updated_at
    }
}

impl Syncable for Category {
    const COLLECTION_NAME: &'static str = collections::CATEGORIES;
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct Manga {
    #[serde(rename = "_id", skip_serializing)]
    pub oid: Option<ObjectId>,
    pub id: i64,
    pub name: String,
    pub link: String,
    #[serde(rename = "imageUrl")]
    pub image_url: String,
    pub description: Option<String>,
    pub author: Option<String>,
    pub artist: Option<String>,
    pub status: i64,
    pub favorite: bool,
    pub source: String,
    #[serde(rename = "sourceId")]
    pub source_id: Option<i64>,
    pub lang: String,
    #[serde(rename = "dateAdded")]
    pub date_added: Option<i64>,
    #[serde(rename = "lastUpdate")]
    pub last_update: Option<i64>,
    #[serde(rename = "lastRead")]
    pub last_read: Option<i64>,
    #[serde(rename = "isLocalArchive")]
    pub is_local_archive: Option<bool>,
    #[serde(rename = "customCoverFromTracker")]
    pub custom_cover_from_tracker: Option<String>,
    #[serde(rename = "itemType")]
    pub item_type: i64,
    pub genre: Option<Vec<String>>,
    pub categories: Option<Vec<i64>>,
    #[serde(skip_serializing)]
    pub user: Option<ObjectId>,
    #[serde(rename = "updatedAt")]
    pub updated_at: i64,
}

impl Model for Manga {
    fn get_id(&self) -> i64 {
        self.id
    }
    fn get_updated_at(&self) -> i64 {
        self.updated_at
    }
}

impl Syncable for Manga {
    const COLLECTION_NAME: &'static str = collections::MANGA;
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct Chapter {
    #[serde(rename = "_id", skip_serializing)]
    pub oid: Option<ObjectId>,
    pub id: i64,
    pub name: String,
    pub url: Option<String>,
    #[serde(rename = "dateUpload")]
    pub date_upload: Option<String>,
    pub scanlator: Option<String>,
    #[serde(rename = "isBookmarked")]
    pub is_bookmarked: Option<bool>,
    #[serde(rename = "isRead")]
    pub is_read: Option<bool>,
    #[serde(rename = "lastPageRead")]
    pub last_page_read: Option<String>,
    #[serde(rename = "archivePath")]
    pub archive_path: Option<String>,
    #[serde(rename = "mangaId")]
    pub manga_id: i64,
    #[serde(skip_serializing)]
    pub user: Option<ObjectId>,
    #[serde(rename = "updatedAt")]
    pub updated_at: i64,
}

impl Model for Chapter {
    fn get_id(&self) -> i64 {
        self.id
    }
    fn get_updated_at(&self) -> i64 {
        self.updated_at
    }
}

impl Syncable for Chapter {
    const COLLECTION_NAME: &'static str = collections::CHAPTERS;
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct Track {
    #[serde(rename = "_id", skip_serializing)]
    pub oid: Option<ObjectId>,
    pub id: i64,
    #[serde(rename = "libraryId")]
    pub library_id: Option<i64>,
    #[serde(rename = "mediaId")]
    pub media_id: Option<i64>,
    #[serde(rename = "mangaId")]
    pub manga_id: Option<i64>,
    pub score: Option<i64>,
    #[serde(rename = "startedReadingDate")]
    pub started_reading_date: Option<i64>,
    #[serde(rename = "finishedReadingDate")]
    pub finished_reading_date: Option<i64>,
    #[serde(rename = "lastChapterRead")]
    pub last_chapter_read: Option<i64>,
    pub status: Option<i64>,
    #[serde(rename = "syncId")]
    pub sync_id: Option<i64>,
    pub title: Option<String>,
    #[serde(rename = "totalChapter")]
    pub total_chapter: Option<i64>,
    #[serde(rename = "trackingUrl")]
    pub tracking_url: Option<String>,
    #[serde(rename = "isManga")]
    pub is_manga: Option<bool>,
    #[serde(rename = "itemType")]
    pub item_type: i64,
    #[serde(skip_serializing)]
    pub user: Option<ObjectId>,
    #[serde(rename = "updatedAt")]
    pub updated_at: i64,
}

impl Model for Track {
    fn get_id(&self) -> i64 {
        self.id
    }
    fn get_updated_at(&self) -> i64 {
        self.updated_at
    }
}

impl Syncable for Track {
    const COLLECTION_NAME: &'static str = collections::TRACKS;
}

/// Manga sync payload containing heterogeneous entity types
///
/// # ARCHITECTURAL NOTE: Why MangaList does NOT implement `SyncList`
///
/// Unlike `HistoryList` or `UpdateList`, this struct contains **FOUR different
/// entity types** (Category, Manga, Chapter, Track) that are stored in **FOUR
/// separate MongoDB collections**.
///
/// The `SyncList` trait (defined in `crate::sync::service`) is designed for
/// **homogeneous** entity lists only - it has a single associated type `Item`.
/// This makes it fundamentally incompatible with `MangaList`.
///
/// ## Attempting to implement `SyncList` for `MangaList` would require:
///
/// 1. **A unified enum type** (enum MangaListItem { Category(Category), Manga(Manga), ... })
///    This would add complexity, require runtime dispatch, and lose type safety.
///
/// 2. **Multiple associated types in the trait** (type Item1; type Item2; type Item3; type Item4;)
///    This would over-engineer the trait for a single use case.
///
/// 3. **Dynamic dispatch with `Box<dyn Any>`**
///    Loses zero-cost abstraction and compile-time type safety.
///
/// ## Current approach (INTENTIONAL AND CORRECT):
///
/// `MangaList` uses its own specialized service function (`sync_manga_list`)
/// that uses `tokio::join!` for parallel fetching across all 4 collections.
/// This is more performant than the generic `sync_entity_list` could ever be
/// for this specific use case.
///
/// ## Summary:
/// - `HistoryList` → 1 entity, 1 collection → implements `SyncList` ✅
/// - `UpdateList` → 1 entity, 1 collection → implements `SyncList` ✅
/// - `MangaList` → 4 entities, 4 collections → CANNOT implement `SyncList` ❌ (by design)
///
/// DO NOT attempt to "unify" this with the `SyncList` pattern. The current
/// architecture is intentional, correct, and optimized for the domain.
#[derive(Serialize, Deserialize, Default)]
pub struct MangaList {
    pub categories: Vec<Category>,
    pub deleted_categories: Vec<i64>,
    pub manga: Vec<Manga>,
    pub deleted_manga: Vec<i64>,
    pub chapters: Vec<Chapter>,
    pub deleted_chapters: Vec<i64>,
    pub tracks: Vec<Track>,
    pub deleted_tracks: Vec<i64>,
    #[serde(rename = "resetAll")]
    pub reset_all: Option<bool>,
}

impl ValidatePayload for MangaList {
    fn item_count(&self) -> usize {
        self.categories.len() + self.manga.len() + self.chapters.len() + self.tracks.len()
    }

    fn validate(&self) -> Result<(), String> {
        use crate::sync::validation;

        // Validate deleted item counts AND IDs (for consistency with History/Update)
        validation::validate_deleted_ids(&self.deleted_categories, "category")?;
        validation::validate_deleted_ids(&self.deleted_manga, "manga")?;
        validation::validate_deleted_ids(&self.deleted_chapters, "chapter")?;
        validation::validate_deleted_ids(&self.deleted_tracks, "track")?;

        // Validate item IDs
        validation::validate_ids(&self.categories, "category", |c| c.id)?;
        validation::validate_ids(&self.manga, "manga", |m| m.id)?;
        validation::validate_ids(&self.chapters, "chapter", |c| c.id)?;
        validation::validate_ids(&self.tracks, "track", |t| t.id)?;

        Ok(())
    }
}

impl crate::sync::handler::SyncResult for MangaList {
    fn modified_count(&self) -> usize {
        self.categories.len() + self.manga.len() + self.chapters.len() + self.tracks.len()
    }
}
