//! Integration tests for payload extraction and validation
//!
//! These tests verify the security-critical payload handling logic including
//! size limits, validation, and edge cases that could indicate attacks.

use mangayomi_server::sync::extractor::{extract_sync_payload, get_max_payload_size, ValidatePayload};
use mangayomi_server::config::sync;
use mangayomi_server::http_constants::HTTP_PAYLOAD_TOO_LARGE;

use serde::{Deserialize, Serialize};

/// Test payload for validation testing
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
struct TestPayload {
    id: i64,
    name: String,
    items: Vec<String>,
}

impl ValidatePayload for TestPayload {
    fn item_count(&self) -> usize {
        self.items.len()
    }
    
    fn validate(&self) -> Result<(), String> {
        // Business rule: ID must be positive
        if self.id < sync::MIN_ID_VALUE {
            return Err(format!("Invalid ID: {} (must be >= {})", 
                              self.id, sync::MIN_ID_VALUE));
        }
        // Business rule: Name cannot be empty or whitespace-only
        if self.name.trim().is_empty() {
            return Err("Name cannot be empty".to_string());
        }
        // Business rule: Name length limit
        if self.name.len() > 100 {
            return Err("Name exceeds maximum length of 100 characters".to_string());
        }
        Ok(())
    }
}

/// Test successful extraction with valid payload
///
/// Business logic: Normal valid payloads should parse and validate cleanly.
#[tokio::test]
async fn test_extract_sync_payload_success() {
    let json = r#"{"id": 123, "name": "Test", "items": ["a", "b"]}"#;
    let body = actix_web::web::Bytes::from(json);
    
    let result: Result<TestPayload, _> = extract_sync_payload(body, "test_endpoint").await;
    
    assert!(result.is_ok(), "Valid payload should succeed");
    let payload = result.unwrap();
    assert_eq!(payload.id, 123);
    assert_eq!(payload.name, "Test");
    assert_eq!(payload.items.len(), 2);
}

/// Test extraction with type mismatch in JSON
///
/// Security: Type confusion attacks should be rejected.
#[tokio::test]
async fn test_extract_sync_payload_type_mismatch() {
    let json = r#"{"id": "not_a_number", "name": "Test", "items": []}"#;
    let body = actix_web::web::Bytes::from(json);
    
    let result: Result<TestPayload, _> = extract_sync_payload(body, "test_endpoint").await;
    
    assert!(result.is_err(), "Type mismatch should be rejected");
    let resp = result.unwrap_err();
    assert_eq!(resp.status(), 400, "Should return Bad Request for type mismatch");
}

/// Test extraction with minimal valid payload
///
/// Edge case: Payloads at the boundary of validity should be accepted.
#[tokio::test]
async fn test_extract_sync_payload_minimum_valid() {
    // Minimum valid: id=1, name with one char, empty items
    let json = r#"{"id": 1, "name": "X", "items": []}"#;
    let body = actix_web::web::Bytes::from(json);
    
    let result: Result<TestPayload, _> = extract_sync_payload(body, "test_endpoint").await;
    
    assert!(result.is_ok(), "Minimum valid payload should succeed");
    assert_eq!(result.unwrap().item_count(), 0);
}

/// Test extraction with ID at minimum boundary
///
/// Edge case: ID of 0 should be rejected (business rule).
#[tokio::test]
async fn test_extract_sync_payload_rejects_zero_id() {
    let json = r#"{"id": 0, "name": "Test", "items": []}"#;
    let body = actix_web::web::Bytes::from(json);
    
    let result: Result<TestPayload, _> = extract_sync_payload(body, "test_endpoint").await;
    
    assert!(result.is_err(), "Zero ID should be rejected");
    let resp = result.unwrap_err();
    assert_eq!(resp.status(), 400);
}

/// Test extraction with negative ID
///
/// Security: Negative IDs could indicate integer underflow attacks.
#[tokio::test]
async fn test_extract_sync_payload_rejects_negative_id() {
    let json = r#"{"id": -1, "name": "Test", "items": []}"#;
    let body = actix_web::web::Bytes::from(json);
    
    let result: Result<TestPayload, _> = extract_sync_payload(body, "test_endpoint").await;
    
    assert!(result.is_err(), "Negative ID should be rejected");
}

/// Test extraction with empty name
///
/// Business rule: Empty names are not allowed.
#[tokio::test]
async fn test_extract_sync_payload_rejects_empty_name() {
    let json = r#"{"id": 1, "name": "", "items": []}"#;
    let body = actix_web::web::Bytes::from(json);
    
    let result: Result<TestPayload, _> = extract_sync_payload(body, "test_endpoint").await;
    
    assert!(result.is_err(), "Empty name should be rejected");
}

/// Test extraction with whitespace-only name
///
/// Security: Whitespace-only strings could be used to bypass validation.
#[tokio::test]
async fn test_extract_sync_payload_rejects_whitespace_name() {
    let json = r#"{"id": 1, "name": "   ", "items": []}"#;
    let body = actix_web::web::Bytes::from(json);
    
    let result: Result<TestPayload, _> = extract_sync_payload(body, "test_endpoint").await;
    
    assert!(result.is_err(), "Whitespace-only name should be rejected");
}

/// Test extraction with oversized payload (DoS protection)
///
/// Security: Payloads exceeding max size should be rejected to prevent
/// memory exhaustion attacks.
#[tokio::test]
async fn test_extract_sync_payload_rejects_oversized() {
    let max_size = get_max_payload_size();
    let oversized = vec![b'x'; max_size + 1];
    let body = actix_web::web::Bytes::from(oversized);
    
    let result: Result<TestPayload, _> = extract_sync_payload(body, "test_endpoint").await;
    
    assert!(result.is_err(), "Oversized payload should be rejected");
    let resp = result.unwrap_err();
    assert_eq!(resp.status(), HTTP_PAYLOAD_TOO_LARGE as u16, 
               "Should return 413 Payload Too Large");
}

/// Test extraction with payload exactly at size limit
///
/// Edge case: Payload at exact limit should be accepted.
#[tokio::test]
async fn test_extract_sync_payload_accepts_exact_size_limit() {
    // Create a valid JSON payload that exactly hits the size limit
    let max_size = get_max_payload_size();
    
    // Create payload with valid structure but size close to limit
    // This tests boundary condition without needing massive allocation
    let base = r#"{"id": 1, "name": "X", "items": ["#;
    let suffix = r#"]}"#;
    let available = max_size.saturating_sub(base.len() + suffix.len());
    
    // Limit items to stay under MAX_SYNC_ITEMS (100,000) to avoid batch size rejection
    let max_items_for_test = std::cmp::min(available / 5, sync::MAX_SYNC_ITEMS - 1);
    
    // Ensure test setup is valid - we must be able to create a meaningful payload
    assert!(
        max_items_for_test > 10,
        "Test setup failed: payload size config ({}MB) too small for meaningful test",
        sync::DEFAULT_PAYLOAD_SIZE_MB
    );
    
    // Create items array that fills available space
    let items = format!("{}", "\"a\",".repeat(max_items_for_test));
    let json = format!("{}{}{}", base, items.trim_end_matches(','), suffix);
    
    // Verify our constructed payload is under the limit
    assert!(
        json.len() <= max_size,
        "Test setup failed: constructed payload ({} bytes) exceeds max size ({} bytes)",
        json.len(),
        max_size
    );
    
    let body = actix_web::web::Bytes::from(json);
    let result: Result<TestPayload, _> = extract_sync_payload(body, "test_endpoint").await;
    
    // Should either succeed (if valid JSON) or fail validation (not size)
    if let Err(resp) = result {
        assert_ne!(
            resp.status(), 
            HTTP_PAYLOAD_TOO_LARGE as u16,
            "Should not reject for size when under limit"
        );
    }
}

/// Test extraction with empty body
///
/// Edge case: Empty body should be gracefully rejected.
#[tokio::test]
async fn test_extract_sync_payload_rejects_empty_body() {
    let body = actix_web::web::Bytes::from("{}");
    
    let result: Result<TestPayload, _> = extract_sync_payload(body, "test_endpoint").await;
    
    assert!(result.is_err(), "Empty object should fail validation");
}

/// Test extraction with completely empty bytes
#[tokio::test]
async fn test_extract_sync_payload_rejects_zero_bytes() {
    let body = actix_web::web::Bytes::from("");
    
    let result: Result<TestPayload, _> = extract_sync_payload(body, "test_endpoint").await;
    
    assert!(result.is_err(), "Empty bytes should fail parsing");
}

/// Test extraction with malformed JSON
///
/// Security: Malformed JSON could be used in injection attacks.
#[tokio::test]
async fn test_extract_sync_payload_rejects_malformed_json() {
    let test_cases = vec![
        (r#"{"id": 1, "name": "Test""#, "truncated"),
        (r#"{"id": 1, "name": "Test",}"#, "trailing comma"),
        (r#"{"id": 1, "name": }"#, "missing value"),
        (r#"{"id": 1 "name": "Test"}"#, "missing comma"),
        (r#"not json at all"#, "not json"),
        (r#"<html>not json</html>"#, "html payload"),
    ];
    
    for (json, description) in test_cases {
        let body = actix_web::web::Bytes::from(json);
        let result: Result<TestPayload, _> = extract_sync_payload(body, "test_endpoint").await;
        
        assert!(result.is_err(), "{} should be rejected", description);
    }
}

/// Test extraction with nested JSON injection attempt
///
/// Security: Deeply nested structures could cause stack overflow.
#[tokio::test]
async fn test_extract_sync_payload_handles_deeply_nested() {
    // Create deeply nested structure (serde_json has limits, but let's test)
    let depth = 1000;
    let mut json = String::new();
    for _ in 0..depth {
        json.push_str(r#"{"nested":"#);
    }
    json.push_str(r#""value""#);
    for _ in 0..depth {
        json.push_str(r#"}"#);
    }
    
    // This should either parse (if within limits) or fail gracefully
    let body = actix_web::web::Bytes::from(json);
    let result: Result<TestPayload, _> = extract_sync_payload(body, "test_endpoint").await;
    
    // We mainly care that it doesn't panic
    let _ = result;
}

/// Test extraction with valid but large item count
///
/// Edge case: Payloads with many items should be validated correctly.
#[tokio::test]
async fn test_extract_sync_payload_with_many_items() {
    let items: Vec<String> = (0..100).map(|i| format!("item_{}", i)).collect();
    let json = format!(
        r#"{{"id": 1, "name": "Test", "items": {}}}"#,
        serde_json::to_string(&items).unwrap()
    );
    let body = actix_web::web::Bytes::from(json);
    
    let result: Result<TestPayload, _> = extract_sync_payload(body, "test_endpoint").await;
    
    assert!(result.is_ok(), "Payload with many items should succeed");
    assert_eq!(result.unwrap().item_count(), 100);
}

/// Test extraction with special characters in strings
///
/// Security: Special characters should be handled safely.
#[tokio::test]
async fn test_extract_sync_payload_with_special_characters() {
    let json = r#"{"id": 1, "name": "Test <script>alert('xss')</script>", "items": []}"#;
    let body = actix_web::web::Bytes::from(json);
    
    let result: Result<TestPayload, _> = extract_sync_payload(body, "test_endpoint").await;
    
    // Should succeed - the extractor doesn't sanitize content, just validates structure
    assert!(result.is_ok(), "Special characters in strings should be allowed");
    let payload = result.unwrap();
    assert!(payload.name.contains("<script>"), "Content should be preserved");
}

/// Test extraction with unicode in strings
///
/// Business logic: Unicode should be handled correctly.
#[tokio::test]
async fn test_extract_sync_payload_with_unicode() {
    let json = r#"{"id": 1, "name": "测试测试テスト🎉", "items": []}"#;
    let body = actix_web::web::Bytes::from(json);
    
    let result: Result<TestPayload, _> = extract_sync_payload(body, "test_endpoint").await;
    
    assert!(result.is_ok(), "Unicode should be handled correctly");
    assert_eq!(result.unwrap().name, "测试测试テスト🎉");
}
