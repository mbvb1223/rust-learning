//! Request and response types with their validation (lesson 10), already implemented.

use serde::{Deserialize, Serialize};

use crate::error::AppError;

pub const MAX_TITLE_CHARS: usize = 200;

/// The JSON shape every endpoint returns: `{"id": 1, "title": "Buy milk", "completed": false}`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, sqlx::FromRow)]
pub struct Task {
    pub id: i64,
    pub title: String,
    pub completed: bool,
}

/// `POST /tasks` body.
#[derive(Debug, Deserialize)]
pub struct CreateTask {
    pub title: String,
}

/// `PATCH /tasks/{id}` body. A missing field and an explicit `null` both mean "leave unchanged".
#[derive(Debug, Default, Deserialize)]
pub struct UpdateTask {
    pub title: Option<String>,
    pub completed: Option<bool>,
}

/// Trims `raw`. The result must be 1..=200 characters (`char`s, not bytes).
///
/// Errors: `Validation("title must not be empty")` or
/// `Validation("title must be at most 200 characters")`.
pub fn validate_title(raw: &str) -> Result<String, AppError> {
    let title = raw.trim();
    if title.is_empty() {
        return Err(AppError::Validation("title must not be empty".to_string()));
    }
    if title.chars().count() > MAX_TITLE_CHARS {
        return Err(AppError::Validation(format!(
            "title must be at most {MAX_TITLE_CHARS} characters"
        )));
    }
    Ok(title.to_string())
}

impl CreateTask {
    /// The trimmed title. Errors as [`validate_title`].
    pub fn validate(self) -> Result<String, AppError> {
        validate_title(&self.title)
    }
}

impl UpdateTask {
    /// The same changes with the title trimmed.
    ///
    /// Errors: `Validation("update must include title or completed")` when both fields
    /// are `None`; otherwise as [`validate_title`].
    pub fn validate(self) -> Result<UpdateTask, AppError> {
        if self.title.is_none() && self.completed.is_none() {
            return Err(AppError::Validation(
                "update must include title or completed".to_string(),
            ));
        }
        let title = self.title.as_deref().map(validate_title).transpose()?;
        Ok(UpdateTask {
            title,
            completed: self.completed,
        })
    }
}
