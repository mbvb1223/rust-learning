//! One handler per endpoint. Each returns `Result<_, AppError>`: use `?` on rejected bodies
//! and repository calls, and turn a missing task into `AppError::NotFound`.

use axum::Json;
use axum::extract::rejection::JsonRejection;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use sqlx::SqlitePool;

use crate::error::AppError;
use crate::model::{CreateTask, Task, UpdateTask};

/// `POST /tasks` → `201 Created` with the new task.
///
/// Errors: rejected body → 400/422 (see `From<JsonRejection>`); invalid title → 422
/// (`CreateTask::validate`); database failure → 500.
pub async fn create_task(
    State(pool): State<SqlitePool>,
    payload: Result<Json<CreateTask>, JsonRejection>,
) -> Result<(StatusCode, Json<Task>), AppError> {
    todo!()
}

/// `GET /tasks` → `200 OK` with every task, ordered by id (`[]` when there are none).
pub async fn list_tasks(State(pool): State<SqlitePool>) -> Result<Json<Vec<Task>>, AppError> {
    todo!()
}

/// `GET /tasks/{id}` → `200 OK` with the task, or 404 if it doesn't exist.
pub async fn get_task(
    State(pool): State<SqlitePool>,
    Path(id): Path<i64>,
) -> Result<Json<Task>, AppError> {
    todo!()
}

/// `PATCH /tasks/{id}` → `200 OK` with the updated task.
///
/// Errors: rejected body → 400/422; `UpdateTask::validate` fails → 422; no such task → 404.
/// Validation runs before the database is touched, so an invalid body for a missing task is 422.
pub async fn update_task(
    State(pool): State<SqlitePool>,
    Path(id): Path<i64>,
    payload: Result<Json<UpdateTask>, JsonRejection>,
) -> Result<Json<Task>, AppError> {
    todo!()
}

/// `DELETE /tasks/{id}` → `204 No Content` with an empty body, or 404 if the task doesn't exist.
pub async fn delete_task(
    State(pool): State<SqlitePool>,
    Path(id): Path<i64>,
) -> Result<StatusCode, AppError> {
    todo!()
}
