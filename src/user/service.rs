use crate::config::{db as db_config, collections};
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
use mongodb::{Client};
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
        let collection = db.database(db_config::DB_NAME).collection(collections::USERS);
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
        let col_users: mongodb::Collection<User> = db.database(db_config::DB_NAME).collection(collections::USERS);
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
pub async fn delete_account(db: web::Data<Client>, user_id: ObjectId) -> Result<bool, String> {
    if find_account_by_id(user_id, &db).await.is_none() {
        return Ok(false);
    }
    
    let database = db.database(db_config::DB_NAME);
    let col_users: mongodb::Collection<User> = database.collection(collections::USERS);
    let col_categories: mongodb::Collection<Category> = database.collection(collections::CATEGORIES);
    let col_manga: mongodb::Collection<Manga> = database.collection(collections::MANGA);
    let col_chapter: mongodb::Collection<Chapter> = database.collection(collections::CHAPTERS);
    let col_track: mongodb::Collection<Track> = database.collection(collections::TRACKS);
    let col_histories: mongodb::Collection<History> = database.collection(collections::HISTORIES);
    let col_updates: mongodb::Collection<Update> = database.collection(collections::UPDATES);
    let col_settings: mongodb::Collection<Settings> = database.collection(collections::SETTINGS);
    
    // Delete all user data from each collection
    let filter = mongodb::bson::doc! { "user": user_id };
    
    // Execute deletions and track any failures
    let mut errors = Vec::new();
    
    if let Err(e) = col_categories.delete_many(filter.clone()).await {
        errors.push(format!("categories: {}", e));
    }
    if let Err(e) = col_manga.delete_many(filter.clone()).await {
        errors.push(format!("manga: {}", e));
    }
    if let Err(e) = col_chapter.delete_many(filter.clone()).await {
        errors.push(format!("chapters: {}", e));
    }
    if let Err(e) = col_track.delete_many(filter.clone()).await {
        errors.push(format!("tracks: {}", e));
    }
    if let Err(e) = col_histories.delete_many(filter.clone()).await {
        errors.push(format!("histories: {}", e));
    }
    if let Err(e) = col_updates.delete_many(filter.clone()).await {
        errors.push(format!("updates: {}", e));
    }
    if let Err(e) = col_settings.delete_many(filter.clone()).await {
        errors.push(format!("settings: {}", e));
    }
    
    // Delete the user record itself
    match col_users.delete_one(mongodb::bson::doc! { "_id": user_id }).await {
        Ok(result) => {
            log::info!("Deleted {} user(s).", result.deleted_count);
        }
        Err(e) => {
            log::error!("Failed to delete user: {}", e);
            errors.push(format!("user: {}", e));
        }
    }
    
    if !errors.is_empty() {
        Err(format!("Failed to delete some data: {:?}", errors))
    } else {
        Ok(true)
    }
}

/// returns an account with the matching id
async fn find_account_by_id(id: ObjectId, db: &Client) -> Option<User> {
    let collection = db.database(db_config::DB_NAME).collection::<User>(collections::USERS);
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
    let collection = db.database(db_config::DB_NAME).collection::<User>(collections::USERS);
    match collection.find_one(doc! { "email": email }).await {
        Ok(user) => user,
        Err(err) => {
            log::error!("Failed to find account by email: {}", err);
            None
        }
    }
}
