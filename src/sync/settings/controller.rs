use crate::sync::settings::model::SettingsObj;
use crate::sync::settings::service::sync_settings;
use actix_identity::Identity;
use actix_web::{HttpResponse, Responder, post, web};
use mongodb::Client;
use mongodb::bson::oid::ObjectId;
use serde_json;

#[post("/settings")]
async fn sync_settings_obj(
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
        log::debug!("[SYNC SETTINGS] Raw body: {}", truncated);
    }
    
    // Parse the JSON manually
    let settings: SettingsObj = match serde_json::from_slice(&body) {
        Ok(s) => s,
        Err(e) => {
            log::error!("[SYNC SETTINGS] Failed to parse JSON: {}", e);
            return HttpResponse::BadRequest().body(format!("JSON parse error: {}", e));
        }
    };
    
    let user_id = match user.id().ok().and_then(|id| ObjectId::parse_str(&id).ok()) {
        Some(id) => id,
        None => return HttpResponse::Unauthorized().finish(),
    };
    match sync_settings(user_id, &settings, client).await {
        Some(data) => HttpResponse::Ok().json(data),
        None => HttpResponse::Ok().json(SettingsObj { settings: None }),
    }
}
