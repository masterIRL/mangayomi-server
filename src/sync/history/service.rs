use crate::config::collections;
use crate::sync::error::SyncError;
use crate::sync::history::model::HistoryList;
use crate::sync::service::sync_entity_list;
use actix_web::web;
use mongodb::bson::oid::ObjectId;
use mongodb::Client;

pub async fn sync_history_list(
    user_id: ObjectId,
    history_list: &HistoryList,
    db: web::Data<Client>,
) -> Result<HistoryList, SyncError> {
    sync_entity_list(user_id, history_list, &db, collections::HISTORIES).await
}
