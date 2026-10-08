# Lesson 16 — Threads and ownership

Count lines and bytes in many files on several threads: implement every `todo!()` in `src/lib.rs`, then fix `src/broken.rs`, until the tests pass.

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

Until a function is implemented, its parameters show `unused variable` warnings. Don't run `cargo fix`.

`--features broken` compiles `src/broken.rs`, which fails to build on purpose until Part F. If `cargo test` hangs, a channel `Sender` is still alive somewhere (see "The worker pool").

## Notes

### PHP shares nothing; Rust threads share everything

| PHP | Parallel work | Shared memory |
|---|---|---|
| PHP-FPM | A pool of processes, each handling one request at a time | None by default; shared state goes through the DB, Redis, or APCu (a shared-memory cache with its own locking) |
| `pcntl_fork()` | A child process with a copy-on-write copy of the parent | None; results come back through pipes, sockets, files, or the exit code (`pcntl_waitpid`) |
| ext-parallel | `parallel\Runtime` threads, each with its own interpreter (ZTS build) | None; arguments and results are copied, `parallel\Channel` passes messages, `Future::value()` waits |

Rust threads live in one process and can read and write the same memory. Nothing is copied unless you copy it. That is faster and riskier, so the compiler checks every value that crosses a thread boundary. The comparison breaks down here: PHP gets safety by isolating everything at runtime; Rust allows sharing and the type system decides which sharing is safe.

### Spawn and join

```rust
let path = PathBuf::from("a.txt");
let handle: JoinHandle<usize> = thread::spawn(move || expensive(&path)); // `path` moves in
let value = handle.join().unwrap(); // waits; Err only if the thread panicked
```

- `spawn` requires `F: FnOnce() -> T + Send + 'static` and `T: Send + 'static`: both the closure and the value it returns cross threads. `'static` means the closure borrows nothing from the caller's stack, because the thread may outlive the function that started it. `move` hands it owned data instead.
- `join()` returns `Result<T, Box<dyn Any + Send>>`. A panic ends only that thread; `Err` carries the panic payload, and `std::panic::resume_unwind(payload)` re-raises it in the caller.
- Dropping a `JoinHandle` detaches the thread. When `main` returns, the process exits and detached threads die mid-work.
- Like `pcntl_waitpid`, but you get back any `T`, not an exit code.

### Scoped threads

```rust
let words = vec!["a", "bb", "ccc"];
let (left, right) = words.split_at(1);
let total = thread::scope(|s| {
    let a = s.spawn(|| left.len());   // borrows `words`: no Arc, no clone, no 'static
    let b = s.spawn(|| right.len());
    a.join().unwrap() + b.join().unwrap()
}); // every thread spawned on `s` has finished here
```

- `scope` joins all its threads before returning, so they may borrow anything that outlives the scope.
- Many threads may share a `&T` (requires `T: Sync`). A `&mut` goes to exactly one thread: split data with `chunks_mut` or `split_at_mut` to give each thread its own part.
- If a loop variable is an owned value, such as an index `i`, write `s.spawn(move || …)`. Without `move` the closure borrows `i`, which dies at the end of the iteration (E0373). Loop variables that are themselves references (from `chunks`, `chunks_mut`) work either way, because the closure reborrows the data they point to. `move` is still the habit to build.
- If a scoped thread panicked and you didn't join it, `scope` panics after joining the rest.

### `Send` and `Sync` in plain words

- `Send`: a value may be **moved** to another thread.
- `Sync`: a value may be **shared** (`&T`) by several threads at once. Formally, `T: Sync` exactly when `&T: Send`.
- Both are auto traits: a struct is `Send`/`Sync` when all its fields are. You rarely write them; you meet them in error messages.

| Type | `Send` | `Sync` | Why |
|---|---|---|---|
| `i64`, `String`, `PathBuf`, `Vec<T>` | yes | yes | plain owned data (`Vec<T>`: if `T` is) |
| `Rc<T>` | no | no | non-atomic refcount: two threads cloning at once corrupt it |
| `Arc<T>` | if `T: Send + Sync` | if `T: Send + Sync` | atomic refcount; hands out `&T` only |
| `RefCell<T>`, `Cell<T>` | if `T: Send` | no | the runtime borrow flag isn't atomic |
| `Mutex<T>` | if `T: Send` | if `T: Send` | the lock makes every access exclusive |
| `MutexGuard<'_, T>` | no | if `T: Sync` | must unlock on the thread that locked |
| `mpsc::Receiver<T>` | if `T: Send` | no | one consumer at a time |

### Why data races don't compile

A data race: two threads touch the same memory at the same time, at least one writes, and nothing orders them. Inside one thread Rust already enforces "many `&T` or one `&mut T`". `Send` and `Sync` extend that rule across threads:

- `thread::spawn` demands `Send + 'static`, so a thread can't keep a reference into another thread's stack, or an `Rc` whose count another thread updates.
- Mutating shared data needs either `&mut` (exclusive by construction) or a `Sync` type that synchronizes internally (`Mutex`, `RwLock`, atomics).

The compiler does **not** catch deadlocks, logic races (check-then-act across two separate lock calls), or nondeterministic ordering. That is why `run_pool` sorts its output.

### `Arc` and `Mutex`

```rust
let total = Arc::new(Mutex::new(Summary::default()));
let for_thread = Arc::clone(&total);              // refcount +1, same Mutex
thread::spawn(move || {
    let mut guard = for_thread.lock().unwrap();   // blocks until the lock is free
    guard.files += 1;
});                                               // guard dropped: unlocked
```

- `Arc` ≈ a PHP object handle with a refcount, except the count is atomic and it only hands out `&T`. Shared **and** mutable needs a `Mutex` inside.
- `Mutex<T>` owns the data. The only way in is `lock()`, which returns a guard; dropping the guard unlocks. Unlike `flock()` or a semaphore next to the data, you can't touch the data without the lock.
- Keep guards short: lock, update, drop. A guard held across slow work makes every other thread wait.

### Poisoning

If a thread panics while holding a guard, the mutex is **poisoned**: every later `lock()` returns `Err(PoisonError)`, because the data may be half-updated.

- `lock().unwrap()` propagates the panic. Right when a half-done update could break invariants (two fields that must agree).
- `lock().unwrap_or_else(PoisonError::into_inner)` takes the guard anyway. Fine when every update leaves the data valid, like `Summary::add` on one struct.
- PHP has no equivalent: a fatal error ends the request, and its memory with it.

### Channels and backpressure

```rust
let (tx, rx) = mpsc::sync_channel::<PathBuf>(8); // bounded: send blocks while 8 are queued
let (tx, rx) = mpsc::channel::<FileReport>();    // unbounded: send never blocks
tx.send(path)?;                                  // `path` moves into the channel
for report in rx {}                              // ends once every Sender is dropped and the queue is empty
```

- `mpsc`: many producers (`Sender` is `Clone`), one consumer (`Receiver` isn't). N workers share one receiver as `Arc<Mutex<Receiver<T>>>`.
- Sending moves the value: the sender can't touch it any more, so the data itself needs no lock.
- **Backpressure**: with `sync_channel(n)`, a fast producer blocks once `n` jobs are waiting, so memory holds `n` queued jobs instead of the whole input. `sync_channel(0)` is a rendezvous: each `send` waits for a `recv`. Compare a RabbitMQ prefetch limit, or a `parallel\Channel` with a capacity.
- **Shutdown is ownership**: `recv()` returns `Err` once all `Sender`s are dropped and the queue is empty. No "stop" message needed.

### The worker pool

```text
caller ──sync_channel(queue)──▶ worker 1..N ──channel()──▶ caller
         paths, bounded          job(&path)    FileReport
```

The order matters:

1. Create both channels; wrap the job `Receiver` in `Arc<Mutex<…>>`.
2. Spawn N workers. Each gets an `Arc` clone of the job receiver and a clone of the result `Sender`.
3. Drop the caller's result `Sender` (or collecting never ends) and its `Arc` of the receiver (if every worker dies, `send` then fails instead of blocking forever).
4. Send each path as you pull it, then drop the job `Sender`. Workers drain the queue and exit.
5. Collect results until the channel closes, join the workers, sort by path.

Gotchas:

- `while let Ok(path) = rx.lock().unwrap().recv() { … }` keeps the guard alive for the **whole loop body** (temporaries in a `while let` or `match` scrutinee live until the body ends), so workers run one job at a time. Receive in its own statement:

  ```rust
  let next = rx.lock().unwrap().recv(); // guard dropped at the `;`
  let Ok(path) = next else { break };
  ```

- If the result channel were bounded too, the pool could deadlock: workers block sending results nobody reads yet, while the caller blocks sending jobs nobody takes.
- Completion order is random. Sort, or keep input order by joining handles in order (as `count_files_spawned` does).

### Deadlocks are still possible

Rust prevents data races, not deadlocks:

- Locking a `Mutex` the current thread already holds never returns (it may deadlock or panic; std leaves it unspecified).
- Two threads taking two locks in opposite orders. Always lock in one fixed order.
- Waiting on a channel whose `Sender` is still alive.

### Results without a global accumulator

| | Channel: `run_pool` + `summarize` | Shared: `summarize_shared` |
|---|---|---|
| Who owns the results | the collector; workers move each report out | every thread, through `Arc<Mutex<Summary>>` |
| Locking | none on results | one lock per file, contended |
| Per-file detail | kept, sortable | lost; only totals |
| A panic mid-update | nothing shared to corrupt | poisons the total |

Prefer moving results to one owner, like PHP workers returning results to a coordinator instead of all updating one row with `SELECT … FOR UPDATE`. Use shared state when threads must read or update one structure during the run (a cache, a rate limiter). For a single counter, atomics (`AtomicUsize`) are lighter; see the roadmap's advanced topics.

## Exercise

Part A — the job:

1. `count`: count `b'\n'` bytes, then add one line if `data` is non-empty and doesn't end in `\n` (`data.last()`).
2. `count_file`: `fs::read(path)`, then `.map(…)` and `.map_err(|e| e.kind())`. `Outcome` stores `io::ErrorKind` because `io::Error` is neither `Clone` nor `PartialEq`.

Part B — spawn and join:

3. `spawn_count`: `thread::spawn(move || …)`. Build the report inside the thread so the path travels in and back out.
4. `count_files_spawned`: collect all handles into a `Vec` first, then join them in order. A single lazy `.map(spawn_count).map(|h| h.join())` chain spawns, joins, spawns, joins: one file at a time.

Part C — scoped threads:

5. `count_in_parts`: chunk length `data.len().div_ceil(parts)`; `data.chunks(len)` then yields at most `parts` chunks (and panics for `len == 0`, hence the empty check). Count only `\n` per chunk and add the final unterminated line once. Try summing `count(chunk)` first and find the input that breaks it.
6. `uppercase_in_parts`: `data.chunks_mut(len)`, one scoped thread per chunk, `make_ascii_uppercase`. No lock: the chunks can't overlap, and the borrow checker knows it.

Part D — the worker pool:

7. `lock_recovering`: one line, see "Poisoning".
8. `run_pool`: follow "The worker pool". `assert!` the worker count first. A `fn` pointer is `Copy + Send + 'static`, so each worker's `move` closure can take `job`. Re-raise a worker panic with `resume_unwind`, or `expect` on `join()`.

Part E — totals:

9. `Summary::add`: one `match`.
10. `summarize`: a loop over `add`.
11. `summarize_shared`: group length `paths.len().div_ceil(workers).max(1)`; `chunks(len)` plus `to_vec()` gives each thread an owned group. Update through `lock_recovering`. After joining, `*lock_recovering(&total)` copies the result out (`Summary` is `Copy`).

Part F — fix `src/broken.rs`:

12. Run `cargo test --features broken`. Fix the three functions in order, reading each full error first: the compiler names the missing `Send`, `Sync`, or `'static`. Tools: `Arc`, `Mutex`, `thread::scope`.

Optional extensions:

- Make `run_pool` generic over `F: Fn(&Path) -> Outcome + Send + Sync + 'static`, share it through `Arc<F>`, and pass a closure that captures a setting. Remove each bound once and read the error.
- Rewrite `run_pool` with `thread::scope`: the receiver needs no `Arc`, and `job` no longer needs `'static`.

## Done when

- `cargo test --features broken`, `cargo fmt --check`, and `cargo clippy --all-targets --features broken -- -D warnings` pass.
- You can explain:
  - why `thread::spawn` needs `move` and `'static`, and why `thread::scope` doesn't need `'static`
  - what `Send` and `Sync` mean, and why `Rc` and `RefCell` failed in `broken.rs`
  - what makes the pool's workers exit, and why a forgotten `Sender` makes the test hang
  - where `run_pool` blocks when workers are slower than the producer, and what that protects
  - why `count_in_parts` can't just add up `count` of each chunk
  - what `summarize_shared` gives up compared with collecting results through a channel
  - when recovering a poisoned mutex is acceptable
