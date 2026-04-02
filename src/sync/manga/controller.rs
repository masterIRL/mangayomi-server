use crate::sync::manga::model::MangaList;
use crate::sync::manga::service::sync_manga_list;
use actix_identity::Identity;
use actix_web::{HttpResponse, Responder, post, web};
use mongodb::Client;
use mongodb::bson::oid::ObjectId;

#[post("/manga")]
async fn sync_manga(
    client: web::Data<Client>,
    user: Identity,
    manga_list: web::Json<MangaList>,
) -> impl Responder {
    let user_id = match user.id().ok().and_then(|id| ObjectId::parse_str(&id).ok()) {
        Some(id) => id,
        None => return HttpResponse::Unauthorized().finish(),
    };
    let result = sync_manga_list(user_id, &manga_list, client);
    HttpResponse::Ok().json(result.await)
}
