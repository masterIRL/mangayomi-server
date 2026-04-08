use actix_identity::Identity;
use actix_web::HttpResponse;
use mongodb::bson::oid::ObjectId;

/// Extract user ID from identity
///
/// Returns Ok(ObjectId) if valid, Err(HttpResponse) otherwise
pub fn extract_user_id(user: Identity) -> Result<ObjectId, HttpResponse> {
    match user.id().ok().and_then(|id| ObjectId::parse_str(&id).ok()) {
        Some(id) => Ok(id),
        None => Err(HttpResponse::Unauthorized().finish()),
    }
}
