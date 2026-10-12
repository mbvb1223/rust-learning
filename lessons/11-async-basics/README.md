# Lesson 11 — Enough async to start

Run timers one after another, concurrently, and as spawned tasks in `src/lib.rs`, then fix the spawn errors in `src/broken.rs`.

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

`src/broken.rs` only compiles with `--features broken` (or `--all-features`), so plain `cargo test` works while it's still broken. Don't run `cargo fix`.

## Notes

### Async needs a runtime

`async` and `.await` are part of the language, but the runtime that runs async code is a crate. This course uses Tokio; Axum and SQLx run on it.

```rust
#[tokio::main]
async fn main() {
    tokio::time::sleep(Duration::from_millis(100)).await;
    println!("done"); // done (after 100 ms)
}
```

`#[tokio::main]` starts the runtime and makes `main` async. `.await` only works inside `async` code. Tokio plays the role of Amp's or ReactPHP's event loop, but uses several threads.

### Futures are lazy

Calling an `async fn` doesn't run it. It returns a **future**, and the body starts only when you `.await` it.

```rust
let future = timer("a", 100);           // nothing runs yet
sleep(Duration::from_millis(50)).await;
let report = future.await;              // the timer starts now, at +50 ms
```

An Amp or React promise has already started; a Rust future hasn't. A future you never await is a bug, and the compiler warns about it.

### Sequential: await one by one

Each `.await` waits for its future to finish before the next line runs.

```rust
let a = timer("a", 100).await; // 0–100 ms
let b = timer("b", 200).await; // 100–300 ms
// total: 300 ms
```

### Concurrent: `tokio::join!`

`tokio::join!` awaits several futures at once and returns their results as a tuple, in argument order.

```rust
let (a, b) = tokio::join!(timer("a", 100), timer("b", 200));
// a: 0–100 ms, b: 0–200 ms, total: 200 ms
```

One task polls both futures and switches between them at each `.await`. That's concurrency without parallelism, which is enough for waiting on timers and I/O.

### Spawned: `tokio::spawn`

`tokio::spawn` hands a future to the runtime, which starts it at once, without waiting for an `.await`. It returns a `JoinHandle`.

```rust
let handle = tokio::spawn(async { 1 + 1 });
let n = handle.await.unwrap();
println!("{n}"); // 2
```

- Spawned tasks can run in parallel on different threads.
- Awaiting a `JoinHandle<T>` gives `Result<T, JoinError>`. It's `Err` if the task panicked.
- Dropping a `JoinHandle` doesn't stop the task; it keeps running in the background.

### Spawned tasks must own their data

A spawned task can outlive the function that spawned it, so it can't borrow that function's variables: it must be `'static`. Give it owned data and capture it with `async move`.

```rust
let name = name.to_owned();            // an owned String, no borrow
let handle = tokio::spawn(async move { // `move`: the block takes ownership of `name`
    timer(&name, 100).await
});
let report = handle.await?;
```

`timer(name, 100)` returns a future that borrows `name`, so `tokio::spawn(timer(name, 100))` fails with `borrowed data escapes outside of function`. Without `move`, an `async` block borrows what it uses, like a closure.

### Spawned tasks must be `Send`

The runtime may move a task to another thread at any `.await`, so everything the task holds must be `Send`.

```rust
let count = Rc::new(5);
tokio::spawn(async move {
    tokio::time::sleep(Duration::from_millis(10)).await;
    println!("{count}");
});
// error: future cannot be sent between threads safely
```

- `Rc` isn't `Send` (its counter isn't atomic); `Arc` is.
- A `std::sync::MutexGuard` isn't `Send` either (lesson 12).
- `Send` and `'static` are checked at compile time, even in `#[tokio::test]`, which runs on one thread.

### Testing with a paused clock

`#[tokio::test]` gives each test its own runtime. With `start_paused = true`, Tokio jumps the clock to the next timer whenever all tasks are waiting, so timings are exact and the test finishes instantly.

```rust
#[tokio::test(start_paused = true)]
async fn sleeps_100_ms() {
    let t0 = tokio::time::Instant::now();
    tokio::time::sleep(Duration::from_millis(100)).await;
    assert_eq!(t0.elapsed(), Duration::from_millis(100)); // passes, in no real time
}
```

Only Tokio's clock is paused: use `tokio::time::Instant`, not `std::time::Instant`.

### Timeouts cancel by dropping

`tokio::time::timeout` races a future against a deadline. If the deadline wins, the future is dropped, which stops it at its current `.await`.

```rust
let result = timeout(Duration::from_millis(100), timer("slow", 300)).await;
println!("{}", result.is_err()); // true: the timer was dropped mid-sleep
```

`timeout` returns `Result<T, Elapsed>`.

### Don't block in async code

A thread runs many tasks by switching between them at `.await` points. A blocking call has no `.await`, so every task on that thread waits for it.

```rust
async fn bad(ms: u64) { std::thread::sleep(Duration::from_millis(ms)); }       // blocks the thread
async fn good(ms: u64) { tokio::time::sleep(Duration::from_millis(ms)).await; } // lets others run

tokio::join!(bad(100), bad(200));   // 300 ms
tokio::join!(good(100), good(200)); // 200 ms
```

Blocking calls include `std::thread::sleep`, `std::fs`, synchronous database clients, and heavy CPU work like password hashing. Under PHP-FPM blocking is harmless, because each process serves one request.

### `spawn_blocking`

`tokio::task::spawn_blocking` runs a blocking closure on a separate thread pool and returns a `JoinHandle`.

```rust
let sum = tokio::task::spawn_blocking(|| (1..=1_000_000u64).sum::<u64>())
    .await
    .unwrap();
println!("{sum}"); // 500000500000
```

The closure must be `Send + 'static`, for the same reasons as `spawn`.

## Exercise

Implement in this order:

1. `timer`: `Instant::now()`, then `tokio::time::sleep(Duration::from_millis(ms)).await`, then `Instant::now()` again. Add `use std::time::Duration;`.
2. `run_sequential`: two `.await`s.
3. `run_concurrent`: `tokio::join!` returns a tuple in argument order.
4. `create_then_wait`: bind the timer future to a variable, sleep, then await it.
5. `run_spawned`: try `tokio::spawn(timer(first.0, first.1))` and read the error, then use owned names and `async move`. A `fn spawn_timer(name: String, ms: u64) -> JoinHandle<TimerReport>` helper keeps steps 5–7 short.
6. `spawn_then_wait`: spawn, sleep, then await the handle.
7. `run_many`: spawn everything into a `Vec<JoinHandle<TimerReport>>` first, then await the handles in order; `for (name, ms) in timers` moves each `String` without a clone.
8. `with_timeout`: `tokio::time::timeout(…).await` returns a `Result`; `.ok()` turns it into an `Option`.
9. `run_blocking`: one expression around `tokio::task::spawn_blocking`.
10. `cargo run`, and explain why each timeline's numbers are what they are (real time adds a few ms of jitter).
11. `cargo test --features broken`: fix the three functions in `src/broken.rs` without changing their signatures or results, and note whether each error comes from `Send` or `'static`.

Optional: rewrite `run_many` with `tokio::task::JoinSet`; `join_next()` returns results in completion order, so spawn `(index, report)` pairs and sort them back.

## Done when

- `cargo test`, `cargo test --features broken`, `cargo fmt --check`, and `cargo clippy --all-targets --all-features -- -D warnings` pass.
- You can explain:
  - why `create_then_wait` reports `started_at` at +50 ms, but `spawn_then_wait` at +0 ms
  - why `tokio::spawn` requires `Send` and `'static`, and how each fix in `src/broken.rs` satisfies them
  - why `std::thread::sleep` in async code is a bug, and what `spawn_blocking` does instead
