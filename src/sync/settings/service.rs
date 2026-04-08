use crate::config::collections;
use crate::db::utils::{find_all_by_user, get_database};
use crate::sync::error::SyncError;
use crate::sync::settings::model::{Settings, SettingsObj};
use crate::sync::service;
use actix_web::web;
use mongodb::bson::oid::ObjectId;
use mongodb::{Client, Collection};

pub async fn sync_settings(
    user_id: ObjectId,
    settings_obj: &SettingsObj,
    db: web::Data<Client>,
) -> Result<SettingsObj, SyncError> {
    let database = get_database(&db);
    let col: Collection<Settings> = database.collection(collections::SETTINGS);

    if let Some(ref settings) = settings_obj.settings {
        service::upsert_batch(&col, user_id, &[settings.clone()], collections::SETTINGS).await?;
    }

    let settings_list: Vec<Settings> = find_all_by_user(&col, user_id)
        .await
        .map_err(|e| SyncError::DatabaseError(e.to_string()))?;
    let settings = settings_list.into_iter().next();

    Ok(SettingsObj { settings })
}
