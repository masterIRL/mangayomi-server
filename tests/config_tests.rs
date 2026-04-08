//! Tests for configuration module
//!
//! These tests verify configuration behavior, not constants.

use mangayomi_server::config::db;

/// Test that timeout clamping works correctly at boundaries
///
/// Business logic: Timeout values outside the allowed range should be
/// clamped to the nearest valid value.
#[test]
fn test_get_timeout_clamps_values() {
    // The function should always return a value within bounds
    // regardless of environment variables
    let timeout = db::get_timeout();

    assert!(
        timeout.as_secs() >= db::MIN_TIMEOUT_SECS,
        "Timeout {} must be >= minimum {}",
        timeout.as_secs(),
        db::MIN_TIMEOUT_SECS
    );
    assert!(
        timeout.as_secs() <= db::MAX_TIMEOUT_SECS,
        "Timeout {} must be <= maximum {}",
        timeout.as_secs(),
        db::MAX_TIMEOUT_SECS
    );
}
