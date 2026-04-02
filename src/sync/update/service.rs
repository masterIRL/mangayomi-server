use crate::sync::update::model::{Update, UpdateList};
use crate::db::bulk_upsert::{get_global_upserter};
use actix_web::web;
use futures::TryStreamExt;
use mongodb::bson::oid::ObjectId;
use mongodb::bson::doc;
use mongodb::{Client, Collection};
use serde::de::DeserializeOwned;

pub async fn sync_update_list(
    user_id: ObjectId,
    update_list: &UpdateList,
    db: web::Data<Client>,
) -> UpdateList {
    let database = db.database("mangayomi");
    let col_updates: Collection<Update> = database.collection("updates");
    let reset_all = update_list.reset_all.unwrap_or(false);

    if reset_all {
        delete_many(&col_updates, user_id, &[0], true).await;
    }

    // Use bulk upserter (auto-detects MongoDB version)
    if let Some(upserter) = get_global_upserter() {
        match upserter.upsert_batch(
            &col_updates,
            &update_list.updates,
            user_id,
            |u: &Update| (u.id, u.updated_at)
        ).await {
            Ok(result) => {
                if !result.failed_items.is_empty() {
                    log::error!("Update upsert had {} failures", result.failed_items.len());
                }
            }
            Err(e) => log::error!("Failed to upsert updates: {}", e),
        }
    } else {
        log::error!("BulkUpserter not initialized!");
    }

    if !reset_all {
        delete_many(
            &col_updates,
            user_id,
            &update_list.deleted_updates,
            false,
        )
        .await;
    }

    UpdateList {
        updates: find_all(&col_updates, user_id).await,
        deleted_updates: vec![],
        reset_all: update_list.reset_all,
    }
}

async fn delete_many<T: Send + Sync>(
    collection: &Collection<T>,
    user_id: ObjectId,
    ids: &[i64],
    reset_all: bool,
) {
    if ids.is_empty() {
        return;
    }
    let del_result = collection
        .delete_many(if reset_all {
            doc! {
                "user": user_id,
            }
        } else {
            doc! {
                "id": doc! {
                    "$in": ids
                },
                "user": user_id,
            }
        })
        .await;
    match del_result {
        Ok(result) => log::info!("Deleted {} {}.", result.deleted_count, collection.name()),
        Err(err) => log::error!("Failed to delete {}: {}", collection.name(), err),
    }
}

async fn find_all<T: DeserializeOwned + Unpin + Send + Sync>(
    collection: &Collection<T>,
    user_id: ObjectId,
) -> Vec<T> {
    match collection.find(doc! { "user": user_id }).await {
        Ok(result) => result.try_collect().await.unwrap_or_else(|err| {
            log::error!("Failed to collect results from {}: {}", collection.name(), err);
            vec![]
        }),
        Err(err) => {
            log::error!("Failed to query {}: {}", collection.name(), err);
            vec![]
        }
    }
}