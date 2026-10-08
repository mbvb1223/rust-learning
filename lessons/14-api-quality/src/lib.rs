//! Lesson 14 — the task API with real error responses, configuration, logging, tests, and
//! shutdown. Replace every `todo!()` in `src/` (and the two "your turn" tests in `tests/api.rs`)
//! until `cargo test` passes.

pub mod config;
pub mod error;
pub mod handlers;
pub mod model;
pub mod repo;

use std::str::FromStr;

use axum::Router;
use sqlx::SqlitePool;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use tokio::net::TcpListener;

pub use config::Config;
pub use error::AppError;

/// Opens the SQLite database at `database_url`, creating the file if needed, and runs the migrations.
pub async fn connect(database_url: &str) -> Result<SqlitePool, sqlx::Error> {
    let options = SqliteConnectOptions::from_str(database_url)?.create_if_missing(true);
    let pool = SqlitePoolOptions::new().connect_with(options).await?;
    migrate(&pool).await?;
    Ok(pool)
}

/// Applies `migrations/`. `sqlx::migrate!` embeds the SQL files into the binary at compile time.
pub async fn migrate(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    sqlx::migrate!().run(pool).await?;
    Ok(())
}

/// The whole API, ready for [`serve`] or for in-process tests.
///
/// | Route | Handler |
/// |---|---|
/// | `POST /tasks` | [`handlers::create_task`] |
/// | `GET /tasks` | [`handlers::list_tasks`] |
/// | `GET /tasks/{id}` | [`handlers::get_task`] |
/// | `PATCH /tasks/{id}` | [`handlers::update_task`] |
/// | `DELETE /tasks/{id}` | [`handlers::delete_task`] |
///
/// `pool` is the router state. Every route is wrapped in
/// `tower_http::trace::TraceLayer::new_for_http()`, which opens a span per request (method, URI)
/// and logs each response's status and latency.
pub fn app(pool: SqlitePool) -> Router {
    todo!()
}

/// Installs the global `tracing` subscriber: `tracing_subscriber::fmt` output, filtered by
/// `RUST_LOG` (for example `RUST_LOG=api_quality=trace,tower_http=info`).
/// If `RUST_LOG` is unset or invalid, the filter is `api_quality=debug,tower_http=debug,info`.
///
/// Call it once, first thing in `main`. Tests don't call it.
pub fn init_tracing() {
    todo!()
}

/// Serves `app` on `listener` until `shutdown` completes. Then it stops accepting connections,
/// lets in-flight requests finish, and returns `Ok(())`.
pub async fn serve(
    listener: TcpListener,
    app: Router,
    shutdown: impl Future<Output = ()> + Send + 'static,
) -> std::io::Result<()> {
    todo!()
}

/// Completes when the process receives Ctrl+C (SIGINT) or, on Unix, SIGTERM — the signal
/// `docker stop` sends. Logs which signal arrived at `info` level.
pub async fn shutdown_signal() {
    todo!()
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::io;
    use std::net::SocketAddr;
    use std::sync::{Arc, Mutex};

    use axum::Json;
    use axum::body::Body;
    use axum::extract::FromRequest;
    use axum::extract::rejection::JsonRejection;
    use axum::http::{Request, StatusCode, header};
    use axum::response::IntoResponse;
    use serde_json::{Value, json};

    use crate::config::{Config, ConfigError};
    use crate::error::AppError;
    use crate::model::CreateTask;

    // ---- Config ----

    fn lookup_from(pairs: &[(&str, &str)]) -> impl Fn(&str) -> Option<String> {
        let vars: HashMap<String, String> = pairs
            .iter()
            .map(|(name, value)| (name.to_string(), value.to_string()))
            .collect();
        move |name| vars.get(name).cloned()
    }

    fn addr(text: &str) -> SocketAddr {
        text.parse().unwrap()
    }

    #[test]
    fn config_uses_defaults_when_unset() {
        assert_eq!(
            Config::from_lookup(|_| None),
            Ok(Config {
                database_url: "sqlite:tasks.db".to_string(),
                bind_addr: addr("127.0.0.1:3000"),
            })
        );
    }

    #[test]
    fn config_reads_and_trims_values() {
        let vars = [
            ("DATABASE_URL", " sqlite:/data/app.db "),
            ("BIND_ADDR", "\t0.0.0.0:8080\n"),
        ];
        assert_eq!(
            Config::from_lookup(lookup_from(&vars)),
            Ok(Config {
                database_url: "sqlite:/data/app.db".to_string(),
                bind_addr: addr("0.0.0.0:8080"),
            })
        );

        let config = Config::from_lookup(lookup_from(&[("BIND_ADDR", "[::1]:3000")])).unwrap();
        assert_eq!(config.bind_addr, addr("[::1]:3000"));
        assert_eq!(config.database_url, "sqlite:tasks.db");
    }

    #[test]
    fn config_rejects_blank_values_instead_of_defaulting() {
        assert_eq!(
            Config::from_lookup(lookup_from(&[("DATABASE_URL", "   ")])),
            Err(ConfigError::Empty("DATABASE_URL"))
        );
        assert_eq!(
            Config::from_lookup(lookup_from(&[("BIND_ADDR", "")])),
            Err(ConfigError::Empty("BIND_ADDR"))
        );
    }

    #[test]
    fn config_requires_an_ip_and_port() {
        for bad in ["localhost:3000", "0.0.0.0", "3000", "127.0.0.1:70000"] {
            assert_eq!(
                Config::from_lookup(lookup_from(&[("BIND_ADDR", bad)])),
                Err(ConfigError::InvalidBindAddr(bad.to_string())),
                "BIND_ADDR={bad:?}"
            );
        }
        assert_eq!(
            Config::from_lookup(lookup_from(&[("BIND_ADDR", " nope ")])),
            Err(ConfigError::InvalidBindAddr("nope".to_string()))
        );
    }

    #[test]
    fn config_reports_database_url_first() {
        assert_eq!(
            Config::from_lookup(lookup_from(&[("DATABASE_URL", ""), ("BIND_ADDR", "nope")])),
            Err(ConfigError::Empty("DATABASE_URL"))
        );
    }

    // ---- AppError → response ----

    async fn parts(err: AppError) -> (StatusCode, Option<String>, Value) {
        let response = err.into_response();
        let status = response.status();
        let content_type = response
            .headers()
            .get(header::CONTENT_TYPE)
            .map(|value| value.to_str().unwrap().to_string());
        let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let body = serde_json::from_slice(&bytes).expect("error body is JSON");
        (status, content_type, body)
    }

    #[tokio::test]
    async fn client_errors_carry_their_message() {
        let (status, content_type, body) = parts(AppError::BadRequest("bad body".into())).await;
        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert_eq!(content_type.as_deref(), Some("application/json"));
        assert_eq!(body, json!({"error": "bad body"}));

        let (status, content_type, body) =
            parts(AppError::Validation("title must not be empty".into())).await;
        assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
        assert_eq!(content_type.as_deref(), Some("application/json"));
        assert_eq!(body, json!({"error": "title must not be empty"}));
    }

    #[tokio::test]
    async fn not_found_uses_the_contract_message() {
        let (status, content_type, body) = parts(AppError::NotFound).await;
        assert_eq!(status, StatusCode::NOT_FOUND);
        assert_eq!(content_type.as_deref(), Some("application/json"));
        assert_eq!(body, json!({"error": "task not found"}));
    }

    #[tokio::test]
    async fn internal_errors_hide_their_source() {
        let (status, content_type, body) = parts(AppError::Internal("disk on fire".into())).await;
        assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
        assert_eq!(content_type.as_deref(), Some("application/json"));
        assert_eq!(body, json!({"error": "internal server error"}));
    }

    #[derive(Clone, Default)]
    struct LogBuffer(Arc<Mutex<Vec<u8>>>);

    impl io::Write for LogBuffer {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            self.0.lock().unwrap().extend_from_slice(buf);
            Ok(buf.len())
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    #[test]
    fn internal_errors_log_their_source() {
        let logs = LogBuffer::default();
        let writer = logs.clone();
        let subscriber = tracing_subscriber::fmt()
            .with_ansi(false)
            .with_writer(move || writer.clone())
            .finish();

        // Captures events from this thread only, so parallel tests don't interfere.
        tracing::subscriber::with_default(subscriber, || {
            let _response = AppError::Internal("disk on fire".into()).into_response();
        });

        let logs = String::from_utf8(logs.0.lock().unwrap().clone()).unwrap();
        assert!(
            logs.contains("ERROR"),
            "expected an ERROR event, got: {logs:?}"
        );
        assert!(
            logs.contains("disk on fire"),
            "expected the source in the log, got: {logs:?}"
        );
    }

    // ---- conversions into AppError ----

    #[test]
    fn sqlx_errors_become_not_found_or_internal() {
        assert!(matches!(
            AppError::from(sqlx::Error::RowNotFound),
            AppError::NotFound
        ));
        match AppError::from(sqlx::Error::PoolTimedOut) {
            AppError::Internal(source) => {
                assert_eq!(source.to_string(), sqlx::Error::PoolTimedOut.to_string());
            }
            other => panic!("expected Internal, got {other:?}"),
        }
    }

    async fn rejection(content_type: Option<&str>, body: &'static str) -> JsonRejection {
        let mut request = Request::builder().method("POST").uri("/tasks");
        if let Some(content_type) = content_type {
            request = request.header(header::CONTENT_TYPE, content_type);
        }
        let request = request.body(Body::from(body)).unwrap();
        Json::<CreateTask>::from_request(request, &())
            .await
            .unwrap_err()
    }

    #[tokio::test]
    async fn unreadable_json_is_a_bad_request() {
        let cases = [
            (Some("application/json"), r#"{"title": "#),
            (None, r#"{"title": "Buy milk"}"#),
        ];
        for (content_type, body) in cases {
            let rejection = rejection(content_type, body).await;
            let expected = rejection.body_text();
            match AppError::from(rejection) {
                AppError::BadRequest(message) => assert_eq!(message, expected),
                other => panic!("{body:?}: expected BadRequest, got {other:?}"),
            }
        }
    }

    #[tokio::test]
    async fn wrongly_shaped_json_is_a_validation_error() {
        for body in ["{}", r#"{"title": 5}"#] {
            let rejection = rejection(Some("application/json"), body).await;
            let expected = rejection.body_text();
            match AppError::from(rejection) {
                AppError::Validation(message) => assert_eq!(message, expected),
                other => panic!("{body:?}: expected Validation, got {other:?}"),
            }
        }
    }
}
