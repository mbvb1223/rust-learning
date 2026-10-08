//! Lesson 12 — HTTP with Axum. Replace every `todo!()` until `cargo test` passes.

use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};

/// Maximum title length in `char`s, measured after trimming.
pub const MAX_TITLE_CHARS: usize = 200;

/// A stored task, exactly as the API returns it:
/// `{"id": 1, "title": "Buy milk", "completed": false}`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Task {
    pub id: i64,
    pub title: String,
    pub completed: bool,
}

/// Body of `POST /tasks`: `{"title": "Buy milk"}`.
#[derive(Debug, Deserialize)]
pub struct CreateTask {
    pub title: String,
}

/// Body of `PATCH /tasks/{id}`: `{"title"?: "...", "completed"?: bool}`.
/// A missing key and `null` both become `None`.
#[derive(Debug, Deserialize)]
pub struct UpdateTask {
    pub title: Option<String>,
    pub completed: Option<bool>,
}

/// An error a handler returns. Every variant becomes a JSON response
/// (`content-type: application/json`) with the body `{"error": "<message>"}`.
#[derive(Debug, PartialEq, Eq)]
pub enum ApiError {
    /// 422 Unprocessable Entity; the message is the `error` value.
    Validation(String),
    /// 404 Not Found with `{"error": "task not found"}`.
    NotFound,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        todo!()
    }
}

/// Trims `raw` and returns the trimmed title as an owned `String`.
///
/// Fails with `ApiError::Validation` and exactly this message:
/// - empty after trimming: `"title must not be empty"`
/// - more than `MAX_TITLE_CHARS` characters after trimming: `"title must be at most 200 characters"`
///
/// Count `char`s, not bytes: `"é"` is one character but two bytes.
pub fn validate_title(raw: &str) -> Result<String, ApiError> {
    todo!()
}

/// In-memory task storage. Plain synchronous Rust: no HTTP and no locking.
/// `AppState` wraps it in `Arc<Mutex<…>>` to share it between requests.
#[derive(Debug, Default)]
pub struct Store {
    // A `BTreeMap` iterates in key order, so listing by id needs no sort.
    tasks: BTreeMap<i64, Task>,
    // Id of the most recently created task; 0 before the first one.
    last_id: i64,
}

impl Store {
    /// Stores a new task with `completed: false` and returns a copy of it.
    ///
    /// Ids start at 1 and increase by 1 per created task. They are never reused,
    /// even after the task with the highest id is deleted.
    /// Doesn't validate `title`: callers pass a title that already went through `validate_title`.
    pub fn create(&mut self, title: String) -> Task {
        todo!()
    }

    /// Copies of all tasks, ordered by id ascending.
    pub fn list(&self) -> Vec<Task> {
        todo!()
    }

    /// The task with `id`, borrowed from the store (like `HashMap::get`).
    pub fn get(&self, id: i64) -> Option<&Task> {
        todo!()
    }

    /// Replaces the title if `title` is `Some` and the completion flag if `completed` is `Some`;
    /// a `None` argument leaves that field unchanged. Returns a copy of the updated task,
    /// or `None` (changing nothing) if no task has `id`. Doesn't validate `title`.
    pub fn update(
        &mut self,
        id: i64,
        title: Option<String>,
        completed: Option<bool>,
    ) -> Option<Task> {
        todo!()
    }

    /// Removes the task with `id`. Returns `true` if it existed.
    pub fn delete(&mut self, id: i64) -> bool {
        todo!()
    }
}

/// Shared application state. Axum clones it for every request; cloning an `Arc` copies a
/// pointer and increments a counter, so every request uses the same `Store`.
#[derive(Debug, Clone, Default)]
pub struct AppState {
    pub store: Arc<Mutex<Store>>,
}

/// Builds the router with `state` attached:
///
/// | Request | Handler |
/// |---|---|
/// | `GET /tasks` | `list_tasks` |
/// | `POST /tasks` | `create_task` |
/// | `GET /tasks/{id}` | `get_task` |
/// | `PATCH /tasks/{id}` | `update_task` |
/// | `DELETE /tasks/{id}` | `delete_task` |
pub fn app(state: AppState) -> Router {
    todo!()
}

/// `GET /tasks` → 200 with a JSON array of every task, ordered by id ascending (`[]` when empty).
pub async fn list_tasks(State(state): State<AppState>) -> Json<Vec<Task>> {
    todo!()
}

/// `GET /tasks/{id}` → 200 with the task, or `ApiError::NotFound`.
///
/// A non-integer `{id}` never reaches this function: the `Path` extractor rejects it with 400.
pub async fn get_task(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<Task>, ApiError> {
    todo!()
}

/// `POST /tasks` → 201 Created with the new task.
///
/// The title goes through `validate_title`; if that fails, return its error (422) and store nothing.
///
/// Before this function runs, the `Json` extractor rejects syntactically invalid JSON with 400
/// and valid JSON of the wrong shape (e.g. no `title`, or a number as `title`) with 422.
pub async fn create_task(
    State(state): State<AppState>,
    Json(input): Json<CreateTask>,
) -> Result<(StatusCode, Json<Task>), ApiError> {
    todo!()
}

/// `PATCH /tasks/{id}` → 200 with the updated task.
///
/// Validate the body first, without touching the store, so an invalid body gets 422
/// even when `{id}` doesn't exist:
/// - neither `title` nor `completed` given: `ApiError::Validation("update must include title or completed")`
/// - `title` given: it goes through `validate_title`
///
/// Then `ApiError::NotFound` if no task has `id`. Extractor rejections are the same as for
/// `create_task` (`Json`) and `get_task` (`Path`).
pub async fn update_task(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(input): Json<UpdateTask>,
) -> Result<Json<Task>, ApiError> {
    todo!()
}

/// `DELETE /tasks/{id}` → 204 No Content with an empty body, or `ApiError::NotFound`.
/// A non-integer `{id}` is rejected with 400, as for `get_task`.
pub async fn delete_task(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<StatusCode, ApiError> {
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::header;
    use http_body_util::BodyExt;
    use serde_json::{Value, json};

    fn task(id: i64, title: &str, completed: bool) -> Task {
        Task {
            id,
            title: title.to_string(),
            completed,
        }
    }

    fn validation(message: &str) -> ApiError {
        ApiError::Validation(message.to_string())
    }

    fn store_with(titles: &[&str]) -> Store {
        let mut store = Store::default();
        for title in titles {
            store.create(title.to_string());
        }
        store
    }

    async fn body_json(response: Response) -> Value {
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        serde_json::from_slice(&bytes).expect("body is JSON")
    }

    #[test]
    fn validate_title_trims() {
        assert_eq!(validate_title("Buy milk"), Ok("Buy milk".to_string()));
        assert_eq!(validate_title("  Buy milk \n"), Ok("Buy milk".to_string()));
    }

    #[test]
    fn validate_title_rejects_empty_and_blank() {
        let expected = Err(validation("title must not be empty"));
        assert_eq!(validate_title(""), expected);
        assert_eq!(validate_title("   \t\n"), expected);
    }

    #[test]
    fn validate_title_counts_chars_not_bytes() {
        let too_long = Err(validation("title must be at most 200 characters"));
        assert_eq!(validate_title(&"a".repeat(200)), Ok("a".repeat(200)));
        assert_eq!(validate_title(&"a".repeat(201)), too_long);
        // 200 chars, 400 bytes.
        assert_eq!(validate_title(&"é".repeat(200)), Ok("é".repeat(200)));
        assert_eq!(validate_title(&"é".repeat(201)), too_long);
    }

    #[test]
    fn validate_title_measures_length_after_trimming() {
        let padded = format!("  {}  ", "a".repeat(200));
        assert_eq!(validate_title(&padded), Ok("a".repeat(200)));
    }

    #[test]
    fn create_assigns_ids_from_one() {
        let mut store = Store::default();
        assert_eq!(store.create("a".to_string()), task(1, "a", false));
        assert_eq!(store.create("b".to_string()), task(2, "b", false));
        assert_eq!(store.create("c".to_string()), task(3, "c", false));
    }

    #[test]
    fn list_is_empty_for_a_new_store() {
        assert_eq!(Store::default().list(), vec![]);
    }

    #[test]
    fn list_is_ordered_by_id() {
        let store = store_with(&["first", "second", "third"]);
        assert_eq!(
            store.list(),
            vec![
                task(1, "first", false),
                task(2, "second", false),
                task(3, "third", false),
            ]
        );
    }

    #[test]
    fn get_borrows_a_stored_task() {
        let store = store_with(&["a", "b"]);
        assert_eq!(store.get(2), Some(&task(2, "b", false)));
        assert_eq!(store.get(3), None);
        assert_eq!(store.get(0), None);
    }

    #[test]
    fn update_changes_only_given_fields() {
        let mut store = store_with(&["a"]);

        assert_eq!(store.update(1, None, Some(true)), Some(task(1, "a", true)));
        assert_eq!(
            store.update(1, Some("b".to_string()), None),
            Some(task(1, "b", true))
        );
        assert_eq!(
            store.update(1, Some("c".to_string()), Some(false)),
            Some(task(1, "c", false))
        );
        assert_eq!(store.update(1, None, None), Some(task(1, "c", false)));
        assert_eq!(store.get(1), Some(&task(1, "c", false)));
    }

    #[test]
    fn update_unknown_id_changes_nothing() {
        let mut store = store_with(&["a"]);
        assert_eq!(store.update(2, Some("b".to_string()), Some(true)), None);
        assert_eq!(store.list(), vec![task(1, "a", false)]);
    }

    #[test]
    fn delete_removes_once() {
        let mut store = store_with(&["a", "b"]);
        assert!(store.delete(1));
        assert!(!store.delete(1));
        assert!(!store.delete(99));
        assert_eq!(store.list(), vec![task(2, "b", false)]);
    }

    #[test]
    fn ids_are_not_reused_after_delete() {
        let mut store = store_with(&["a", "b"]);
        assert!(store.delete(2));
        assert_eq!(store.create("c".to_string()).id, 3);
    }

    #[tokio::test]
    async fn validation_error_is_422_json() {
        let response = validation("title must not be empty").into_response();
        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
        assert_eq!(response.headers()[header::CONTENT_TYPE], "application/json");
        assert_eq!(
            body_json(response).await,
            json!({"error": "title must not be empty"})
        );
    }

    #[tokio::test]
    async fn not_found_error_is_404_json() {
        let response = ApiError::NotFound.into_response();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
        assert_eq!(response.headers()[header::CONTENT_TYPE], "application/json");
        assert_eq!(
            body_json(response).await,
            json!({"error": "task not found"})
        );
    }
}
