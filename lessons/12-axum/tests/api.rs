//! Drives the router in-process with `tower::ServiceExt::oneshot`: no port, no network.

use axum::Router;
use axum::body::Body;
use axum::http::{Method, Request, StatusCode, header};
use axum_lesson::{AppState, MAX_TITLE_CHARS, Task, app};
use http_body_util::BodyExt;
use serde_json::{Value, json};
use tower::ServiceExt;

struct TestResponse {
    status: StatusCode,
    content_type: Option<String>,
    body: Vec<u8>,
}

impl TestResponse {
    fn json(&self) -> Value {
        serde_json::from_slice(&self.body).expect("body is JSON")
    }

    fn task(&self) -> Task {
        serde_json::from_slice(&self.body).expect("body is a task")
    }
}

async fn send(app: &Router, method: Method, uri: &str, body: Option<String>) -> TestResponse {
    let mut request = Request::builder().method(method).uri(uri);
    let body = match body {
        Some(json) => {
            request = request.header(header::CONTENT_TYPE, "application/json");
            Body::from(json)
        }
        None => Body::empty(),
    };

    // `oneshot` consumes the router. Cloning it is cheap and shares the same `AppState`.
    let response = app
        .clone()
        .oneshot(request.body(body).unwrap())
        .await
        .unwrap();

    let status = response.status();
    let content_type = response
        .headers()
        .get(header::CONTENT_TYPE)
        .map(|value| value.to_str().unwrap().to_string());
    let body = response.into_body().collect().await.unwrap().to_bytes();
    TestResponse {
        status,
        content_type,
        body: body.to_vec(),
    }
}

async fn get(app: &Router, uri: &str) -> TestResponse {
    send(app, Method::GET, uri, None).await
}

async fn post(app: &Router, uri: &str, body: Value) -> TestResponse {
    send(app, Method::POST, uri, Some(body.to_string())).await
}

async fn patch(app: &Router, uri: &str, body: Value) -> TestResponse {
    send(app, Method::PATCH, uri, Some(body.to_string())).await
}

async fn delete(app: &Router, uri: &str) -> TestResponse {
    send(app, Method::DELETE, uri, None).await
}

async fn create(app: &Router, title: &str) -> Task {
    let response = post(app, "/tasks", json!({ "title": title })).await;
    assert_eq!(response.status, StatusCode::CREATED);
    response.task()
}

fn new_app() -> Router {
    app(AppState::default())
}

fn task(id: i64, title: &str, completed: bool) -> Task {
    Task {
        id,
        title: title.to_string(),
        completed,
    }
}

fn error(message: &str) -> Value {
    json!({ "error": message })
}

#[tokio::test]
async fn list_starts_empty() {
    let response = get(&new_app(), "/tasks").await;
    assert_eq!(response.status, StatusCode::OK);
    assert_eq!(response.content_type.as_deref(), Some("application/json"));
    assert_eq!(response.json(), json!([]));
}

#[tokio::test]
async fn create_returns_201_with_the_task() {
    let response = post(&new_app(), "/tasks", json!({ "title": "  Buy milk " })).await;
    assert_eq!(response.status, StatusCode::CREATED);
    assert_eq!(response.content_type.as_deref(), Some("application/json"));
    assert_eq!(
        response.json(),
        json!({ "id": 1, "title": "Buy milk", "completed": false })
    );
}

#[tokio::test]
async fn created_tasks_are_listed_in_id_order() {
    let app = new_app();
    create(&app, "first").await;
    create(&app, "second").await;

    let response = get(&app, "/tasks").await;
    assert_eq!(response.status, StatusCode::OK);
    assert_eq!(
        response.json(),
        json!([
            { "id": 1, "title": "first", "completed": false },
            { "id": 2, "title": "second", "completed": false },
        ])
    );
}

#[tokio::test]
async fn create_rejects_a_blank_title() {
    let app = new_app();
    let response = post(&app, "/tasks", json!({ "title": "   " })).await;
    assert_eq!(response.status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(response.json(), error("title must not be empty"));
    assert_eq!(get(&app, "/tasks").await.json(), json!([]));
}

#[tokio::test]
async fn create_limits_the_title_to_200_chars() {
    let app = new_app();

    let too_long = "a".repeat(MAX_TITLE_CHARS + 1);
    let response = post(&app, "/tasks", json!({ "title": too_long })).await;
    assert_eq!(response.status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(
        response.json(),
        error("title must be at most 200 characters")
    );

    // 200 chars but 400 bytes.
    let longest = "é".repeat(MAX_TITLE_CHARS);
    let created = create(&app, &longest).await;
    assert_eq!(created, task(1, &longest, false));
}

#[tokio::test]
async fn create_rejects_malformed_json_with_400() {
    let app = new_app();
    let response = send(
        &app,
        Method::POST,
        "/tasks",
        Some(r#"{"title": "Buy milk""#.to_string()),
    )
    .await;
    assert_eq!(response.status, StatusCode::BAD_REQUEST);
    assert_eq!(get(&app, "/tasks").await.json(), json!([]));
}

#[tokio::test]
async fn create_rejects_the_wrong_shape_with_422() {
    let app = new_app();
    let missing = post(&app, "/tasks", json!({})).await;
    assert_eq!(missing.status, StatusCode::UNPROCESSABLE_ENTITY);

    let wrong_type = post(&app, "/tasks", json!({ "title": 5 })).await;
    assert_eq!(wrong_type.status, StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn get_returns_a_task() {
    let app = new_app();
    create(&app, "a").await;
    create(&app, "b").await;

    let response = get(&app, "/tasks/2").await;
    assert_eq!(response.status, StatusCode::OK);
    assert_eq!(response.task(), task(2, "b", false));
}

#[tokio::test]
async fn get_unknown_task_is_404() {
    let response = get(&new_app(), "/tasks/42").await;
    assert_eq!(response.status, StatusCode::NOT_FOUND);
    assert_eq!(response.content_type.as_deref(), Some("application/json"));
    assert_eq!(response.json(), error("task not found"));
}

#[tokio::test]
async fn non_integer_id_is_400() {
    let app = new_app();
    assert_eq!(
        get(&app, "/tasks/abc").await.status,
        StatusCode::BAD_REQUEST
    );
    assert_eq!(
        delete(&app, "/tasks/1.5").await.status,
        StatusCode::BAD_REQUEST
    );
}

#[tokio::test]
async fn update_completed_keeps_the_title() {
    let app = new_app();
    create(&app, "Buy milk").await;

    let response = patch(&app, "/tasks/1", json!({ "completed": true })).await;
    assert_eq!(response.status, StatusCode::OK);
    assert_eq!(response.task(), task(1, "Buy milk", true));
    assert_eq!(
        get(&app, "/tasks/1").await.task(),
        task(1, "Buy milk", true)
    );
}

#[tokio::test]
async fn update_title_is_trimmed() {
    let app = new_app();
    create(&app, "Buy milk").await;

    let response = patch(&app, "/tasks/1", json!({ "title": " Buy oat milk  " })).await;
    assert_eq!(response.status, StatusCode::OK);
    assert_eq!(response.task(), task(1, "Buy oat milk", false));
}

#[tokio::test]
async fn update_both_fields() {
    let app = new_app();
    create(&app, "Buy milk").await;

    let response = patch(
        &app,
        "/tasks/1",
        json!({ "title": "Buy bread", "completed": true }),
    )
    .await;
    assert_eq!(response.status, StatusCode::OK);
    assert_eq!(response.task(), task(1, "Buy bread", true));
}

#[tokio::test]
async fn update_rejects_an_invalid_title() {
    let app = new_app();
    create(&app, "Buy milk").await;

    let response = patch(&app, "/tasks/1", json!({ "title": "", "completed": true })).await;
    assert_eq!(response.status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(response.json(), error("title must not be empty"));
    assert_eq!(
        get(&app, "/tasks/1").await.task(),
        task(1, "Buy milk", false)
    );
}

#[tokio::test]
async fn update_without_fields_is_422() {
    let app = new_app();
    create(&app, "Buy milk").await;

    for body in [json!({}), json!({ "title": null })] {
        let response = patch(&app, "/tasks/1", body).await;
        assert_eq!(response.status, StatusCode::UNPROCESSABLE_ENTITY);
        assert_eq!(
            response.json(),
            error("update must include title or completed")
        );
    }
}

#[tokio::test]
async fn update_unknown_task_is_404() {
    let response = patch(&new_app(), "/tasks/7", json!({ "completed": true })).await;
    assert_eq!(response.status, StatusCode::NOT_FOUND);
    assert_eq!(response.json(), error("task not found"));
}

#[tokio::test]
async fn update_validates_before_looking_up_the_task() {
    let app = new_app();
    let response = patch(&app, "/tasks/7", json!({})).await;
    assert_eq!(response.status, StatusCode::UNPROCESSABLE_ENTITY);

    let response = patch(&app, "/tasks/7", json!({ "title": " " })).await;
    assert_eq!(response.status, StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn update_rejects_malformed_json_with_400() {
    let app = new_app();
    create(&app, "Buy milk").await;

    let response = send(
        &app,
        Method::PATCH,
        "/tasks/1",
        Some("{completed: true}".to_string()),
    )
    .await;
    assert_eq!(response.status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn delete_returns_204_then_404() {
    let app = new_app();
    create(&app, "a").await;
    create(&app, "b").await;

    let response = delete(&app, "/tasks/1").await;
    assert_eq!(response.status, StatusCode::NO_CONTENT);
    assert!(response.body.is_empty());

    assert_eq!(get(&app, "/tasks/1").await.status, StatusCode::NOT_FOUND);
    assert_eq!(
        get(&app, "/tasks").await.json(),
        json!([{ "id": 2, "title": "b", "completed": false }])
    );

    let again = delete(&app, "/tasks/1").await;
    assert_eq!(again.status, StatusCode::NOT_FOUND);
    assert_eq!(again.json(), error("task not found"));
}

#[tokio::test]
async fn ids_are_not_reused_after_delete() {
    let app = new_app();
    create(&app, "a").await;
    create(&app, "b").await;
    delete(&app, "/tasks/2").await;

    assert_eq!(create(&app, "c").await.id, 3);
}

#[tokio::test]
async fn the_router_shares_the_store_with_the_caller() {
    let state = AppState::default();
    let app = app(state.clone());

    // Writes through the `Arc` are visible to the router, and the other way round.
    state.store.lock().unwrap().create("seeded".to_string());
    assert_eq!(get(&app, "/tasks/1").await.task(), task(1, "seeded", false));

    create(&app, "over HTTP").await;
    assert_eq!(
        state.store.lock().unwrap().list(),
        vec![task(1, "seeded", false), task(2, "over HTTP", false)]
    );
}
