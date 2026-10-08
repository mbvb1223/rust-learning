//! Lesson 10 — JSON with Serde. Add the missing `#[serde(...)]` attributes and replace
//! every `todo!()` until `cargo test` passes.

use std::fmt;

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

/// A list response with a count: `{"tasks": [...], "count": 2}`. (`GET /tasks` in lessons 12
/// and 14 returns a bare array; this envelope is here to practise borrowing in a response.)
///
/// Borrows the tasks, so building a response doesn't clone them.
#[derive(Debug, Serialize)]
pub struct TaskList<'a> {
    tasks: &'a [Task],
    count: usize,
}

impl<'a> TaskList<'a> {
    /// `count` is always `tasks.len()`.
    pub fn new(tasks: &'a [Task]) -> Self {
        todo!()
    }
}

/// Error response body: `{"error": "task not found"}`.
///
/// The Rust field is `message`; the JSON key is `error`, in both directions.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ErrorBody {
    pub message: String,
}

/// Raw body of `POST /tasks`: `{"title": "Buy milk"}`.
///
/// Shape only: the title rule is applied later, by `NewTask::try_from`.
/// Any key other than `title` is rejected, e.g. `{"title": "x", "completed": true}`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CreateTaskInput {
    pub title: String,
}

/// Raw body of `PATCH /tasks/{id}`: `{"title"?: "...", "completed"?: bool}`.
///
/// - Deserializing: a missing key and `null` both become `None`. Any other key is rejected.
/// - Serializing (a client building a request): `None` fields are omitted, never written as `null`.
///   `UpdateTaskInput { title: None, completed: Some(true) }` → `{"completed": true}`.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct UpdateTaskInput {
    pub title: Option<String>,
    pub completed: Option<bool>,
}

/// One row of the old PHP app's export: `{"taskId": 7, "taskTitle": "Buy milk", "isDone": true}`.
///
/// Every JSON key is the camelCase form of the field name. Rows exported before `isDone`
/// existed omit it; a missing `isDone` means `false`. Other keys are ignored.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct LegacyTask {
    pub task_id: i64,
    pub task_title: String,
    pub is_done: bool,
}

impl From<LegacyTask> for Task {
    /// `task_id` → `id`, `task_title` → `title` (unchanged, not validated), `is_done` → `completed`.
    fn from(legacy: LegacyTask) -> Self {
        todo!()
    }
}

/// A domain rule failed. The JSON itself had the right shape.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidationError {
    /// The title is empty after trimming.
    EmptyTitle,
    /// The trimmed title is longer than [`MAX_TITLE_CHARS`]; `chars` is its length in `char`s.
    TitleTooLong { chars: usize },
    /// An update with neither `title` nor `completed`.
    EmptyUpdate,
}

impl fmt::Display for ValidationError {
    /// - `EmptyTitle` → `title must not be empty`
    /// - `TitleTooLong { chars: 201 }` → `title must be at most 200 characters (got 201)`
    /// - `EmptyUpdate` → `update must include title or completed`
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        todo!()
    }
}

impl std::error::Error for ValidationError {}

/// Trims `raw` with `str::trim`, then measures it in `char`s, not bytes.
///
/// Returns the trimmed title, `EmptyTitle` if it is empty, or `TitleTooLong` if it has
/// more than [`MAX_TITLE_CHARS`] chars.
pub fn validate_title(raw: &str) -> Result<String, ValidationError> {
    todo!()
}

/// A create request that passed validation.
///
/// The field is private, so `NewTask::try_from` is the only way to build one:
/// holding a `NewTask` proves the title is valid.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewTask {
    title: String,
}

impl NewTask {
    /// The trimmed, validated title.
    pub fn title(&self) -> &str {
        todo!()
    }

    /// The stored task: the given `id`, this title, and `completed: false`.
    pub fn into_task(self, id: i64) -> Task {
        todo!()
    }
}

impl TryFrom<CreateTaskInput> for NewTask {
    type Error = ValidationError;

    /// Applies [`validate_title`] to `input.title`.
    fn try_from(input: CreateTaskInput) -> Result<Self, Self::Error> {
        todo!()
    }
}

/// An update that passed validation: at least one field is `Some`, and a `Some` title is
/// trimmed and valid.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskPatch {
    title: Option<String>,
    completed: Option<bool>,
}

impl TaskPatch {
    /// The new title, if this patch changes it.
    pub fn title(&self) -> Option<&str> {
        todo!()
    }

    /// The new completion status, if this patch changes it.
    pub fn completed(&self) -> Option<bool> {
        todo!()
    }

    /// Overwrites only the fields this patch sets. `task.id` never changes.
    pub fn apply(self, task: &mut Task) {
        todo!()
    }
}

impl TryFrom<UpdateTaskInput> for TaskPatch {
    type Error = ValidationError;

    /// `EmptyUpdate` if both fields are `None`. Otherwise a `Some` title goes through
    /// [`validate_title`], even when `completed` is also set.
    fn try_from(input: UpdateTaskInput) -> Result<Self, Self::Error> {
        todo!()
    }
}

/// Everything that can go wrong handling a task request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ApiError {
    /// The body isn't JSON: bad syntax or unexpected end of input.
    ///
    /// `message` is the serde_json error's `Display` text (it already ends with the
    /// position); `line` and `column` are its `line()` and `column()`.
    MalformedJson {
        message: String,
        line: usize,
        column: usize,
    },
    /// The body is valid JSON of the wrong shape: wrong type, missing field, unknown field,
    /// or not an object (e.g. `true`).
    ///
    /// `message`, `line`, and `column` are filled exactly as for `MalformedJson`.
    InvalidBody {
        message: String,
        line: usize,
        column: usize,
    },
    /// The body had the right shape but broke a domain rule.
    Validation(ValidationError),
    /// No task has the requested id.
    NotFound,
}

impl ApiError {
    /// HTTP status: `MalformedJson` → 400, `InvalidBody` and `Validation` → 422, `NotFound` → 404.
    pub fn status(&self) -> u16 {
        todo!()
    }

    /// The response body: `{"error": <this error's Display text>}`.
    pub fn body(&self) -> ErrorBody {
        todo!()
    }
}

impl fmt::Display for ApiError {
    /// - `MalformedJson` → `malformed JSON: <message>`
    /// - `InvalidBody` → `invalid body: <message>`
    /// - `Validation(e)` → the `Display` text of `e`
    /// - `NotFound` → `task not found`
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        todo!()
    }
}

impl std::error::Error for ApiError {}

impl From<serde_json::Error> for ApiError {
    /// Split by `err.classify()`: `Category::Data` → `InvalidBody`; `Syntax`, `Eof`, and `Io` →
    /// `MalformedJson`. The same split as Axum's `Json` extractor.
    fn from(err: serde_json::Error) -> Self {
        todo!()
    }
}

impl From<ValidationError> for ApiError {
    /// Always `Validation`.
    fn from(err: ValidationError) -> Self {
        todo!()
    }
}

/// Deserializes a `POST /tasks` body into a [`CreateTaskInput`], then validates it into a
/// [`NewTask`].
pub fn parse_create(body: &str) -> Result<NewTask, ApiError> {
    todo!()
}

/// Deserializes a `PATCH /tasks/{id}` body into an [`UpdateTaskInput`], then validates it into
/// a [`TaskPatch`].
pub fn parse_update(body: &str) -> Result<TaskPatch, ApiError> {
    todo!()
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    fn task(id: i64, title: &str, completed: bool) -> Task {
        Task {
            id,
            title: title.to_string(),
            completed,
        }
    }

    fn malformed_message(err: ApiError) -> String {
        match err {
            ApiError::MalformedJson { message, .. } => message,
            other => panic!("expected MalformedJson, got {other:?}"),
        }
    }

    fn invalid_body_message(err: ApiError) -> String {
        match err {
            ApiError::InvalidBody { message, .. } => message,
            other => panic!("expected InvalidBody, got {other:?}"),
        }
    }

    #[test]
    fn task_serializes_to_the_contract_shape() {
        let value = serde_json::to_value(task(1, "Buy milk", false)).unwrap();
        assert_eq!(
            value,
            json!({"id": 1, "title": "Buy milk", "completed": false})
        );

        let parsed: Task =
            serde_json::from_str(r#"{"completed": true, "title": "Walk", "id": 2}"#).unwrap();
        assert_eq!(parsed, task(2, "Walk", true));
    }

    #[test]
    fn task_list_serializes_tasks_and_count() {
        let tasks = vec![task(1, "Buy milk", false), task(2, "Walk", true)];
        assert_eq!(
            serde_json::to_value(TaskList::new(&tasks)).unwrap(),
            json!({
                "tasks": [
                    {"id": 1, "title": "Buy milk", "completed": false},
                    {"id": 2, "title": "Walk", "completed": true}
                ],
                "count": 2
            })
        );
        assert_eq!(
            serde_json::to_value(TaskList::new(&tasks[1..])).unwrap(),
            json!({"tasks": [{"id": 2, "title": "Walk", "completed": true}], "count": 1})
        );
        assert_eq!(
            serde_json::to_value(TaskList::new(&[])).unwrap(),
            json!({"tasks": [], "count": 0})
        );
    }

    #[test]
    fn error_body_uses_the_error_key() {
        let body = ErrorBody {
            message: "task not found".to_string(),
        };
        assert_eq!(
            serde_json::to_value(&body).unwrap(),
            json!({"error": "task not found"})
        );
        assert_eq!(
            serde_json::from_value::<ErrorBody>(json!({"error": "task not found"})).unwrap(),
            body
        );
    }

    #[test]
    fn create_input_reads_the_title() {
        let input: CreateTaskInput = serde_json::from_value(json!({"title": "Buy milk"})).unwrap();
        assert_eq!(
            input,
            CreateTaskInput {
                title: "Buy milk".to_string()
            }
        );
    }

    #[test]
    fn create_input_rejects_unknown_fields() {
        let err =
            serde_json::from_value::<CreateTaskInput>(json!({"title": "x", "completed": true}))
                .unwrap_err();
        assert!(
            err.to_string().contains("unknown field `completed`"),
            "{err}"
        );
    }

    #[test]
    fn update_input_fields_are_optional() {
        let empty: UpdateTaskInput = serde_json::from_value(json!({})).unwrap();
        assert_eq!(empty, UpdateTaskInput::default());

        let both: UpdateTaskInput =
            serde_json::from_value(json!({"title": "Walk", "completed": true})).unwrap();
        assert_eq!(
            both,
            UpdateTaskInput {
                title: Some("Walk".to_string()),
                completed: Some(true),
            }
        );
    }

    #[test]
    fn update_input_treats_null_as_absent() {
        let input: UpdateTaskInput =
            serde_json::from_value(json!({"title": null, "completed": false})).unwrap();
        assert_eq!(
            input,
            UpdateTaskInput {
                title: None,
                completed: Some(false),
            }
        );
    }

    #[test]
    fn update_input_rejects_unknown_fields() {
        let err = serde_json::from_value::<UpdateTaskInput>(json!({"done": true})).unwrap_err();
        assert!(err.to_string().contains("unknown field `done`"), "{err}");
    }

    #[test]
    fn update_input_omits_none_fields_when_serialized() {
        let only_completed = UpdateTaskInput {
            title: None,
            completed: Some(true),
        };
        assert_eq!(
            serde_json::to_value(&only_completed).unwrap(),
            json!({"completed": true})
        );

        let only_title = UpdateTaskInput {
            title: Some("Walk".to_string()),
            completed: None,
        };
        assert_eq!(
            serde_json::to_value(&only_title).unwrap(),
            json!({"title": "Walk"})
        );

        assert_eq!(
            serde_json::to_value(UpdateTaskInput::default()).unwrap(),
            json!({})
        );
    }

    #[test]
    fn legacy_task_reads_camel_case_keys() {
        let legacy: LegacyTask = serde_json::from_value(
            json!({"taskId": 7, "taskTitle": "Buy milk", "isDone": true, "createdBy": "admin"}),
        )
        .unwrap();
        assert_eq!(Task::from(legacy), task(7, "Buy milk", true));
    }

    #[test]
    fn legacy_task_defaults_missing_is_done_to_false() {
        let legacy: LegacyTask =
            serde_json::from_value(json!({"taskId": 8, "taskTitle": " Walk "})).unwrap();
        assert_eq!(Task::from(legacy), task(8, " Walk ", false));
    }

    #[test]
    fn legacy_task_rejects_snake_case_keys() {
        let err = serde_json::from_value::<LegacyTask>(
            json!({"task_id": 7, "task_title": "Buy milk", "is_done": true}),
        )
        .unwrap_err();
        assert!(err.to_string().contains("missing field `taskId`"), "{err}");
    }

    #[test]
    fn validate_title_trims_whitespace() {
        assert_eq!(validate_title("Buy milk"), Ok("Buy milk".to_string()));
        assert_eq!(
            validate_title(" \t Buy milk \n"),
            Ok("Buy milk".to_string())
        );
        assert_eq!(validate_title("  a  b  "), Ok("a  b".to_string()));
    }

    #[test]
    fn validate_title_rejects_empty_and_blank_titles() {
        assert_eq!(validate_title(""), Err(ValidationError::EmptyTitle));
        assert_eq!(validate_title(" \t\n "), Err(ValidationError::EmptyTitle));
    }

    #[test]
    fn validate_title_limits_length_in_chars_not_bytes() {
        assert_eq!(validate_title(&"a".repeat(200)), Ok("a".repeat(200)));
        assert_eq!(
            validate_title(&"a".repeat(201)),
            Err(ValidationError::TitleTooLong { chars: 201 })
        );

        // 200 chars, 400 bytes.
        assert_eq!(validate_title(&"é".repeat(200)), Ok("é".repeat(200)));
        assert_eq!(
            validate_title(&"é".repeat(250)),
            Err(ValidationError::TitleTooLong { chars: 250 })
        );
    }

    #[test]
    fn validate_title_measures_after_trimming() {
        let padded = format!("   {}   ", "a".repeat(200));
        assert_eq!(validate_title(&padded), Ok("a".repeat(200)));

        let padded = format!("   {}   ", "a".repeat(201));
        assert_eq!(
            validate_title(&padded),
            Err(ValidationError::TitleTooLong { chars: 201 })
        );
    }

    #[test]
    fn validation_errors_have_human_messages() {
        assert_eq!(
            ValidationError::EmptyTitle.to_string(),
            "title must not be empty"
        );
        assert_eq!(
            ValidationError::TitleTooLong { chars: 201 }.to_string(),
            "title must be at most 200 characters (got 201)"
        );
        assert_eq!(
            ValidationError::EmptyUpdate.to_string(),
            "update must include title or completed"
        );
    }

    #[test]
    fn new_task_comes_from_valid_input() {
        let input = CreateTaskInput {
            title: "  Buy milk  ".to_string(),
        };
        let new_task = NewTask::try_from(input).unwrap();
        assert_eq!(new_task.title(), "Buy milk");
        assert_eq!(new_task.into_task(7), task(7, "Buy milk", false));
    }

    #[test]
    fn new_task_rejects_invalid_titles() {
        let blank = CreateTaskInput {
            title: "   ".to_string(),
        };
        assert_eq!(NewTask::try_from(blank), Err(ValidationError::EmptyTitle));

        let long = CreateTaskInput {
            title: "x".repeat(300),
        };
        assert_eq!(
            NewTask::try_from(long),
            Err(ValidationError::TitleTooLong { chars: 300 })
        );
    }

    #[test]
    fn patch_requires_at_least_one_field() {
        assert_eq!(
            TaskPatch::try_from(UpdateTaskInput::default()),
            Err(ValidationError::EmptyUpdate)
        );
    }

    #[test]
    fn patch_validates_the_title() {
        let patch = TaskPatch::try_from(UpdateTaskInput {
            title: Some("  Walk  ".to_string()),
            completed: None,
        })
        .unwrap();
        assert_eq!(patch.title(), Some("Walk"));
        assert_eq!(patch.completed(), None);

        let blank_title_and_completed = UpdateTaskInput {
            title: Some(" ".to_string()),
            completed: Some(true),
        };
        assert_eq!(
            TaskPatch::try_from(blank_title_and_completed),
            Err(ValidationError::EmptyTitle)
        );
    }

    #[test]
    fn patch_with_only_completed() {
        let patch = TaskPatch::try_from(UpdateTaskInput {
            title: None,
            completed: Some(false),
        })
        .unwrap();
        assert_eq!(patch.title(), None);
        assert_eq!(patch.completed(), Some(false));
    }

    #[test]
    fn patch_changes_only_the_fields_it_sets() {
        let patch = |title: Option<&str>, completed: Option<bool>| {
            TaskPatch::try_from(UpdateTaskInput {
                title: title.map(str::to_string),
                completed,
            })
            .unwrap()
        };

        let mut current = task(3, "Buy milk", false);
        patch(None, Some(true)).apply(&mut current);
        assert_eq!(current, task(3, "Buy milk", true));

        patch(Some("Buy oat milk"), None).apply(&mut current);
        assert_eq!(current, task(3, "Buy oat milk", true));

        patch(Some("Walk"), Some(false)).apply(&mut current);
        assert_eq!(current, task(3, "Walk", false));
    }

    #[test]
    fn api_errors_map_to_statuses() {
        let malformed = ApiError::MalformedJson {
            message: "expected value at line 1 column 1".to_string(),
            line: 1,
            column: 1,
        };
        assert_eq!(malformed.status(), 400);
        let invalid = ApiError::InvalidBody {
            message: "missing field `title` at line 1 column 2".to_string(),
            line: 1,
            column: 2,
        };
        assert_eq!(invalid.status(), 422);
        assert_eq!(
            ApiError::Validation(ValidationError::EmptyTitle).status(),
            422
        );
        assert_eq!(ApiError::NotFound.status(), 404);
    }

    #[test]
    fn api_errors_render_error_bodies() {
        let malformed = ApiError::MalformedJson {
            message: "expected value at line 1 column 1".to_string(),
            line: 1,
            column: 1,
        };
        assert_eq!(
            malformed.to_string(),
            "malformed JSON: expected value at line 1 column 1"
        );
        assert_eq!(
            serde_json::to_value(malformed.body()).unwrap(),
            json!({"error": "malformed JSON: expected value at line 1 column 1"})
        );
        let invalid = ApiError::InvalidBody {
            message: "missing field `title` at line 1 column 2".to_string(),
            line: 1,
            column: 2,
        };
        assert_eq!(
            serde_json::to_value(invalid.body()).unwrap(),
            json!({"error": "invalid body: missing field `title` at line 1 column 2"})
        );
        assert_eq!(
            serde_json::to_value(ApiError::Validation(ValidationError::EmptyUpdate).body())
                .unwrap(),
            json!({"error": "update must include title or completed"})
        );
        assert_eq!(
            serde_json::to_value(ApiError::NotFound.body()).unwrap(),
            json!({"error": "task not found"})
        );
    }

    #[test]
    fn api_error_from_serde_json_error_keeps_message_and_position() {
        let err = serde_json::from_str::<Task>("{\n  \"id\": 1,\n  \"title\": tru").unwrap_err();
        let expected = ApiError::MalformedJson {
            message: err.to_string(),
            line: err.line(),
            column: err.column(),
        };
        assert_eq!(ApiError::from(err), expected);
    }

    #[test]
    fn api_error_from_serde_json_data_error_is_invalid_body() {
        let err = serde_json::from_str::<Task>("{\n  \"id\": \"1\"\n}").unwrap_err();
        let expected = ApiError::InvalidBody {
            message: err.to_string(),
            line: err.line(),
            column: err.column(),
        };
        assert_eq!(ApiError::from(err), expected);
    }

    #[test]
    fn api_error_from_validation_error() {
        assert_eq!(
            ApiError::from(ValidationError::EmptyTitle),
            ApiError::Validation(ValidationError::EmptyTitle)
        );
    }

    #[test]
    fn parse_create_accepts_a_valid_body() {
        let new_task = parse_create(r#"{"title": "  Buy milk  "}"#).unwrap();
        assert_eq!(new_task.title(), "Buy milk");
    }

    #[test]
    fn parse_create_reports_syntax_errors_with_position() {
        let err = parse_create("{\n  \"title\": \"Buy milk\",\n}").unwrap_err();
        assert_eq!(err.status(), 400);
        let ApiError::MalformedJson {
            message,
            line,
            column,
        } = &err
        else {
            panic!("expected MalformedJson, got {err:?}");
        };
        assert!(message.contains("trailing comma"), "{message}");
        assert_eq!((*line, *column), (3, 1));
        assert!(
            err.to_string()
                .starts_with("malformed JSON: trailing comma")
        );
    }

    #[test]
    fn parse_create_treats_empty_body_as_malformed() {
        let err = parse_create("").unwrap_err();
        assert_eq!(err.status(), 400);
        assert!(malformed_message(err).contains("EOF while parsing"));
    }

    #[test]
    fn parse_create_reports_wrong_shapes_as_invalid_body() {
        let err = parse_create("{}").unwrap_err();
        assert_eq!(err.status(), 422);
        assert!(
            err.to_string()
                .starts_with("invalid body: missing field `title`")
        );

        assert!(
            invalid_body_message(parse_create(r#"{"title": 42}"#).unwrap_err())
                .contains("invalid type")
        );
        assert!(
            invalid_body_message(
                parse_create(r#"{"title": "x", "completed": false}"#).unwrap_err()
            )
            .contains("unknown field `completed`")
        );
        assert!(invalid_body_message(parse_create("true").unwrap_err()).contains("invalid type"));
    }

    #[test]
    fn parse_create_reports_validation_failures_as_422() {
        let err = parse_create(r#"{"title": "   "}"#).unwrap_err();
        assert_eq!(err, ApiError::Validation(ValidationError::EmptyTitle));
        assert_eq!(err.status(), 422);
        assert_eq!(
            serde_json::to_value(err.body()).unwrap(),
            json!({"error": "title must not be empty"})
        );
    }

    #[test]
    fn parse_update_accepts_partial_bodies() {
        let patch = parse_update(r#"{"completed": true}"#).unwrap();
        assert_eq!((patch.title(), patch.completed()), (None, Some(true)));

        let patch = parse_update(r#"{"title": " Walk ", "completed": null}"#).unwrap();
        assert_eq!((patch.title(), patch.completed()), (Some("Walk"), None));
    }

    #[test]
    fn parse_update_rejects_empty_updates() {
        for body in [
            "{}",
            r#"{"title": null}"#,
            r#"{"title": null, "completed": null}"#,
        ] {
            assert_eq!(
                parse_update(body),
                Err(ApiError::Validation(ValidationError::EmptyUpdate)),
                "{body}"
            );
        }
    }

    #[test]
    fn parse_update_splits_malformed_json_from_wrong_shapes() {
        let not_json = parse_update(r#"{"completed": yes}"#).unwrap_err();
        assert_eq!(not_json.status(), 400);
        assert!(malformed_message(not_json).contains("expected value"));

        let wrong_type = parse_update(r#"{"completed": "yes"}"#).unwrap_err();
        assert_eq!(wrong_type.status(), 422);
        assert!(invalid_body_message(wrong_type).contains("invalid type"));

        assert!(
            invalid_body_message(parse_update(r#"{"done": true}"#).unwrap_err())
                .contains("unknown field `done`")
        );
        assert!(invalid_body_message(parse_update("true").unwrap_err()).contains("invalid type"));
    }

    #[test]
    fn create_then_update_flow() {
        let mut stored = parse_create(r#"{"title": "Buy milk"}"#)
            .unwrap()
            .into_task(1);
        parse_update(r#"{"completed": true}"#)
            .unwrap()
            .apply(&mut stored);

        let tasks = [stored];
        assert_eq!(
            serde_json::to_value(TaskList::new(&tasks)).unwrap(),
            json!({"tasks": [{"id": 1, "title": "Buy milk", "completed": true}], "count": 1})
        );
    }
}
