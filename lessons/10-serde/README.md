# Lesson 10 — JSON with Serde

Parse task API request bodies into validated types and serialize responses: add the missing `#[serde(...)]` attributes and implement every `todo!()` in `src/lib.rs`.

## Run

Inside the container:

```bash
cd /workspace/lessons/10-serde
cargo test
cargo fmt --check
cargo clippy --all-targets -- -D warnings
```

Don't run `cargo fix`. The package is named `serde-lesson` so it doesn't clash with the `serde` dependency.

## Notes

### Serialize and Deserialize

Derive `Serialize` and `Deserialize`, and `serde_json` can turn a struct into JSON and back.

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct Task {
    id: i64,
    title: String,
    completed: bool,
}

let task: Task = serde_json::from_str(r#"{"id": 1, "title": "Buy milk", "completed": false}"#)?;
println!("{}", task.title);                    // Buy milk
println!("{}", serde_json::to_string(&task)?); // {"id":1,"title":"Buy milk","completed":false}
```

`from_str` does PHP's `json_decode`, the hydration into an object, and the type checks in one step.

### Strict types

Each JSON value must match its field's type, or `from_str` returns an error before your code runs.

```rust
serde_json::from_str::<Task>(r#"{"id": "1", "title": "Buy milk", "completed": false}"#)
// Err: invalid type: string "1", expected i64 at line 1 column 10
```

- `"1"` is not an `i64`, and `null` is not a `String`.
- A missing key is an error, unless the field is an `Option` or has `#[serde(default)]`.
- An unknown key is silently ignored.

### Comparing JSON in tests

`serde_json::to_value` turns a value into a `serde_json::Value`, and `json!` builds one by hand. Objects compare as maps, so key order doesn't matter.

```rust
use serde_json::json;

let value = serde_json::to_value(&task)?;
assert_eq!(value, json!({"completed": false, "id": 1, "title": "Buy milk"})); // passes
```

### Renaming keys

`rename` changes one field's JSON key. `rename_all` on the struct changes every key. Both work in both directions.

```rust
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Report {
    task_id: i64,
    #[serde(rename = "error")]
    message: String,
}

let report = Report { task_id: 7, message: "task not found".to_string() };
println!("{}", serde_json::to_string(&report)?); // {"taskId":7,"error":"task not found"}
```

Struct attributes go below `#[derive]`; field attributes go on the field.

### Rejecting unknown keys

`#[serde(deny_unknown_fields)]` on the struct turns an unknown key into an error.

```rust
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Signup {
    email: String,
}

serde_json::from_str::<Signup>(r#"{"email": "a@b.c", "admin": true}"#)
// Err: unknown field `admin`, expected `email` at line 1 column 26
```

### Default values

`#[serde(default)]` on a field turns a missing key into the type's default: `false`, `0`, `""`, an empty `Vec`.

```rust
#[derive(Deserialize)]
struct Settings {
    name: String,
    #[serde(default)]
    dark_mode: bool,
}

let s: Settings = serde_json::from_str(r#"{"name": "Ana"}"#)?;
println!("{}", s.dark_mode); // false
```

### Optional fields

An `Option<T>` field needs no attribute to deserialize:

```text
{}                  → title: None
{"title": null}     → title: None
{"title": "Walk"}   → title: Some("Walk")
```

When serializing, `None` becomes `null`. `skip_serializing_if = "Option::is_none"` leaves the key out instead:

```rust
#[derive(Serialize)]
struct Profile {
    name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    nickname: Option<String>,
}

let p = Profile { name: "Ana".to_string(), nickname: None };
println!("{}", serde_json::to_string(&p)?); // {"name":"Ana"}
// without the attribute:                      {"name":"Ana","nickname":null}
```

PHP can tell a missing key from `null` with `array_key_exists`; a plain `Option` can't.

### Shape first, then rules

Serde checks the shape. Domain rules, like "1–200 characters after trimming", are your code. Put them in a `TryFrom` that builds a type with a private field:

```rust
pub struct Email(String); // private field: `try_from` is the only way in

impl TryFrom<String> for Email {
    type Error = String;

    fn try_from(raw: String) -> Result<Self, Self::Error> {
        if raw.contains('@') {
            Ok(Email(raw))
        } else {
            Err(format!("not an email: {raw}"))
        }
    }
}

Email::try_from("ana@example.com".to_string()); // Ok(Email("ana@example.com"))
Email::try_from("ana".to_string());             // Err("not an email: ana")
```

An invalid `Email` can't exist, so code that receives one never checks again. In PHP: a value object with a private constructor and a static factory that throws.

### Borrowing in responses

A struct that is only serialized can borrow its data, so building a response doesn't clone anything.

```rust
#[derive(Serialize)]
pub struct TaskList<'a> {
    tasks: &'a [Task], // borrowed from the caller
    count: usize,
}
```

Request structs own their data (`String`, not `&str`), because a JSON string like `"a\nb"` must be decoded into new memory.

### serde_json errors

A `serde_json::Error` has a message, a position, and a category.

```rust
let err = serde_json::from_str::<CreateTaskInput>(r#"{"title": 42}"#).unwrap_err();
println!("{err}");                // invalid type: integer `42`, expected a string at line 1 column 12
println!("{}", err.line());       // 1
println!("{}", err.column());     // 12
println!("{:?}", err.classify()); // Data
```

`err.classify()` returns a `serde_json::error::Category`:

- `Syntax`: not JSON, e.g. `{"title": yes}` or a trailing comma
- `Eof`: the input ended too early, e.g. `""`
- `Data`: valid JSON of the wrong shape: wrong type, missing or unknown field
- `Io`: the reader failed (only `from_reader` does this)

`serde_json::Error` is neither `Clone` nor `PartialEq`, so copy its message, line, and column into your own enum.

### 400 or 422

Every failure becomes a status and an `{"error": "…"}` body:

- 400: not JSON (`Syntax`, `Eof`, `Io`) → `malformed JSON: …`
- 422: valid JSON, wrong shape (`Data`) → `invalid body: …`
- 422: a domain rule failed → `title must not be empty`
- 404: unknown task → `task not found`

So `{"completed": yes}` is 400 and `{"completed": "yes"}` is 422, the same split as Axum's `Json` extractor in lessons 12 and 14.

### `?` converts errors

`?` passes the error through `From`. Implement `From<TheirError> for YourError`, and `?` converts it for you:

```rust
impl From<ParseIntError> for AppError {
    fn from(err: ParseIntError) -> Self {
        AppError::BadNumber(err.to_string())
    }
}

fn parse_age(s: &str) -> Result<u8, AppError> {
    let age = s.parse::<u8>()?; // a ParseIntError becomes an AppError
    Ok(age)
}
```

### Option helpers

- `as_deref()` turns `Option<String>` into `Option<&str>`.
- `transpose()` turns `Option<Result<T, E>>` into `Result<Option<T>, E>`, ready for `?`.

```rust
let n: Option<i32> = Some("42").map(|s| s.parse::<i32>()).transpose()?;
println!("{n:?}"); // Some(42)

let title = Some("Walk".to_string());
println!("{:?}", title.as_deref()); // Some("Walk")
```

### Counting characters

`len()` counts bytes, like PHP's `strlen`. `chars().count()` counts characters, like `mb_strlen`.

```rust
let s = "  é  ".trim();
println!("{}", s.len());           // 2
println!("{}", s.chars().count()); // 1
```

## Exercise

Steps 1–4 are attributes only; until each is right, its tests fail with assertion or `unwrap` panics, not `todo!()`.

Part A — wire format:

1. `ErrorBody`: the field is `message`, the JSON key is `error`.
2. `CreateTaskInput`: reject unknown keys.
3. `UpdateTaskInput`: reject unknown keys, and omit `None` fields when serializing (one attribute per field).
4. `LegacyTask`: one struct attribute for the camelCase keys, one field attribute so a missing `isDone` is `false`.
5. `From<LegacyTask> for Task`, then `TaskList::new` (`count` is `tasks.len()`).

Part B — validation:

6. `validate_title`: `trim`, then `chars().count()`; return an owned `String`.
7. `Display` for `ValidationError`: `match self` and `write!(f, …)`, using `MAX_TITLE_CHARS`, not `200`.
8. `NewTask`: `try_from`, `title`, `into_task`. `into_task(self, …)` moves the title into the `Task` without a clone.
9. `TaskPatch`: check `EmptyUpdate` first, then `input.title.map(|t| validate_title(&t)).transpose()?`; `title()` uses `as_deref()`.

Part C — errors:

10. `ApiError`: both `From` impls (`match err.classify()`: `Category::Data` → `InvalidBody`, the rest → `MalformedJson`), then `status`, `Display`, and `body` (`self.to_string()`).
11. `parse_create`, `parse_update`: `serde_json::from_str` into the raw input, then `try_from`, two lines each with `?`.

Optional: add a nullable `note` that `PATCH` can clear, as `Option<Option<String>>` with `#[serde(default, deserialize_with = …)]` (see `serde_with::rust::double_option`), with tests for missing, `null`, and a value.

## Done when

- `cargo test`, `cargo fmt --check`, and `cargo clippy --all-targets -- -D warnings` pass.
- You can explain:
  - why `CreateTaskInput` and `NewTask` are separate types, and why `NewTask`'s field is private
  - which bodies produce 400 and which produce 422, and where `?` converts one error type into another
  - what a missing key, `null`, and `skip_serializing_if` each do to an `Option` field
