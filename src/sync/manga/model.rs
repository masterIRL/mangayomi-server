use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

pub trait Model {
    fn get_id(&self) -> i64;
    fn get_updated_at(&self) -> i64;
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
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

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
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

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
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

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
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

#[derive(Serialize, Deserialize)]
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
