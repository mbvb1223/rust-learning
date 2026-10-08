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

The server needs a published port, and a shell opened with `docker compose exec` has none. Start it from the repository root on your Mac:

```bash
docker compose run --rm -p 3000:3000 -e BIND_ADDR=0.0.0.0:3000 -w /workspace/lessons/14-api-quality rust cargo run
```

The first build compiles SQLite from C source (the `sqlite` feature of `sqlx` bundles it), so expect a few minutes.

Until a function is implemented, its parameters show `unused variable` warnings. Don't run `cargo fix`; it renames them to `_name`.

`cargo test` stops after the first test binary that fails. Add `--no-fail-fast` to see every failure, or run one binary at a time: `cargo test --lib`, `cargo test --test api`, `cargo test --test shutdown`.

## Notes

### What's here

| File | Contents | You implement |
|---|---|---|
| `src/model.rs` | `Task`, `CreateTask`, `UpdateTask`, title validation (lesson 10) | — |
| `src/repo.rs`, `migrations/` | SQLite queries (lesson 13) | — |
| `src/main.rs` | Startup: tracing → config → database → listener → serve → close pool | — |
| `src/config.rs` | `Config::from_lookup` | yes |
| `src/error.rs` | `AppError` → HTTP response, `From` conversions | yes |
| `src/handlers.rs` | One handler per endpoint | yes |
| `src/lib.rs` | `app`, `init_tracing`, `serve`, `shutdown_signal` | yes |
| `tests/` | API and shutdown tests | two tests |

| Request | Success | Failures |
|---|---|---|
| `POST /tasks` `{"title": "..."}` | 201 + task | 400 malformed JSON, 422 invalid input |
| `GET /tasks` | 200 + tasks ordered by id | — |
| `GET /tasks/{id}` | 200 + task | 404 |
| `PATCH /tasks/{id}` `{"title"?, "completed"?}` | 200 + task | 400, 422, 404 |
| `DELETE /tasks/{id}` | 204, empty body | 404 |

A task is `{"id": 1, "title": "Buy milk", "completed": false}`. Titles are trimmed, then must be 1–200 characters. Every error body is `{"error": "<message>"}`. Any database failure is a 500 with `{"error": "internal server error"}`.

### Error responses: one enum, one `IntoResponse`

```rust
async fn show_user(State(pool): State<SqlitePool>, Path(id): Path<i64>) -> Result<Json<User>, AppError> {
    let user = users::find(&pool, id).await?  // sqlx::Error → AppError through From
        .ok_or(AppError::NotFound)?;          // Option<User> → Result<User, AppError>
    Ok(Json(user))
}
```

- PHP code throws anywhere, and one exception handler (Laravel's `Handler::render`, Symfony's `kernel.exception`) builds the response. In Rust the error is a return value in the signature. `?` converts it with `From`, and axum calls `IntoResponse` on it. The mapping still lives in one place, and the compiler checks every path. Nothing can go uncaught.
- The comparison breaks down at panics. A panicking handler drops its connection without a response, and the server keeps running. Use `Result` for expected failures. A panic means a bug.
- A `(StatusCode, impl IntoResponse)` tuple is itself `IntoResponse`: `(StatusCode::NOT_FOUND, Json(json!({"error": "task not found"})))`.
- `Box<dyn std::error::Error + Send + Sync>` is "any error type", like catching `\Throwable`. `Send + Sync` lets it cross threads on Tokio's multi-threaded runtime.
- **Never leak internals.** Error text such as `no such table: tasks` reveals your schema. The 500 body is fixed text; the real error goes to the log: `tracing::error!(error = %source, "internal error")`.

### Extractor rejections

```rust
pub async fn create_task(
    State(pool): State<SqlitePool>,
    payload: Result<Json<CreateTask>, JsonRejection>, // extraction failure arrives as a value
) -> Result<(StatusCode, Json<Task>), AppError> {
    let Json(input) = payload?;                        // uses From<JsonRejection> for AppError
```

With a plain `Json<CreateTask>` argument, axum rejects a bad body before your handler runs, using its own plain-text body. Taking `Result<Json<_>, JsonRejection>` hands the rejection to you, so the response keeps the `{"error": ...}` shape.

| `JsonRejection` variant | Example | axum default | This API |
|---|---|---|---|
| `JsonSyntaxError` | `{"title": ` | 400 | 400 |
| `JsonDataError` | `{}`, `{"title": 5}` | 422 | 422 |
| `MissingJsonContentType` | no `Content-Type: application/json` | 415 | 400 |
| `BytesRejection` | body couldn't be read | 400/413 | 400 |

- `JsonRejection` is `#[non_exhaustive]`: axum may add variants in a minor release, so a `match` on it must have a `_` arm. For your own enums, lesson 02's advice still applies: list every variant.
- `rejection.body_text()` describes the client's input (serde's message, e.g. ``missing field `title` ``). Returning it to the client is safe.

### Configuration from the environment

```rust
Config::from_lookup(|name| std::env::var(name).ok())   // production: Config::from_env
Config::from_lookup(|name| vars.get(name).cloned())    // tests: a HashMap
```

- This is the same idea as PHP's `getenv()` or Laravel's `env('DB_HOST', 'default')`. Nothing loads a `.env` file here; the `dotenvy` crate does that.
- Edition 2024 made `std::env::set_var` `unsafe`, because changing the environment while another thread reads it is undefined behaviour on some platforms. `cargo test` runs tests on parallel threads in one process. Passing the lookup in keeps tests free of global state.
- Unset means "use the default". Set but blank (`DATABASE_URL=`) is almost always a deployment mistake, so it's an error, not a silent fallback.
- `"localhost:3000".parse::<SocketAddr>()` fails: `SocketAddr` is an IP address plus a port, with no DNS lookup.
- Inside a container, `127.0.0.1` is the container's own loopback, which Docker's published port can't reach. That's why the run command sets `BIND_ADDR=0.0.0.0:3000`. The default stays `127.0.0.1`, so a run on your laptop isn't exposed to the network.
- `main` validates the config and exits before binding the port. A bad deploy fails at startup, not on the first request.

### Tracing

| PHP | Rust |
|---|---|
| PSR-3 `LoggerInterface` | `tracing` macros (`info!`, `error!`, …). Libraries depend only on this. |
| Monolog handlers and formatters | `tracing-subscriber`, installed once by the binary |
| `$logger->error('msg', ['id' => $id])` | `tracing::error!(task_id = id, "msg")` |
| Monolog processors that add request context | Spans: every event inside a span carries the span's fields |

```rust
let filter = EnvFilter::try_from_default_env()          // RUST_LOG, if set and valid
    .unwrap_or_else(|_| EnvFilter::new("info"));
tracing_subscriber::fmt().with_env_filter(filter).init();
```

- `RUST_LOG=api_quality=debug,tower_http=info,warn` sets per-target levels. The bare `warn` is the level for every other target.
- `TraceLayer::new_for_http()` opens a `request{method=GET uri=/tasks/1 version=HTTP/1.1}` span per request and logs its start and end (status, latency) at `DEBUG`. That's why the default filter includes `tower_http=debug`. sqlx logs each query at `DEBUG` inside that same span.
- In a field, `%value` records it with `Display` and `?value` with `Debug`.
- With no subscriber installed, events go nowhere: no error, no output. Tests don't install one. `internal_errors_log_their_source` captures output with a subscriber that applies to the current thread only (`tracing::subscriber::with_default`).

### Graceful shutdown

`axum::serve(listener, app)` returns a builder. `.with_graceful_shutdown(signal)` takes any `Future<Output = ()> + Send + 'static`, and awaiting the result runs the server.

- When `signal` completes, the server stops accepting connections, lets in-flight requests finish, and returns. PHP-FPM's master process handles graceful stops and reloads for you. Here, your process is the server.
- `tokio::signal::ctrl_c()` is a future. Waiting for SIGTERM needs `tokio::signal::unix::signal(SignalKind::terminate())` behind `#[cfg(unix)]`. `tokio::select!` waits for whichever comes first.
- `docker stop` sends SIGTERM, waits 10 s, then sends SIGKILL. `compose.yaml` sets `init: true`, so a tiny init process is PID 1 and forwards SIGTERM to your server. Without a SIGTERM handler, the default action kills the server at once and drops in-flight requests. Without `init`, your server would be PID 1, and Linux ignores any signal sent to PID 1 that it has no handler for (SIGKILL aside), so `docker stop` would always wait the full 10 s.
- `serve` takes the signal as a parameter, so `tests/shutdown.rs` can pass a channel instead of Ctrl+C. This is the same trick as `from_lookup`: inject what the OS normally provides.

### Integration tests with isolated databases

```rust
let response = router.clone().oneshot(request).await.unwrap(); // tower::ServiceExt
```

- A `Router` is a tower `Service`: request in, response out. `oneshot` calls it in-process, with no port and no network, like Laravel's `$this->postJson()` or Symfony's `KernelBrowser`. `tests/shutdown.rs` uses a real socket because shutdown is about sockets.
- Every `TestApp::new()` creates its own migrated in-memory database. Laravel's `RefreshDatabase` resets one shared database, which works because PHPUnit runs tests one at a time. Rust tests run **in parallel**, so a shared database would make `"id": 1` assertions fail at random.
- Gotcha: an in-memory SQLite database exists only while a connection to it is open. If the pool's idle timeout closed the connection, the data would vanish mid-test. `TestApp` therefore keeps exactly one connection with no idle timeout and no maximum lifetime.
- Shared helpers live in `tests/common/mod.rs`, not `tests/common.rs`, so Cargo doesn't build them as a test binary of their own.
- `#[sqlx::test]` is an alternative: it creates and migrates a database per test and passes you the pool. For SQLite it writes files under `target/sqlx/test-dbs/`.
- To force a server failure, `database_errors_are_500_without_details` drops the table. That's cheaper than mocking the repository.

### Blocking work: `spawn_blocking`

```rust
let hash = tokio::task::spawn_blocking(move || hash_password(&password)) // runs on a separate thread pool
    .await
    .map_err(|err| AppError::Internal(err.into()))?;                    // JoinError: the closure panicked
```

- PHP-FPM gives every request its own process, so a slow `password_hash()` blocks only that request. Tokio runs many requests on a few worker threads. 100 ms of CPU work inside an async handler stalls every request on that worker.
- Async I/O is fine to `.await` directly. Move CPU-heavy work (password hashing with argon2 or bcrypt, image resizing) and blocking calls (`std::fs`, synchronous drivers) into `spawn_blocking`. The closure must be `Send + 'static`, so `move` owned data into it.
- sqlx is async, so the repository doesn't need this. Lesson 17 covers CPU parallelism.

## Exercise

Implement in this order:

1. `Config::from_lookup` (`src/config.rs`).
   - A closure `read(name, default) -> Result<String, ConfigError>` avoids repeating the unset, blank, and trim logic.
   - Use `str::parse::<SocketAddr>()`, then `map_err`.
2. `IntoResponse for AppError` (`src/error.rs`).
   - `match self` into a `(StatusCode, String)`, and log in the `Internal` arm.
   - Then return `(status, Json(json!({ "error": message }))).into_response()`.
   - Add the imports the compiler asks for.
3. `From<sqlx::Error>` and `From<JsonRejection>`.
   - `Box::new(err)` turns a concrete error into the boxed trait object.
   - The pattern `JsonRejection::JsonDataError(_)` binds nothing, so `rejection` is still usable inside that arm.
   - `cargo test --lib` should now pass.
4. The handlers (`src/handlers.rs`).
   - Use `let Json(input) = payload?;`, then `input.validate()?`, then a `repo::…` call with `?`.
   - `.ok_or(AppError::NotFound)?` turns an `Option` into an error.
5. `app` (`src/lib.rs`).
   - Paths use `"/tasks/{id}"` (axum 0.8; older docs show `/:id`).
   - Chain methods: `get(list).post(create)`.
   - Finish with `.layer(TraceLayer::new_for_http())` and `.with_state(pool)`.
   - `cargo test --test api` should now pass, apart from your two tests.
6. `init_tracing`: build an `EnvFilter` from `RUST_LOG`, falling back to the doc comment's default, and install a `fmt` subscriber with it.
7. `serve` and `shutdown_signal`.
   - On non-Unix targets, `let terminate = std::future::pending::<()>();` keeps the `select!` compiling.
8. Write the two tests at the bottom of `tests/api.rs` with the `create` and `assert_error` helpers.
   - Watch each test fail before you trust it: temporarily break the handler (for example, ignore `completed`) and check that the test fails.
9. Run the API, try these requests from another terminal, and watch the log. Then press Ctrl+C.

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

Restart the server and run `curl -i localhost:3000/tasks`: the last task you created is still there (id 2 on a fresh database), because the data lives in `tasks.db` in the lesson directory, which is git-ignored. Then try `-e RUST_LOG=debug` to see sqlx's queries, and `-e BIND_ADDR=localhost:3000` to see the configuration error.

Optional extensions:

- Write a restart test: `api_quality::connect` to a file under `std::env::temp_dir()` with a unique name per test. Create a task, drop the router and pool, connect again, and `GET` the task.
- `GET /tasks/abc` still returns axum's plain-text 400. Take `Result<Path<i64>, PathRejection>` and add `From<PathRejection> for AppError` so it gets the JSON shape too.

## Done when

- `cargo test`, `cargo fmt --check`, and `cargo clippy --all-targets -- -D warnings` pass.
- `cargo run` answers the curl requests above, logs every request, and logs `shut down cleanly` after Ctrl+C.
- You can explain:
  - step by step, how `?` in a handler turns a `sqlx::Error` into a 500 response
  - why the 500 body is fixed text, and where the real error goes
  - why `Config` takes a lookup closure instead of calling `std::env::var`
  - why each test gets its own database, and what would break if they shared one
  - what happens to an in-flight request when Ctrl+C arrives
  - when a handler needs `spawn_blocking`
