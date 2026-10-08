# Lesson 17 — CPU parallelism and measurement

Hash files with SHA-256 sequentially and in parallel, prove the results are identical, and measure the difference: implement every `todo!()` in `src/lib.rs` until the tests pass, then run the benchmark in `src/main.rs`.

## Run

From the repository root:

```bash
docker compose run --rm -w /workspace/lessons/17-parallelism rust cargo test
docker compose run --rm -w /workspace/lessons/17-parallelism rust cargo fmt --check
docker compose run --rm -w /workspace/lessons/17-parallelism rust cargo clippy --all-targets -- -D warnings
docker compose run --rm -w /workspace/lessons/17-parallelism rust cargo run --release
```

Until a function is implemented, its parameters show `unused variable` warnings. Don't run `cargo fix`; it renames them to `_name`.

Plain `cargo test` stops at the first failing test binary, so the file tests in `tests/files.rs` run only after every unit test in `src/lib.rs` passes. To check steps 4, 5, 8 and 9 earlier, run `cargo test --test files` (or `cargo test --no-fail-fast`).

## Notes

### New capability: many cores, one process

PHP-FPM runs each request on one thread of one worker process. To use more cores you start more processes — FPM workers, queue consumers, `pcntl_fork`, `xargs -P` — each with its own memory, passing results through Redis, a database, or pipes.

Rayon runs work on several threads **inside one process**, all reading the same memory. The compiler checks that this is safe: closures handed to Rayon must be `Send + Sync` (lesson 16) and can't take `&mut` to what they capture (shared mutation needs a `Mutex` or atomics). A data race is a compile error, not an intermittent production bug.

| PHP | This lesson |
|---|---|
| `hash('sha256', $s, true)` | `hash_bytes(s)` → `[u8; 32]` |
| `hash_file('sha256', $path)` | `to_hex(&hash_file(path, &mut buf)?)` |
| `hash_init` / `hash_update` / `hash_final` | `Sha256::new()` / `update` / `finalize` |
| N worker processes reading a queue | one Rayon pool of N threads |

### Why `sha2`, not `blake3`

- Same algorithm as PHP's `hash('sha256', …)` and `sha256sum`, so any result can be cross-checked.
- One SHA-256 stream is strictly sequential: each 64-byte block needs the state left by the previous one. All parallelism in this lesson is yours — across files or independent chunks — so the numbers show what Rayon does.
- `blake3` is faster and built as a tree, so a single large input can be split across cores; its `rayon` feature (`Hasher::update_rayon`) does that. Great in production, but it would mix a second source of parallelism into the measurement.
- Pure Rust. `sha2` detects SHA CPU instructions (x86 SHA-NI, AArch64 `sha2`) at runtime and falls back to portable code otherwise.

```rust
use sha2::{Digest, Sha256}; // `Digest` is a trait: its methods only exist where it's in scope

let mut hasher = Sha256::new();
hasher.update(b"hello ");
hasher.update(b"world");
let checksum: [u8; 32] = hasher.finalize().into();

let same: [u8; 32] = Sha256::digest(b"hello world").into();
```

`finalize` returns a fixed-size array type from the `hybrid-array` crate; `.into()` converts it to a plain `[u8; 32]`. That's `Copy` and lives on the stack: hashing allocates nothing. Only `to_hex` allocates, once, for display.

### Streaming with a reusable buffer

```rust
let mut buf = vec![0u8; BUF_SIZE];          // one heap allocation
for path in paths {
    let checksum = hash_file(path, &mut buf); // lend the buffer; it's back after the call
}
```

- `read(buf)` returns how many bytes it wrote, `n`. Only `&buf[..n]` is new data; the rest is left over from earlier reads. `Ok(0)` means end of stream.
- A read can fail with `ErrorKind::Interrupted` (a signal arrived). The correct response is to retry; `read_to_end` and `io::copy` do that internally.
- `fs::read(path)` is simpler — it's `file_get_contents` — but it holds the whole file in memory. Streaming needs 64 KiB for a file of any size.
- `BufReader` pays off for many small reads (line by line). Reads at least as large as its 8 KiB buffer bypass it, so here it would only add an allocation per file.
- `impl Read` accepts a `File`, a `&[u8]`, a socket, or a test double — the unit tests use readers that return 3 bytes at a time or fail on purpose.

### Rayon in one page

```rust
use rayon::prelude::*;

let sizes: Vec<u64> = paths.iter().map(file_size).collect();     // sequential
let sizes: Vec<u64> = paths.par_iter().map(file_size).collect(); // parallel, same order
```

| Sequential | Parallel |
|---|---|
| `slice.iter()` | `slice.par_iter()` |
| `vec.into_iter()` | `vec.into_par_iter()` |
| `slice.chunks(n)` | `slice.par_chunks(n)` |
| `.map(f).collect::<Vec<_>>()` | same — and the result keeps input order |
| `.sum()`, `.fold(init, f)` | `.sum()`, `.reduce(…)` — see below |

- Rayon splits the input into jobs; idle threads steal jobs from busy ones, so a mix of large and small files balances itself.
- `collect()` puts each result at its input position, whichever thread finished first. Pushing into a shared `Mutex<Vec<_>>` would record completion order instead.
- `reduce(|| 0, |a, b| a + b)` takes the identity as a closure, not a value: every job starts its own accumulator. `.reduce(0, …)` doesn't compile.
- Rayon has its own `fold(|| init, f)`, but it returns a parallel iterator of partial results, one per job, not a single value. Finish with `.sum()` or `.reduce(…)`.
- Closures run on several threads at once, so Rayon requires `Fn`, not `FnMut`:

```rust
let mut total = 0;
sizes.par_iter().for_each(|s| total += s);
// error[E0596]: cannot borrow `total` as mutable, as it is a captured variable in a `Fn` closure
let total: u64 = sizes.par_iter().sum(); // combine results instead of sharing a counter
```

### Controlling the thread count

```rust
let pool = ThreadPoolBuilder::new().num_threads(4).build()?; // Result: spawning threads can fail
let total: u64 = pool.install(|| sizes.par_iter().sum());
```

- Without `install`, `par_iter` uses Rayon's global pool, sized by `RAYON_NUM_THREADS` or else `available_parallelism()`.
- `install` runs the closure on this pool; every Rayon call made inside it uses this pool's threads. That's how `par_chunk_checksums` runs on whichever pool calls it.
- `num_threads(0)` means "Rayon's default", the same rule as the global pool.
- Building a pool spawns OS threads. In an application, build it once and reuse it. `hash_files_parallel` builds one per call so the thread count can be an argument; that costs microseconds, small next to hashing megabytes.
- `std::thread::available_parallelism()` returns `io::Result<NonZeroUsize>`: how many threads can actually run at once. On Linux it respects CPU affinity and cgroup CPU quotas, so in a container it reports the container's limit; under Docker Desktop it reports the VM's CPUs, not the Mac's.

### Allocation costs: `map_init` and `par_chunks`

The obvious parallel version allocates scratch space for every item. Here, a checksum of each line with its whitespace removed:

```rust
let checksums: Vec<Checksum> = lines
    .par_iter()
    .map(|line| {
        let squeezed: String = line.split_whitespace().collect(); // a new String per line
        hash_bytes(squeezed.as_bytes())
    })
    .collect();
```

`map_init` creates a value per Rayon job and lends it, as `&mut`, to every item that job processes:

```rust
let checksums: Vec<Checksum> = lines
    .par_iter()
    .map_init(String::new, |scratch, line| {
        scratch.clear(); // keeps the capacity from earlier lines
        scratch.extend(line.split_whitespace());
        hash_bytes(scratch.as_bytes())
    })
    .collect();
```

The closure gets `&mut String` here. With a `Vec<u8>` as the init value it gets `&mut Vec<u8>`, which coerces to `&mut [u8]` wherever a function expects one.

Rayon calls `init` "only as needed for a value to be paired with the group of items in each rayon job": usually far fewer times than there are items, but possibly more than once per thread. Don't use it to count threads.

Predict, then measure: the benchmark's small-file table compares both. A 64 KiB allocation is cheap — the allocator reuses the block just freed — next to `open` + `read` + `close`, so the gain may be small or within noise. `map_init` matters when per-item setup is expensive: large buffers, a compiled regex, a connection. If measurement shows no difference, prefer the simpler code.

`par_chunks(n)` splits a slice into borrowed sub-slices without copying; every thread reads `&[u8]` views of the same buffer. One checksum per chunk is **not** the SHA-256 of the whole buffer — a single SHA-256 can't be split across threads.

### Parallel results must be identical

Parallelism may change speed, never output. The tests compare every parallel function against the sequential one, entry by entry, including errors at the same positions. That holds here because each checksum depends only on its own input, `collect` keeps input order, and SHA-256 is exact integer arithmetic.

Floating point is the classic trap: `(a + b) + c` can differ from `a + (b + c)`, and a parallel `sum()` of `f64` groups additions differently from a loop. If a result depends on the order of combination, define what "identical" means before parallelizing.

`io::Error` implements neither `PartialEq` nor `Clone`, so tests compare `error.kind()`.

### Measuring

| Rule | Why |
|---|---|
| `--release` | Debug timings measure missing optimization, not your code |
| one discarded warm-up run | The first run pays for page faults, cold caches, and lazy setup |
| several runs; report **min** and **median** | Min is the least-disturbed run; median ignores outliers. One slow run skews a mean |
| compare against a sequential baseline | "8 threads: 120 ms" means nothing without "sequential: 700 ms" |
| `std::hint::black_box(result)` | The optimizer may delete work whose result is never used |
| verify results before timing | A fast wrong answer is not a speedup |

**Debug builds are useless for timing.** `cargo run` and `cargo test` use the dev profile: `opt-level = 0`, overflow checks on, almost nothing inlined. Iterator chains and generic code that compile to tight loops in release become chains of function calls — often 10× slower or worse. The ratios lie too: in debug, the work is inflated so much that coordination overhead and memory bandwidth disappear, and parallel speedups look better than they are. Run `cargo run` once and compare; `main` prints a warning in debug builds.

`std::time::Instant` is monotonic, like PHP's `hrtime()` rather than `microtime()`: it never jumps when the system clock is adjusted.

`measure` is enough to learn what benchmark harnesses automate. For real comparisons use `criterion` or `divan` (statistics, outlier detection, regression tracking).

### CPU-bound vs disk-bound, warm vs cold cache

**Amdahl's law:** if a fraction `p` of the work runs in parallel on `n` threads, the best speedup is `1 / ((1 − p) + p / n)`. With `p = 0.9` and 8 threads that's 4.7×, not 8×. Thread start-up, result collection, and anything single-threaded live in `1 − p`.

- **CPU-bound** — data already in memory: speedup grows with cores until it hits a hardware limit. Hyper-threads share execution units, efficiency cores (Apple, recent Intel) are slower than performance cores, and memory bandwidth is shared.
- **Disk-bound** — data must come from storage: if the disk delivers 500 MB/s and one core hashes 2 GB/s, more threads can't help; time is at least `bytes / disk throughput`. SSDs, especially NVMe, can gain substantially from several reads in flight (queue depth) when files are small; spinning disks often lose from seeking. Measure.
- **Warm cache** — recently read files are served from the OS page cache in RAM; no disk involved. The benchmark writes its fixtures and hashes them immediately, so it measures the warm, CPU-bound case.
- **Cold cache** — the first read after a reboot or after dropping caches (Linux: `sync; echo 3 | sudo tee /proc/sys/vm/drop_caches`; macOS: `sudo purge`). That needs privileges you don't have in this container. Run order matters too: the first variant warms the cache for the next, another reason for the warm-up run.

The benchmark writes fixtures to the container's `/tmp`: `/workspace` is a bind mount from macOS and much slower, which would turn the file tables into a file-sharing benchmark.

## Exercise

Implement in this order:

1. `to_hex`: `String::with_capacity`, then `write!(out, "{byte:02x}")` with `use std::fmt::Write as _;`. Writing to a `String` can't fail, so `.expect(…)` the `fmt::Result`.
2. `hash_bytes`: the one-shot form from the `sha2` snippet in Notes.
3. `hash_reader`: `assert!` first, then a `loop` over `match reader.read(buf)` with arms for `Ok(0)`, `Ok(n)`, `Interrupted`, and other errors. Declare the parameter `mut reader`.
4. `hash_file`: `File::open(path)?`, then delegate.
5. `hash_files_sequential`: one `vec![0u8; BUF_SIZE]` before the loop; `path.clone()` into each result.
6. `chunk_checksums`: `chunks` + `map` + `collect`; `hash_bytes` can be passed directly to `map`.
7. `par_chunk_checksums`: the parallel counterpart of `chunks` (needs `use rayon::prelude::*;`). Compare the two bodies.
8. `hash_files_parallel_alloc`: build a pool with `ThreadPoolBuilder`, run `par_iter` inside `pool.install`, and allocate a buffer inside the per-path closure. `build()` already returns the error type this function needs.
9. `hash_files_parallel`: as step 8, with the buffer from `map_init`.
10. `default_threads`: `available_parallelism` returns `io::Result<NonZeroUsize>`; one `Result` combinator turns it into a `usize`.
11. `Timing::from_samples`: `assert!`, `sort_unstable`, then index. Declare `mut samples`.
12. `measure`: `assert!` before anything else, `Instant::now()` / `elapsed()`, `std::hint::black_box`. Declare `mut f`: calling an `FnMut` mutates it.

Then run `cargo run --release` (twice — results vary between runs) and `cargo run` once. Note:

- the speedup at each thread count for in-memory chunks and for files, and where it stops growing;
- how a pool with 1 thread compares with the sequential baseline, and why;
- whether `map_init` beat per-file allocation on your machine;
- how the debug-build numbers differ, including the speedup column.

Optional extensions:

- Restrict the process to 2 CPUs and watch `available_parallelism` and the tables change: `docker compose run --rm -w /workspace/lessons/17-parallelism rust taskset -c 0,1 cargo run --release`.
- Change `BUF_SIZE` to 4 KiB and to 1 MiB and measure the file tables. Explain the result.
- Add `blake3` (without its `rayon` feature) and compare single-thread throughput with SHA-256 on the in-memory data.

## Done when

- `cargo test`, `cargo fmt --check`, and `cargo clippy --all-targets -- -D warnings` pass.
- `cargo run --release` prints all three tables and ends with `all parallel results matched the sequential baseline`.
- You can explain:
  - why debug-build timings are useless, and what `black_box` prevents
  - why `sizes.par_iter().for_each(|s| total += s)` doesn't compile, and what to write instead
  - why `hash_files_parallel` returns input order even though threads finish in any order
  - what `map_init` saves, why `init` can run more than once per thread, and what your measurement showed
  - why in-memory hashing scales better than cold-cache file hashing would (Amdahl)
  - why chunk checksums differ from the SHA-256 of the whole buffer
