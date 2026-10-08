# Lesson 10 — JSON with Serde

Parse task API request bodies into validated types and serialize responses: add the missing `#[serde(...)]` attributes and implement every `todo!()` in `src/lib.rs` until the tests pass.

## Run

From the repository root:

```bash
docker compose run --rm -w /workspace/lessons/10-serde rust cargo test
docker compose run --rm -w /workspace/lessons/10-serde rust cargo fmt --check
docker compose run --rm -w /workspace/lessons/10-serde rust cargo clippy --all-targets -- -D warnings
```

Until a function is implemented, its parameters show `unused variable` warnings. Don't run `cargo fix`.

The package is named `serde-lesson`, not `serde`, so its library name doesn't collide with the `serde` dependency.

## Notes

### Serde and serde_json

```bash
cargo add serde --features derive   # the framework: Serialize/Deserialize traits + derive macros
cargo add serde_json                # one format; TOML, YAML, CSV crates plug into the same traits
```

```rust
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Task {
    pub id: i64,
    pub title: String,
    pub completed: bool,
}

let task: Task = serde_json::from_str(body)?;     // decode + hydrate + type-check in one step
let json: String = serde_json::to_string(&task)?; // {"id":1,"title":"Buy milk","completed":false}
```

| serde_json | PHP |
|---|---|
| `from_str::<T>(&str)`, `from_slice` | `json_decode($s, true)` + hand-written hydration and checks |
| `to_string(&v)`, `to_string_pretty` | `json_encode($v)`, `JSON_PRETTY_PRINT` |
| `Value`, `json!({"a": 1})` | an untyped assoc array |
| `to_value(&v)`, `from_value::<T>(value)` | convert between typed and untyped |

- The derive generates code at compile time; there is no runtime reflection. `Serialize` ≈ `JsonSerializable`. PHP has no built-in inverse; `Deserialize` replaces `$data['title'] ?? null` plus `is_string()` checks.
- Serialization ignores Rust visibility: private fields are written too.
- Serde is strict where `json_decode` is loose: `"42"` is not an `i64`, `1.0` is not an `i64`, and `null` is not a `String`. Each is an error before your code runs.
- Tests compare `serde_json::to_value(&x)` with `json!(…)`. `Value` objects compare as maps, so key order doesn't matter; raw string comparison would depend on it.

### Shape first, then rules

```rust
#[derive(Deserialize)]
pub struct CreateTaskInput { pub title: String }  // shape: serde checks it

pub struct NewTask { title: String }              // rules: private field, built only via TryFrom

impl TryFrom<CreateTaskInput> for NewTask {
    type Error = ValidationError;
    fn try_from(input: CreateTaskInput) -> Result<Self, Self::Error> { … }
}

let new_task = NewTask::try_from(input)?;         // or `let new_task: NewTask = input.try_into()?;`
```

- Serde answers "is this the right shape?". Domain rules (trimmed, 1–200 chars) are your code.
- Because `NewTask`'s `title` field is private and `try_from` is the only constructor, a `NewTask` with a bad title can't exist. Code that receives one doesn't re-check. This is "parse, don't validate".
- PHP equivalent: a request DTO plus a value object with a private constructor and a static factory that throws. Here the failure is an `Err` in the signature, and the caller can't reach the value without handling it.
- `TryFrom` is lesson 09's fallible conversion trait. Implementing it gives you `try_into()` for free.

### Attributes

Container attributes go on the struct, below `#[derive]`; field attributes go on the field. Combine them in one: `#[serde(default, skip_serializing_if = "…")]`.

| Attribute | Where | Effect |
|---|---|---|
| `rename = "error"` | field | The JSON key differs from the Rust name, in both directions |
| `rename_all = "camelCase"` | container | `task_id` ↔ `taskId` for every field. Also `"snake_case"`, `"kebab-case"`, `"SCREAMING_SNAKE_CASE"`, … |
| `deny_unknown_fields` | container | An unknown key is an error. By default it is silently ignored |
| `default` | field | A missing key becomes `Default::default()`: `false`, `0`, `""`, empty `Vec` |
| `default = "path::to_fn"` | field | A missing key becomes `to_fn()` |
| `default` | container | Missing fields come from the struct's own `Default` impl |
| `skip_serializing_if = "Option::is_none"` | field | Omit the key when the predicate returns `true` |
| `skip` | field | Never written; deserializing fills it with `Default::default()` |

Gotcha: a derived struct also deserializes from a JSON **array** of its field values in order. `[null, true]` is a valid `UpdateTaskInput`. `deny_unknown_fields` doesn't change that.

### Optional fields

| JSON | `title: Option<String>` |
|---|---|
| key missing | `None`. No `#[serde(default)]` needed |
| `"title": null` | `None` |
| `"title": "Walk"` | `Some("Walk")` |

- Serializing `None` writes `"title": null` unless the field has `skip_serializing_if = "Option::is_none"`. `UpdateTaskInput` derives `Serialize` so a Rust client, such as an API test, can build `PATCH` bodies from it. Sending only the changed keys is what makes it a partial update.
- PHP can tell "missing" from `null` with `array_key_exists`. A plain `Option<T>` can't (see the extension).

### Borrow in responses, own in requests

```rust
#[derive(Serialize)]
pub struct TaskList<'a> {
    tasks: &'a [Task], // borrowed from the store, not cloned
    count: usize,
}
```

- `Serialize` only reads, so a response struct can borrow (lesson 07). The `'a` says a `TaskList` can't outlive the tasks it points at.
- Input structs own their data (`String`, not `&str`). A JSON string with an escape such as `"a\nb"` must be decoded into new memory, so a `&'de str` field fails on it: `invalid type: string "a\nb", expected a borrowed string`.

### serde_json errors

```rust
let err = serde_json::from_str::<CreateTaskInput>(r#"{"title": 42}"#).unwrap_err();
err.to_string(); // "invalid type: integer `42`, expected a string at line 1 column 12"
err.line();      // 1
err.column();    // 12
err.classify();  // Category::Data   (also Syntax, Eof, Io)
```

| `Category` | Meaning |
|---|---|
| `Syntax` | Not JSON: `{"title": yes}`, trailing comma, trailing characters |
| `Eof` | Input ended mid-value: `""`, `{"title": "a` |
| `Data` | Valid JSON that doesn't fit the type: wrong type, missing or unknown field, `true` instead of an object |
| `Io` | The reader failed. Only `from_reader` produces it |

- `serde_json::Error` is neither `Clone` nor `PartialEq`. Copy what you need (message, line, column) into your own enum; then you can derive both and compare errors in tests.
- `?` converts the error through `From`. With `From<serde_json::Error>` and `From<ValidationError>` implemented for `ApiError`, one function can `?` both. PHP equivalent: catching `JsonException` and rethrowing a domain exception, except here it's visible in the signature.

| Failure | Status | Body |
|---|---|---|
| Not JSON: `Syntax`, `Eof` (and `Io`) | 400 | `{"error": "malformed JSON: …"}` |
| Valid JSON, wrong shape: `Data` | 422 | `{"error": "invalid body: …"}` |
| A domain rule: blank or too-long title, empty update | 422 | `{"error": "title must not be empty"}` |
| Unknown task | 404 | `{"error": "task not found"}` |

This lesson uses the same split as Axum's `Json` extractor (lessons 12 and 14), so a body gets the same status everywhere: 400 means "I couldn't read it", 422 means "I read it and it's wrong". `{"completed": yes}` is 400; `{"completed": "yes"}` is 422.

### Counting characters

- `str::len()` counts bytes, like PHP's `strlen`. `chars().count()` counts Unicode scalar values, close to `mb_strlen`. `"é".len()` is 2. The title limit counts chars.
- `str::trim` strips Unicode whitespace, including the non-breaking space `U+00A0`. PHP's `trim()` only strips ASCII ` \t\n\r\0\x0B`. The sets aren't nested: `str::trim` does not strip `\0`, which PHP's `trim()` does.

## Exercise

Steps 1–4 are attributes only. Until the attribute is right, their tests fail with assertion or `unwrap` panics, not `todo!()`. The `LegacyTask` tests then stop at step 5's `todo!()`.

Part A — wire format:

1. `ErrorBody`: the field is `message`, the JSON key is `error`.
2. `CreateTaskInput`: reject unknown keys.
3. `UpdateTaskInput`: reject unknown keys, and omit `None` fields when serializing (one attribute per field).
4. `LegacyTask`: one container attribute for the camelCase keys, one field attribute for the optional `isDone`.
5. `From<LegacyTask> for Task`, then `TaskList::new`.

Part B — validation:

6. `validate_title`: `trim`, then `chars().count()`. Return an owned `String`.
7. `Display` for `ValidationError`: `match self` and `write!(f, …)`. Use `MAX_TITLE_CHARS` in the message, not a literal `200`.
8. `NewTask`: `try_from`, `title`, `into_task`. `into_task(self, …)` takes ownership, so the title moves into the `Task` without a clone.
9. `TaskPatch`: `try_from`, `title`, `completed`, `apply`.
   - Check `EmptyUpdate` first.
   - `input.title.map(|t| validate_title(&t))` is an `Option<Result<…>>`; `.transpose()` turns it into `Result<Option<…>>`, ready for `?`.
   - `title()` returns `Option<&str>` from an `Option<String>`: `as_deref()`.

Part C — errors:

10. `From<serde_json::Error>` and `From<ValidationError>` for `ApiError`, then `status`, `Display`, and `body` (reuse `Display` with `self.to_string()`).
    - `use serde_json::error::Category;`, then `match err.classify()`: `Data` → `InvalidBody`, the other three → `MalformedJson`.
    - Read `to_string()`, `line()`, and `column()` once, before the `match`; both arms use them.
11. `parse_create`, `parse_update`: `serde_json::from_str` into the raw input, then `try_from`. Two lines each, using `?`.

Optional extension — absent versus `null`. `PATCH` can't clear a value yet, because a missing key and `null` both become `None`. Add a nullable note:

- `Task`: `note: Option<String>` with `skip_serializing_if = "Option::is_none"`, so tasks without a note keep the contract's JSON. Fix each `Task { … }` the compiler points at.
- `UpdateTaskInput` and `TaskPatch`: `note: Option<Option<String>>`. `None` = leave unchanged, `Some(None)` = clear, `Some(Some(text))` = set. Count it in the `EmptyUpdate` check.
- A plain `Option<Option<T>>` still turns `null` into the outer `None`. Serde calls a `deserialize_with` function only when the key is present:

```rust
fn double_option<'de, D, T>(de: D) -> Result<Option<Option<T>>, D::Error>
where
    D: serde::Deserializer<'de>,
    T: Deserialize<'de>,
{
    Option::<T>::deserialize(de).map(Some)
}

#[serde(default, deserialize_with = "double_option", skip_serializing_if = "Option::is_none")]
pub note: Option<Option<String>>,
```

`default` is required: with `deserialize_with`, a missing key is otherwise a `missing field` error. Add tests for all three cases. The `serde_with` crate ships the same helper as `serde_with::rust::double_option`.

## Done when

- `cargo test`, `cargo fmt --check`, and `cargo clippy --all-targets -- -D warnings` pass.
- You can explain:
  - why `CreateTaskInput` and `NewTask` are separate types, and why `NewTask`'s field is private
  - why `TaskList` can borrow its tasks but `CreateTaskInput` owns its `String`
  - which bodies produce 400 and which produce 422, and where `?` converts one error type into another
  - what a missing key, `null`, and `skip_serializing_if` each do to an `Option` field
