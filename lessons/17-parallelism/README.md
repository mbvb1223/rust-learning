# Lesson 17 — CPU parallelism and measurement

Hash files with SHA-256 sequentially and in parallel, check the results match, and measure the speedup: implement every `todo!()` in `src/lib.rs`, then run the benchmark in `src/main.rs`.

## Run

Inside the container:

```bash
cd /workspace/lessons/17-parallelism
cargo test
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo run --release
```

Don't run `cargo fix`. Timings only mean something with `cargo run --release`.

Plain `cargo test` runs `tests/files.rs` only after every unit test passes. To check steps 4, 5, 8 and 9 earlier, run `cargo test --test files` (or `cargo test --no-fail-fast`).

## Notes

### Many cores, one process

PHP uses more cores by starting more processes (FPM workers, queue consumers). Rayon runs work on several threads inside one process, and the compiler checks that it's safe, so a data race is a compile error.

### SHA-256 with sha2

```rust
use sha2::{Digest, Sha256};

let mut hasher = Sha256::new();
hasher.update(b"ab");
hasher.update(b"c");
let checksum: [u8; 32] = hasher.finalize().into();

let same: [u8; 32] = Sha256::digest(b"abc").into();
println!("{}", checksum == same); // true
```

- `Digest` is a trait: without the `use`, the methods don't exist.
- `.into()` turns the result into a plain `[u8; 32]`.
- `new` / `update` / `finalize` are PHP's `hash_init` / `hash_update` / `hash_final`.

### Hex output

`write!` can append to a `String`. `{:02x}` prints a byte as two lowercase hex digits:

```rust
use std::fmt::Write as _;

let mut out = String::new();
write!(out, "{:02x}", 10).unwrap();
write!(out, "{:02x}", 255).unwrap();
println!("{out}"); // 0aff
```

### Streaming with a reusable buffer

`read` fills part of a buffer and returns how many bytes it wrote. Only those bytes are new:

```rust
let mut file = File::open("big.iso")?;
let mut buf = vec![0u8; 64 * 1024]; // allocated once
let n = file.read(&mut buf)?;        // 0 means end of file
hasher.update(&buf[..n]);
```

- A read can fail with `ErrorKind::Interrupted`. Retry it; return any other error.
- `fs::read` is `file_get_contents`: the whole file in memory. Streaming needs 64 KiB for any file size.
- `impl Read` accepts a `File`, a `&[u8]`, or a test reader.
- Lend the same buffer to every file: `hash_file(path, &mut buf)`.

### Rayon basics

Swap `iter()` for `par_iter()` and the work runs on all cores:

```rust
use rayon::prelude::*;

let nums = vec![1, 2, 3, 4];
let squares: Vec<i32> = nums.par_iter().map(|n| n * n).collect();
println!("{squares:?}"); // [1, 4, 9, 16]
```

- `iter()` → `par_iter()`, `into_iter()` → `into_par_iter()`, `chunks(n)` → `par_chunks(n)`.
- `collect()` keeps input order, whichever thread finishes first.

### Combine results, don't share a counter

Rayon closures run on several threads at once, so they can't change captured variables:

```rust
let mut total = 0;
nums.par_iter().for_each(|n| total += n);
// error[E0596]: cannot borrow `total` as mutable, as it is a captured variable in a `Fn` closure

let total: i32 = nums.par_iter().sum();
println!("{total}"); // 10
```

### Thread pools

`ThreadPoolBuilder` builds a pool with a fixed number of threads. Rayon calls inside `install` use it:

```rust
use rayon::ThreadPoolBuilder;

let pool = ThreadPoolBuilder::new().num_threads(4).build()?;
let total: i32 = pool.install(|| nums.par_iter().sum());
println!("{total}"); // 10
```

- Without `install`, Rayon uses its global pool: one thread per CPU. `num_threads(0)` means the same default.
- `std::thread::available_parallelism()` returns `io::Result<NonZeroUsize>`. In Docker it counts the container's CPUs, not the Mac's.

### Reuse scratch space with map_init

`map_init` creates a value once per Rayon job and lends it as `&mut` to every item in that job:

```rust
let lines = vec!["a b", " c  d "];
let squeezed: Vec<String> = lines
    .par_iter()
    .map_init(String::new, |scratch, line| {
        scratch.clear();
        scratch.extend(line.split_whitespace());
        scratch.clone()
    })
    .collect();
println!("{squeezed:?}"); // ["ab", "cd"]
```

- `init` runs far fewer times than there are items, but can run more than once per thread.
- A `&mut Vec<u8>` works wherever a function expects `&mut [u8]`.
- It pays off when setup is expensive. A 64 KiB buffer is cheap, so measure.

### Parallel chunks

`par_chunks(n)` splits a slice into borrowed pieces, without copying:

```rust
let data = vec![0u8; 10];
let sizes: Vec<usize> = data.par_chunks(4).map(|chunk| chunk.len()).collect();
println!("{sizes:?}"); // [4, 4, 2]
```

One checksum per chunk is not the SHA-256 of the whole buffer. SHA-256 processes blocks in order, so one hash can't be split across threads.

### Same results, any thread count

Parallel code may change the speed, never the output. The tests compare each parallel function with the sequential one, entry by entry, errors included.

- `io::Error` isn't `PartialEq`, so the tests compare `error.kind()`.
- Floating-point sums can differ: `(a + b) + c` isn't always `a + (b + c)`.

### Measuring

```rust
use std::time::Instant;

let start = Instant::now();
let result = work();
std::hint::black_box(result);
println!("{:?}", start.elapsed()); // 6.536ms (varies)
```

- Use `--release`. Debug builds are often 10× slower, and their speedups look better than they are.
- Run once untimed (warm-up), then several times. Report the **min** and the **median**.
- `black_box` stops the optimizer from deleting work whose result is never used.
- Check results before timing: a fast wrong answer isn't a speedup.

### CPU-bound vs disk-bound

**Amdahl's law:** if a fraction `p` of the work runs in parallel on `n` threads, the best speedup is `1 / ((1 − p) + p / n)`. With `p = 0.9` and 8 threads, that's 4.7×, not 8×.

- CPU-bound (data in memory): the speedup grows with cores, up to a hardware limit.
- Disk-bound: the disk's speed is the limit, and more threads don't help.
- The benchmark hashes files it just wrote, so they come from the OS cache in RAM: CPU-bound. It writes them to `/tmp` because `/workspace` is a slow macOS bind mount.

## Exercise

Implement in this order:

1. `to_hex`: `String::with_capacity`, then `write!(out, "{byte:02x}")` with `use std::fmt::Write as _;`; `.expect(…)` the `fmt::Result`.
2. `hash_bytes`: the one-shot `Sha256::digest` from "SHA-256 with sha2".
3. `hash_reader`: `assert!` first, then a `loop` over `match reader.read(buf)` with arms for `Ok(0)`, `Ok(n)`, `Interrupted`, and other errors. Declare `mut reader`.
4. `hash_file`: `File::open(path)?`, then delegate.
5. `hash_files_sequential`: one `vec![0u8; BUF_SIZE]` before the loop; `path.clone()` into each result.
6. `chunk_checksums`: `chunks` + `map` + `collect`; pass `hash_bytes` directly to `map`.
7. `par_chunk_checksums`: the same with `par_chunks` (needs `use rayon::prelude::*;`).
8. `hash_files_parallel_alloc`: `ThreadPoolBuilder`, `par_iter` inside `pool.install`, a new buffer per path. `build()` already returns the error type you need.
9. `hash_files_parallel`: as step 8, with the buffer from `map_init`.
10. `default_threads`: one `Result` combinator turns `available_parallelism()` into a `usize`.
11. `Timing::from_samples`: `assert!`, `sort_unstable`, then index. Declare `mut samples`.
12. `measure`: `assert!` before anything else, `Instant::now()` / `elapsed()`, `std::hint::black_box`. Declare `mut f`.

Then run `cargo run --release` twice and `cargo run` once. Note:

- the speedup at each thread count, in memory and for files, and where it stops growing;
- how a 1-thread pool compares with the sequential baseline, and why;
- whether `map_init` beat per-file allocation on your machine;
- how the debug-build numbers differ, including the speedup column.

Optional: restrict the process to 2 CPUs with `taskset -c 0,1 cargo run --release` and watch the tables change.

## Done when

- `cargo test`, `cargo fmt --check`, and `cargo clippy --all-targets -- -D warnings` pass.
- `cargo run --release` prints all three tables and ends with `all parallel results matched the sequential baseline`.
- You can explain:
  - why debug-build timings are useless, and what `black_box` prevents
  - why `sizes.par_iter().for_each(|s| total += s)` doesn't compile, and what to write instead
  - why `hash_files_parallel` returns input order even though threads finish in any order
