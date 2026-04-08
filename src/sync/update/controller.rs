use crate::sync::handler::{handle_sync, SyncEndpointConfig};
use crate::sync::update::service::sync_update_list;
use actix_identity::Identity;
use actix_web::{post, web, Responder};
use mongodb::Client;

/// Sync updates endpoint
/// 
/// Handles synchronization of library update entries.
#[post("/updates")]
pub async fn sync_updates(
    client: web::Data<Client>,
    user: Identity,
    body: web::Bytes,
) -> impl Responder {
    handle_sync(
        SyncEndpointConfig::new("update", "/updates"),
        client,
        user,
        body,
        |user_id, cli, payload| async move {
            sync_update_list(user_id, &payload, cli).await
        },
    )
    .await
}
