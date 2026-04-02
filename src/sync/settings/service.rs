use crate::sync::settings::model::{Settings, SettingsObj};
use crate::db::bulk_upsert::{get_global_upserter, UpsertBatchResult};
use actix_web::web;
use mongodb::bson::oid::ObjectId;
use mongodb::bson::{doc, to_document};
use mongodb::{Client, Collection};
use serde::de::DeserializeOwned;

pub async fn sync_settings(
    user_id: ObjectId,
    settings: &SettingsObj,
    db: web::Data<Client>,
) -> Option<SettingsObj> {
    let database = db.database("mangayomi");
    let col_settings: Collection<Settings> = database.collection("settings");

    if let Some(ref settings_data) = settings.settings {
        // Use the global upserter (auto-detects MongoDB version)
        if let Some(upserter) = get_global_upserter() {
            let result = upserter.upsert_batch(
                &col_settings,
                &[settings_data.clone()],
                user_id,
                |s: &Settings| (s.id, s.updated_at)
            ).await;
            
            match result {
                Ok(UpsertBatchResult { modified_count, failed_items, .. }) => {
                    if !failed_items.is_empty() {
                        log::error!("Settings upsert had {} failures", failed_items.len());
                    } else {
                        log::info!("Upserted {} settings", modified_count);
                    }
                }
                Err(e) => {
                    log::error!("Failed to upsert settings: {}", e);
                }
            }
        } else {
            log::error!("BulkUpserter not initialized! Falling back to direct update.");
            // Fallback: direct update_one
            fallback_upsert_settings(&col_settings, user_id, settings_data).await;
        }
    }

    match find_one(&col_settings, user_id).await {
        Some(obj) => Some(SettingsObj { settings: Some(obj) }),
        None => None,
    }
}

async fn fallback_upsert_settings(
    collection: &Collection<Settings>,
    user_id: ObjectId,
    settings: &Settings,
) {
    let mut doc = match to_document(&settings) {
        Ok(doc) => doc,
        Err(err) => {
            log::error!("Failed to serialize settings: {}", err);
            return;
        }
    };
    doc.insert("user", user_id);
    
    let filter = doc! {
        "id": settings.id,
        "user": user_id,
        "updatedAt": { "$lt": settings.updated_at }
    };
    
    match collection.update_one(filter, doc! { "$set": doc }).upsert(true).await {
        Ok(_) => log::info!("Settings upserted (fallback)"),
        Err(e) => log::error!("Fallback upsert failed: {}", e),
    }
}

async fn find_one<T: DeserializeOwned + Unpin + Send + Sync>(
    collection: &Collection<T>,
    user_id: ObjectId,
) -> Option<T> {
    match collection.find_one(doc! { "user": user_id }).await {
        Ok(result) => result,
        Err(err) => {
            log::error!("Failed to find document: {}", err);
            None
        }
    }
}