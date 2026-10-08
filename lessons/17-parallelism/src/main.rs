//! Benchmark: `cargo run --release`. Every parallel result is checked against the sequential
//! baseline before it is timed.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::Duration;

use parallelism::{
    Checksum, FileHash, Timing, chunk_checksums, default_threads, hash_files_parallel,
    hash_files_parallel_alloc, hash_files_sequential, measure, par_chunk_checksums,
};
use rayon::ThreadPoolBuilder;

const RUNS: usize = 5;
const MIB: usize = 1024 * 1024;
const CHUNK_SIZE: usize = MIB;

fn main() -> io::Result<()> {
    if cfg!(debug_assertions) {
        eprintln!("warning: debug build, timings are meaningless; use `cargo run --release`\n");
    }

    let max_threads = default_threads();
    let thread_counts = thread_counts(max_threads);
    println!("available_parallelism: {max_threads}, runs per row: {RUNS} (+1 warm-up)");

    // The container's /tmp, not the slow macOS bind mount at /workspace.
    let dir = std::env::temp_dir().join(format!("lesson17-bench-{}", std::process::id()));
    let large = write_fixtures(&dir.join("large"), 32, 2 * MIB)?;
    let small = write_fixtures(&dir.join("small"), 2000, 4 * 1024)?;

    let data = pseudo_random_bytes(64 * MIB, 0);
    print_header("64 MiB in memory, 1 MiB chunks (CPU-bound)");
    let expected = chunk_checksums(&data, CHUNK_SIZE);
    let baseline = measure(RUNS, || chunk_checksums(&data, CHUNK_SIZE));
    print_row("sequential", baseline, baseline);
    for &threads in &thread_counts {
        let pool = ThreadPoolBuilder::new()
            .num_threads(threads)
            .build()
            .expect("thread pool");
        assert_eq!(
            pool.install(|| par_chunk_checksums(&data, CHUNK_SIZE)),
            expected
        );
        let timing = measure(RUNS, || {
            pool.install(|| par_chunk_checksums(&data, CHUNK_SIZE))
        });
        print_row(&threads_label(threads), timing, baseline);
    }

    print_header("32 files x 2 MiB, warm page cache");
    let expected = checksums(hash_files_sequential(&large));
    let baseline = measure(RUNS, || hash_files_sequential(&large));
    print_row("sequential", baseline, baseline);
    for &threads in &thread_counts {
        assert_eq!(checksums(parallel(&large, threads)), expected);
        let timing = measure(RUNS, || parallel(&large, threads));
        print_row(&threads_label(threads), timing, baseline);
    }

    print_header("2000 files x 4 KiB: per-file buffer vs map_init");
    let expected = checksums(hash_files_sequential(&small));
    let baseline = measure(RUNS, || hash_files_sequential(&small));
    print_row("sequential", baseline, baseline);
    assert_eq!(checksums(parallel_alloc(&small, max_threads)), expected);
    assert_eq!(checksums(parallel(&small, max_threads)), expected);
    let alloc = measure(RUNS, || parallel_alloc(&small, max_threads));
    print_row(&format!("{max_threads} thr, alloc"), alloc, baseline);
    let reuse = measure(RUNS, || parallel(&small, max_threads));
    print_row(&format!("{max_threads} thr, map_init"), reuse, baseline);

    println!("\nall parallel results matched the sequential baseline");
    fs::remove_dir_all(&dir)
}

fn parallel(paths: &[PathBuf], threads: usize) -> Vec<FileHash> {
    hash_files_parallel(paths, threads).expect("thread pool")
}

fn parallel_alloc(paths: &[PathBuf], threads: usize) -> Vec<FileHash> {
    hash_files_parallel_alloc(paths, threads).expect("thread pool")
}

/// 1, 2, 4, … up to and including `max`.
fn thread_counts(max: usize) -> Vec<usize> {
    let mut counts: Vec<usize> = std::iter::successors(Some(1), |n| Some(n * 2))
        .take_while(|&n| n < max)
        .collect();
    counts.push(max);
    counts
}

/// Fixtures must hash cleanly: an error here is a broken benchmark, not a result to time.
fn checksums(results: Vec<FileHash>) -> Vec<(PathBuf, Checksum)> {
    results
        .into_iter()
        .map(|(path, result)| {
            let checksum = result.unwrap_or_else(|e| panic!("{}: {e}", path.display()));
            (path, checksum)
        })
        .collect()
}

fn write_fixtures(dir: &Path, count: usize, size: usize) -> io::Result<Vec<PathBuf>> {
    fs::create_dir_all(dir)?;
    (0..count)
        .map(|i| {
            let path = dir.join(format!("{i:05}.bin"));
            fs::write(&path, pseudo_random_bytes(size, i as u64 + 1))?;
            Ok(path)
        })
        .collect()
}

/// Deterministic bytes (xorshift64), so every run hashes the same content.
fn pseudo_random_bytes(len: usize, seed: u64) -> Vec<u8> {
    let mut state = seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1;
    let mut bytes = Vec::with_capacity(len + 8);
    while bytes.len() < len {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        bytes.extend_from_slice(&state.to_le_bytes());
    }
    bytes.truncate(len);
    bytes
}

fn threads_label(threads: usize) -> String {
    format!("{threads} thread{}", if threads == 1 { "" } else { "s" })
}

fn print_header(title: &str) {
    println!("\n{title}");
    println!(
        "  {:<18} {:>10} {:>10} {:>8}",
        "variant", "min", "median", "speedup"
    );
}

/// Speedup compares medians: baseline median / this median.
fn print_row(label: &str, timing: Timing, baseline: Timing) {
    println!(
        "  {label:<18} {:>10} {:>10} {:>7.2}x",
        millis(timing.min),
        millis(timing.median),
        baseline.median.as_secs_f64() / timing.median.as_secs_f64()
    );
}

fn millis(duration: Duration) -> String {
    format!("{:.2} ms", duration.as_secs_f64() * 1000.0)
}
