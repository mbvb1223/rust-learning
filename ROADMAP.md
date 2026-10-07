# Rust Roadmap — fast track for a senior PHP developer

27 lessons · 3 checkpoint projects · 1 capstone · ≈6–8 weeks to production backend (at 1–2 h/day)

Syntax is compressed. Ownership, lifetimes and async are kept at full depth — they have no PHP equivalent.

## How each lesson works

1. Folder `lessons/NN-slug/` is scaffolded with notes + failing tests.
2. You implement until `cargo test` passes.
3. `cargo fmt && cargo clippy` must be clean.
4. Review → mark ✅ here → next lesson.

Status: ⬜ todo · 🟡 in progress · ✅ done

---

## Phase 1 — Syntax sprint (2–3 days)

| # | Lesson | Key topics | PHP analogy | Exercise | Status |
|---|--------|------------|-------------|----------|--------|
| 01 | [Cargo, types, control flow](lessons/01-cargo-basics) | `cargo`, `Cargo.toml`, `let`/`mut`/shadowing, scalar types, `as`, expressions vs statements, `if`/`loop`/`for` as values | Composer; `strict_types` taken to the extreme | Temperature converter + FizzBuzz variants | ⬜ |
| 02 | [Structs, enums, match, Option](lessons/02-structs-enums) | `struct`/`impl`, `::new()`, enums with data, exhaustive `match`, `if let`, `let else`, `Option<T>` | `class` without inheritance, PHP 8.1 enums + `match`, `?Type`/`??` | `Money` value object + order state machine | ⬜ |

## Phase 2 — Ownership, the core (1–2 weeks) ⚠️ don't rush

| # | Lesson | Key topics | PHP analogy | Exercise | Status |
|---|--------|------------|-------------|----------|--------|
| 03 | [Ownership & move](lessons/03-ownership) | stack vs heap, move semantics, `Copy` vs `Clone`, scope & drop | copy-on-write arrays vs object handles; GC → none | Fix 10 ownership compile errors | ⬜ |
| 04 | [Borrowing](lessons/04-borrowing) | `&T`, `&mut T`, "many readers XOR one writer", reading borrow-checker errors | `&$var` references (but checked at compile time) | Shopping-cart mutations | ⬜ |
| 05 | [Strings & slices](lessons/05-strings-slices) | `String` vs `&str`, `&[T]`, UTF-8, bytes vs `char`, why no `s[0]` | strings as byte arrays, `mb_*` | Slugify, word reverser | ⬜ |
| 06 | [Lifetimes](lessons/06-lifetimes) | `'a`, elision rules, structs holding refs, `'static` | — (new concept) | Longest-word parser returning `&str` | ⬜ |

🧩 **Checkpoint A — `text-stats` CLI** (`projects/a-text-stats`): read a file, count words/lines, top-N words, zero needless clones.

## Phase 3 — Errors, collections, iterators (3–4 days)

| # | Lesson | Key topics | PHP analogy | Exercise | Status |
|---|--------|------------|-------------|----------|--------|
| 07 | [Errors](lessons/07-errors) | `Result<T, E>`, `?`, `panic!` vs recoverable, error enums, `From`, `thiserror`, `anyhow` | exceptions → values; custom exception hierarchy | Bank account domain errors | ⬜ |
| 08 | [Collections, iterators, closures](lessons/08-iterators) | `Vec`, `HashMap`, `HashSet`, entry API, `map`/`filter`/`fold`/`collect`, `iter` vs `into_iter`, `Fn`/`FnMut`/`FnOnce`, `move` | PHP `array` split into real types; `array_map`, `Closure`, `use ($x)` | Rewrite loops as iterator chains | ⬜ |

## Phase 4 — Abstraction (≈1 week)

| # | Lesson | Key topics | PHP analogy | Exercise | Status |
|---|--------|------------|-------------|----------|--------|
| 09 | [Traits & generics](lessons/09-traits-generics) | traits, default methods, `impl Trait`, `<T: Trait>`, `where`, monomorphization | `interface` + traits; PHPStan `@template` but enforced | Generic `Repository<T>` | ⬜ |
| 10 | [Standard traits & derive](lessons/10-std-traits) | `Debug`, `Display`, `Clone`, `PartialEq`/`Eq`, `Hash`, `Ord`, `Default`, `From`/`Into` | `__toString`, magic methods | Make `Money` fully idiomatic | ⬜ |
| 11 | [Trait objects & associated types](lessons/11-dyn-traits) | `dyn Trait`, `Box<dyn Trait>`, static vs dynamic dispatch, `type Item`, custom `Iterator`, `Add`/`Index`/`Deref` | interface-typed params, `IteratorAggregate`, `ArrayAccess` | Notification plugin system | ⬜ |
| 12 | [Modules, testing, docs](lessons/12-modules-testing) | `mod`, `pub(crate)`, `use`, lib vs bin, workspaces, unit/integration/doc tests, `cargo doc` | namespaces + PSR-4, PHPUnit | Split + fully test a crate | ⬜ |

🧩 **Checkpoint B — `validator` library** (`projects/b-validator`): Symfony-Validator-style lib with rules, errors, tests, docs.

## Phase 5 — Smart pointers (3–4 days)

| # | Lesson | Key topics | PHP analogy | Exercise | Status |
|---|--------|------------|-------------|----------|--------|
| 13 | [Box, Rc, RefCell, Weak](lessons/13-smart-pointers) | heap, recursive types, shared ownership, interior mutability, ref cycles | refcounting (PHP does it for you) | Expression tree + tree with parent links | ⬜ |
| 14 | [RAII & flexible APIs](lessons/14-raii-apis) | `Drop`, guard pattern, `Cow`, `AsRef`, `impl Into<String>` | `__destruct` (but guaranteed timing) | Lock guard + zero-copy normalizer | ⬜ |

## Phase 6 — Concurrency & async (≈1 week)

| # | Lesson | Key topics | PHP analogy | Exercise | Status |
|---|--------|------------|-------------|----------|--------|
| 15 | [Threads & shared state](lessons/15-threads) | `spawn`, scoped threads, `Arc`, `Mutex`, `RwLock`, atomics, `Send`/`Sync` | share-nothing requests; Redis as shared state | Thread-safe cache | ⬜ |
| 16 | [Channels & worker pools](lessons/16-channels) | `mpsc`, worker pool, backpressure | Laravel Queue / Messenger workers | Job queue with N workers | ⬜ |
| 17 | [Async & tokio](lessons/17-async) | `Future`, `async`/`await`, tasks, `join!`, `select!`, timeouts, `reqwest` | Fibers, ReactPHP, Swoole, Guzzle promises | Concurrent HTTP fetcher | ⬜ |

🧩 **Checkpoint C — `url-checker`** (`projects/c-url-checker`): check 1000 URLs concurrently with rate limit + timeouts.

## Phase 7 — Real-world backend (≈2 weeks)

| # | Lesson | Key topics | PHP analogy | Exercise | Status |
|---|--------|------------|-------------|----------|--------|
| 18 | [Serde & clap](lessons/18-serde-clap) | `Serialize`/`Deserialize`, renames, enums; CLI args, subcommands | `json_encode`, Symfony Serializer, Artisan | `mytool` CLI with JSON I/O | ⬜ |
| 19 | [HTTP with axum](lessons/19-axum) | router, handlers, extractors, shared state, JSON, error responses | Laravel routes + controllers | CRUD API (in-memory) | ⬜ |
| 20 | [Database with sqlx](lessons/20-sqlx) | Postgres, pool, compile-time checked queries, migrations, transactions | Eloquent / Doctrine | Persist the CRUD API | ⬜ |
| 21 | [Middleware, logging, auth](lessons/21-middleware-auth) | tower layers, `tracing`, config from env, argon2, JWT, validation | middleware, Monolog, Sanctum, Form Requests | Request-ID logs + register/login | ⬜ |
| 22 | [Testing & shipping](lessons/22-test-ship) | API integration tests, test DB, release profile, multi-stage Dockerfile, CI | Laravel feature tests, deploy pipelines | Test suite + prod image < 30 MB | ⬜ |

🏆 **Capstone — port a PHP service** (`projects/capstone`): REST API with auth + Postgres + tests + Docker, ported from one of your PHP apps.

## Phase 8 — Expert (ongoing)

| # | Lesson | Key topics | PHP analogy | Exercise | Status |
|---|--------|------------|-------------|----------|--------|
| 23 | [Advanced lifetimes & traits](lessons/23-advanced-types) | variance, HRTB `for<'a>`, blanket impls, orphan rule, newtype, `PhantomData`, typestate | — | Type-safe builder + zero-copy parser | ⬜ |
| 24 | [Macros](lessons/24-macros) | `macro_rules!`, derive macros, `syn`, `quote` | PHP 8 attributes + code generation | `hashmap!{}` + `#[derive(Builder)]` | ⬜ |
| 25 | [Unsafe & FFI](lessons/25-unsafe-ffi) | raw pointers, safety contracts, calling C, `ext-php-rs` | writing PHP extensions in C | PHP extension written in Rust | ⬜ |
| 26 | [Performance & async internals](lessons/26-performance) | `criterion`, flamegraphs, allocation avoidance, `Pin`, `Waker`, hand-written `Future` | Blackfire, Xdebug profiler | Optimize a hot path 10× + mini executor | ⬜ |
| 27 | [API design & publishing](lessons/27-api-design) | idioms, semver, error design, docs, crates.io | publishing to Packagist | Publish a crate | ⬜ |

---

## Resources

| When | Resource |
|------|----------|
| Phases 1–4 | [The Rust Book](https://doc.rust-lang.org/book/) (skim ch. 1–6, read ch. 4 + 10 carefully) · [Rustlings](https://github.com/rust-lang/rustlings) |
| Phase 6 | [Tokio tutorial](https://tokio.rs/tokio/tutorial) |
| Phase 7 | *Zero To Production in Rust* — Luca Palmieri |
| Phase 8 | *Rust for Rustaceans* — Jon Gjengset · [The Rustonomicon](https://doc.rust-lang.org/nomicon/) |
| Always | [Clippy lint list](https://rust-lang.github.io/rust-clippy/master/) · [std docs](https://doc.rust-lang.org/std/) |
