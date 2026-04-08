//! Settings model with schemaless storage

use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

/// User settings stored as opaque JSON
///
/// The server only tracks metadata for sync conflict resolution.
/// Actual settings content is stored opaquely as `serde_json::Value`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Default)]
pub struct Settings {
    #[serde(rename = "_id", skip_serializing)]
    pub oid: Option<ObjectId>,
    pub id: i64,
    #[serde(skip_serializing)]
    pub user: Option<ObjectId>,
    #[serde(rename = "updatedAt")]
    pub updated_at: i64,
    #[serde(flatten)]
    pub content: serde_json::Value,
}

/// Settings wrapper for sync API
#[derive(Serialize, Deserialize, Default, Clone)]
pub struct SettingsObj {
    pub settings: Option<Settings>,
}
