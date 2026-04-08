use crate::config::collections;
use crate::sync::extractor::ValidatePayload;
use crate::sync::model::{Model, Syncable};
use crate::sync::validation;
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
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

impl Model for History {
    fn get_id(&self) -> i64 {
        self.id
    }
    fn get_updated_at(&self) -> i64 {
        self.updated_at
    }
}

impl Syncable for History {
    const COLLECTION_NAME: &'static str = collections::HISTORIES;
}

#[derive(Serialize, Deserialize, Default, Clone)]
pub struct HistoryList {
    pub histories: Vec<History>,
    pub deleted_histories: Vec<i64>,
    #[serde(rename = "resetAll")]
    pub reset_all: Option<bool>,
}

impl ValidatePayload for HistoryList {
    fn item_count(&self) -> usize {
        self.histories.len()
    }

    fn validate(&self) -> Result<(), String> {
        validation::validate_deleted_count(self.deleted_histories.len(), "history")?;
        validation::validate_ids(&self.histories, "history", |h| h.id)?;
        validation::validate_deleted_ids(&self.deleted_histories, "history")?;
        Ok(())
    }
}

impl crate::sync::handler::SyncResult for HistoryList {
    fn modified_count(&self) -> usize {
        self.histories.len()
    }
}

impl crate::sync::service::SyncList for HistoryList {
    type Item = History;

    fn should_reset_all(&self) -> bool {
        self.reset_all.unwrap_or(false)
    }

    fn items(&self) -> &[Self::Item] {
        &self.histories
    }

    fn deleted_ids(&self) -> &[i64] {
        &self.deleted_histories
    }

    fn into_result(self, items: Vec<Self::Item>) -> Self {
        HistoryList {
            histories: items,
            deleted_histories: vec![],
            reset_all: self.reset_all,
        }
    }
}
