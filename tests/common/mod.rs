use serde::de::DeserializeOwned;
use std::fmt::Debug;

/// DRY abstraction for payload parsing.
/// Attempts to deserialize a JSON string into `T` and panics with a clean
/// error message containing the failing string and error info if it fails.
pub fn parse_model<T>(json: &str) -> T
where
    T: DeserializeOwned + Debug,
{
    match serde_json::from_str::<T>(json) {
        Ok(model) => model,
        Err(err) => panic!(
            "Deserialization failed!\nError: {}\nPayload: {}",
            err, json
        ),
    }
}
