# Lesson 16 — Threads and ownership

Count lines and bytes in many files on several threads: implement every `todo!()` in `src/lib.rs`, then fix `src/broken.rs`.

## Run

Inside the container:

```bash
cd /workspace/lessons/16-threads
cargo test
cargo test --features broken
cargo fmt --check
cargo clippy --all-targets --features broken -- -D warnings
cargo run -- Cargo.toml README.md src missing.txt
```

Don't run `cargo fix`. `src/broken.rs` only compiles with `--features broken`, and fails on purpose until Part F.

If `cargo test` hangs, a channel `Sender` is still alive somewhere (see "Channels").

## Notes

### Threads share memory

PHP runs parallel work in separate processes (PHP-FPM, `pcntl_fork`) that share nothing. Rust threads run inside one process and share its memory, so the compiler checks every value that crosses to another thread.

### Spawn and join

`thread::spawn` starts a thread. `join` waits for it and returns its result:

```rust
use std::thread;

let name = String::from("a.txt");
let handle = thread::spawn(move || name.len()); // `name` moves into the thread
let len = handle.join().unwrap();
println!("{len}"); // 5
```

- `spawn` can't borrow from the caller (`'static`): the thread may outlive the function. `move` gives it owned data instead.
- `join()` returns `Err` if the thread panicked. `std::panic::resume_unwind(payload)` re-raises that panic.
- Spawn all threads first, then join them. Spawning and joining one by one runs one thread at a time.

### Scoped threads

`thread::scope` joins all its threads before it returns, so they may borrow local data:

```rust
let data = vec![1, 2, 3, 4];
let (left, right) = data.split_at(2);
let total = thread::scope(|s| {
    let a = s.spawn(|| left.iter().sum::<i32>());
    let b = s.spawn(|| right.iter().sum::<i32>());
    a.join().unwrap() + b.join().unwrap()
});
println!("{total}"); // 10
```

Many threads can share a `&T`, but a `&mut` goes to one thread only. `chunks_mut` splits a slice into parts that can't overlap:

```rust
let mut nums = [1, 2, 3, 4, 5, 6];
thread::scope(|s| {
    for chunk in nums.chunks_mut(2) {
        s.spawn(move || chunk.iter_mut().for_each(|n| *n *= 10));
    }
});
println!("{nums:?}"); // [10, 20, 30, 40, 50, 60]
```

Write `s.spawn(move || …)` in loops. Without `move`, a loop variable like `i` is borrowed and dies too early (E0373).

### Send and Sync

- `Send`: a value can be **moved** to another thread.
- `Sync`: a value can be **shared** (`&T`) by several threads.
- A type gets them automatically when all its fields have them. You meet them in error messages.

```rust
let rc = Rc::new(5);
thread::spawn(move || println!("{rc}"));
// error[E0277]: `Rc<i32>` cannot be sent between threads safely
```

- `Rc<T>` isn't `Send`: use `Arc<T>`.
- `RefCell<T>` isn't `Sync`: use `Mutex<T>`.
- `i64`, `String`, `Vec<T>`, `Arc<T>`, `Mutex<T>` are thread-safe when what's inside is.

That's why a data race is a compile error. A deadlock is not.

### Arc and Mutex

`Arc` shares one value between threads. `Mutex` lets one thread at a time change it:

```rust
use std::sync::{Arc, Mutex};

let count = Arc::new(Mutex::new(0));
let mut handles = Vec::new();
for _ in 0..4 {
    let count = Arc::clone(&count);
    handles.push(thread::spawn(move || *count.lock().unwrap() += 1));
}
for handle in handles {
    handle.join().unwrap();
}
println!("{}", count.lock().unwrap()); // 4
```

- `lock()` returns a guard. Dropping the guard unlocks.
- The data lives inside the `Mutex`, so you can't touch it without the lock. Keep guards short.

### Poisoning

If a thread panics while holding the lock, the mutex is **poisoned**: later `lock()` calls return `Err`.

```rust
let guard = mutex.lock().unwrap();                                // panics if poisoned
let guard = mutex.lock().unwrap_or_else(PoisonError::into_inner); // takes the guard anyway
```

Recover only when every update leaves the data valid, like one `Summary::add`.

### Channels

A channel moves values from one thread to another:

```rust
use std::sync::mpsc;

let (tx, rx) = mpsc::channel();
thread::spawn(move || {
    for n in 1..=3 {
        tx.send(n).unwrap();
    }
}); // `tx` is dropped when the thread ends
for n in rx {
    println!("{n}"); // 1, then 2, then 3
}
```

- `send` moves the value. The sender can't touch it afterwards, so it needs no lock.
- `Sender` can be cloned (many producers). `Receiver` can't (one consumer): workers share it as `Arc<Mutex<Receiver<T>>>`.
- `for n in rx` ends once every `Sender` is dropped. A forgotten `Sender` makes it wait forever.

### Backpressure

`sync_channel(n)` is a bounded channel: `send` blocks while `n` items are waiting.

```rust
let (tx, rx) = mpsc::sync_channel::<PathBuf>(8);
```

- A fast producer waits for slow workers, so memory holds at most 8 jobs, not the whole input.
- `sync_channel(0)`: each `send` waits for a `recv`.
- Like a RabbitMQ prefetch limit.

### The worker pool

```text
caller ──sync_channel(queue)──▶ worker 1..N ──channel()──▶ caller
          paths, bounded         job(&path)    FileReport
```

1. Create both channels. Wrap the job `Receiver` in `Arc<Mutex<…>>`.
2. Spawn N workers, each with an `Arc` clone of the job receiver and a clone of the result `Sender`.
3. Drop the caller's result `Sender` (or collecting never ends) and its `Arc` of the job receiver.
4. Send each path, then drop the job `Sender`. Workers finish the queue and exit.
5. Collect results until the channel closes, join the workers, sort by path.

Receive in its own statement, so the lock is released before the job runs:

```rust
let next = rx.lock().unwrap().recv(); // lock released at the `;`
let Ok(path) = next else { break };
```

`while let Ok(path) = rx.lock().unwrap().recv() { … }` keeps the lock for the whole loop body, so only one job runs at a time.

### Channel or shared state?

- Channel (`run_pool` + `summarize`): workers move each report to one owner. No lock on results, and per-file detail is kept.
- Shared (`summarize_shared`): every thread updates one `Arc<Mutex<Summary>>`. One lock per file, and only totals survive.

Prefer moving results to one owner. Share state only when threads must see each other's updates during the run (a cache, a rate limiter).

## Exercise

Part A — the job:

1. `count`: count `b'\n'` bytes, then add one line if `data` is non-empty and doesn't end in `\n` (`data.last()`).
2. `count_file`: `fs::read(path)`, then `.map(…)` and `.map_err(|e| e.kind())`.

Part B — spawn and join:

3. `spawn_count`: `thread::spawn(move || …)`; build the report inside the thread.
4. `count_files_spawned`: collect all handles into a `Vec` first, then join them in order.

Part C — scoped threads:

5. `count_in_parts`: chunk length `data.len().div_ceil(parts)`, then `data.chunks(len)`. Count only `\n` per chunk and add the final unterminated line once (summing `count(chunk)` breaks; find the input).
6. `uppercase_in_parts`: `data.chunks_mut(len)`, one scoped thread per chunk, `make_ascii_uppercase`.

Part D — the worker pool:

7. `lock_recovering`: one line, see "Poisoning".
8. `run_pool`: follow "The worker pool". `assert!` the worker count first. A `fn` pointer is `Copy + Send + 'static`, so each `move` closure can take `job`. Re-raise a worker panic with `resume_unwind`, or `expect` on `join()`.

Part E — totals:

9. `Summary::add`: one `match`.
10. `summarize`: a loop over `add`.
11. `summarize_shared`: group length `paths.len().div_ceil(workers).max(1)`; `chunks(len)` plus `to_vec()` gives each thread an owned group. Update through `lock_recovering`, then return `*lock_recovering(&total)`.

Part F — fix `src/broken.rs`:

12. Run `cargo test --features broken`. Fix the three functions in order, reading each full error first. Tools: `Arc`, `Mutex`, `thread::scope`.

Optional: rewrite `run_pool` with `thread::scope`: the receiver needs no `Arc`, and `job` no longer needs `'static`.

## Done when

- `cargo test --features broken`, `cargo fmt --check`, and `cargo clippy --all-targets --features broken -- -D warnings` pass.
- You can explain:
  - why `thread::spawn` needs `move` and `'static`, and why `thread::scope` doesn't need `'static`
  - what `Send` and `Sync` mean, and why `Rc` and `RefCell` failed in `broken.rs`
  - what makes the pool's workers exit, and why a forgotten `Sender` makes the test hang
