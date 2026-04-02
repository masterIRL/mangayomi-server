use crate::sync::history::model::{History, HistoryList};
use crate::db::bulk_upsert::{get_global_upserter};
use actix_web::web;
use futures::TryStreamExt;
use mongodb::bson::oid::ObjectId;
use mongodb::bson::doc;
use mongodb::{Client, Collection};
use serde::de::DeserializeOwned;

pub async fn sync_history_list(
    user_id: ObjectId,
    history_list: &HistoryList,
    db: web::Data<Client>,
) -> HistoryList {
    let database = db.database("mangayomi");
    let col_histories: Collection<History> = database.collection("histories");
    let reset_all = history_list.reset_all.unwrap_or(false);

    if reset_all {
        delete_many(&col_histories, user_id, &[0], true).await;
    }

    // Use bulk upserter (auto-detects MongoDB version)
    if let Some(upserter) = get_global_upserter() {
        match upserter.upsert_batch(
            &col_histories,
            &history_list.histories,
            user_id,
            |h: &History| (h.id, h.updated_at)
        ).await {
            Ok(result) => {
                if !result.failed_items.is_empty() {
                    log::error!("History upsert had {} failures", result.failed_items.len());
                }
            }
            Err(e) => log::error!("Failed to upsert histories: {}", e),
        }
    } else {
        log::error!("BulkUpserter not initialized!");
    }

    if !reset_all {
        delete_many(
            &col_histories,
            user_id,
            &history_list.deleted_histories,
            false,
        )
        .await;
    }

    HistoryList {
        histories: find_all(&col_histories, user_id).await,
        deleted_histories: vec![],
        reset_all: history_list.reset_all,
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