//! Settings deserialization tests
//!
//! Tests for the schemaless settings storage approach.
//!
//! NOTE: JSON fixtures are duplicated across test files rather than shared.
//! This is intentional: Rust integration tests compile each file as a separate
//! binary. Sharing fixtures via a common module causes dead_code warnings
//! because each binary only uses a subset of the shared fixtures.
//! Duplication is cleaner than suppressing warnings or using #[allow(dead_code)].
use mangayomi_server::sync::settings::model::SettingsObj;

/// Timestamp value from 2026 (DateTime.now().millisecondsSinceEpoch)
const TIMESTAMP_2026: i64 = 1_775_156_240_310_i64;

const SETTINGS_FULL_DART_PAYLOAD: &str = r#"{
    "settings": {
        "id": 227,
        "updatedAt": 1775156240310,
        "displayType": 0,
        "themeIsDark": false
    }
}"#;

const SETTINGS_EMPTY_PAYLOAD: &str = r#"{}"#;

const SETTINGS_WITH_NULL: &str = r#"{
    "settings": null
}"#;

const SETTINGS_WITH_UNKNOWN_FIELDS: &str = r#"{
    "settings": {
        "id": 227,
        "updatedAt": 1775156240310,
        "displayType": 0,
        "themeIsDark": false,
        "futureFieldNotYetDefined": "some_value",
        "nestedObject": {
            "key": "value"
        }
    }
}"#;

#[test]
fn test_settings_full_dart_client_payload() {
    // This is an exact simulation of what the Dart client sends via _getSettingsData()
    // when download=false (normal sync). Settings always has id=227.
    // updatedAt is DateTime.now().millisecondsSinceEpoch which in 2026 is ~1.77 trillion.
    //
    // NOTE: With schemaless storage, all client fields are captured in `content`
    // and the server doesn't validate individual fields.
    let result: Result<SettingsObj, _> = serde_json::from_str(SETTINGS_FULL_DART_PAYLOAD);
    assert!(
        result.is_ok(),
        "Failed to deserialize full Dart settings payload: {:?}",
        result.err()
    );

    let obj = result.unwrap();
    assert!(obj.settings.is_some(), "Settings should be present");
    let settings = obj.settings.unwrap();
    assert_eq!(settings.id, 227, "Settings ID should be 227");
    assert_eq!(
        settings.updated_at, TIMESTAMP_2026,
        "updatedAt should match"
    );

    // With schemaless storage, verify the content blob captured the extra fields
    let content = settings.content;
    assert!(content.is_object(), "Content should be a JSON object");
    assert_eq!(
        content.get("displayType").and_then(|v| v.as_i64()),
        Some(0),
        "displayType should be captured in content"
    );
    assert_eq!(
        content.get("themeIsDark").and_then(|v| v.as_bool()),
        Some(false),
        "themeIsDark should be captured in content"
    );
}

#[test]
fn test_settings_download_only_empty_payload() {
    // When download=true, the Dart client sends {} (no "settings" key at all).
    // This must deserialize to SettingsObj { settings: None } — NOT panic.
    let result: Result<SettingsObj, _> = serde_json::from_str(SETTINGS_EMPTY_PAYLOAD);
    assert!(
        result.is_ok(),
        "Empty payload must deserialize: {:?}",
        result.err()
    );

    let obj = result.unwrap();
    assert!(
        obj.settings.is_none(),
        "settings should be None for download-only payload"
    );
}

#[test]
fn test_settings_with_explicit_null() {
    // Edge case: some clients might send {"settings": null} explicitly.
    let result: Result<SettingsObj, _> = serde_json::from_str(SETTINGS_WITH_NULL);
    assert!(
        result.is_ok(),
        "Explicit null settings must deserialize: {:?}",
        result.err()
    );
    assert!(result.unwrap().settings.is_none());
}

#[test]
fn test_settings_timestamp_at_i32_boundary() {
    // Exact i32::MAX boundary — this is the value that JUST fits in i32.
    // Values above this cause the original bug.
    let json = format!(
        r#"{{
        "settings": {{
            "id": 227,
            "updatedAt": {},
            "displayType": 0
        }}
    }}"#,
        i32::MAX as i64 + 1
    );

    let result: Result<SettingsObj, _> = serde_json::from_str(&json);
    assert!(
        result.is_ok(),
        "Timestamp just above i32::MAX must work: {:?}",
        result.err()
    );
    assert_eq!(
        result.unwrap().settings.unwrap().updated_at,
        i32::MAX as i64 + 1
    );
}

#[test]
fn test_settings_schemaless_ignores_unknown_fields() {
    // With schemaless storage, the server should accept ANY fields
    // sent by the client without validation errors.
    let result: Result<SettingsObj, _> = serde_json::from_str(SETTINGS_WITH_UNKNOWN_FIELDS);
    assert!(
        result.is_ok(),
        "Unknown fields should be accepted: {:?}",
        result.err()
    );

    let settings = result.unwrap().settings.unwrap();
    let content = settings.content;

    assert!(
        content.get("futureFieldNotYetDefined").is_some(),
        "Unknown fields should be captured in content"
    );
    assert!(
        content.get("nestedObject").is_some(),
        "Nested objects should be captured"
    );
}
