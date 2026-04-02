use crate::sync::update::model::UpdateList;
use crate::sync::update::service::sync_update_list;
use actix_identity::Identity;
use actix_web::{HttpResponse, Responder, post, web};
use mongodb::Client;
use mongodb::bson::oid::ObjectId;

#[post("/updates")]
async fn sync_updates(
    client: web::Data<Client>,
    user: Identity,
    update_list: web::Json<UpdateList>,
) -> impl Responder {
    let user_id = match user.id().ok().and_then(|id| ObjectId::parse_str(&id).ok()) {
        Some(id) => id,
        None => return HttpResponse::Unauthorized().finish(),
    };
    let result = sync_update_list(user_id, &update_list, client);
    HttpResponse::Ok().json(result.await)
}
