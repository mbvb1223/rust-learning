# Lesson 11 — Enough async to start

Run timers one after another, concurrently, and as spawned tasks: implement every `todo!()` in `src/lib.rs` until the tests pass, then fix the spawn errors in `src/broken.rs`.

## Run

Inside the container:

```bash
cd /workspace/lessons/11-async-basics
cargo test
cargo test --features broken
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo run
```

Until a function is implemented, its parameters show `unused variable` warnings. Don't run `cargo fix`; it renames them to `_name`.

`--features broken` (and `--all-features`) also compiles `src/broken.rs`, which fails on purpose until you fix it in step 11.

## Notes

### The runtime is a library

PHP-FPM gives each request its own process, so a blocking MySQL query or `sleep()` stalls only that request. Concurrency inside one PHP process needs an add-on: Fibers (PHP 8.1) can suspend a function, but an event loop has to resume them: Revolt, used by Amp and by ReactPHP's `react/async`. The Swoole extension replaces both with its own coroutines and scheduler.

Rust splits it the same way: `async`/`await` and the `Future` trait are part of the language; the runtime that drives futures is a crate. Tokio is the usual choice, and Axum and SQLx (lessons 12–13) run on it.

| Rust | Closest PHP | Where the comparison breaks down |
|---|---|---|
| `async fn` returns a `Future` | Amp `Future`, React promise | A Rust future is **lazy**: nothing runs until it is awaited or spawned. Amp's `async()` and React promises have already started |
| `.await` | `$future->await()` (Amp), `React\Async\await()` | Fibers are stackful. A Rust future is a compiler-generated state machine with a size fixed at compile time |
| `tokio::join!(a, b)` | `Amp\Future\await([$a, $b])`, `React\Promise\all()` | — |
| `tokio::spawn(future)` | `Amp\async($fn)`, Swoole `go($fn)` | The task may run on another thread, in parallel |
| Tokio runtime | Revolt loop, Swoole scheduler | Multi-threaded by default; a PHP event loop runs on one thread |

### Futures are lazy

```rust
async fn timer(name: &str, ms: u64) -> TimerReport { … }
// roughly: fn timer<'a>(name: &'a str, ms: u64) -> impl Future<Output = TimerReport> + 'a

let future = timer("a", 100);           // builds a state machine: no clock read, no sleep
sleep(Duration::from_millis(50)).await;
let report = future.await;              // the body starts now: started_at is +50 ms
```

- `.await` polls the future. When it can't finish yet (a timer, a socket), it returns `Pending`, and the runtime runs other tasks on that thread until it is woken.
- `.await` only works inside `async` code. `#[tokio::main]` makes `main` async.
- An un-awaited future triggers a warning: "futures do nothing unless you `.await` or poll them". Treat it as a bug.
- The future from `timer(name, …)` **borrows** `name` (the `+ 'a`). That matters when you spawn it.

### The runtime

```rust
#[tokio::main]                        // builds a multi-thread runtime; main's body runs on the main thread via block_on, spawned tasks on worker threads
async fn main() { … }

#[tokio::test]                        // a fresh single-thread runtime per test
#[tokio::test(start_paused = true)]   // the same, with a virtual clock
```

| Tokio feature | Gives you |
|---|---|
| `rt-multi-thread` | The multi-thread scheduler: one worker thread per CPU core by default |
| `macros` | `#[tokio::main]`, `#[tokio::test]`, `join!`, `select!` |
| `time` | `sleep`, `timeout`, `Instant` |
| `test-util` (dev-dependency) | `start_paused`, `tokio::time::advance` |

Tokio's features are opt-in, and `full` enables almost all of them. This crate lists only what it uses.

With `start_paused = true` the clock is frozen. When every task is waiting, Tokio jumps it to the next timer, so a 300 ms test finishes instantly with exact timings. Only Tokio's clock is virtual: use `tokio::time::Instant`, not `std::time::Instant`.

### Await, join, or spawn

Two timers, `a` = 100 ms and `b` = 200 ms:

| | Code | `a` runs | `b` runs | Total |
|---|---|---|---|---|
| Sequential | `let a = fa.await; let b = fb.await;` | 0–100 | 100–300 | 300 ms |
| Concurrent | `let (a, b) = tokio::join!(fa, fb);` | 0–100 | 0–200 | 200 ms |
| Spawned | `tokio::spawn` both, then await both handles | 0–100 | 0–200 | 200 ms |

- `join!` is concurrency without parallelism: one task polls both futures in turn and switches at each `.await`. That's enough when the work is waiting on I/O or timers.
- `spawn` hands the future to the runtime, which starts it without waiting for an `.await`. On `rt-multi-thread`, spawned tasks can run in parallel on different worker threads.
- `spawn` returns a `JoinHandle<T>`. Awaiting it gives `Result<T, JoinError>`, which is `Err` when the task panicked or was aborted.
- Dropping a `JoinHandle` detaches the task: it keeps running until it finishes or the runtime shuts down (e.g. `main` returns), which drops it at its next `.await`. Lesson 14 covers graceful shutdown. Dropping a future that isn't spawned cancels it (see below).

### `spawn` needs `Send + 'static`

```rust
pub fn spawn<F>(future: F) -> JoinHandle<F::Output>
where
    F: Future + Send + 'static,
    F::Output: Send + 'static,
```

- **`'static`**: the task can outlive the function that spawned it, so it can't borrow that function's locals or parameters. Give it owned data — `to_owned()`, `clone()`, or an `Arc` — and capture it with `async move`.
- **`Send`**: a multi-thread runtime can resume a task on a different worker thread after any `.await`, so every value held across an `.await` must be `Send`. `Rc` isn't, because its reference count isn't atomic; `Arc` is. A `std::sync::MutexGuard` isn't either — lesson 12.
- Both are compile errors. The single-thread runtime in `#[tokio::test]` doesn't relax them, because they're part of `spawn`'s signature.

```rust
let name = name.to_owned();               // an owned String: no borrow of the caller
let handle = tokio::spawn(async move {    // `move`: the block takes ownership of `name`
    timer(&name, 100).await
});
let report = handle.await?;               // JoinError if the task panicked
```

`async move` works like a `move` closure: without `move`, the block borrows what it uses.

PHP's reference counting isn't atomic either, but PHP never shares an object between threads: FPM uses processes, and `ext-parallel` copies values into each thread. Rust lets threads share memory and checks that sharing at compile time.

### Cancellation is dropping

```rust
match tokio::time::timeout(Duration::from_millis(100), timer("slow", 300)).await {
    Ok(report) => …,
    Err(_elapsed) => …,   // the deadline won; the timer future was dropped mid-sleep
}
```

A future only makes progress when it is polled, so dropping it stops it at its current `.await`, and the code after that point never runs. `timeout` and `select!` cancel this way. Amp instead passes a `Cancellation` object that your code must check. In Rust, any future can be cancelled at any `.await` without its cooperation.

### Blocking inside async code

```rust
async fn bad(ms: u64) { std::thread::sleep(Duration::from_millis(ms)); }       // holds the thread
async fn good(ms: u64) { tokio::time::sleep(Duration::from_millis(ms)).await; } // yields
```

A worker thread runs many tasks by switching between them at `.await` points. A blocking call has no `.await`, so the thread and every task queued on it stall until the call returns. `tokio::join!(bad(100), bad(200))` takes 300 ms, not 200 ms; `cargo run` shows it. Common blocking calls: `std::thread::sleep`, `std::fs`, synchronous HTTP or database clients, and long CPU work such as password hashing.

- Under FPM, blocking is harmless because one process serves one request. Swoole has the same trap as Tokio: a blocking call that Swoole doesn't hook stalls every coroutine in the worker.
- `tokio::task::spawn_blocking(closure)` runs the closure on a separate pool of threads (up to 512 by default) and returns a `JoinHandle`. Use it for bounded blocking work. The closure must be `FnOnce() -> T + Send + 'static`, for the same reasons as `spawn`.
- Don't use it for work that never ends (use `std::thread::spawn`) or for splitting CPU work across cores (Rayon, lesson 17).
- A paused test clock hides this bug: `std::thread::sleep` takes real time but doesn't move Tokio's virtual clock.

## Exercise

Implement in this order:

1. `timer`: `Instant::now()`, then `tokio::time::sleep(Duration::from_millis(ms)).await`, then `Instant::now()` again. Add `use std::time::Duration;`.
2. `run_sequential`: two `.await`s.
3. `run_concurrent`: `tokio::join!` returns a tuple in argument order.
4. `create_then_wait`: bind the timer future to a variable, sleep, then await it. Check `started_at` against the test.
5. `run_spawned`: first try `tokio::spawn(timer(first.0, first.1))` and read the error (`borrowed data escapes outside of function`). Then give each task an owned name and use `async move`. A small `fn spawn_timer(name: String, ms: u64) -> JoinHandle<TimerReport>` helper keeps steps 5–7 short. `?` returns a `JoinError` as `Err`.
6. `spawn_then_wait`: spawn, sleep, then await the handle.
7. `run_many`: spawn everything into a `Vec<JoinHandle<TimerReport>>` first, then await the handles in order. `timers.into_iter()` (or `for (name, ms) in timers`) consumes the `Vec`, so each `String` moves into its task without a clone.
8. `with_timeout`: `tokio::time::timeout(…).await` returns `Result<TimerReport, Elapsed>`; `.ok()` turns it into an `Option`.
9. `run_blocking`: a single expression around `tokio::task::spawn_blocking`.
10. `cargo run`. For each timeline, explain why the numbers are what they are. Real time adds a few milliseconds of jitter.
11. `cargo test --features broken`. Fix the three functions in `src/broken.rs` without changing their signatures or results. Note which bound — `Send` or `'static` — each error comes from.

Optional extension: rewrite `run_many` with `tokio::task::JoinSet`. `join_next()` returns results in completion order, so spawn `(index, report)` pairs and put the reports back in input order.

## Done when

- `cargo test`, `cargo test --features broken`, `cargo fmt --check`, and `cargo clippy --all-targets --all-features -- -D warnings` pass.
- You can explain:
  - why `create_then_wait` reports `started_at` at +50 ms, but `spawn_then_wait` at +0 ms
  - why `join!` is concurrent but not parallel, and when spawned tasks run in parallel
  - why `tokio::spawn` requires `Send` and `'static`, and how each fix in `src/broken.rs` satisfies them
  - what happens to the timer inside `with_timeout` when the deadline wins
  - why `std::thread::sleep` in async code is a bug, and what `spawn_blocking` does instead
