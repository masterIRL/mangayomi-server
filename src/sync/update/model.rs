use crate::config::collections;
use crate::sync::extractor::ValidatePayload;
use crate::sync::model::{Model, Syncable};
use crate::sync::validation;
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct Update {
    #[serde(rename = "_id", skip_serializing)]
    pub oid: Option<ObjectId>,
    pub id: i64,
    #[serde(rename = "mangaId")]
    pub manga_id: i64,
    #[serde(rename = "chapterName")]
    pub chapter_name: String,
    pub date: String,
    #[serde(skip_serializing)]
    pub user: Option<ObjectId>,
    #[serde(rename = "updatedAt")]
    pub updated_at: i64,
}

impl Model for Update {
    fn get_id(&self) -> i64 {
        self.id
    }
    fn get_updated_at(&self) -> i64 {
        self.updated_at
    }
}

impl Syncable for Update {
    const COLLECTION_NAME: &'static str = collections::UPDATES;
}

#[derive(Serialize, Deserialize, Default, Clone)]
pub struct UpdateList {
    pub updates: Vec<Update>,
    pub deleted_updates: Vec<i64>,
    #[serde(rename = "resetAll")]
    pub reset_all: Option<bool>,
}

impl ValidatePayload for UpdateList {
    fn item_count(&self) -> usize {
        self.updates.len()
    }

    fn validate(&self) -> Result<(), String> {
        validation::validate_deleted_count(self.deleted_updates.len(), "update")?;
        validation::validate_ids(&self.updates, "update", |u| u.id)?;
        validation::validate_deleted_ids(&self.deleted_updates, "update")?;
        Ok(())
    }
}

impl crate::sync::handler::SyncResult for UpdateList {
    fn modified_count(&self) -> usize {
        self.updates.len()
    }
}

impl crate::sync::service::SyncList for UpdateList {
    type Item = Update;

    fn should_reset_all(&self) -> bool {
        self.reset_all.unwrap_or(false)
    }

    fn items(&self) -> &[Self::Item] {
        &self.updates
    }

    fn deleted_ids(&self) -> &[i64] {
        &self.deleted_updates
    }

    fn into_result(self, items: Vec<Self::Item>) -> Self {
        UpdateList {
            updates: items,
            deleted_updates: vec![],
            reset_all: self.reset_all,
        }
    }
}
