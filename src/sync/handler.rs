//! Generic sync endpoint handler to eliminate controller duplication

use crate::config::logging;
use crate::sync::extractor::{self, ValidatePayload};
use crate::user::extractor::extract_user_id;
use actix_identity::Identity;
use actix_web::{HttpResponse, Responder, web};
use mongodb::bson::oid::ObjectId;
use mongodb::Client;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::time::Instant;

/// Trait for sync results that can provide a count of modified items
pub trait SyncResult {
    fn modified_count(&self) -> usize;
}

/// Configuration for a sync endpoint handler
#[derive(Debug, Clone, Copy)]
pub struct SyncEndpointConfig {
    pub entity_name: &'static str,
    pub path: &'static str,
}

impl SyncEndpointConfig {
    pub const fn new(entity_name: &'static str, path: &'static str) -> Self {
        Self { entity_name, path }
    }
}

/// Standard error response for sync operations
#[derive(Debug, Serialize)]
pub struct SyncErrorResponse {
    pub error: String,
    pub entity: &'static str,
}

impl SyncErrorResponse {
    pub const fn new(error: String, entity: &'static str) -> Self {
        Self { error, entity }
    }
}

/// Generic sync handler that can be used directly in controllers
///
/// # Arguments
/// * `config` - Endpoint configuration
/// * `client` - MongoDB client
/// * `user` - User identity
/// * `body` - Request body
/// * `service_fn` - Service function to process the sync
///
/// # Returns
/// HTTP response with sync result or error
pub async fn handle_sync<T, R, F, Fut>(
    config: SyncEndpointConfig,
    client: web::Data<Client>,
    user: Identity,
    body: web::Bytes,
    service_fn: F,
) -> impl Responder
where
    T: ValidatePayload + DeserializeOwned + Send + Sync + 'static,
    R: SyncResult + Serialize + Send + Sync + 'static,
    F: FnOnce(ObjectId, web::Data<Client>, T) -> Fut,
    Fut: std::future::Future<Output = Result<R, crate::sync::error::SyncError>>,
{
    let start = Instant::now();

    let user_id = match extract_user_id(user) {
        Ok(id) => id,
        Err(resp) => return resp,
    };

    let payload = match extractor::extract_sync_payload::<T>(body, config.entity_name).await {
        Ok(data) => data,
        Err(resp) => return resp,
    };

    match service_fn(user_id, client, payload).await {
        Ok(result) => {
            let duration = start.elapsed().as_millis();
            let modified_count = result.modified_count();
            extractor::log_sync_complete(config.entity_name, modified_count, 0, duration);

            HttpResponse::Ok().json(result)
        }
        Err(e) => {
            log::error!(
                target: logging::SYNC_API,
                "[SYNC] {} sync failed: {}",
                config.entity_name,
                e
            );
            HttpResponse::InternalServerError().json(SyncErrorResponse::new(
                format!("Sync failed: {}", e),
                config.entity_name,
            ))
        }
    }
}
