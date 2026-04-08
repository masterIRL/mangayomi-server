//! Trait implementations for Settings models

use crate::config::collections;
use crate::sync::extractor::ValidatePayload;
use crate::sync::model::{Model, Syncable};
use crate::sync::settings::model::{Settings, SettingsObj};
use crate::sync::validation;

impl Model for Settings {
    fn get_id(&self) -> i64 {
        self.id
    }

    fn get_updated_at(&self) -> i64 {
        self.updated_at
    }
}

impl Syncable for Settings {
    const COLLECTION_NAME: &'static str = collections::SETTINGS;
}

impl ValidatePayload for SettingsObj {
    fn item_count(&self) -> usize {
        self.settings.as_ref().map(|_| 1).unwrap_or(0)
    }

    fn validate(&self) -> Result<(), String> {
        if let Some(ref settings) = self.settings {
            validation::validate_id(settings.id, "settings")?;

            if settings.updated_at < 0 {
                return Err("Invalid updated_at timestamp".to_string());
            }
        }
        Ok(())
    }
}

impl crate::sync::handler::SyncResult for SettingsObj {
    fn modified_count(&self) -> usize {
        self.settings.as_ref().map(|_| 1).unwrap_or(0)
    }
}
