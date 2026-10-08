# Lesson 12 — HTTP with Axum

Build an in-memory task API: implement every `todo!()` in `src/lib.rs` until the tests in `src/lib.rs` and `tests/api.rs` pass.

## Run

Inside the container:

```bash
cd /workspace/lessons/12-axum
cargo test
cargo fmt --check
cargo clippy --all-targets -- -D warnings
```

The server needs a published port, and a shell opened with `docker compose exec` has none. Start it from the repository root on your Mac:

```bash
docker compose run --rm -p 3000:3000 -w /workspace/lessons/12-axum rust cargo run
```

Until a function is implemented, its parameters show `unused variable` warnings and `Store`'s fields show `never read`. Don't run `cargo fix`.

`cargo test` stops at the first failing test binary, so `tests/api.rs` only runs once the unit tests pass. `cargo test --test api` runs the HTTP tests alone.

With the server running (`-p` publishes the port), from another terminal:

```bash
curl -i localhost:3000/tasks -H 'content-type: application/json' -d '{"title": "Buy milk"}'      # 201
curl -s localhost:3000/tasks                                                                # [{"id":1,…}]
curl -s -X PATCH localhost:3000/tasks/1 -H 'content-type: application/json' -d '{"completed": true}'
curl -i -X DELETE localhost:3000/tasks/1                                                    # 204
curl -i localhost:3000/tasks/1                                                              # 404 {"error":"task not found"}
```

`-d` makes curl send a POST. Without the `content-type` header, curl sends a form content type and `Json` answers 415. Ctrl+C stops the server, and restarting it empties the store. Lesson 13 adds SQLite.

## Notes

### One long-running process instead of shared-nothing

| | PHP-FPM | Axum |
|---|---|---|
| Process lifetime | The framework boots for every request | One process serves every request until it stops |
| Memory after a response | Freed | Request-local values are dropped; values put in shared state stay for later requests |
| Concurrency | One request per worker process | Many requests at once, on a few Tokio worker threads |
| State shared by requests | DB, Redis, APCu, sessions | Any value in memory, if it's safe to share between threads |

Swoole, RoadRunner, and FrankenPHP worker mode are closer to the Axum model. The new problem is **shared mutable memory**: two requests on two threads can touch the store at the same moment. Rust makes you choose the sharing strategy explicitly, and it rejects unsafe sharing at compile time.

### Routes and handlers

```rust
use axum::routing::get;

Router::new()
    .route("/tasks", get(list_tasks).post(create_task))   // chain methods on one path
    .route("/tasks/{id}", get(get_task).delete(delete_task))
    .with_state(state)                                    // `Router<AppState>` → `Router`
```

- A handler is an `async fn` whose parameters are **extractors** and whose return type implements `IntoResponse`. You don't receive a request object by default; you declare the parts you need. Taking the whole `Request` is possible, as the last parameter. There is no controller base class.
- Axum 0.8 captures path parameters with `/{id}`. The `/:id` syntax from 0.7 and older blog posts panics when you build the router.
- `Router` without a type parameter means `Router<()>`: every piece of state it needs has been provided. Until `.with_state`, the router is `Router<AppState>` and can't be served.
- An unmatched path gets an empty 404, and a known path with the wrong method gets 405.

### Extractors

| Extractor | Reads | PHP / Laravel |
|---|---|---|
| `State<AppState>` | Shared state given to `with_state` | Service container |
| `Path<i64>` | `{id}` from the URL, parsed to the type | Route parameter `$id` |
| `Query<T>` | Query string into a `Deserialize` struct | `$request->query()` |
| `HeaderMap` | All headers | `$request->headers` |
| `Json<T>` | Body, parsed into a `Deserialize` type | `$request->json()` + type checks |

```rust
async fn update_task(
    State(state): State<AppState>,   // destructure the wrapper in the parameter pattern
    Path(id): Path<i64>,
    Json(input): Json<UpdateTask>,   // body extractor: last
) -> Result<Json<Task>, ApiError>
```

- **Ordering rule:** the request body is a stream that can be read only once. `Json`, `String`, `Bytes`, and `Form` consume it, so a handler can have at most one of them, and it must be the **last** parameter. Axum enforces this with traits: every parameter except the last must implement `FromRequestParts`, which sees the URL, headers, and app state but not the body (`State`, `Path`, `Query`, `HeaderMap`); these can come in any order. Only the last parameter may be a `FromRequest`-only extractor, which receives the body. A function that breaks this doesn't implement `Handler`, hence the error below.
- If an extractor fails, the handler **never runs**; Axum responds with the extractor's rejection:

  | Request | Status |
  |---|---|
  | `Path<i64>` gets `/tasks/abc` | 400 |
  | Body isn't valid JSON | 400 |
  | Valid JSON with a missing or wrongly typed field | 422 |
  | `content-type` isn't `application/json` | 415 |

  The rejection body is plain text, not `{"error": …}`. Lesson 14 converts rejections into the API's JSON error format.
- A wrong parameter order fails with a long error saying `Handler<_, _>` is not implemented. Put `#[axum::debug_handler]` on the handler (the `macros` feature is enabled in `Cargo.toml`) to get a direct message: `` `Json<_>` consumes the request body and thus must be the last argument ``.

### Responses: `IntoResponse`

Any type that implements `IntoResponse` can be a handler's return type:

| Return value | Response |
|---|---|
| `Json(task)` | 200, `content-type: application/json`, serialized body |
| `StatusCode::NO_CONTENT` | 204, empty body |
| `(StatusCode::CREATED, Json(task))` | 201 + JSON: a tuple sets the status, then the body |
| `Result<T, E>` | `T`'s response for `Ok`, `E`'s response for `Err` |

Make your own error type a response, and `?` works inside handlers:

```rust
impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, message) = match self { … };
        (status, Json(json!({ "error": message }))).into_response()  // `serde_json::json!`
    }
}

let title = validate_title(&input.title)?;   // Err(ApiError) becomes the response
```

This replaces throwing an exception and letting a Laravel/Symfony exception handler render it. The error is part of the handler's return type, so the compiler checks that every error has a response.

### Shared state: `Arc<Mutex<Store>>`

```rust
#[derive(Clone, Default)]
pub struct AppState {
    pub store: Arc<Mutex<Store>>,
}
```

Read the type from the outside in:

- **`Arc`** (atomically reference-counted) gives several owners the same value. Axum clones `AppState` for every request; cloning an `Arc` copies a pointer and increments a counter. The `Store` is never copied. It's dropped when the last `Arc` goes.
- **`Mutex`** allows one thread at a time to access the inner value. `lock()` returns a **guard** that you use like a `&mut Store`; dropping it unlocks. There is no `unlock()` call.
- `Arc` alone gives only shared `&` access, so mutating requires a `Mutex` inside it.
- `Rc` (non-atomic reference count, not `Send`) and `RefCell` (non-atomic borrow flag, not `Sync`) are the single-threaded versions. Handlers run on several threads, so Axum requires state to be `Clone + Send + Sync + 'static`, and `Rc<RefCell<Store>>` fails to compile.
- `lock()` returns a `Result`: if a thread panics while holding the guard, the mutex is "poisoned". `.unwrap()`, which passes the panic on, is the usual response: the data may be half-modified.

PHP refcounts values implicitly, and everything is freed when the request ends. `Arc` is opt-in, uses atomic operations so threads can share it, and lives as long as its last owner, possibly for the whole process.

**`std::sync::Mutex`, not `tokio::sync::Mutex`.** Every lock here is held for a few microseconds of in-memory work and never across an `.await`. The standard mutex is faster for that, and Tokio's docs recommend it. A Tokio mutex is needed only if the guard must stay alive across an `.await`, such as around an async connection. The compiler enforces this: a `std` guard is not `Send`, so a handler that keeps it across an `.await` returns a future that isn't `Send`, and Axum rejects the handler. `#[axum::debug_handler]` then reports `future cannot be sent between threads safely`.

```rust
let task = {
    let store = state.store.lock().unwrap();
    store.get(id).cloned()            // `&Task` borrows from the guard: clone it out
};                                    // guard dropped, lock released
some_async_call().await;              // fine: no guard alive here
```

Gotchas:

- `store.get(id)` returns `Option<&Task>`, and that reference can't outlive the guard. Clone what you need into the response.
- Do a read-modify-write under **one** guard. Separate `get` and `update` calls under two locks let another request change the task in between.
- Locking the `Mutex` again while the same thread still holds a guard (e.g. calling a helper that locks) deadlocks or panics; `std::sync::Mutex` is not re-entrant.
- `state.store.lock().unwrap().create(title)` holds the guard only until the end of that statement, because the guard is a temporary.

### Testing without a server

`Router` implements Tower's `Service` trait, so a test can call it like a function. No port is opened:

```rust
use tower::ServiceExt; // for `oneshot`

let response = app(AppState::default())
    .oneshot(Request::builder().uri("/tasks").body(Body::empty()).unwrap())
    .await
    .unwrap();
assert_eq!(response.status(), StatusCode::OK);
```

This is like Laravel's `$this->getJson('/tasks')` or Symfony's `KernelBrowser`. `oneshot` consumes the router; `tests/api.rs` sends each request to `app.clone()`, and every clone shares the same `Arc`, so the requests see each other's changes.

## Exercise

Implement in this order:

Part A — plain Rust, no HTTP (unit tests in `src/lib.rs`):

1. `validate_title`: `str::trim`, then `chars().count()`. `len()` counts bytes.
2. `Store::create`: increment `last_id`, insert a `Task`, return a clone. The map keeps one copy and the caller gets another.
3. `Store::list` and `Store::get`: `values().cloned().collect()`; `BTreeMap::get` already returns `Option<&Task>`.
4. `Store::update` and `Store::delete`: `get_mut(&id)?` plus `if let Some(…)` per field; `remove(&id).is_some()`.
5. `ApiError::into_response`: match to `(status, message)`, then the tuple shown above.

Part B — HTTP (`tests/api.rs`):

6. `app`: two `.route` calls and `.with_state(state)`. Import `axum::routing::get`; `post`, `patch`, and `delete` are methods on what `get` returns.
7. `list_tasks` and `get_task`: lock, clone out, and wrap in `Json`. `Option::ok_or(ApiError::NotFound)` turns a missing task into an error.
8. `create_task`: validate before locking, so an invalid title never touches the store. Return `(StatusCode::CREATED, Json(task))`.
9. `update_task`: reject a body with neither field, then validate the title. `input.title.as_deref().map(validate_title).transpose()?` gives `Option<String>`. Only then lock and update.
10. `delete_task`: `StatusCode::NO_CONTENT` or `ApiError::NotFound`.

Then `cargo run` and try the curl commands above.

Optional extension: support `GET /tasks?completed=true`. Add a `#[derive(Deserialize)] struct ListFilter { completed: Option<bool> }`, take `Query<ListFilter>` in `list_tasks`, and add tests in `tests/api.rs`. Then move `Json` before `Path` in `update_task`, read the compiler error, and add `#[axum::debug_handler]` to see the clearer message. Finally, swap `AppState.store` to `Rc<RefCell<Store>>` (`lock().unwrap()` becomes `borrow_mut()`), run `cargo build --lib`, find `cannot be sent between threads safely` among the `Handler<_, _>` errors, and revert.

## Done when

- `cargo test`, `cargo fmt --check`, and `cargo clippy --all-targets -- -D warnings` pass.
- The curl commands above work against `cargo run`.
- You can explain:
  - why `Json` must be the last handler parameter, and what happens when an extractor fails
  - what `Arc` and `Mutex` each contribute, and why `Rc<RefCell<Store>>` wouldn't compile
  - why `std::sync::Mutex` is right here, and when you would need `tokio::sync::Mutex`
  - why `get_task` clones the task instead of returning the `&Task` from `Store::get`
  - why a task created in one request is visible in the next, unlike in PHP-FPM
