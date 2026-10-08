# Rust roadmap — basics, a first API, then new territory

For a developer with 10+ years of PHP experience who is starting Rust.

The learning order is **Rust fundamentals → a basic API backend → projects beyond the usual PHP workflow**. Move quickly through familiar programming ideas, and give ownership, borrowing, and Rust's type system time to settle. Use the milestones below to decide when to move on; there is no fixed deadline or production-readiness promise.

| Stage | Goal | Milestone |
|---|---|---|
| 1. Learn the language | Write and understand small Rust programs | A tested text-statistics CLI |
| 2. Build a basic API | Apply Rust to a problem you already understand | A small CRUD API with persistence |
| 3. Explore new territory | Learn native software, parallel computation, or another chosen area | A native tool followed by a project from an optional track |

PHP can already build CLI tools, network services, and background workers. The later projects explore capabilities that Rust makes practical: native executables, CPU parallelism within one process, WebAssembly modules, and programs for constrained hardware. Each project names the new capability it teaches.

## How each lesson works

1. Scaffold one lesson at a time in `lessons/NN-slug/`, with short notes, examples, and exercise tests. Paths in this roadmap are planned locations.
2. Implement the exercise, read compiler diagnostics, and run `cargo check` frequently.
3. Run `cargo test`, `cargo fmt --check`, and `cargo clippy --all-targets -- -D warnings` from the lesson's crate directory. Use `cargo fmt` to apply formatting.
4. Explain why important values are owned, borrowed, moved, or cloned. Passing tests is one part of completion.
5. Review the code, mark the lesson complete, and continue. A lesson may take several sessions.

Use the stable Rust toolchain and edition 2024 for new crates. Commit application `Cargo.lock` files. Introduce dependencies when an exercise needs them.

Run Cargo inside the container: `docker compose up -d`, then `docker compose exec rust bash`, then `cd /workspace/lessons/01-cargo-basics && cargo test`. Adjust the lesson path and Cargo subcommand as needed. Docker builds Linux programs; use a native toolchain when a later project needs macOS integration or desktop/hardware tooling.

Status: ⬜ todo · 🟡 in progress · ✅ done

## Stage 1 — Rust fundamentals

Goal: understand how data moves through a Rust program and build a useful small tool.

| # | Lesson / folder | Key topics | Exercise | Status |
|---|---|---|---|---|
| 01 | Cargo and basic syntax — `01-cargo-basics` | Cargo, `Cargo.toml`, `fn`, scalar types, type inference, `let`/`mut`, shadowing, expressions, `if`, loops, basic `#[test]` assertions | Temperature converter with tests | ✅ |
| 02 | Model data — `02-structs-enums` | `struct`, `impl`, associated functions, enums with data, exhaustive `match`, `if let`, `let else`, `Option<T>`, `Debug` and `PartialEq` derives | Order states and a money type using integer minor units | 🟡 |
| 03 | Ownership and borrowing — `03-ownership-borrowing` | Moves, `Copy`/`Clone`, stack and heap basics, scope and cleanup, `&T`/`&mut T`, borrow-checker diagnostics | Fix ownership errors and implement shopping-cart mutations; explain each fix | ⬜ |
| 04 | Strings and slices — `04-strings-slices` | `String`/`&str`, `&[T]`, UTF-8, bytes versus `char`, borrowed function parameters | Text helpers with empty-input and Unicode tests | ⬜ |
| 05 | Errors and file I/O — `05-errors-io` | `Result<T, E>`, `?`, `match`, recoverable failures versus panics, `std::fs`, `std::io`, command-line arguments | Read a text file and report missing-file or invalid-input errors | ⬜ |
| 06 | Collections and iteration — `06-collections` | `Vec`, `HashMap`, `HashSet`, entry API, closures, `iter`/`iter_mut`/`into_iter`, `map`/`filter`/`collect`, sorting | Word frequencies and top-N results using loops, then selected iterator operations | ⬜ |
| 07 | Practical lifetimes — `07-lifetimes` | Borrowed return values, elision, basic `'a` annotations, owned return values; annotations describe relationships | Return the longest word borrowed from an input string | ⬜ |
| 08 | Modules and tests — `08-modules-testing` | `mod`, `use`, visibility, lib versus bin, unit/integration/doc tests, `cargo doc` | Separate text processing from CLI I/O and test both | ⬜ |
| 09 | Practical traits and generics — `09-traits-generics` | Traits, generic bounds, `impl Trait`, `Display`, `Default`, `From`/`TryFrom`, common derives | Add plain-text and CSV report formatters, then share the code that uses them | ⬜ |

PHP comparisons are starting points, with limits:

- Cargo covers dependency management plus building and testing; Composer is the familiar entry point.
- PHP references alias variable content. Rust references borrow a value without owning it and must obey borrowing rules.
- Rust traits describe behavior and can provide default implementations. Learn them through small examples before introducing a generic repository or framework architecture.
- `if` and `loop { break value; }` can produce useful result values. `for` and `while` evaluate to `()`.
- Cleanup normally follows ownership and scope exit, but Rust does not guarantee that destructors run in every circumstance. Lifetime annotations do not keep data alive.

**Checkpoint A — `text-stats`** (`projects/a-text-stats`)

Read a UTF-8 text file, count lines and whitespace-separated words, and print the top-N words. Define case handling and tie ordering explicitly. Handle missing files, empty input, invalid UTF-8, and invalid arguments. Test the processing functions and CLI behavior.

You are ready for Stage 2 when you can explain the ownership in this program, choose between `String` and `&str`, propagate errors, and modify the code without adding clones blindly. Reasonable owned data and deliberate clones are fine.

## Stage 2 — A basic API backend

Goal: use familiar HTTP and SQL concepts to practice Rust. Build one small task API, starting with in-memory data and then adding SQLite persistence. Learn the Rust parts in detail while using your existing backend experience.

| # | Lesson / folder | Key topics | Exercise | Status |
|---|---|---|---|---|
| 10 | JSON with Serde — `10-serde` | `Serialize`/`Deserialize`, request/response structs, optional fields, validation, JSON errors | Parse task input and serialize responses | ⬜ |
| 11 | Enough async to start — `11-async-basics` | Futures, `async`/`await`, Tokio runtime, awaiting versus spawning, `move`, spawned-task `Send`/`'static` bounds, blocking versus async work | Run two timer tasks, await their results, and explain their execution | ⬜ |
| 12 | HTTP with Axum — `12-axum` | Routes, handlers, extractors, JSON, status codes, shared state; introduce `Arc` and `Mutex` through the in-memory store | Create, list, update, and delete tasks | ⬜ |
| 13 | Persistence with SQLx — `13-sqlx` | SQLite, pools, parameterized queries, migrations, transactions, mapping rows to structs | Persist tasks across restarts | ⬜ |
| 14 | Errors, tests, and operation — `14-api-quality` | Error responses, environment configuration, `tracing`, API integration tests, isolated test databases, basic graceful shutdown | Test success and failure cases and run the API locally | ⬜ |

Learn these runtime rules alongside the API: shared application state outlives individual requests; a synchronous mutex guard must be released before `.await`; blocking I/O and substantial CPU work need an appropriate execution strategy. Introduce `spawn_blocking` for bounded blocking work, and revisit CPU parallelism in Stage 3.

Start SQLx with runtime-checked, parameterized queries. Compile-time query macros and their build-time database or offline metadata setup can be a later exercise.

**Checkpoint B — `task-api`** (`projects/b-task-api`)

| Endpoint | Behavior |
|---|---|
| `POST /tasks` | Validate a title and create a task |
| `GET /tasks` | List tasks |
| `GET /tasks/{id}` | Return a task or a consistent 404 response |
| `PATCH /tasks/{id}` | Change a title or completion status |
| `DELETE /tasks/{id}` | Delete a task with documented missing-task behavior |

Done means data survives restarts, invalid input produces useful client errors, internal failures do not expose database details, and integration tests cover the endpoint behavior. Document how to start and call the API.

Optional follow-ups: PostgreSQL, pagination, authentication, authorization, Docker packaging, and CI. Add them when you want more backend practice; completing the basic API is enough to move into Stage 3. Authentication introduces additional work such as password hashing away from async runtime workers and testing access controls.

## Stage 3 — Native tools and parallel computation

Goal: experience how Rust works outside a web application. This is a suggested first exploration after the API; you can instead choose an optional track below and learn its prerequisites as needed.

| # | Lesson / folder | Key topics | Exercise | Status |
|---|---|---|---|---|
| 15 | Native CLI and filesystem work — `15-native-cli` | `clap`, `Path`/`PathBuf`, directory traversal, buffered I/O, binary data, release builds, target-specific executables | Scan a directory and summarize file sizes, with explicit symlink and error policies | ⬜ |
| 16 | Threads and ownership — `16-threads` | OS threads, scoped threads, `Send`/`Sync`, `Arc`, `Mutex`, bounded channels, joining workers | Process a bounded stream of file jobs and collect results without a global mutable accumulator | ⬜ |
| 17 | CPU parallelism and measurement — `17-parallelism` | Rayon parallel iterators, controlled thread counts, allocation costs, sequential baselines, repeatable release-build measurements | Compute file hashes sequentially and in parallel, verify identical results, and compare timings | ⬜ |

**Checkpoint C — `duplicate-finder`** (`projects/c-duplicate-finder`)

Build a native executable that groups files by size, hashes candidate files, and byte-compares matching candidates before reporting duplicates. Begin with a sequential version, then add controlled parallel work. Report unreadable or changing files, define symlink handling, and keep this project report-only. Use a stable fixture tree to test correctness.

The new learning is native distribution and CPU work across multiple cores in one process. PHP can also scan files and hash data. Explain where time goes in your Rust implementation; disk throughput may limit the benefit of parallelism. A measured result matters more than an arbitrary speedup target.

## Choose a further direction

Choose one project based on your interests. These tracks are independent; completing all of them is unnecessary.

| Track | Starter project | Learn next | Capability to explore |
|---|---|---|---|
| Networking and systems | A TCP key-value server with a small line protocol | Async socket I/O, message framing, bounded request sizes and connections, channels, timeouts, cancellation, shutdown | Design a long-running service and manage its resources explicitly |
| Desktop applications | A Tauri desktop interface for the task API's domain or the file scanner | Native development setup, Rust commands, UI integration, file access, packaging | Combine a web UI with native application functionality |
| Browser WebAssembly | An image-filter module called from a simple JavaScript page | `wasm32` target, `wasm-bindgen`, byte buffers, browser interop, Web Workers for heavy work | Compile Rust computation into a module that runs in the browser |
| Embedded hardware | Blink an LED, then read a sensor on a supported microcontroller | Board-specific setup, `no_std`, hardware abstraction layers, interrupts, debugging | Run code directly on a device with constrained memory and hardware access |
| PHP interoperability | A small Rust-powered PHP extension | C ABI, FFI, ownership across language boundaries, extension tooling and compatibility | Move a measured CPU-heavy operation into native code while keeping a PHP application |

For the networking track, first build a bounded HTTP URL checker: process a list of URLs with at most N requests in flight, a separate requests-per-second limit, request timeouts, and cancellation. Learn task ownership and shutdown before applying them to the TCP server. An in-memory key-value store is sufficient for its first version.

For the WebAssembly track, start with one pure image-processing function and its Rust tests, then connect it to JavaScript. Reuse existing HTML/CSS knowledge. For embedded work, select the board and its supported toolchain before buying hardware or choosing crates.

**Personal capstone:** finish one chosen project, write setup instructions, test its failure cases, and explain which Rust capability it helped you learn.

## Advanced topics — learn when a project needs them

| Topic | A useful trigger |
|---|---|
| `Box`, `Rc`, `RefCell`, `Weak`, custom `Drop` | Recursive structures, single-threaded shared ownership, resource wrappers; understand reference cycles and runtime borrow checks |
| Trait objects, associated types, newtypes, orphan rules | Multiple implementations, iterator APIs, or stronger domain types; compare static and dynamic dispatch |
| `Cow`, `AsRef`, conversion traits | An API needs to accept borrowed and owned inputs clearly |
| Advanced lifetimes, HRTB, variance, `PhantomData`, typestate | A concrete library design needs these relationships |
| Declarative and procedural macros | Repeated code justifies learning `macro_rules!`, then `syn`/`quote` |
| Unsafe Rust and FFI | A hardware or interoperability boundary requires explicit safety contracts |
| Profiling, benchmarks, atomics, async internals | Measurements reveal a bottleneck, or you want to study `Pin`, `Waker`, and executors |
| Library design and publishing | A reusable crate has stable behavior, documented errors, examples, and tests |

## Resources

Read the relevant section while doing each exercise. Each resource is a reference, not an additional course to finish first.

| When | Resource |
|---|---|
| Stage 1 | [The Rust Book](https://doc.rust-lang.org/book/): skim familiar syntax; study ownership, collections, errors, generics, lifetimes, and testing as they appear in the lessons |
| Stage 1 practice | [Rustlings](https://github.com/rust-lang/rustlings): select exercises matching the current topic |
| Stage 2 | [Serde](https://serde.rs/), [Tokio tutorial](https://tokio.rs/tokio/tutorial), [Axum docs](https://docs.rs/axum/latest/axum/), [SQLx docs](https://docs.rs/sqlx/latest/sqlx/) |
| Async operation | [Tokio blocking work](https://docs.rs/tokio/latest/tokio/task/fn.spawn_blocking.html) and [graceful shutdown](https://tokio.rs/tokio/topics/shutdown) |
| Stage 3 | [Clap](https://docs.rs/clap/latest/clap/), [Rayon](https://docs.rs/rayon/latest/rayon/), and the Book's concurrency chapters |
| Desktop | [Tauri guide](https://tauri.app/start/) |
| WebAssembly | [Rust and WebAssembly](https://rust-lang.org/what/wasm/) and [wasm-bindgen guide](https://rustwasm.github.io/docs/wasm-bindgen/) |
| Embedded | [The Embedded Rust Book](https://docs.rust-embedded.org/book/) |
| Advanced, as needed | [The Rustonomicon](https://doc.rust-lang.org/nomicon/), especially for unsafe code and FFI |
| Language details | [Borrowing](https://doc.rust-lang.org/book/ch04-02-references-and-borrowing.html), [loop expressions](https://doc.rust-lang.org/reference/expressions/loop-expr.html), and [destructor guarantees](https://doc.rust-lang.org/std/mem/fn.forget.html) |
| Always | [Standard library docs](https://doc.rust-lang.org/std/) and [Clippy](https://doc.rust-lang.org/clippy/) |
