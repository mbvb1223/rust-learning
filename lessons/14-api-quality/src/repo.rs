//! The SQLite repository (lesson 13), already implemented.
//! Every function fails with `sqlx::Error` only when the database itself fails.

use sqlx::SqlitePool;

use crate::model::{Task, UpdateTask};

/// Inserts a task with `completed = false`. In an empty database the first id is 1.
pub async fn create(pool: &SqlitePool, title: &str) -> Result<Task, sqlx::Error> {
    sqlx::query_as("INSERT INTO tasks (title) VALUES (?) RETURNING id, title, completed")
        .bind(title)
        .fetch_one(pool)
        .await
}

/// All tasks, ordered by id.
pub async fn list(pool: &SqlitePool) -> Result<Vec<Task>, sqlx::Error> {
    sqlx::query_as("SELECT id, title, completed FROM tasks ORDER BY id")
        .fetch_all(pool)
        .await
}

/// `None` if no task has this id.
pub async fn get(pool: &SqlitePool, id: i64) -> Result<Option<Task>, sqlx::Error> {
    sqlx::query_as("SELECT id, title, completed FROM tasks WHERE id = ?")
        .bind(id)
        .fetch_optional(pool)
        .await
}

/// Applies the `Some` fields of `changes`; `None` fields keep their value.
/// Returns the updated task, or `None` if no task has this id.
pub async fn update(
    pool: &SqlitePool,
    id: i64,
    changes: &UpdateTask,
) -> Result<Option<Task>, sqlx::Error> {
    sqlx::query_as(
        "UPDATE tasks
         SET title = COALESCE(?, title), completed = COALESCE(?, completed)
         WHERE id = ?
         RETURNING id, title, completed",
    )
    .bind(changes.title.as_deref())
    .bind(changes.completed)
    .bind(id)
    .fetch_optional(pool)
    .await
}

/// `true` if a task was deleted, `false` if no task has this id.
pub async fn delete(pool: &SqlitePool, id: i64) -> Result<bool, sqlx::Error> {
    let result = sqlx::query("DELETE FROM tasks WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(result.rows_affected() > 0)
}
