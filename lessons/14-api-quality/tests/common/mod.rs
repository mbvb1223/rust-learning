//! Helpers shared by integration tests. `tests/common/mod.rs` is not a test binary itself;
//! each test file opts in with `mod common;`.

use axum::Router;
use axum::body::Body;
use axum::http::{Method, Request, StatusCode, header};
use serde_json::Value;
use sqlx::SqlitePool;
use sqlx::sqlite::SqlitePoolOptions;
use tower::ServiceExt;

/// The API on top of its own fresh, migrated in-memory database.
pub struct TestApp {
    pub router: Router,
    pub pool: SqlitePool,
}

pub struct TestResponse {
    pub status: StatusCode,
    pub content_type: Option<String>,
    pub body: Vec<u8>,
}

impl TestResponse {
    pub fn json(&self) -> Value {
        serde_json::from_slice(&self.body).unwrap_or_else(|err| {
            panic!(
                "body is not JSON ({err}): {:?}",
                String::from_utf8_lossy(&self.body)
            )
        })
    }
}

impl TestApp {
    pub async fn new() -> TestApp {
        // Every parse of "sqlite::memory:" names a new in-memory database, and SQLite drops it
        // when its last connection closes. One connection that never expires keeps it alive.
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .idle_timeout(None)
            .max_lifetime(None)
            .connect("sqlite::memory:")
            .await
            .expect("open in-memory database");
        api_quality::migrate(&pool).await.expect("run migrations");

        TestApp {
            router: api_quality::app(pool.clone()),
            pool,
        }
    }

    /// Runs `request` through the router in-process: no socket, no port.
    pub async fn send(&self, request: Request<Body>) -> TestResponse {
        let response = self.router.clone().oneshot(request).await.unwrap();
        let status = response.status();
        let content_type = response
            .headers()
            .get(header::CONTENT_TYPE)
            .map(|value| value.to_str().unwrap().to_string());
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap()
            .to_vec();
        TestResponse {
            status,
            content_type,
            body,
        }
    }

    pub async fn get(&self, uri: &str) -> TestResponse {
        self.send(request(Method::GET, uri, None)).await
    }

    pub async fn delete(&self, uri: &str) -> TestResponse {
        self.send(request(Method::DELETE, uri, None)).await
    }

    /// Sends `body` as-is with `Content-Type: application/json`, so tests can send broken JSON.
    pub async fn post_json(&self, uri: &str, body: &str) -> TestResponse {
        self.send(request(Method::POST, uri, Some(body))).await
    }

    pub async fn patch_json(&self, uri: &str, body: &str) -> TestResponse {
        self.send(request(Method::PATCH, uri, Some(body))).await
    }
}

fn request(method: Method, uri: &str, json: Option<&str>) -> Request<Body> {
    let builder = Request::builder().method(method).uri(uri);
    match json {
        Some(json) => builder
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(json.to_string())),
        None => builder.body(Body::empty()),
    }
    .unwrap()
}
