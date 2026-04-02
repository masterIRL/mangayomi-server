use crate::sync::history::model::{History, HistoryList};
use actix_web::web;
use futures::TryStreamExt;
use mongodb::bson::oid::ObjectId;
use mongodb::bson::{doc, to_document};
use mongodb::options::{UpdateOneModel, WriteModel};
use mongodb::{Client, Collection, Namespace};
use serde::de::DeserializeOwned;

pub async fn sync_history_list(
    user_id: ObjectId,
    history_list: &web::Json<HistoryList>,
    db: web::Data<Client>,
) -> HistoryList {
    let col_histories = db.database("mangayomi").collection("histories");
    let reset_all = history_list.reset_all.unwrap_or(false);

    if reset_all {
        delete_many(&col_histories, user_id, &[0], true).await;
    }

    upsert(
        &db,
        col_histories.namespace(),
        user_id,
        &history_list.histories,
    )
    .await;

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

async fn upsert(
    db: &web::Data<Client>,
    namespace: Namespace,
    user_id: ObjectId,
    histories: &Vec<History>,
) {
    let mut ops = vec![];
    for history in histories {
        let mut doc = match to_document(&history) {
            Ok(doc) => doc,
            Err(err) => {
                log::error!("Failed to serialize history to BSON document: {}", err);
                continue;
            }
        };
        doc.insert("user", user_id);
        ops.push(WriteModel::UpdateOne(
            UpdateOneModel::builder()
                .namespace(namespace.to_owned())
                .filter(doc! {
                    "id": history.id,
                    "user": user_id,
                    "updatedAt": { "$lt": history.updated_at },
                })
                .update(doc! {
                    "$set": doc
                })
                .upsert(true)
                .build(),
        ));
    }
    if !ops.is_empty() {
        match db.bulk_write(ops).ordered(false).await {
            Ok(result) => log::info!("Upserted {} histories.", result.modified_count),
            Err(err) => log::error!("Failed to upsert histories: {}", err),
        }
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
