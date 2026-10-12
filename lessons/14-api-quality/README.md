# Lesson 14 — Errors, tests, and operation

Finish the task API: implement every `todo!()` in `src/` and the two "your turn" tests in `tests/api.rs` until the tests pass, then run the API locally.

## Run

Inside the container:

```bash
cd /workspace/lessons/14-api-quality
cargo test
cargo fmt --check
cargo clippy --all-targets -- -D warnings
```

Start the server from the repository root on your Mac (a `docker compose exec` shell has no published port):

```bash
docker compose run --rm -p 3000:3000 -e BIND_ADDR=0.0.0.0:3000 -w /workspace/lessons/14-api-quality rust cargo run
```

The first build compiles SQLite from C, so it takes a few minutes. Don't run `cargo fix`.

`cargo test` stops at the first failing test binary. Add `--no-fail-fast`, or run one at a time: `cargo test --lib`, `cargo test --test api`, `cargo test --test shutdown`.

## Notes

### The API

A task is `{"id": 1, "title": "Buy milk", "completed": false}`. Titles are trimmed, then must be 1–200 characters.

- `POST /tasks` `{"title": "…"}` → 201 + task
- `GET /tasks` → 200 + all tasks, ordered by id
- `GET /tasks/{id}` → 200 + task, or 404
- `PATCH /tasks/{id}` `{"title"?, "completed"?}` → 200 + task, or 404
- `DELETE /tasks/{id}` → 204 with an empty body, or 404

Malformed JSON is a 400, invalid input a 422. Every error body is `{"error": "<message>"}`; any database failure is a 500 with `{"error": "internal server error"}`.

`src/model.rs` (validation) and `src/repo.rs` (queries) are already done.

### Errors as responses

A handler returns `Result<_, AppError>`. axum turns an `Err` into a response with `AppError`'s `IntoResponse`, so the error-to-status mapping lives in one place:

```rust
async fn show_user(
    State(pool): State<SqlitePool>,
    Path(id): Path<i64>,
) -> Result<Json<User>, AppError> {
    let user = users::find(&pool, id)
        .await? // sqlx::Error → AppError via From
        .ok_or(AppError::NotFound)?; // None → 404
    Ok(Json(user))
}
```

Like Laravel's exception handler, except the error is a return value and `?` converts it with `From`.

A `(StatusCode, body)` tuple is a response too:

```rust
let response = (StatusCode::NOT_FOUND, Json(json!({"error": "task not found"}))).into_response();
println!("{}", response.status()); // 404 Not Found
```

- `Box<dyn std::error::Error + Send + Sync>` holds any error type; `Box::new(err)` makes one.
- Never send internal error text (`no such table: tasks`) to the client. Return fixed text and log the real error: `tracing::error!(error = %source, "internal error")`.

### JSON body errors

With a plain `Json<T>` argument, axum rejects a bad body before your handler runs, with a plain-text body. Take `Result<Json<T>, JsonRejection>` instead to keep the `{"error": …}` shape:

```rust
pub async fn create_task(
    State(pool): State<SqlitePool>,
    payload: Result<Json<CreateTask>, JsonRejection>,
) -> Result<(StatusCode, Json<Task>), AppError> {
    let Json(input) = payload?; // From<JsonRejection> for AppError
    // …
}
```

- `JsonRejection::JsonDataError` (valid JSON, wrong shape: `{}`, `{"title": 5}`) → 422.
- Anything else (`{"title": `, no `Content-Type: application/json`, unreadable body) → 400.
- `rejection.body_text()` describes the client's input, so it's safe to send back.
- `JsonRejection` is `#[non_exhaustive]`, so a `match` on it needs a `_` arm.

### Config from the environment

`Config::from_lookup` takes a function that reads a variable, instead of calling `std::env::var` itself:

```rust
Config::from_lookup(|name| std::env::var(name).ok()) // production: Config::from_env
Config::from_lookup(|name| vars.get(name).cloned())  // tests: a HashMap
```

Tests run in parallel threads, and edition 2024 made `std::env::set_var` `unsafe`, so tests pass their own variables instead.

- Unset → the default. Set but blank → an error, since it's almost always a deploy mistake.
- `SocketAddr` needs an IP address, not a host name:

```rust
let ok = "127.0.0.1:3000".parse::<SocketAddr>();
let bad = "localhost:3000".parse::<SocketAddr>();
println!("{} {}", ok.is_ok(), bad.is_ok()); // true false
```

- Inside Docker, the server must bind `0.0.0.0` for the published port to reach it. That's why the run command sets `BIND_ADDR`.

### Logging with tracing

`tracing` is the logging API (like PSR-3); `tracing-subscriber` prints the events (like Monolog's handlers). Install a subscriber once, at startup:

```rust
let filter = EnvFilter::try_from_default_env() // RUST_LOG, if set and valid
    .unwrap_or_else(|_| EnvFilter::new("info"));
tracing_subscriber::fmt().with_env_filter(filter).init();

tracing::info!(task_id = 1, "created");
```

```text
2026-10-12T09:00:00.000000Z  INFO api_quality: created task_id=1
```

- `RUST_LOG=api_quality=debug,tower_http=info,warn` sets a level per crate; the bare `warn` covers the rest.
- In fields, `%value` logs with `Display` and `?value` with `Debug`.
- `TraceLayer::new_for_http()` logs each request and its response (status, latency) at `DEBUG`.
- With no subscriber installed, events go nowhere. Tests don't install one.

### Graceful shutdown

`with_graceful_shutdown(signal)` runs the server until `signal` completes. Then it stops accepting connections, lets in-flight requests finish, and returns:

```rust
axum::serve(listener, app)
    .with_graceful_shutdown(signal)
    .await?;
```

- `tokio::signal::ctrl_c()` waits for Ctrl+C. SIGTERM (what `docker stop` sends) needs `tokio::signal::unix::signal(SignalKind::terminate())` behind `#[cfg(unix)]`.
- `tokio::select!` waits for whichever future finishes first:

```rust
tokio::select! {
    _ = tokio::time::sleep(Duration::from_secs(1)) => println!("slow"),
    _ = tokio::time::sleep(Duration::from_millis(10)) => println!("fast"),
}
// fast
```

- `serve` takes the signal as a parameter, so `tests/shutdown.rs` can pass a channel instead of Ctrl+C.

PHP-FPM's master process handles graceful stops for you. Here, your process is the server.

### Integration tests

`oneshot` sends one request through the router in-process: no port, no network. It's like Laravel's `$this->postJson()`.

```rust
let response = router.clone().oneshot(request).await.unwrap(); // tower::ServiceExt
```

`TestApp` in `tests/common/mod.rs` wraps it; `tests/api.rs` adds the `create` and `assert_error` helpers:

```rust
#[tokio::test]
async fn get_unknown_task_is_404() {
    let app = TestApp::new().await;
    let response = app.get("/tasks/42").await;
    assert_error(&response, StatusCode::NOT_FOUND, "task not found");
}
```

- Each `TestApp::new()` gets its own in-memory database, so tests run in parallel and can still expect `"id": 1`.
- Its pool keeps one connection open for the whole test, because an in-memory SQLite database disappears when its last connection closes.
- Helpers live in `tests/common/mod.rs`, not `tests/common.rs`, so Cargo doesn't build them as a test of their own.

### Blocking work

Tokio runs many requests on a few threads, so CPU-heavy or blocking work inside a handler stalls other requests. Move it to `spawn_blocking`:

```rust
let hash = tokio::task::spawn_blocking(move || hash_password(&password))
    .await
    .map_err(|err| AppError::Internal(err.into()))?; // Err: the closure panicked
```

PHP-FPM gives each request its own process, so a slow `password_hash()` blocks only that request. SQLx is already async, so this lesson doesn't need it.

## Exercise

Implement in this order:

1. `Config::from_lookup` (`src/config.rs`): a closure `read(name, default) -> Result<String, ConfigError>` holds the unset/blank/trim logic; then `parse::<SocketAddr>()` with `map_err`.
2. `IntoResponse for AppError` (`src/error.rs`): `match self` into a `(StatusCode, String)`, log in the `Internal` arm, then `(status, Json(json!({ "error": message }))).into_response()`.
3. `From<sqlx::Error>` and `From<JsonRejection>`: `Box::new(err)` for `Internal`; match `JsonRejection::JsonDataError(_)`. `cargo test --lib` should now pass.
4. The handlers (`src/handlers.rs`): `let Json(input) = payload?;`, `input.validate()?`, a `repo::…` call with `?`, and `.ok_or(AppError::NotFound)?`.
5. `app` (`src/lib.rs`): paths `"/tasks"` and `"/tasks/{id}"` (not `/:id`), `get(list).post(create)`, then `.layer(TraceLayer::new_for_http())` and `.with_state(pool)`. `cargo test --test api` should now pass, apart from your two tests.
6. `init_tracing`: an `EnvFilter` from `RUST_LOG`, falling back to the doc comment's default, and a `fmt` subscriber.
7. `serve` and `shutdown_signal`: `with_graceful_shutdown` and `tokio::select!`; on non-Unix, `let terminate = std::future::pending::<()>();`.
8. The two tests at the bottom of `tests/api.rs`, with `create` and `assert_error`. Break the handler once (e.g. ignore `completed`) to see each fail.
9. Run the API, send these requests from another terminal, watch the log, then press Ctrl+C.

```bash
curl -i -X POST localhost:3000/tasks -H 'content-type: application/json' -d '{"title": "Buy milk"}'
curl -i localhost:3000/tasks
curl -i localhost:3000/tasks/1
curl -i -X PATCH localhost:3000/tasks/1 -H 'content-type: application/json' -d '{"completed": true}'
curl -i -X DELETE localhost:3000/tasks/1
curl -i localhost:3000/tasks/1                                                                # 404
curl -i -X POST localhost:3000/tasks -H 'content-type: application/json' -d '{"title": '      # 400
curl -i -X POST localhost:3000/tasks -H 'content-type: application/json' -d '{"title": "  "}'  # 422
curl -i -X POST localhost:3000/tasks -H 'content-type: application/json' -d '{"title": "Survive a restart"}'
```

Restart the server and `curl -i localhost:3000/tasks`: the last task is still there, in `tasks.db` (git-ignored). Then try `-e RUST_LOG=debug` to see the SQL, and `-e BIND_ADDR=localhost:3000` to see the config error.

Optional: make `GET /tasks/abc` return the JSON error shape with `Result<Path<i64>, PathRejection>` and `From<PathRejection> for AppError`.

## Done when

- `cargo test`, `cargo fmt --check`, and `cargo clippy --all-targets -- -D warnings` pass.
- `cargo run` answers the curl requests above, logs every request, and logs `shut down cleanly` after Ctrl+C.
- You can explain:
  - how `?` in a handler turns a `sqlx::Error` into a 500, and why its body is fixed text
  - why `Config` takes a lookup closure instead of calling `std::env::var`
  - what happens to an in-flight request when Ctrl+C arrives
