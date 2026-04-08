use mangayomi_server::sync::history::model::History;
use mangayomi_server::sync::manga::model::{Category, Chapter, Manga, Track};
use mangayomi_server::sync::update::model::Update;
use actix_governor::governor::middleware::NoOpMiddleware;
use actix_governor::{Governor, GovernorConfig, GovernorConfigBuilder, PeerIpKeyExtractor};
use actix_identity::IdentityMiddleware;
use actix_session::SessionMiddleware;
use actix_session::config::{CookieContentSecurity, PersistentSession, TtlExtensionPolicy};
use actix_session::storage::CookieSessionStore;
use actix_web::cookie::{Key, SameSite};
use actix_web::middleware::{Logger, NormalizePath};
use actix_web::error::JsonPayloadError;
use actix_web::{
    App, HttpResponse, HttpServer, Scope, cookie::time::Duration as CookieDuration, web,
};
use mongodb::bson::doc;
use mongodb::options::{ClientOptions, IndexOptions};
use mongodb::{Client, IndexModel};
use std::fs;
use tera::Tera;
use walkdir::WalkDir;

use mangayomi_server::{app, db, globals, sync, user, http_constants};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenvy::dotenv().ok();
    
    // Initialize logging - uses default if RUST_LOG is not set
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let db_url = globals::DATABASE_URL.as_str();
    let host = globals::HOST.as_str();
    let port = globals::PORT.as_str();
    let session_ttl = &globals::SESSION_TTL;
    let key = &globals::SECRET_KEY;
    let secret_key = if key.len() < 64 {
        log::info!("Generated a random key because SECRET_KEY is not set in .env");
        Key::generate()
    } else {
        Key::from(key.as_bytes())
    };

    log::info!("Connecting to {}...", db_url);

    db::CONN
        .get_or_init(|| async {
            let client_options = ClientOptions::parse(db_url).await
                .expect("Failed to parse MongoDB connection options");
            let result = Client::with_options(client_options)
                .expect("Failed to create MongoDB client");
            log::info!("Connected to MongoDB.");
            result
        })
        .await;
    log::info!("Initializing Tera...");
    let mut tera = Tera::default();
    match tera.add_raw_templates(get_templates()) {
        Ok(t) => t,
        Err(e) => {
            println!("Parsing error(s): {}", e);
            ::std::process::exit(1);
        }
    };
    tera.autoescape_on(vec![".html"]);
    log::info!("Initialized Tera.");

    let conn = db::CONN.get()
        .expect("Database connection not initialized");

    init_db_indexes(conn).await;

    // Initialize bulk upserter with MongoDB version detection
    db::bulk_upsert::initialize_global_upserter(conn).await;

    /*
    if *globals::USE_REDIS {
                let redis_store = RedisSessionStore::new(redis_url)
                    .await
                    .unwrap();
                SessionMiddleware::new(
                    redis_store.clone(),
                    secret_key.clone(),
                )
            } else {
                SessionMiddleware::new(
                    CookieSessionStore::default(),
                    secret_key.clone(),
                )
            }
     */

    HttpServer::new(move || {
        // Configure JSON payload limits to match sync payload limits
        let max_payload_size = sync::extractor::get_max_payload_size();
        let json_config = web::JsonConfig::default()
            .limit(max_payload_size)
            .error_handler(|err, _req| {
                match err {
                    JsonPayloadError::Overflow { limit } => {
                        let error_msg = format!("JSON payload too large (limit: {}MB)", limit >> 20);
                        log::error!("{}", error_msg);
                        actix_web::error::InternalError::from_response(
                            err,
                            http_constants::payload_too_large(error_msg)
                        ).into()
                    }
                    _ => {
                        actix_web::error::InternalError::from_response(
                            err,
                            http_constants::bad_request("Invalid JSON payload")
                        ).into()
                    }
                }
            });

        App::new()
            .wrap(Logger::default())
            .wrap(NormalizePath::trim())
            .wrap(IdentityMiddleware::default())
            .wrap(Governor::new(&rate_limiter()))
            .wrap(
                SessionMiddleware::builder(CookieSessionStore::default(), secret_key.clone())
                    .session_lifecycle(
                        PersistentSession::default()
                            .session_ttl(CookieDuration::days(**session_ttl))
                            .session_ttl_extension_policy(TtlExtensionPolicy::OnEveryRequest),
                    )
                    .cookie_secure(true)
                    .cookie_same_site(SameSite::Strict)
                    .cookie_content_security(CookieContentSecurity::Private)
                    .cookie_http_only(true)
                    .build(),
            )
            .app_data(web::Data::new(conn.clone()))
            .app_data(web::Data::new(tera.clone()))
            .app_data(json_config)
            .service(actix_files::Files::new("/assets", "./resources/assets"))
            .service(actix_files::Files::new("/static", "./frontend/dist/browser"))
            .service(user::controller::profile)
            .service(user::controller::delete)
            .service(user::controller::register)
            .service(user::controller::login)
            .service(user::controller::logout)
            .service(user::controller::home)
            .service(sync_controller())
            .service(app::app_routes::basic_controller())
            .default_service(web::to(|| HttpResponse::NotFound()))
    })
    .bind(format!("{}:{}", host, port))?
    .run()
    .await
}

fn sync_controller() -> Scope {
    let max_size = sync::extractor::get_max_payload_size();
    log::info!("Sync endpoints configured with max payload size: {}MB", max_size >> 20);
    
    web::scope("/sync")
        .app_data(web::PayloadConfig::new(max_size))
        .service(sync::manga::controller::sync_manga)
        .service(sync::history::controller::sync_histories)
        .service(sync::update::controller::sync_updates)
        .service(sync::settings::controller::sync_settings_obj)
}

fn rate_limiter() -> GovernorConfig<PeerIpKeyExtractor, NoOpMiddleware> {
    GovernorConfigBuilder::default()
        .const_requests_per_minute(30)
        .burst_size(15)
        .finish()
        .expect("Failed to build rate limiter configuration")
}

fn get_templates() -> Vec<(String, String)> {
    let mut templates: Vec<(String, String)> = Vec::new();

    for file in WalkDir::new("./resources/templates")
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if file.metadata().map(|m| m.is_file()).unwrap_or(false) {
            let template_name = file
                .path()
                .file_name()
                .and_then(|n| n.to_str())
                .map(|s| s.to_string());
            
            if let Some(name) = template_name {
                if let Ok(template_raw) = fs::read_to_string(file.path()) {
                    log::info!("Adding template: {}", name);
                    templates.push((name, template_raw));
                }
            }
        }
    }
    templates
}

use mangayomi_server::config::{collections, db as db_config};

async fn create_index_for_collection<T: Send + Sync>(db: &mongodb::Database, collection_name: &str, idx: &IndexModel) {
    let col: mongodb::Collection<T> = db.collection(collection_name);
    match col.create_index(idx.clone()).await {
        Ok(result) => log::info!("Created {} index: {}", collection_name, result.index_name),
        Err(e) => log::info!("Failed to create {} index: {}", collection_name, e),
    }
}

async fn init_db_indexes(conn: &Client) {
    let db = conn.database(db_config::DB_NAME);
    let opts = IndexOptions::builder().unique(true).build();
    let idx = IndexModel::builder()
        .keys(doc! { "id": -1, "user": -1 })
        .options(opts)
        .build();

    create_index_for_collection::<Category>(&db, collections::CATEGORIES, &idx).await;
    create_index_for_collection::<Manga>(&db, collections::MANGA, &idx).await;
    create_index_for_collection::<Chapter>(&db, collections::CHAPTERS, &idx).await;
    create_index_for_collection::<Track>(&db, collections::TRACKS, &idx).await;
    create_index_for_collection::<History>(&db, collections::HISTORIES, &idx).await;
    create_index_for_collection::<Update>(&db, collections::UPDATES, &idx).await;
    create_index_for_collection::<mangayomi_server::sync::settings::model::Settings>(&db, collections::SETTINGS, &idx).await;
}
