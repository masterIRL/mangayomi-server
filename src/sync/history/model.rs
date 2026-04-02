use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct History {
    #[serde(rename = "_id", skip_serializing)]
    pub oid: Option<ObjectId>,
    pub id: i64,
    pub date: String,
    #[serde(rename = "mangaId")]
    pub manga_id: i64,
    #[serde(rename = "chapterId")]
    pub chapter_id: i64,
    #[serde(rename = "itemType")]
    pub item_type: i64,
    #[serde(skip_serializing)]
    pub user: Option<ObjectId>,
    #[serde(rename = "updatedAt")]
    pub updated_at: i64,
}

#[derive(Serialize, Deserialize)]
pub struct HistoryList {
    pub histories: Vec<History>,
    pub deleted_histories: Vec<i64>,
    #[serde(rename = "resetAll")]
    pub reset_all: Option<bool>,
}
