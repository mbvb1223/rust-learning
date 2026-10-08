use axum::extract::rejection::JsonRejection;
use axum::response::{IntoResponse, Response};

/// Every way a request can fail. Handlers return `Result<_, AppError>`; axum turns the error
/// into a response through `IntoResponse`. Every error body is `{"error": "<message>"}`.
#[derive(Debug)]
pub enum AppError {
    /// 400 — the request body isn't usable JSON. Holds the message for the client.
    BadRequest(String),
    /// 422 — well-formed JSON that breaks an input rule. Holds the message for the client.
    Validation(String),
    /// 404 — the task doesn't exist.
    NotFound,
    /// 500 — a failure the client can't fix. Holds the source error for the logs, never for the client.
    Internal(Box<dyn std::error::Error + Send + Sync>),
}

/// | Variant | Status | `error` message |
/// |---|---|---|
/// | `BadRequest(msg)` | 400 | `msg` |
/// | `Validation(msg)` | 422 | `msg` |
/// | `NotFound` | 404 | `"task not found"` |
/// | `Internal(source)` | 500 | `"internal server error"` |
///
/// The body is JSON (`Content-Type: application/json`).
/// `Internal` logs `source` with `tracing::error!`, including its `Display` text.
/// The response body must contain none of it.
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        todo!()
    }
}

/// `sqlx::Error::RowNotFound` → `NotFound`. Any other error → `Internal`, wrapping the original error.
impl From<sqlx::Error> for AppError {
    fn from(err: sqlx::Error) -> Self {
        todo!()
    }
}

/// Lets a handler write `let Json(input) = payload?;` with `payload: Result<Json<T>, JsonRejection>`.
///
/// - `JsonRejection::JsonDataError` (valid JSON, wrong shape: missing field, wrong type) → `Validation`.
/// - Every other rejection (syntax error, missing `Content-Type: application/json`, unreadable body) → `BadRequest`.
///
/// The message is the rejection's `body_text()`.
impl From<JsonRejection> for AppError {
    fn from(rejection: JsonRejection) -> Self {
        todo!()
    }
}
