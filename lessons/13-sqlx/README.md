# Lesson 13 — Persistence with SQLx

Store tasks in SQLite so they survive restarts: implement every `todo!()` in `src/lib.rs` until the tests pass.

## Run

Inside the container:

```bash
cd /workspace/lessons/13-sqlx
cargo test
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo run -- add "Buy milk"
cargo run -- list
```

The first build compiles SQLite from C, so it takes a few minutes. `cargo run` keeps its data in `tasks.db` here (git-ignored); delete it to start over. Don't run `cargo fix`.

## Notes

### SQLx

SQLx is an async SQL library: every call returns a future you `.await`, so tests use `#[tokio::test]`. Its `sqlite` feature bundles SQLite itself, so nothing else needs installing.

### Connection pool

A `SqlitePool` keeps a few connections open and lends one to each query.

```rust
use std::str::FromStr;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};

let options = SqliteConnectOptions::from_str("sqlite:tasks.db")?.create_if_missing(true);
let pool = SqlitePoolOptions::new()
    .max_connections(5)
    .connect_with(options)
    .await?;
```

- Without `create_if_missing(true)`, a missing file is an error.
- `pool.clone()` is cheap: clones share the same connections. That's why `TaskRepo` derives `Clone`.

PHP-FPM opens a PDO connection per request. A Rust server is one long-running process serving many requests at once, so it shares a pool.

### Queries

`sqlx::query_as` runs SQL and turns each row into a struct. Each `?` in the SQL is a placeholder, filled in order by `.bind()`:

```rust
let task: Option<Task> = sqlx::query_as("SELECT id, title, completed FROM tasks WHERE id = ?")
    .bind(1)
    .fetch_optional(&pool)
    .await?;
println!("{task:?}"); // Some(Task { id: 1, title: "Buy milk", completed: false })
```

Pick the fetch method by how many rows you expect:

- `fetch_one`: exactly one row; no row is `Err(RowNotFound)`.
- `fetch_optional`: `Option<T>`.
- `fetch_all`: `Vec<T>`.

For SQL that returns no rows, use `sqlx::query(…).execute(&pool)`. Its `.rows_affected()` counts the matched rows; in SQLite that includes rows whose value didn't change.

### Mapping rows with `FromRow`

`#[derive(sqlx::FromRow)]` on `Task` maps columns to fields **by name, at runtime**. A missing column fails when the query runs, not when it compiles:

```rust
// with one row in the table
let result = sqlx::query_as::<_, Task>("SELECT id, title FROM tasks")
    .fetch_one(&pool)
    .await;
println!("{result:?}"); // Err(ColumnNotFound("completed"))
```

- List the columns instead of `SELECT *`.
- `::<_, Task>` names the row type; the `_` (the database) is inferred. Annotating the variable works too.
- SQLite has no boolean type: `completed` is stored as 0/1, and SQLx decodes it into a `bool`.

### `RETURNING`

`RETURNING` gives back the inserted or updated row in the same statement:

```rust
let task: Task =
    sqlx::query_as("INSERT INTO tasks (title) VALUES (?) RETURNING id, title, completed")
        .bind("Buy milk")
        .fetch_one(&pool)
        .await?;
println!("{}", task.id); // 1
```

This replaces PDO's `lastInsertId()` plus a second `SELECT`.

### Partial updates with `COALESCE`

Binding `None` sends SQL `NULL`. `COALESCE(?, title)` then keeps the old value:

```rust
let new_title: Option<&str> = None;
sqlx::query("UPDATE tasks SET title = COALESCE(?, title) WHERE id = ?")
    .bind(new_title) // NULL: the title stays as it was
    .bind(1)
    .execute(&pool)
    .await?;
```

### No SQL built from strings

SQLx 0.9 rejects SQL built at runtime, at compile time:

```rust
sqlx::query(format!("SELECT * FROM tasks WHERE title = '{title}'"))
// error[E0277]: dynamic SQL strings should be audited for possible injections
```

`query` takes a string literal (`&'static str`), so values must go through `.bind()`.

### Migrations

`sqlx::migrate!()` embeds the files in `migrations/` into the binary. `.run()` applies the ones not applied yet:

```rust
sqlx::migrate!().run(&pool).await?; // first run: creates the tasks table
sqlx::migrate!().run(&pool).await?; // later runs: nothing to do
```

- Files are named `<version>_<description>.sql` and run in version order. SQLx records applied ones in `_sqlx_migrations`, like Doctrine's migration table.
- Never edit an applied migration: its checksum changes and the next run fails with `VersionMismatch`. Add a new file instead.
- `build.rs` tells Cargo to rebuild when you add a migration file.

### Transactions

`pool.begin()` starts a transaction. Run each query on `&mut *tx`, then `commit()`:

```rust
let mut tx = pool.begin().await?;
sqlx::query("INSERT INTO tasks (title) VALUES (?)")
    .bind("a")
    .execute(&mut *tx)
    .await?;
tx.commit().await?;
```

Dropping a transaction without `commit()` rolls it back:

```rust
{
    let mut tx = pool.begin().await?;
    sqlx::query("DELETE FROM tasks").execute(&mut *tx).await?;
} // tx dropped: rolled back, nothing deleted
```

- So every `?` or early `return` before `commit()` is a rollback. No `try { … } catch { rollBack(); }` needed.
- `&mut *tx` lends the transaction's connection to one query. Plain `&mut tx` doesn't compile.
- A query on `&self.pool` runs **outside** the transaction, on another connection. With a one-connection pool it waits for the connection the transaction holds and fails with `PoolTimedOut`. So `create_many` can't call `self.create()`.

### In-memory databases for tests

`SqliteConnectOptions::new()` opens an in-memory database. Each connection gets its own empty one, and it disappears when that connection closes. So keep exactly one connection, open for the pool's whole life:

```rust
let pool = SqlitePoolOptions::new()
    .max_connections(1)
    .idle_timeout(None)
    .max_lifetime(None)
    .connect_with(SqliteConnectOptions::new())
    .await?;
```

Every call gives a new, separate database, so tests run in parallel without sharing data.

### Validation errors vs database errors

`RepoError::Validation` is the client's mistake (a 422 in lesson 14). `RepoError::Database` is a server failure (a 500 that hides the details). `From<sqlx::Error> for RepoError` lets `?` convert SQLx errors:

```rust
let result = sqlx::query("DELETE FROM tasks").execute(&self.pool).await?; // sqlx::Error → RepoError
```

### Counting characters

`trim()` returns a slice of the input, without copying. `len()` counts bytes; `chars().count()` counts characters:

```rust
let title = "  café ".trim();
println!("{title}");                   // café
println!("{}", title.len());           // 5
println!("{}", title.chars().count()); // 4
```

Like PHP's `strlen()` vs `mb_strlen()`.

## Exercise

Implement in this order:

1. `From<sqlx::Error> for RepoError`: wrap the error in `Database`.
2. `validate_title`: `trim()`, then check `chars().count()`.
3. `connect_in_memory`: the one-connection pool above, then `sqlx::migrate!().run(&pool).await?`.
4. `connect`: `SqliteConnectOptions::from_str(url)?.create_if_missing(true)` (needs `use std::str::FromStr;`), a pool, then migrations.
5. `create`: validate first, then `INSERT … RETURNING id, title, completed` with `query_as` and `fetch_one`.
6. `get` and `list`: `fetch_optional` and `fetch_all`; the list needs `ORDER BY id`.
7. `delete`: `execute`, then `rows_affected()`.
8. `update`: validate before any query, then one `UPDATE … COALESCE … RETURNING` with `fetch_optional`. For the `Option<String>` title, look up `Option::as_deref` and `Option::transpose`.
9. `create_many`: `begin`, insert each title on `&mut *tx`, `commit`; `?` handles the rollback.
10. `complete_many`: `UPDATE tasks SET completed = 1 WHERE id = ?` per id; if `rows_affected()` is 0, `return Ok(false)` without committing.

Then try the CLI: `add` a few tasks, `done` or `rm` some, and `list` again. Each `cargo run` is a new process reading the same `tasks.db`.

Optional: add `migrations/0002_add_priority.sql` (`ALTER TABLE tasks ADD COLUMN priority INTEGER NOT NULL DEFAULT 0`) and a `priority: i64` field on `Task`, then fix each `ColumnNotFound`.

## Done when

- `cargo test`, `cargo fmt --check`, and `cargo clippy --all-targets -- -D warnings` pass.
- `cargo run -- list` shows tasks added by earlier runs.
- You can explain:
  - why `.bind()` is safe where `format!` into SQL isn't
  - what happens to `create_many`'s transaction when `?` returns early
  - why queries inside a transaction use `&mut *tx`, not `&self.pool`
