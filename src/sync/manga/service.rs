use crate::sync::manga::model::{MangaList, Model};
use actix_web::web;
use futures::TryStreamExt;
use mongodb::bson::oid::ObjectId;
use mongodb::bson::{doc, to_document};
use mongodb::options::{UpdateOneModel, WriteModel};
use mongodb::{Client, Collection, Namespace};
use serde::de::DeserializeOwned;

pub async fn sync_manga_list(
    user_id: ObjectId,
    manga_list: &web::Json<MangaList>,
    db: web::Data<Client>,
) -> MangaList {
    let col_categories = db.database("mangayomi").collection("categories");
    let col_manga = db.database("mangayomi").collection("manga");
    let col_chapter = db.database("mangayomi").collection("chapters");
    let col_track = db.database("mangayomi").collection("tracks");
    let reset_all = manga_list.reset_all.unwrap_or(false);

    if reset_all {
        delete_many(&col_categories, user_id, &[0], true).await;
        delete_many(&col_manga, user_id, &[0], true).await;
        delete_many(&col_chapter, user_id, &[0], true).await;
        delete_many(&col_track, user_id, &[0], true).await;
    }

    upsert(
        &db,
        col_categories.namespace(),
        user_id,
        &manga_list.categories,
    )
    .await;
    upsert(&db, col_manga.namespace(), user_id, &manga_list.manga).await;
    upsert(&db, col_chapter.namespace(), user_id, &manga_list.chapters).await;
    upsert(&db, col_track.namespace(), user_id, &manga_list.tracks).await;

    if !reset_all {
        delete_many(
            &col_categories,
            user_id,
            &manga_list.deleted_categories,
            false,
        )
        .await;
        delete_many(&col_manga, user_id, &manga_list.deleted_manga, false).await;
        delete_many(&col_chapter, user_id, &manga_list.deleted_chapters, false).await;
        delete_many(&col_track, user_id, &manga_list.deleted_tracks, false).await;
    }

    MangaList {
        categories: find_all(&col_categories, user_id).await,
        manga: find_all(&col_manga, user_id).await,
        chapters: find_all(&col_chapter, user_id).await,
        tracks: find_all(&col_track, user_id).await,
        deleted_categories: vec![],
        deleted_manga: vec![],
        deleted_chapters: vec![],
        deleted_tracks: vec![],
        reset_all: manga_list.reset_all,
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

async fn upsert<T: Send + Sync + serde::Serialize + Model>(
    db: &web::Data<Client>,
    namespace: Namespace,
    user_id: ObjectId,
    items: &Vec<T>,
) {
    let mut ops = vec![];
    for item in items {
        let mut doc = match to_document(&item) {
            Ok(doc) => doc,
            Err(err) => {
                log::error!("Failed to serialize item to BSON document: {}", err);
                continue;
            }
        };
        doc.insert("user", user_id);
        ops.push(WriteModel::UpdateOne(
            UpdateOneModel::builder()
                .namespace(namespace.to_owned())
                .filter(doc! {
                    "id": item.get_id(),
                    "user": user_id,
                    "updatedAt": { "$lt": item.get_updated_at() },
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
            Ok(result) => log::info!("Upserted {} {}.", result.modified_count, namespace.coll),
            Err(err) => log::error!("Failed to upsert {}: {}", namespace.coll, err),
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
