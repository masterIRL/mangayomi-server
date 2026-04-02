use crate::sync::history::model::History;
use crate::sync::manga::model::{Category, Chapter, Manga, Track};
use crate::sync::settings::model::Settings;
use crate::sync::update::model::Update;
use crate::user::model::{BasicUser, UpdateUser, User};
use actix_web::web;
use argon2::Argon2;
use std::convert::TryFrom;
use mongodb::bson::oid::ObjectId;
use mongodb::bson::{doc, to_document};
use mongodb::{Client, Collection};
use password_hash::rand_core::OsRng;
use password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString};
use std::time::{SystemTime, UNIX_EPOCH};

fn get_timestamp() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| i64::try_from(d.as_millis()).unwrap_or(0))
        .unwrap_or(0)
}

/// inserts a new account if it does not exist yet
pub async fn register_account(db: web::Data<Client>, user: &web::Json<BasicUser>) -> Option<User> {
    if user.password.chars().count() < 8 {
        return None;
    }
    let usr = find_account(&user.email, &db).await;
    if usr.is_none() {
        let salt = SaltString::generate(&mut OsRng);
        let password_hash = match Argon2::default().hash_password(user.password.as_bytes(), &salt) {
            Ok(hash) => hash,
            Err(err) => {
                log::error!("Failed to hash password: {}", err);
                return None;
            }
        };
        let collection = db.database("mangayomi").collection("users");
        let timestamp = get_timestamp();
        let account = User {
            id: None,
            email: user.email.to_owned(),
            password: password_hash.to_string(),
            salt: salt.to_string(),
            role: "BASIC".to_string(),
            created_at: timestamp,
            updated_at: timestamp,
        };
        return match collection.insert_one(account).await {
            Ok(_result) => find_account(&user.email, &db).await,
            Err(err) => {
                log::error!("Failed to insert user: {}", err);
                None
            }
        };
    }
    None
}

// returns account if the email and password matches
pub async fn login_account(db: web::Data<Client>, user: &web::Json<BasicUser>) -> Option<User> {
    if let Some(account) = find_account(&user.email, &db).await {
        let hash = match PasswordHash::new(&account.password) {
            Ok(h) => h,
            Err(err) => {
                log::error!("Failed to parse password hash: {}", err);
                return None;
            }
        };
        if Argon2::default()
            .verify_password(user.password.as_bytes(), &hash)
            .is_ok()
        {
            return Some(account);
        }
    }
    None
}

// update account details
pub async fn update_account(
    db: web::Data<Client>,
    user_id: ObjectId,
    data: &web::Json<UpdateUser>,
) -> bool {
    let exist_user = find_account(&data.email, &db).await;
    if let Some(usr) = exist_user {
        if usr.id != Some(user_id) {
            return false;
        }
    }
    if let Some(mut account) = find_account_by_id(user_id, &db).await {
        let hash = match PasswordHash::new(&account.password) {
            Ok(h) => h,
            Err(err) => {
                log::error!("Failed to parse password hash: {}", err);
                return false;
            }
        };
        let allow_pw = Argon2::default()
            .verify_password(data.password_old.as_bytes(), &hash)
            .is_ok();
        if data.password_old.chars().count() >= 8 && !allow_pw {
            return false;
        }
        if allow_pw {
            let salt = SaltString::generate(&mut OsRng);
            let password_hash = match Argon2::default()
                .hash_password(data.password.as_bytes(), &salt) {
                    Ok(h) => h,
                    Err(err) => {
                        log::error!("Failed to hash new password: {}", err);
                        return false;
                    }
                };
            account.salt = salt.to_string();
            account.password = password_hash.to_string();
        }
        account.email = data.email.to_owned();
        let timestamp = get_timestamp();
        account.updated_at = timestamp;
        let doc = match to_document(&account) {
            Ok(d) => d,
            Err(err) => {
                log::error!("Failed to convert account to bson document: {}", err);
                return false;
            }
        };
        let col_users: mongodb::Collection<User> = db.database("mangayomi").collection("users");
        let result = col_users
            .update_one(
                doc! { "_id": user_id },
                doc! { "$set": doc },
            )
            .await;
        return result.is_ok();
    }
    false
}

// delete account and related collections
pub async fn delete_account(db: web::Data<Client>, user_id: ObjectId) -> bool {
    if find_account_by_id(user_id, &db).await.is_some() {
        let col_users: mongodb::Collection<User> = db.database("mangayomi").collection("users");
        let col_categories: mongodb::Collection<Category> =
            db.database("mangayomi").collection("categories");
        let col_manga: mongodb::Collection<Manga> = db.database("mangayomi").collection("manga");
        let col_chapter: mongodb::Collection<Chapter> =
            db.database("mangayomi").collection("chapters");
        let col_track: mongodb::Collection<Track> = db.database("mangayomi").collection("tracks");
        let col_histories: mongodb::Collection<History> =
            db.database("mangayomi").collection("histories");
        let col_updates: mongodb::Collection<Update> =
            db.database("mangayomi").collection("updates");
        let col_settings: mongodb::Collection<Settings> =
            db.database("mangayomi").collection("settings");
        
        let empty_ids: &[i64] = &[0];
        delete_many(&col_categories, user_id, empty_ids, false).await;
        delete_many(&col_manga, user_id, empty_ids, false).await;
        delete_many(&col_chapter, user_id, empty_ids, false).await;
        delete_many(&col_track, user_id, empty_ids, false).await;
        delete_many(&col_histories, user_id, empty_ids, false).await;
        delete_many(&col_updates, user_id, empty_ids, false).await;
        delete_many(&col_settings, user_id, empty_ids, false).await;
        
        let user_empty: &[i64] = &[0];
        // For User table we still pass &[i64] to delete_many_user
        delete_many_user(&col_users, user_id, user_empty, true).await;
        return true;
    }
    false
}

async fn delete_many<T: Send + Sync>(
    collection: &Collection<T>,
    user_id: ObjectId,
    ids: &[i64],
    is_user: bool,
) {
    if ids.is_empty() {
        return;
    }
    let del_result = collection
        .delete_many(if is_user {
            doc! { "_id": user_id }
        } else {
            doc! { "user": user_id }
        })
        .await;
    match del_result {
        Ok(result) => log::info!("Deleted {} {}.", result.deleted_count, collection.name()),
        Err(err) => log::error!("Failed to delete {}: {}", collection.name(), err),
    }
}

async fn delete_many_user<T: Send + Sync>(
    collection: &Collection<T>,
    user_id: ObjectId,
    ids: &[i64],
    is_user: bool,
) {
    if ids.is_empty() {
        return;
    }
    let del_result = collection
        .delete_many(if is_user {
            doc! { "_id": user_id }
        } else {
            doc! { "user": user_id }
        })
        .await;
    match del_result {
        Ok(result) => log::info!("Deleted {} {}.", result.deleted_count, collection.name()),
        Err(err) => log::error!("Failed to delete {}: {}", collection.name(), err),
    }
}

/// returns an account with the matching id
async fn find_account_by_id(id: ObjectId, db: &Client) -> Option<User> {
    let collection = db.database("mangayomi").collection("users");
    match collection.find_one(doc! { "_id": id }).await {
        Ok(user) => user,
        Err(err) => {
            log::error!("Failed to find account by id: {}", err);
            None
        }
    }
}

/// returns an account with the matching email
async fn find_account(email: &String, db: &Client) -> Option<User> {
    let collection = db.database("mangayomi").collection("users");
    match collection.find_one(doc! { "email": email }).await {
        Ok(user) => user,
        Err(err) => {
            log::error!("Failed to find account by email: {}", err);
            None
        }
    }
}
