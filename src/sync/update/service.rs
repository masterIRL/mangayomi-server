use crate::config::collections;
use crate::sync::error::SyncError;
use crate::sync::update::model::UpdateList;
use crate::sync::service::sync_entity_list;
use actix_web::web;
use mongodb::bson::oid::ObjectId;
use mongodb::Client;

pub async fn sync_update_list(
    user_id: ObjectId,
    update_list: &UpdateList,
    db: web::Data<Client>,
) -> Result<UpdateList, SyncError> {
    sync_entity_list(user_id, update_list, &db, collections::UPDATES).await
}
