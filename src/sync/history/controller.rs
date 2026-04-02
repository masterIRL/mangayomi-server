use crate::sync::history::model::HistoryList;
use crate::sync::history::service::sync_history_list;
use actix_identity::Identity;
use actix_web::{HttpResponse, Responder, post, web};
use mongodb::Client;
use mongodb::bson::oid::ObjectId;

#[post("/histories")]
async fn sync_histories(
    client: web::Data<Client>,
    user: Identity,
    history_list: web::Json<HistoryList>,
) -> impl Responder {
    let user_id = match user.id().ok().and_then(|id| ObjectId::parse_str(&id).ok()) {
        Some(id) => id,
        None => return HttpResponse::Unauthorized().finish(),
    };
    let result = sync_history_list(user_id, &history_list, client);
    HttpResponse::Ok().json(result.await)
}
