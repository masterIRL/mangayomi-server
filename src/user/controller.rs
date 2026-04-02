use actix_files::NamedFile;
use crate::user::model::{BasicUser, UpdateUser};
use crate::user::service::{delete_account, login_account, register_account, update_account};
use actix_http::HttpMessage;
use actix_identity::Identity;
use actix_web::error::ErrorBadRequest;
use actix_web::web::Data;
use actix_web::{HttpRequest, HttpResponse, Responder, Result, get, post, web, delete};
use mongodb::bson::oid::ObjectId;
use mongodb::Client;
use validator::Validate;

/// register a new account with the given email and password
#[post("/register")]
async fn register(client: Data<Client>, user: web::Json<BasicUser>) -> Result<String> {
    let is_valid = user.validate();
    if is_valid.is_err() {
        return Err(ErrorBadRequest("Username or password is invalid!"));
    }
    let result = register_account(client, &user);
    match result.await {
        Some(_account) => Ok("Account registered!".to_owned()),
        None => Ok(format!("Account already exists {}!", user.email)),
    }
}

/// login into the session
#[post("/login")]
async fn login(
    request: HttpRequest,
    client: Data<Client>,
    user: web::Json<BasicUser>,
) -> Result<String> {
    let is_valid = user.validate();
    if is_valid.is_err() {
        return Err(ErrorBadRequest("Username or password is invalid!"));
    }
    let result = login_account(client, &user);
    match result.await {
        Some(account) => {
            Identity::login(&request.extensions(), account.id.unwrap().to_string())?;
            Ok(format!("Welcome {}!", account.email))
        }
        None => Ok(format!("Account not found {}!", user.email)),
    }
}

/// logout from session
#[get("/logout")]
async fn logout(user: Option<Identity>) -> Result<HttpResponse> {
    if let Some(u) = user {
        u.logout();
        Ok(HttpResponse::Ok().body("Logged out!"))
    } else {
        // Already logged out or session expired
        Ok(HttpResponse::Ok().body("Already logged out!"))
    }
}

/// update account
#[post("/profile")]
async fn profile(client: Data<Client>, user: Identity, data: web::Json<UpdateUser>) -> impl Responder {
    let user_id = match user.id().ok().and_then(|id| ObjectId::parse_str(&id).ok()) {
        Some(id) => id,
        None => return HttpResponse::Unauthorized().body("Unauthorized"),
    };
    let success = update_account(client, user_id, &data).await;
    if success {
        return HttpResponse::Ok().body("Account updated!".to_string());
    }
    HttpResponse::BadRequest().body("".to_string())
}

/// delete account
#[delete("/delete")]
async fn delete(client: Data<Client>, user: Identity) -> impl Responder {
    let user_id = match user.id().ok().and_then(|id| ObjectId::parse_str(&id).ok()) {
        Some(id) => id,
        None => return HttpResponse::Unauthorized().body("Unauthorized"),
    };
    let success = delete_account(client, user_id).await;
    if success {
        return HttpResponse::Ok().body("Account successfully deleted!".to_string());
    }
    HttpResponse::BadRequest().body("".to_string())
}

#[get("/")]
async fn home() -> impl Responder {
    NamedFile::open_async("./frontend/dist/browser/index.html").await
}
