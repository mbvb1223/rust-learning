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

The first build compiles SQLite itself from C source, so it takes a while. `cargo run` keeps its data in `tasks.db` in this directory (git-ignored); delete the file to start over.

Until a function is implemented, its parameters show `unused variable` warnings. Don't run `cargo fix`.

## Notes

### The pieces

| File | Role | PHP equivalent |
|---|---|---|
| `migrations/0001_create_tasks.sql` | Schema, applied once per database | A Doctrine migration |
| `build.rs` | Rebuilds the crate when a migration is added | — |
| `src/lib.rs` | `connect`, `TaskRepo` — the exercise | A repository class over PDO |
| `src/main.rs` | A small CLI over the repository (given) | A `bin/console` command |
| `tests/persistence.rs` | File-backed tests through the public API | PHPUnit integration tests |

```toml
sqlx = { version = "0.9.0", default-features = false, features = ["runtime-tokio", "sqlite", "macros", "migrate"] }
```

- `runtime-tokio`: every SQLx call is async and needs a runtime (lesson 11). Tests use `#[tokio::test]`.
- `sqlite`: the driver plus a bundled SQLite; no system library needed.
- `migrate` + `macros`: `sqlx::migrate!()`. `macros` also provides `#[derive(sqlx::FromRow)]`.
- The package is named `sqlx-tasks`, not `sqlx`, so that `sqlx::…` always means the dependency.

### A pool, not a connection

```rust
let options = SqliteConnectOptions::from_str("sqlite:tasks.db")?.create_if_missing(true);
let pool: SqlitePool = SqlitePoolOptions::new()
    .max_connections(5)
    .connect_with(options)
    .await?;
```

- PHP-FPM opens a PDO connection per request and drops it when the request ends. A Rust server is one long-running process handling many requests at once. A pool keeps up to N connections open and lends one to each query.
- `SqlitePool` wraps an `Arc`: `clone()` is cheap and every clone shares the same connections. `TaskRepo` derives `Clone` for the same reason. Shared app state (lesson 12) can hold it directly; the pool already handles concurrent use, so no `Arc<Mutex<…>>` is needed.
- URLs: `sqlite:tasks.db` is relative to the working directory; `sqlite:///abs/path/tasks.db` is absolute. Without `create_if_missing(true)`, a missing file is an error. PDO's SQLite driver creates it without asking.
- SQLite allows one writer at a time. Concurrent writers wait for each other up to `busy_timeout` (SQLx default: 5 s).
- SQLite's C API is blocking. SQLx gives each SQLite connection its own background thread and awaits its replies, so queries don't block the Tokio runtime (lesson 11). That's why you don't need `spawn_blocking` here.

### Queries: `query`, `query_as`, `bind`

```rust
let task: Option<Task> = sqlx::query_as("SELECT id, title, completed FROM tasks WHERE id = ?")
    .bind(id)
    .fetch_optional(&self.pool)
    .await?;
```

| SQLx | PDO |
|---|---|
| `query_as::<_, Task>(sql)` | `prepare($sql)` + `PDO::FETCH_CLASS` |
| `.bind(value)` | `bindValue()` / `execute([$value])` |
| `.fetch_one(&pool)` | `fetch()`, but no row is an error (`RowNotFound`), not `false` |
| `.fetch_optional(&pool)` | `fetch()` → `Option<T>` |
| `.fetch_all(&pool)` | `fetchAll()` |
| `query(sql).execute(&pool)` then `.rows_affected()` | `exec()` / `rowCount()` |
| `.last_insert_rowid()` on that result | `lastInsertId()` |

- Every query is a prepared statement. SQLite's placeholder is `?`; Postgres uses `$1`.
- Binding an `Option<T>` sends `NULL` for `None` and the value for `Some`.
- `query_as::<_, Task>`: the `_` lets the compiler infer the database from the executor. Annotating the variable (`let task: Option<Task> = …`) works too.
- `#[derive(sqlx::FromRow)]` maps columns to fields **by name, at runtime**. A missing column is `sqlx::Error::ColumnNotFound`, a wrong type is `ColumnDecode`; neither is a compile error. List columns instead of `SELECT *`.
- SQLite has no boolean storage: `completed` is an `INTEGER` 0/1 that SQLx decodes into `bool`.
- `RETURNING id, title, completed` gives back the inserted or updated row in the same statement, with no `lastInsertId()` + `SELECT` round trip.

SQLx 0.9 rejects dynamically built SQL at compile time:

```rust
sqlx::query(format!("SELECT * FROM tasks WHERE title = '{title}'"))
// error[E0277]: dynamic SQL strings should be audited for possible injections
```

`query` takes only a `&'static str` (a literal) or an explicit `AssertSqlSafe(string)`. It's the same idea as Psalm/PHPStan's `literal-string` type. Values always go through `.bind()`.

### Why not `query!`?

`sqlx::query!` and `query_as!` check SQL against a real database **at compile time**: column names, types, and nullability. The cost is build setup: a reachable `DATABASE_URL` for every build, or offline metadata from `cargo sqlx prepare` committed to the repo, plus `sqlx-cli`. This lesson uses the runtime-checked functions, so your tests catch SQL mistakes. The roadmap leaves the macros for a later exercise.

### Migrations

```rust
sqlx::migrate!().run(&pool).await?; // embeds ./migrations into the binary at compile time
```

- Files are named `<version>_<description>.sql` and run in version order. Applied versions are recorded in `_sqlx_migrations`, like Doctrine's `doctrine_migration_versions`. A second run applies only new files, so `connect` can run migrations on every start.
- The SQL is compiled into the binary: deploys need no `migrations/` folder.
- SQLx stores a checksum per applied migration. Edit an applied file and the next run fails with `VersionMismatch(1)`. Add a new migration instead. Doctrine doesn't check this.
- Cargo doesn't watch `migrations/` for new files on its own. `build.rs` prints `cargo:rerun-if-changed=migrations` so it does; `sqlx migrate build-script` generates that file.
- The `?` compiles because `sqlx::Error` implements `From<MigrateError>`.

### Transactions

```rust
let mut tx = self.pool.begin().await?;                     // PDO::beginTransaction()
sqlx::query("…").bind(value).execute(&mut *tx).await?;     // runs inside the transaction
tx.commit().await?;                                        // PDO::commit()
```

- **Dropping a `Transaction` without `commit()` rolls it back.** Every early `return` and every `?` is a rollback, with no `try { … } catch { rollBack(); throw; }`. This is ownership doing cleanup: SQLx queues the rollback and runs it when the connection is next used or returns to the pool.
- `&mut *tx`: `Transaction` derefs to the connection. `&mut *tx` lends that connection to one query, so `tx` is still yours for the next query and for `commit()`. Plain `&mut tx` doesn't compile (E0277: `&mut Transaction` is not an `Executor`). `Executor` is implemented for `&mut SqliteConnection`, and trait bounds don't auto-deref, so you deref by hand.
- Trap: a query given `&self.pool` inside a transaction runs on another connection, **outside** the transaction. With a one-connection pool it waits for the connection the transaction holds and fails with `PoolTimedOut` after 30 s. So inside `create_many`, don't call `self.create()`; run the SQL on `&mut *tx`.
- For `UPDATE`, SQLite's `rows_affected()` counts matched rows even when the value doesn't change. MySQL's `rowCount()` counts only changed rows by default.

### In-memory databases for tests

| Connect options | Do the pool's connections share data? |
|---|---|
| `SqliteConnectOptions::new()` or `.in_memory(true)` | No: each connection opens its own empty database |
| `SqliteConnectOptions::from_str("sqlite::memory:")` | Yes: SQLx 0.9 names it uniquely in shared-cache mode; each parse is a new database |

In PHP, `new PDO('sqlite::memory:')` is one database per PDO object. With one connection per request, you never notice. With a pool and the first row's options, a migration on connection A creates tables that connection B doesn't have: `no such table: tasks`, depending on which connection the pool hands out.

Even a shared in-memory database disappears when its last connection closes. The pool closes idle connections after 10 minutes and old ones after 30 minutes by default. So `connect_in_memory` keeps exactly one connection, open for the pool's whole life:

```rust
SqlitePoolOptions::new()
    .max_connections(1)
    .idle_timeout(None)
    .max_lifetime(None)
```

A single connection also makes the pool-inside-a-transaction bug fail loudly. Each test calls `connect_in_memory()` and gets its own database, so tests run in parallel without interfering. The file-backed tests use `tempfile::tempdir()` (a dev-dependency); the directory is deleted when the guard is dropped.

### Errors

`RepoError` separates client mistakes (`Validation`, a 422 response later) from infrastructure failures (`Database`, a 500 that hides the details). `impl From<sqlx::Error> for RepoError` makes `?` convert SQLx errors. `connect` returns plain `sqlx::Error`: a failed startup is not a client error.

## Exercise

Implement in this order:

1. `From<sqlx::Error> for RepoError`.
2. `validate_title`: `trim()` returns a sub-slice of the input, with no allocation. Count with `chars().count()`; `len()` counts bytes.
3. `connect_in_memory`: the pool options above with `SqliteConnectOptions::new()` (its default filename is `:memory:`), then `sqlx::migrate!().run(&pool).await?`. Import `sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions}`.
4. `connect`: `SqliteConnectOptions::from_str(url)?.create_if_missing(true)`, a pool, migrations. `from_str` needs `use std::str::FromStr;`.
5. `create`: validate first, then `INSERT … RETURNING id, title, completed` with `query_as` and `fetch_one`. A `RowNotFound` error means the `RETURNING` clause is missing.
6. `get` and `list`: `fetch_optional` and `fetch_all`. Don't rely on SQLite's default row order; say `ORDER BY id`.
7. `delete`: `execute` and `rows_affected()`.
8. `update`: reject an empty update or an invalid title before any query. One `UPDATE … RETURNING` handles the partial update if a `NULL` bind means "keep the old value" (SQL `COALESCE`). `fetch_optional` returns `None` for a missing id. To validate an `Option<String>` with `?`, look up `Option::as_deref` and `Option::transpose`; `?` then returns the `Err` early and leaves an `Option<&str>`.
9. `create_many`: `begin`, insert each title on `&mut *tx`, `commit`. Let `?` handle the rollback.
10. `complete_many`: `UPDATE tasks SET completed = 1 WHERE id = ?` per id. If `rows_affected()` is 0, `return Ok(false)` without committing.

Then use the CLI: `add` a few tasks, `done` or `rm` some, and run `list` again. Every `cargo run` is a new process using the same data.

Optional extension: add `migrations/0002_add_priority.sql` with `ALTER TABLE tasks ADD COLUMN priority INTEGER NOT NULL DEFAULT 0`, then add `priority: i64` to `Task`. Follow the `ColumnNotFound` failures to every query that must select the new column, and update the `Task` values the tests build. Then edit `0001_create_tasks.sql` and run `cargo run -- list` against your existing `tasks.db` to see the checksum check. Revert that edit afterwards.

## Done when

- `cargo test`, `cargo fmt --check`, and `cargo clippy --all-targets -- -D warnings` pass.
- `cargo run -- list` shows tasks added by earlier runs.
- You can explain:
  - why `.bind()` is safe where `format!` into SQL isn't, and how SQLx 0.9 stops the latter at compile time
  - what happens to `create_many`'s transaction when `?` returns early, and why it needs no explicit rollback
  - why queries inside a transaction use `&mut *tx`, and what goes wrong with `&self.pool`
  - why `connect_in_memory` uses one connection with no idle timeout or maximum lifetime
  - why restarting the CLI neither re-runs nor breaks the migration
