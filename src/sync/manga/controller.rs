use crate::sync::handler::{handle_sync, SyncEndpointConfig};
use crate::sync::manga::service::sync_manga_list;
use actix_identity::Identity;
use actix_web::{post, web, Responder};
use mongodb::Client;

/// Sync manga endpoint
/// 
/// Handles synchronization of manga, categories, chapters, and tracks.
#[post("/manga")]
pub async fn sync_manga(
    client: web::Data<Client>,
    user: Identity,
    body: web::Bytes,
) -> impl Responder {
    handle_sync(
        SyncEndpointConfig::new("manga", "/manga"),
        client,
        user,
        body,
        |user_id, cli, payload| async move {
            sync_manga_list(user_id, &payload, cli).await
        },
    )
    .await
}
