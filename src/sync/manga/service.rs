use crate::sync::manga::model::{MangaList, Model};
use crate::db::bulk_upsert::get_global_upserter;
use actix_web::web;
use futures::TryStreamExt;
use mongodb::bson::oid::ObjectId;
use mongodb::bson::doc;
use mongodb::{Client, Collection};
use serde::de::DeserializeOwned;

pub async fn sync_manga_list(
    user_id: ObjectId,
    manga_list: &MangaList,
    db: web::Data<Client>,
) -> MangaList {
    let database = db.database("mangayomi");
    let col_categories: Collection<crate::sync::manga::model::Category> = database.collection("categories");
    let col_manga: Collection<crate::sync::manga::model::Manga> = database.collection("manga");
    let col_chapter: Collection<crate::sync::manga::model::Chapter> = database.collection("chapters");
    let col_track: Collection<crate::sync::manga::model::Track> = database.collection("tracks");
    let reset_all = manga_list.reset_all.unwrap_or(false);

    if reset_all {
        delete_many(&col_categories, user_id, &[0], true).await;
        delete_many(&col_manga, user_id, &[0], true).await;
        delete_many(&col_chapter, user_id, &[0], true).await;
        delete_many(&col_track, user_id, &[0], true).await;
    }

    // Use bulk upserter (auto-detects MongoDB version)
    if let Some(upserter) = get_global_upserter() {
        upserter.upsert_batch(&col_categories, &manga_list.categories, user_id, |item: &crate::sync::manga::model::Category| {
            (item.get_id(), item.get_updated_at())
        }).await.ok();
        
        upserter.upsert_batch(&col_manga, &manga_list.manga, user_id, |item: &crate::sync::manga::model::Manga| {
            (item.get_id(), item.get_updated_at())
        }).await.ok();
        
        upserter.upsert_batch(&col_chapter, &manga_list.chapters, user_id, |item: &crate::sync::manga::model::Chapter| {
            (item.get_id(), item.get_updated_at())
        }).await.ok();
        
        upserter.upsert_batch(&col_track, &manga_list.tracks, user_id, |item: &crate::sync::manga::model::Track| {
            (item.get_id(), item.get_updated_at())
        }).await.ok();
    } else {
        log::error!("BulkUpserter not initialized!");
    }

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