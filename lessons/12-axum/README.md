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

The server needs a published port, so start it from the repository root on your Mac:

```bash
docker compose run --rm -p 3000:3000 -w /workspace/lessons/12-axum rust cargo run
```

Then, from another terminal:

```bash
curl -i localhost:3000/tasks -H 'content-type: application/json' -d '{"title": "Buy milk"}'      # 201
curl -s localhost:3000/tasks                                                                # [{"id":1,…}]
curl -s -X PATCH localhost:3000/tasks/1 -H 'content-type: application/json' -d '{"completed": true}'
curl -i -X DELETE localhost:3000/tasks/1                                                    # 204
curl -i localhost:3000/tasks/1                                                              # 404 {"error":"task not found"}
```

- Don't run `cargo fix`.
- `tests/api.rs` only runs once the unit tests pass; `cargo test --test api` runs it alone.
- Without the `content-type` header, `Json` answers 415. Restarting the server empties the store.

## Notes

### One process for every request

PHP-FPM boots your app for every request and frees everything afterwards. An Axum server is one long-running process, so data in shared state stays there for the next request.

```text
POST /tasks {"title": "Buy milk"}  → 201, stored in memory
GET  /tasks                        → [{"id":1,"title":"Buy milk","completed":false}]
```

Many requests run at the same time on several threads, so shared data must be safe to use from more than one thread.

### Routes

`.route(path, …)` maps a path to handlers. Chain several methods on one path, and capture path segments with `{name}`.

```rust
use axum::routing::get;

let app = Router::new()
    .route("/tasks", get(list_tasks).post(create_task))
    .route("/tasks/{id}", get(get_task).delete(delete_task))
    .with_state(state);
```

- Axum 0.8 uses `/{id}`. The older `/:id` syntax panics.
- `.with_state(state)` gives every handler access to `state`.
- An unknown path gets 404; a known path with the wrong method gets 405.

### Handlers and extractors

A handler is an `async fn`. Each parameter is an **extractor** that pulls one part out of the request.

```rust
async fn hello(Path(name): Path<String>) -> String {
    format!("hello, {name}")
}
// GET /hello/Ana → 200 hello, Ana
```

- `State<T>`: the shared state from `with_state`
- `Path<T>`: path segments like `{id}`, parsed into `T`
- `Query<T>`: the query string, into a `Deserialize` struct
- `HeaderMap`: the headers
- `Json<T>`: the body, into a `Deserialize` type

Instead of reading `$request->json()`, you declare the parts you need.

### The body extractor goes last

The body can be read only once. `Json`, `String`, `Bytes`, and `Form` read it, so a handler takes at most one of them, as its **last** parameter.

```rust
async fn rename(Path(id): Path<i64>, Json(input): Json<CreateTask>) -> String {
    format!("task {id} is now {}", input.title)
}
// PATCH /tasks/3 {"title": "Walk"} → 200 task 3 is now Walk
```

In the wrong order, the error says `Handler<_, _>` is not implemented. `#[axum::debug_handler]` on the handler gives a clearer message.

### When an extractor fails

If an extractor fails, the handler never runs and Axum answers on its own:

- `Path<i64>` gets `/tasks/abc` → 400
- the body isn't valid JSON → 400
- valid JSON with a missing or wrongly typed field → 422
- `content-type` isn't `application/json` → 415

These bodies are plain text, not `{"error": …}`. Lesson 14 changes that.

### Responses

A handler can return any type that implements `IntoResponse`:

- `Json(task)` → 200 with a JSON body
- `StatusCode::NO_CONTENT` → 204 with an empty body
- `(StatusCode::CREATED, Json(task))` → 201 with a JSON body
- `Result<T, E>` → `T`'s response for `Ok`, `E`'s response for `Err`

### Errors as responses

Implement `IntoResponse` for your error type, and every `Err` a handler returns becomes a response:

```rust
enum AppError {
    Forbidden,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AppError::Forbidden => (StatusCode::FORBIDDEN, "not allowed"),
        };
        (status, Json(json!({ "error": message }))).into_response()
    }
}
// Err(AppError::Forbidden) → 403 {"error":"not allowed"}
```

Now `?` works in handlers. It replaces throwing an exception for Laravel's exception handler to render.

### Shared state: `Arc<Mutex<Store>>`

```rust
#[derive(Clone, Default)]
pub struct AppState {
    pub store: Arc<Mutex<Store>>,
}
```

- `Arc` lets many owners share one value. Axum clones `AppState` for every request, which copies a pointer, never the `Store`.
- `Mutex` lets one thread at a time use the value. `lock()` returns a guard that acts like `&mut Store`; dropping the guard unlocks.
- `lock()` returns a `Result`; call `.unwrap()` on it.
- `Rc<RefCell<Store>>` is the single-thread version, so it doesn't compile here: handlers run on several threads.

### Locking

Lock, copy out what you need, and let the guard drop.

```rust
let task = {
    let store = state.store.lock().unwrap();
    store.get(id).cloned() // `&Task` borrows from the guard, so clone it out
};                         // guard dropped: unlocked
```

- Do a read-modify-write under one lock, not two.
- Locking again while you still hold the guard deadlocks or panics.
- Never hold a `std::sync::Mutex` guard across an `.await`: the handler stops being `Send`, and Axum rejects it.

### Testing without a server

`oneshot` sends one request straight to a `Router`, so tests need no port:

```rust
use tower::ServiceExt; // for `oneshot`

let response = app(AppState::default())
    .oneshot(Request::builder().uri("/tasks").body(Body::empty()).unwrap())
    .await
    .unwrap();
assert_eq!(response.status(), StatusCode::OK);
```

`oneshot` consumes the router, so `tests/api.rs` sends each request to `app.clone()`; every clone shares the same store. It's like Laravel's `$this->getJson('/tasks')`.

## Exercise

Implement in this order:

Part A — plain Rust, no HTTP (unit tests in `src/lib.rs`):

1. `validate_title`: `str::trim`, then `chars().count()` (`len()` counts bytes).
2. `Store::create`: increment `last_id`, insert a `Task`, return a clone.
3. `Store::list` and `Store::get`: `values().cloned().collect()`; `BTreeMap::get` already returns `Option<&Task>`.
4. `Store::update` and `Store::delete`: `get_mut(&id)?` plus `if let Some(…)` per field; `remove(&id).is_some()`.
5. `ApiError::into_response`: match to `(status, message)`, then the tuple from "Errors as responses".

Part B — HTTP (`tests/api.rs`):

6. `app`: two `.route` calls and `.with_state(state)`. Import `axum::routing::get`; `post`, `patch`, and `delete` chain on it.
7. `list_tasks` and `get_task`: lock, clone out, wrap in `Json`; `ok_or(ApiError::NotFound)` turns a missing task into an error.
8. `create_task`: validate before locking, then return `(StatusCode::CREATED, Json(task))`.
9. `update_task`: reject a body with neither field, then `input.title.as_deref().map(validate_title).transpose()?`, and only then lock and update.
10. `delete_task`: `StatusCode::NO_CONTENT` or `ApiError::NotFound`.

Then `cargo run` and try the curl commands above.

Optional: support `GET /tasks?completed=true` with a `Query<ListFilter>` extractor (`completed: Option<bool>`), plus tests in `tests/api.rs`.

## Done when

- `cargo test`, `cargo fmt --check`, and `cargo clippy --all-targets -- -D warnings` pass.
- The curl commands above work against `cargo run`.
- You can explain:
  - why `Json` must be the last handler parameter, and what happens when an extractor fails
  - what `Arc` and `Mutex` each contribute, and why `Rc<RefCell<Store>>` wouldn't compile
  - why `get_task` clones the task instead of returning the `&Task` from `Store::get`
