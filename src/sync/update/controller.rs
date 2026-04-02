use crate::sync::update::model::UpdateList;
use crate::sync::update::service::sync_update_list;
use actix_identity::Identity;
use actix_web::{HttpResponse, Responder, post, web};
use mongodb::Client;
use mongodb::bson::oid::ObjectId;
use serde_json;

#[post("/updates")]
async fn sync_updates(
    client: web::Data<Client>,
    user: Identity,
    body: web::Bytes,
) -> impl Responder {
    // Log the raw body for debugging
    if let Ok(body_str) = std::str::from_utf8(&body) {
        let truncated = if body_str.len() > 3000 {
            format!("{}... (truncated, {} bytes total)", &body_str[..3000], body_str.len())
        } else {
            body_str.to_string()
        };
        log::debug!("[SYNC UPDATE] Raw body: {}", truncated);
    }
    
    // Parse the JSON manually
    let update_list: UpdateList = match serde_json::from_slice(&body) {
        Ok(u) => u,
        Err(e) => {
            log::error!("[SYNC UPDATE] Failed to parse JSON: {}", e);
            return HttpResponse::BadRequest().body(format!("JSON parse error: {}", e));
        }
    };
    
    let user_id = match user.id().ok().and_then(|id| ObjectId::parse_str(&id).ok()) {
        Some(id) => id,
        None => return HttpResponse::Unauthorized().finish(),
    };
    let result = sync_update_list(user_id, &update_list, client);
    HttpResponse::Ok().json(result.await)
}