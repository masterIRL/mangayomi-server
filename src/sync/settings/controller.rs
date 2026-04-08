use crate::sync::handler::{handle_sync, SyncEndpointConfig};
use crate::sync::settings::service::sync_settings;
use actix_identity::Identity;
use actix_web::{post, web, Responder};
use mongodb::Client;

/// Sync settings endpoint
/// 
/// Handles synchronization of user settings.
#[post("/settings")]
pub async fn sync_settings_obj(
    client: web::Data<Client>,
    user: Identity,
    body: web::Bytes,
) -> impl Responder {
    handle_sync(
        SyncEndpointConfig::new("settings", "/settings"),
        client,
        user,
        body,
        |user_id, cli, payload| async move {
            sync_settings(user_id, &payload, cli).await
        },
    )
    .await
}
