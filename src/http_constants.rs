//! HTTP status code constants and response helpers

use actix_web::HttpResponse;
use serde::Serialize;

/// 200 OK
pub const HTTP_OK: u16 = 200;

/// 201 Created
pub const HTTP_CREATED: u16 = 201;

/// 204 No Content
pub const HTTP_NO_CONTENT: u16 = 204;

/// 400 Bad Request
pub const HTTP_BAD_REQUEST: u16 = 400;

/// 401 Unauthorized
pub const HTTP_UNAUTHORIZED: u16 = 401;

/// 403 Forbidden
pub const HTTP_FORBIDDEN: u16 = 403;

/// 404 Not Found
pub const HTTP_NOT_FOUND: u16 = 404;

/// 413 Payload Too Large
pub const HTTP_PAYLOAD_TOO_LARGE: u16 = 413;

/// 422 Unprocessable Entity
pub const HTTP_UNPROCESSABLE_ENTITY: u16 = 422;

/// 429 Too Many Requests
pub const HTTP_TOO_MANY_REQUESTS: u16 = 429;

/// 500 Internal Server Error
pub const HTTP_INTERNAL_SERVER_ERROR: u16 = 500;

/// 503 Service Unavailable
pub const HTTP_SERVICE_UNAVAILABLE: u16 = 503;

/// Standardized error response body
#[derive(Debug, Serialize, Clone)]
pub struct ErrorResponse {
    pub status: u16,
    pub error: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
}

impl ErrorResponse {
    /// Creates a new error response
    pub fn new(status: u16, error: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            status,
            error: error.into(),
            message: message.into(),
            code: None,
        }
    }

    /// Creates a new error response with a specific error code
    pub fn with_code(
        status: u16,
        error: impl Into<String>,
        message: impl Into<String>,
        code: impl Into<String>,
    ) -> Self {
        Self {
            status,
            error: error.into(),
            message: message.into(),
            code: Some(code.into()),
        }
    }

    /// Creates a bad request response
    pub fn bad_request(message: impl Into<String>) -> Self {
        Self::new(HTTP_BAD_REQUEST, "Bad Request", message)
    }

    /// Creates an unauthorized response
    pub fn unauthorized(message: impl Into<String>) -> Self {
        Self::new(HTTP_UNAUTHORIZED, "Unauthorized", message)
    }

    /// Creates a not found response
    pub fn not_found(message: impl Into<String>) -> Self {
        Self::new(HTTP_NOT_FOUND, "Not Found", message)
    }

    /// Creates a payload too large response
    pub fn payload_too_large(message: impl Into<String>) -> Self {
        Self::new(HTTP_PAYLOAD_TOO_LARGE, "Payload Too Large", message)
    }

    /// Creates an internal server error response
    pub fn internal_error(message: impl Into<String>) -> Self {
        Self::new(HTTP_INTERNAL_SERVER_ERROR, "Internal Server Error", message)
    }
}

/// Creates a standardized bad request response
pub fn bad_request(message: impl Into<String>) -> HttpResponse {
    HttpResponse::BadRequest().json(ErrorResponse::bad_request(message))
}

/// Creates a standardized unauthorized response
pub fn unauthorized(message: impl Into<String>) -> HttpResponse {
    HttpResponse::Unauthorized().json(ErrorResponse::unauthorized(message))
}

/// Creates a standardized not found response
pub fn not_found(message: impl Into<String>) -> HttpResponse {
    HttpResponse::NotFound().json(ErrorResponse::not_found(message))
}

/// Creates a standardized payload too large response
pub fn payload_too_large(message: impl Into<String>) -> HttpResponse {
    HttpResponse::PayloadTooLarge().json(ErrorResponse::payload_too_large(message))
}

/// Creates a standardized internal server error response
pub fn internal_error(message: impl Into<String>) -> HttpResponse {
    HttpResponse::InternalServerError().json(ErrorResponse::internal_error(message))
}

/// JSON content type header value
pub const CONTENT_TYPE_JSON: &str = "application/json";

/// Plain text content type header value
pub const CONTENT_TYPE_TEXT: &str = "text/plain";

/// HTML content type header value
pub const CONTENT_TYPE_HTML: &str = "text/html";
