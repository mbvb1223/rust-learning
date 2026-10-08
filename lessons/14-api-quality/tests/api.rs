//! Every endpoint, success and failure, through the real router. Each test builds its own
//! `TestApp`, so each test has its own database: ids start at 1 and lists start empty, even
//! though tests run in parallel.

mod common;

use axum::body::Body;
use axum::http::{Method, Request, StatusCode};
use common::TestApp;
use serde_json::{Value, json};

async fn create(app: &TestApp, title: &str) -> Value {
    let response = app
        .post_json("/tasks", &json!({ "title": title }).to_string())
        .await;
    assert_eq!(response.status, StatusCode::CREATED);
    response.json()
}

fn assert_error(response: &common::TestResponse, status: StatusCode, message: &str) {
    assert_eq!(response.status, status);
    assert_eq!(response.content_type.as_deref(), Some("application/json"));
    assert_eq!(response.json(), json!({ "error": message }));
}

/// For rejections whose wording comes from axum/serde: check the shape, not the text.
fn assert_error_shape(response: &common::TestResponse, status: StatusCode) {
    assert_eq!(response.status, status);
    assert_eq!(response.content_type.as_deref(), Some("application/json"));
    let body = response.json();
    let message = body["error"].as_str().expect("`error` is a string");
    assert!(!message.is_empty());
    assert_eq!(body.as_object().unwrap().len(), 1, "only `error`: {body}");
}

// ---- POST /tasks ----

#[tokio::test]
async fn create_returns_201_with_the_task() {
    let app = TestApp::new().await;
    let response = app.post_json("/tasks", r#"{"title": "Buy milk"}"#).await;

    assert_eq!(response.status, StatusCode::CREATED);
    assert_eq!(response.content_type.as_deref(), Some("application/json"));
    assert_eq!(
        response.json(),
        json!({"id": 1, "title": "Buy milk", "completed": false})
    );
}

#[tokio::test]
async fn create_trims_the_title() {
    let app = TestApp::new().await;
    assert_eq!(create(&app, "  Buy milk \n").await["title"], "Buy milk");
}

#[tokio::test]
async fn create_counts_title_length_in_chars() {
    let app = TestApp::new().await;
    // 200 chars, 400 bytes.
    let title = "é".repeat(200);
    assert_eq!(create(&app, &title).await["title"], title.as_str());

    let response = app
        .post_json("/tasks", &json!({ "title": "a".repeat(201) }).to_string())
        .await;
    assert_error(
        &response,
        StatusCode::UNPROCESSABLE_ENTITY,
        "title must be at most 200 characters",
    );
}

#[tokio::test]
async fn create_rejects_a_blank_title() {
    let app = TestApp::new().await;
    let response = app.post_json("/tasks", r#"{"title": "   "}"#).await;
    assert_error(
        &response,
        StatusCode::UNPROCESSABLE_ENTITY,
        "title must not be empty",
    );

    assert_eq!(app.get("/tasks").await.json(), json!([]));
}

#[tokio::test]
async fn create_rejects_malformed_json_with_400() {
    let app = TestApp::new().await;
    let response = app.post_json("/tasks", r#"{"title": "Buy milk""#).await;
    assert_error_shape(&response, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn create_rejects_a_missing_or_mistyped_title_with_422() {
    let app = TestApp::new().await;
    for body in ["{}", r#"{"title": 5}"#, r#"{"title": null}"#] {
        let response = app.post_json("/tasks", body).await;
        assert_error_shape(&response, StatusCode::UNPROCESSABLE_ENTITY);
    }
}

#[tokio::test]
async fn create_requires_a_json_content_type() {
    let app = TestApp::new().await;
    let request = Request::builder()
        .method(Method::POST)
        .uri("/tasks")
        .body(Body::from(r#"{"title": "Buy milk"}"#))
        .unwrap();
    assert_error_shape(&app.send(request).await, StatusCode::BAD_REQUEST);
}

// ---- GET /tasks ----

#[tokio::test]
async fn list_is_empty_in_a_fresh_database() {
    let app = TestApp::new().await;
    let response = app.get("/tasks").await;
    assert_eq!(response.status, StatusCode::OK);
    assert_eq!(response.json(), json!([]));
}

#[tokio::test]
async fn list_returns_tasks_in_id_order() {
    let app = TestApp::new().await;
    create(&app, "first").await;
    create(&app, "second").await;

    let response = app.get("/tasks").await;
    assert_eq!(response.status, StatusCode::OK);
    assert_eq!(
        response.json(),
        json!([
            {"id": 1, "title": "first", "completed": false},
            {"id": 2, "title": "second", "completed": false},
        ])
    );
}

// ---- GET /tasks/{id} ----

#[tokio::test]
async fn get_returns_the_task() {
    let app = TestApp::new().await;
    create(&app, "first").await;
    create(&app, "second").await;

    let response = app.get("/tasks/2").await;
    assert_eq!(response.status, StatusCode::OK);
    assert_eq!(
        response.json(),
        json!({"id": 2, "title": "second", "completed": false})
    );
}

#[tokio::test]
async fn get_unknown_task_is_404() {
    let app = TestApp::new().await;
    assert_error(
        &app.get("/tasks/42").await,
        StatusCode::NOT_FOUND,
        "task not found",
    );
}

// ---- PATCH /tasks/{id} ----

#[tokio::test]
async fn update_changes_the_given_fields() {
    let app = TestApp::new().await;
    create(&app, "Buy milk").await;

    let response = app
        .patch_json("/tasks/1", r#"{"title": " Buy oat milk "}"#)
        .await;
    assert_eq!(response.status, StatusCode::OK);
    assert_eq!(
        response.json(),
        json!({"id": 1, "title": "Buy oat milk", "completed": false})
    );

    let response = app
        .patch_json("/tasks/1", r#"{"title": "Buy cheese", "completed": true}"#)
        .await;
    assert_eq!(response.status, StatusCode::OK);
    let expected = json!({"id": 1, "title": "Buy cheese", "completed": true});
    assert_eq!(response.json(), expected);
    assert_eq!(app.get("/tasks/1").await.json(), expected);
}

#[tokio::test]
async fn update_without_any_field_is_422() {
    let app = TestApp::new().await;
    create(&app, "Buy milk").await;

    for body in ["{}", r#"{"title": null}"#] {
        assert_error(
            &app.patch_json("/tasks/1", body).await,
            StatusCode::UNPROCESSABLE_ENTITY,
            "update must include title or completed",
        );
    }
}

#[tokio::test]
async fn update_rejects_an_invalid_title() {
    let app = TestApp::new().await;
    create(&app, "Buy milk").await;

    assert_error(
        &app.patch_json("/tasks/1", r#"{"title": ""}"#).await,
        StatusCode::UNPROCESSABLE_ENTITY,
        "title must not be empty",
    );
    let too_long = json!({ "title": "a".repeat(201) }).to_string();
    assert_error(
        &app.patch_json("/tasks/1", &too_long).await,
        StatusCode::UNPROCESSABLE_ENTITY,
        "title must be at most 200 characters",
    );
}

#[tokio::test]
async fn update_rejects_malformed_json_with_400() {
    let app = TestApp::new().await;
    create(&app, "Buy milk").await;
    assert_error_shape(
        &app.patch_json("/tasks/1", "completed=true").await,
        StatusCode::BAD_REQUEST,
    );
}

#[tokio::test]
async fn update_unknown_task_is_404() {
    let app = TestApp::new().await;
    assert_error(
        &app.patch_json("/tasks/42", r#"{"completed": true}"#).await,
        StatusCode::NOT_FOUND,
        "task not found",
    );
}

#[tokio::test]
async fn update_validates_before_looking_up_the_task() {
    let app = TestApp::new().await;
    assert_error(
        &app.patch_json("/tasks/42", "{}").await,
        StatusCode::UNPROCESSABLE_ENTITY,
        "update must include title or completed",
    );
}

// ---- DELETE /tasks/{id} ----

#[tokio::test]
async fn delete_returns_204_and_removes_the_task() {
    let app = TestApp::new().await;
    create(&app, "first").await;
    create(&app, "second").await;

    let response = app.delete("/tasks/1").await;
    assert_eq!(response.status, StatusCode::NO_CONTENT);
    assert!(response.body.is_empty());

    assert_error(
        &app.get("/tasks/1").await,
        StatusCode::NOT_FOUND,
        "task not found",
    );
    assert_eq!(
        app.get("/tasks").await.json(),
        json!([{"id": 2, "title": "second", "completed": false}])
    );
}

#[tokio::test]
async fn delete_unknown_task_is_404() {
    let app = TestApp::new().await;
    assert_error(
        &app.delete("/tasks/42").await,
        StatusCode::NOT_FOUND,
        "task not found",
    );
}

// ---- failures inside the server ----

#[tokio::test]
async fn database_errors_are_500_without_details() {
    let app = TestApp::new().await;
    sqlx::query("DROP TABLE tasks")
        .execute(&app.pool)
        .await
        .unwrap();

    // SQLite now says "no such table: tasks". The client must not see that.
    for response in [
        app.get("/tasks").await,
        app.get("/tasks/1").await,
        app.post_json("/tasks", r#"{"title": "Buy milk"}"#).await,
        app.delete("/tasks/1").await,
    ] {
        assert_error(
            &response,
            StatusCode::INTERNAL_SERVER_ERROR,
            "internal server error",
        );
    }
}

// ---- your turn: replace each `todo!()` with a test (Exercise step 8) ----

/// `PATCH /tasks/{id}` with only `{"completed": true}` → 200; the task is completed and its title
/// is unchanged, both in the response and in a later `GET /tasks/{id}`.
#[tokio::test]
async fn update_of_completed_alone_keeps_the_title() {
    todo!()
}

/// A `PATCH` that fails validation (pick any 422 case) changes nothing: a later `GET` returns the
/// task exactly as it was created.
#[tokio::test]
async fn rejected_update_changes_nothing() {
    todo!()
}
