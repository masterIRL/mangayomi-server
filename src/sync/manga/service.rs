use crate::config::collections;
use crate::db::utils::{find_all_by_user, get_database};
use crate::sync::error::SyncError;
use crate::sync::manga::model::{Category, Chapter, Manga, MangaList, Track};
use crate::sync::service;
use actix_web::web;
use mongodb::bson::oid::ObjectId;
use mongodb::{Client, Collection};

pub async fn sync_manga_list(
    user_id: ObjectId,
    manga_list: &MangaList,
    db: web::Data<Client>,
) -> Result<MangaList, SyncError> {
    let database = get_database(&db);
    let col_categories: Collection<Category> = database.collection(collections::CATEGORIES);
    let col_manga: Collection<Manga> = database.collection(collections::MANGA);
    let col_chapter: Collection<Chapter> = database.collection(collections::CHAPTERS);
    let col_track: Collection<Track> = database.collection(collections::TRACKS);
    let reset_all = manga_list.reset_all.unwrap_or(false);

    if reset_all {
        service::reset_all_entities(&col_categories, user_id, collections::CATEGORIES).await?;
        service::reset_all_entities(&col_manga, user_id, collections::MANGA).await?;
        service::reset_all_entities(&col_chapter, user_id, collections::CHAPTERS).await?;
        service::reset_all_entities(&col_track, user_id, collections::TRACKS).await?;
    }

    service::upsert_batch(&col_categories, user_id, &manga_list.categories, collections::CATEGORIES).await?;
    service::upsert_batch(&col_manga, user_id, &manga_list.manga, collections::MANGA).await?;
    service::upsert_batch(&col_chapter, user_id, &manga_list.chapters, collections::CHAPTERS).await?;
    service::upsert_batch(&col_track, user_id, &manga_list.tracks, collections::TRACKS).await?;

    if !reset_all {
        service::delete_by_ids(&col_categories, user_id, &manga_list.deleted_categories, collections::CATEGORIES).await?;
        service::delete_by_ids(&col_manga, user_id, &manga_list.deleted_manga, collections::MANGA).await?;
        service::delete_by_ids(&col_chapter, user_id, &manga_list.deleted_chapters, collections::CHAPTERS).await?;
        service::delete_by_ids(&col_track, user_id, &manga_list.deleted_tracks, collections::TRACKS).await?;
    }

    // Parallel fetch for better performance
    let (categories_result, manga_result, chapters_result, tracks_result) = tokio::join!(
        find_all_by_user(&col_categories, user_id),
        find_all_by_user(&col_manga, user_id),
        find_all_by_user(&col_chapter, user_id),
        find_all_by_user(&col_track, user_id)
    );

    // Properly handle errors from find_all_by_user
    let categories = categories_result.map_err(|e| SyncError::DatabaseError(e.to_string()))?;
    let manga = manga_result.map_err(|e| SyncError::DatabaseError(e.to_string()))?;
    let chapters = chapters_result.map_err(|e| SyncError::DatabaseError(e.to_string()))?;
    let tracks = tracks_result.map_err(|e| SyncError::DatabaseError(e.to_string()))?;

    Ok(MangaList {
        categories,
        manga,
        chapters,
        tracks,
        deleted_categories: vec![],
        deleted_manga: vec![],
        deleted_chapters: vec![],
        deleted_tracks: vec![],
        reset_all: manga_list.reset_all,
    })
}
