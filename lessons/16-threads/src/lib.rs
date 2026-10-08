//! Lesson 16 — threads and ownership. Replace every `todo!()` until `cargo test` passes,
//! then fix `src/broken.rs` until `cargo test --features broken` passes.

use std::io;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, MutexGuard};
use std::thread::JoinHandle;

#[cfg(feature = "broken")]
pub mod broken;

/// Line and byte counts for one input.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Counts {
    pub lines: usize,
    pub bytes: usize,
}

/// What one job produced. `io::ErrorKind` instead of `io::Error` because it is `Copy` and
/// comparable with `==`.
pub type Outcome = Result<Counts, io::ErrorKind>;

/// The result of one file job. A failed job is a report with an `Err` outcome, never a panic.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileReport {
    pub path: PathBuf,
    pub outcome: Outcome,
}

/// Totals over many reports.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Summary {
    /// Reports with an `Ok` outcome.
    pub files: usize,
    /// Reports with an `Err` outcome.
    pub failed: usize,
    /// Sum of `lines` over `Ok` outcomes.
    pub lines: usize,
    /// Sum of `bytes` over `Ok` outcomes.
    pub bytes: usize,
}

/// Counts lines and bytes in raw bytes (not necessarily UTF-8).
///
/// - `bytes` is `data.len()`.
/// - A line is a run of bytes ending in `\n`, plus a final run without `\n` if it is non-empty.
///   So `""` → 0 lines, `"a"` → 1, `"a\n"` → 1, `"a\nb"` → 2, `"\n\n"` → 2.
///   `\r\n` is one line break (the `\r` is just a byte of that line).
pub fn count(data: &[u8]) -> Counts {
    todo!()
}

/// Reads the whole file at `path` and [`count`]s it.
/// Any I/O error is returned as its kind; a missing file is `Err(io::ErrorKind::NotFound)`.
pub fn count_file(path: &Path) -> Outcome {
    todo!()
}

/// Starts one OS thread (`std::thread::spawn`) that runs [`count_file`] on `path` and returns
/// the report for it. The path is moved into the thread and comes back inside the report.
pub fn spawn_count(path: PathBuf) -> JoinHandle<FileReport> {
    todo!()
}

/// Counts every file on its own thread via [`spawn_count`]: spawn them all first, then join.
/// Reports are in the same order as `paths`. Missing files are `Err` reports.
pub fn count_files_spawned(paths: Vec<PathBuf>) -> Vec<FileReport> {
    todo!()
}

/// Same result as `count(data)`, computed with `std::thread::scope`: split `data` into at most
/// `parts` contiguous chunks and scan each chunk on its own scoped thread, borrowing `data`
/// (no copying, no `Arc`). Empty `data` returns `Counts::default()`.
///
/// # Panics
/// If `parts` is 0.
pub fn count_in_parts(data: &[u8], parts: usize) -> Counts {
    todo!()
}

/// ASCII-uppercases `data` in place, like `<[u8]>::make_ascii_uppercase`, using at most `parts`
/// scoped threads, each given its own `&mut` chunk. Non-ASCII bytes are unchanged.
/// Empty `data` is left as is.
///
/// # Panics
/// If `parts` is 0.
pub fn uppercase_in_parts(data: &mut [u8], parts: usize) {
    todo!()
}

/// Locks `mutex`, ignoring poisoning: if an earlier holder panicked, returns the guard anyway.
pub fn lock_recovering<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    todo!()
}

/// Runs `job` on every path using `workers` threads started with `std::thread::spawn`.
///
/// - Jobs travel to the workers through one bounded `std::sync::mpsc::sync_channel(queue)`.
///   The calling thread is the producer: it pulls paths from `paths` lazily, one at a time, and
///   blocks while the channel is full (backpressure). Never collect `paths` up front.
///   `queue == 0` is valid (every send waits for a worker to take the job).
/// - Each worker takes one job at a time, runs `job` on it *without* holding any lock, sends a
///   [`FileReport`] back through a results channel, and exits once the job channel is closed
///   and empty. At most `workers` jobs run at the same time.
/// - Every path produces exactly one report, including failures (`Err` outcome).
/// - All workers are joined before returning. Reports are sorted by `path` (`PathBuf`'s `Ord`),
///   so the output doesn't depend on which worker finished first.
///
/// # Panics
/// If `workers` is 0, or if `job` panics (the worker's panic propagates to the caller).
pub fn run_pool(
    paths: impl IntoIterator<Item = PathBuf>,
    workers: usize,
    queue: usize,
    job: fn(&Path) -> Outcome,
) -> Vec<FileReport> {
    todo!()
}

impl Summary {
    /// Adds one outcome: `Ok` increments `files` and adds its lines and bytes;
    /// `Err` increments `failed` only.
    pub fn add(&mut self, outcome: &Outcome) {
        todo!()
    }
}

/// Totals over `reports`, using [`Summary::add`]. No threads, no locks: the caller owns the
/// reports.
pub fn summarize(reports: &[FileReport]) -> Summary {
    todo!()
}

/// The same totals as `summarize(&run_pool(paths, workers, _, count_file))`, built the other
/// way: split `paths` into at most `workers` groups, move each group into a thread started with
/// `std::thread::spawn`, and have every thread [`count_file`] its paths and add each outcome
/// into one shared `Arc<Mutex<Summary>>`. Join all threads, then return the total.
/// Empty `paths` returns `Summary::default()`.
///
/// # Panics
/// If `workers` is 0.
pub fn summarize_shared(paths: Vec<PathBuf>, workers: usize) -> Summary {
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering::SeqCst};
    use std::thread;
    use std::time::Duration;
    use tempfile::TempDir;

    /// Writes each `(name, contents)` into a fresh temp dir; returns the dir and the file paths.
    fn fixture(files: &[(&str, &str)]) -> (TempDir, Vec<PathBuf>) {
        let dir = tempfile::tempdir().unwrap();
        let paths = files
            .iter()
            .map(|(name, contents)| {
                let path = dir.path().join(name);
                fs::write(&path, contents).unwrap();
                path
            })
            .collect();
        (dir, paths)
    }

    fn counts(lines: usize, bytes: usize) -> Outcome {
        Ok(Counts { lines, bytes })
    }

    fn report(path: &Path, outcome: Outcome) -> FileReport {
        FileReport {
            path: path.to_path_buf(),
            outcome,
        }
    }

    fn job_paths(n: usize) -> Vec<PathBuf> {
        (0..n)
            .map(|i| PathBuf::from(format!("job-{i:03}")))
            .collect()
    }

    /// A fake job that needs no files: one line, as many bytes as the path is long.
    fn path_len(path: &Path) -> Outcome {
        counts(1, path.as_os_str().len())
    }

    /// A fake job that fails for paths ending in an odd digit.
    fn fail_odd(path: &Path) -> Outcome {
        let last = path.to_str().unwrap().bytes().last().unwrap();
        if (last - b'0') % 2 == 1 {
            Err(io::ErrorKind::InvalidData)
        } else {
            counts(0, 0)
        }
    }

    #[test]
    fn counts_empty_input() {
        assert_eq!(count(b""), Counts::default());
    }

    #[test]
    fn counts_a_final_line_without_newline() {
        assert_eq!(count(b"a"), Counts { lines: 1, bytes: 1 });
        assert_eq!(count(b"a\n"), Counts { lines: 1, bytes: 2 });
        assert_eq!(count(b"a\nb"), Counts { lines: 2, bytes: 3 });
        assert_eq!(count(b"a\nb\n"), Counts { lines: 2, bytes: 4 });
    }

    #[test]
    fn counts_empty_lines_and_crlf() {
        assert_eq!(count(b"\n"), Counts { lines: 1, bytes: 1 });
        assert_eq!(count(b"\n\n"), Counts { lines: 2, bytes: 2 });
        assert_eq!(count(b"a\r\nb\r\n"), Counts { lines: 2, bytes: 6 });
    }

    #[test]
    fn counts_bytes_not_chars() {
        assert_eq!(count("héllo\n".as_bytes()), Counts { lines: 1, bytes: 7 });
        assert_eq!(count(&[0xff, b'\n', 0xfe]), Counts { lines: 2, bytes: 3 });
    }

    #[test]
    fn count_file_reads_the_file() {
        let (_dir, paths) = fixture(&[("a.txt", "one\ntwo\nthree")]);
        assert_eq!(count_file(&paths[0]), counts(3, 13));
    }

    #[test]
    fn count_file_reports_a_missing_file() {
        let dir = tempfile::tempdir().unwrap();
        assert_eq!(
            count_file(&dir.path().join("missing.txt")),
            Err(io::ErrorKind::NotFound)
        );
    }

    #[test]
    fn spawn_count_returns_the_report_through_join() {
        let (dir, paths) = fixture(&[("a.txt", "x\ny\n")]);
        let ok = spawn_count(paths[0].clone()).join().unwrap();
        assert_eq!(ok, report(&paths[0], counts(2, 4)));

        let missing = dir.path().join("missing.txt");
        let failed = spawn_count(missing.clone()).join().unwrap();
        assert_eq!(failed, report(&missing, Err(io::ErrorKind::NotFound)));
    }

    #[test]
    fn spawned_reports_keep_input_order() {
        let (dir, paths) = fixture(&[("a.txt", "a\n"), ("b.txt", "bb\nbb\n"), ("c.txt", "")]);
        let missing = dir.path().join("missing.txt");
        let input = vec![
            paths[2].clone(),
            missing.clone(),
            paths[0].clone(),
            paths[1].clone(),
        ];
        assert_eq!(
            count_files_spawned(input),
            vec![
                report(&paths[2], counts(0, 0)),
                report(&missing, Err(io::ErrorKind::NotFound)),
                report(&paths[0], counts(1, 2)),
                report(&paths[1], counts(2, 6)),
            ]
        );
    }

    #[test]
    fn spawned_with_no_paths_is_empty() {
        assert!(count_files_spawned(Vec::new()).is_empty());
    }

    #[test]
    fn count_in_parts_matches_count() {
        let long = "line\n".repeat(50) + "tail";
        let inputs = [
            "a",
            "a\n",
            "a\nbb\nccc",
            "\n\n\n",
            "x\r\ny\r\n",
            long.as_str(),
        ];
        for input in inputs {
            let data = input.as_bytes();
            for parts in [1, 2, 3, 4, 5, 7, 16, 300] {
                assert_eq!(
                    count_in_parts(data, parts),
                    count(data),
                    "input {input:?} in {parts} parts"
                );
            }
        }
    }

    #[test]
    fn count_in_parts_handles_empty_input() {
        assert_eq!(count_in_parts(b"", 1), Counts::default());
        assert_eq!(count_in_parts(b"", 4), Counts::default());
    }

    #[test]
    #[should_panic]
    fn count_in_parts_rejects_zero_parts() {
        count_in_parts(b"abc", 0);
    }

    #[test]
    fn uppercase_in_parts_matches_sequential() {
        let original = "Hello, wörld! threads 123 ünïcode".as_bytes();
        for parts in 1..=original.len() + 2 {
            let mut data = original.to_vec();
            uppercase_in_parts(&mut data, parts);
            assert_eq!(
                data,
                original.to_ascii_uppercase(),
                "{parts} parts gave {:?}",
                String::from_utf8_lossy(&data)
            );
        }
    }

    #[test]
    fn uppercase_in_parts_handles_empty_input() {
        let mut data: Vec<u8> = Vec::new();
        uppercase_in_parts(&mut data, 3);
        assert!(data.is_empty());
    }

    #[test]
    #[should_panic]
    fn uppercase_in_parts_rejects_zero_parts() {
        uppercase_in_parts(&mut b"abc".to_vec(), 0);
    }

    #[test]
    fn lock_recovering_locks_a_healthy_mutex() {
        let mutex = Mutex::new(vec![1]);
        lock_recovering(&mutex).push(2);
        assert_eq!(*lock_recovering(&mutex), vec![1, 2]);
    }

    #[test]
    fn lock_recovering_ignores_poison() {
        let mutex = Arc::new(Mutex::new(1));
        let in_thread = Arc::clone(&mutex);
        let result = thread::spawn(move || {
            let _guard = in_thread.lock().unwrap();
            panic!("poisoning the mutex on purpose");
        })
        .join();
        assert!(result.is_err());
        assert!(mutex.is_poisoned());

        *lock_recovering(&mutex) += 1;
        assert_eq!(*lock_recovering(&mutex), 2);
    }

    #[test]
    fn pool_counts_files_sorted_by_path() {
        let (_dir, paths) = fixture(&[
            ("a.txt", "1\n2\n3\n"),
            ("b.txt", ""),
            ("c.txt", "no newline"),
            ("d.txt", "\n"),
        ]);
        let shuffled = vec![
            paths[2].clone(),
            paths[0].clone(),
            paths[3].clone(),
            paths[1].clone(),
        ];
        assert_eq!(
            run_pool(shuffled, 3, 2, count_file),
            vec![
                report(&paths[0], counts(3, 6)),
                report(&paths[1], counts(0, 0)),
                report(&paths[2], counts(1, 10)),
                report(&paths[3], counts(1, 1)),
            ]
        );
    }

    #[test]
    fn pool_reports_failures_as_results() {
        let (dir, paths) = fixture(&[("a.txt", "a\n"), ("c.txt", "c\n")]);
        let missing = dir.path().join("b.txt");
        let input = vec![paths[1].clone(), missing.clone(), paths[0].clone()];
        assert_eq!(
            run_pool(input, 2, 1, count_file),
            vec![
                report(&paths[0], counts(1, 2)),
                report(&missing, Err(io::ErrorKind::NotFound)),
                report(&paths[1], counts(1, 2)),
            ]
        );

        let reports = run_pool(job_paths(10), 3, 2, fail_odd);
        let failed: Vec<_> = reports.iter().filter(|r| r.outcome.is_err()).collect();
        assert_eq!(reports.len(), 10);
        assert_eq!(failed.len(), 5);
    }

    #[test]
    fn pool_handles_no_jobs() {
        assert!(run_pool(Vec::<PathBuf>::new(), 4, 2, count_file).is_empty());
    }

    #[test]
    fn pool_works_with_a_rendezvous_queue_and_one_worker() {
        let expected: Vec<_> = job_paths(20)
            .iter()
            .map(|path| report(path, path_len(path)))
            .collect();
        let mut reversed = job_paths(20);
        reversed.reverse();
        assert_eq!(run_pool(reversed, 1, 0, path_len), expected);
    }

    #[test]
    fn pool_handles_many_jobs_and_more_workers_than_jobs() {
        let reports = run_pool(job_paths(300), 3, 1, path_len);
        assert_eq!(reports.len(), 300);
        assert!(reports.is_sorted_by(|a, b| a.path < b.path));
        assert!(reports.iter().all(|r| r.outcome == counts(1, 7)));

        assert_eq!(run_pool(job_paths(2), 8, 4, path_len).len(), 2);
    }

    static RUNNING: AtomicUsize = AtomicUsize::new(0);
    static PEAK: AtomicUsize = AtomicUsize::new(0);

    fn slow_job(path: &Path) -> Outcome {
        let now = RUNNING.fetch_add(1, SeqCst) + 1;
        PEAK.fetch_max(now, SeqCst);
        thread::sleep(Duration::from_millis(50));
        RUNNING.fetch_sub(1, SeqCst);
        path_len(path)
    }

    #[test]
    fn pool_runs_jobs_in_parallel_on_at_most_n_workers() {
        let reports = run_pool(job_paths(12), 4, 4, slow_job);
        assert_eq!(reports.len(), 12);
        let peak = PEAK.load(SeqCst);
        assert!(
            peak > 1,
            "jobs ran one at a time; is a worker holding the receiver lock while running a job?"
        );
        assert!(peak <= 4, "{peak} jobs ran at once with 4 workers");
    }

    static STARTED: AtomicUsize = AtomicUsize::new(0);

    fn mark_started(path: &Path) -> Outcome {
        STARTED.fetch_add(1, SeqCst);
        thread::sleep(Duration::from_millis(1));
        path_len(path)
    }

    #[test]
    fn pool_pulls_paths_lazily_with_backpressure() {
        let (workers, queue) = (2, 2);
        let mut pulled = 0;
        let mut max_ahead = 0;
        let paths = job_paths(100).into_iter().inspect(|_| {
            pulled += 1;
            max_ahead = max_ahead.max(pulled - STARTED.load(SeqCst));
        });
        assert_eq!(run_pool(paths, workers, queue, mark_started).len(), 100);
        // Pulled but not started: the path just pulled, `queue` in the channel, one per worker.
        assert!(
            max_ahead <= 1 + queue + workers,
            "the producer ran {max_ahead} paths ahead of the workers; \
             are paths pulled one at a time into a bounded channel?"
        );
    }

    fn panic_on_job_005(path: &Path) -> Outcome {
        assert_ne!(path, Path::new("job-005"), "job panicked on purpose");
        path_len(path)
    }

    #[test]
    #[should_panic]
    fn pool_propagates_a_job_panic() {
        run_pool(job_paths(10), 2, 1, panic_on_job_005);
    }

    #[test]
    #[should_panic]
    fn pool_rejects_zero_workers() {
        run_pool(job_paths(1), 0, 1, path_len);
    }

    #[test]
    fn summary_add_counts_successes_and_failures() {
        let mut summary = Summary::default();
        summary.add(&counts(3, 10));
        summary.add(&Err(io::ErrorKind::NotFound));
        summary.add(&counts(2, 5));
        assert_eq!(
            summary,
            Summary {
                files: 2,
                failed: 1,
                lines: 5,
                bytes: 15
            }
        );
    }

    #[test]
    fn summarize_folds_reports() {
        assert_eq!(summarize(&[]), Summary::default());
        let reports = [
            report(Path::new("a"), counts(1, 4)),
            report(Path::new("b"), Err(io::ErrorKind::PermissionDenied)),
            report(Path::new("c"), counts(0, 0)),
            report(Path::new("d"), counts(9, 100)),
        ];
        assert_eq!(
            summarize(&reports),
            Summary {
                files: 3,
                failed: 1,
                lines: 10,
                bytes: 104
            }
        );
    }

    #[test]
    fn shared_summary_counts_every_file_once() {
        let names: Vec<String> = (0..25).map(|i| format!("{i:02}.txt")).collect();
        let files: Vec<(&str, &str)> = names.iter().map(|n| (n.as_str(), "ab\ncd\n")).collect();
        let (dir, mut paths) = fixture(&files);
        paths.push(dir.path().join("missing-1.txt"));
        paths.push(dir.path().join("missing-2.txt"));

        let expected = Summary {
            files: 25,
            failed: 2,
            lines: 50,
            bytes: 150,
        };
        for workers in [1, 2, 4, 26, 27, 100] {
            assert_eq!(
                summarize_shared(paths.clone(), workers),
                expected,
                "{workers} workers"
            );
        }
    }

    #[test]
    fn shared_summary_of_nothing_is_default() {
        assert_eq!(summarize_shared(Vec::new(), 3), Summary::default());
    }

    #[test]
    #[should_panic]
    fn shared_summary_rejects_zero_workers() {
        summarize_shared(job_paths(1), 0);
    }
}
