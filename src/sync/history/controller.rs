use crate::sync::handler::{handle_sync, SyncEndpointConfig};
use crate::sync::history::service::sync_history_list;
use actix_identity::Identity;
use actix_web::{post, web, Responder};
use mongodb::Client;

/// Sync histories endpoint
/// 
/// Handles synchronization of reading history entries.
#[post("/histories")]
pub async fn sync_histories(
    client: web::Data<Client>,
    user: Identity,
    body: web::Bytes,
) -> impl Responder {
    handle_sync(
        SyncEndpointConfig::new("history", "/histories"),
        client,
        user,
        body,
        |user_id, cli, payload| async move {
            sync_history_list(user_id, &payload, cli).await
        },
    )
    .await
}
