//! Lesson 17 — CPU parallelism and measurement. Replace every `todo!()` until `cargo test` passes,
//! then run the benchmark with `cargo run --release`.

use std::io::{self, Read};
use std::path::{Path, PathBuf};
use std::time::Duration;

use rayon::ThreadPoolBuildError;

/// A SHA-256 checksum as 32 raw bytes. Hex is only for display: see [`to_hex`].
pub type Checksum = [u8; 32];

/// One result per input path. An error belongs to its file only; it doesn't stop the others.
pub type FileHash = (PathBuf, io::Result<Checksum>);

/// Read-buffer size for hashing files.
pub const BUF_SIZE: usize = 64 * 1024;

/// Lowercase hex, two characters per byte: `[0x00, 0xab, 0xff]` → `"00abff"`. Empty input → `""`.
///
/// Allocates exactly once: reserve `2 * bytes.len()` up front, then append each byte.
pub fn to_hex(bytes: &[u8]) -> String {
    todo!()
}

/// SHA-256 of `data`.
pub fn hash_bytes(data: &[u8]) -> Checksum {
    todo!()
}

/// SHA-256 of everything `reader` yields until end of stream (a `read` returning `Ok(0)`).
///
/// Reads into `buf` and hashes only the bytes each read returned (`&buf[..n]`), so any buffer
/// length from 1 byte up gives the same checksum. Never allocates.
/// Retries a read that fails with `ErrorKind::Interrupted`; returns any other error unchanged.
///
/// # Panics
///
/// With the message `"buffer must not be empty"` if `buf` is empty. Reading into an empty buffer
/// returns `Ok(0)`, which looks like end of stream and would silently hash nothing.
pub fn hash_reader(reader: impl Read, buf: &mut [u8]) -> io::Result<Checksum> {
    todo!()
}

/// SHA-256 of the file at `path`, streamed through `buf` with [`hash_reader`], so a file of any
/// size needs only `buf.len()` bytes of memory. Open and read errors are returned unchanged
/// (a missing file gives `ErrorKind::NotFound`).
pub fn hash_file(path: &Path, buf: &mut [u8]) -> io::Result<Checksum> {
    todo!()
}

/// The sequential baseline: hashes every path on the current thread, in input order, allocating
/// one [`BUF_SIZE`] buffer and reusing it for every file.
///
/// Returns exactly one entry per input path, in input order; a path listed twice is hashed twice.
pub fn hash_files_sequential(paths: &[PathBuf]) -> Vec<FileHash> {
    todo!()
}

/// Same output as [`hash_files_sequential`] — same entries, same order, errors at the same
/// positions — computed with `par_iter` on a new Rayon pool of exactly `threads` worker threads.
/// `threads == 0` lets Rayon choose (`RAYON_NUM_THREADS`, or one per available CPU).
///
/// The naive version: allocates a fresh [`BUF_SIZE`] buffer for every file. Kept so the benchmark
/// can measure it against [`hash_files_parallel`].
///
/// # Errors
///
/// Only if the thread pool can't be built. Per-file errors are inside the `Vec`.
pub fn hash_files_parallel_alloc(
    paths: &[PathBuf],
    threads: usize,
) -> Result<Vec<FileHash>, ThreadPoolBuildError> {
    todo!()
}

/// Same contract as [`hash_files_parallel_alloc`], but reuses read buffers with `map_init`:
/// a [`BUF_SIZE`] buffer is allocated once per Rayon job, not once per file.
///
/// # Errors
///
/// Only if the thread pool can't be built. Per-file errors are inside the `Vec`.
pub fn hash_files_parallel(
    paths: &[PathBuf],
    threads: usize,
) -> Result<Vec<FileHash>, ThreadPoolBuildError> {
    todo!()
}

/// SHA-256 of each consecutive `chunk_size`-byte piece of `data`, in order. The last piece is
/// shorter when `data.len()` isn't a multiple of `chunk_size`. Empty `data` → empty `Vec`.
///
/// This is one checksum per chunk, not the checksum of `data` as a whole.
///
/// # Panics
///
/// If `chunk_size` is 0.
pub fn chunk_checksums(data: &[u8], chunk_size: usize) -> Vec<Checksum> {
    todo!()
}

/// Same output as [`chunk_checksums`], computed with `par_chunks` on the current Rayon pool:
/// the global pool, or the pool whose `install` is running this call. Doesn't build a pool.
///
/// # Panics
///
/// If `chunk_size` is 0.
pub fn par_chunk_checksums(data: &[u8], chunk_size: usize) -> Vec<Checksum> {
    todo!()
}

/// The number of threads this process can run at once, from
/// `std::thread::available_parallelism`, or `1` if it can't be determined.
pub fn default_threads() -> usize {
    todo!()
}

/// Summary of repeated timings.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Timing {
    pub min: Duration,
    pub median: Duration,
}

impl Timing {
    /// `min` is the smallest sample. `median` is the sample at index `len / 2` after sorting
    /// ascending — for an even count, the upper of the two middle samples.
    ///
    /// # Panics
    ///
    /// With the message `"no samples"` if `samples` is empty.
    pub fn from_samples(samples: Vec<Duration>) -> Timing {
        todo!()
    }
}

/// Calls `f` once as an untimed warm-up, then `runs` more times, timing each call separately
/// with `std::time::Instant`. Returns [`Timing::from_samples`] of those `runs` timings.
///
/// Every result of `f` (the warm-up's too) goes through `std::hint::black_box`, so the optimizer
/// can't treat the work as unused.
///
/// # Panics
///
/// With the message `"runs must be at least 1"` if `runs` is 0, before calling `f`.
pub fn measure<T>(runs: usize, f: impl FnMut() -> T) -> Timing {
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::ErrorKind;

    const EMPTY_SHA256: &str = "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855";
    const ABC_SHA256: &str = "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad";

    /// Yields `data` at most `step` bytes per `read`, and fails with `ErrorKind::Interrupted`
    /// before every successful read.
    struct Stuttering<'a> {
        data: &'a [u8],
        step: usize,
        interrupt_next: bool,
    }

    impl Read for Stuttering<'_> {
        fn read(&mut self, out: &mut [u8]) -> io::Result<usize> {
            if self.interrupt_next {
                self.interrupt_next = false;
                return Err(io::Error::from(ErrorKind::Interrupted));
            }
            self.interrupt_next = true;
            let n = self.step.min(out.len()).min(self.data.len());
            out[..n].copy_from_slice(&self.data[..n]);
            self.data = &self.data[n..];
            Ok(n)
        }
    }

    /// Yields `ok_bytes`, then fails with `ErrorKind::PermissionDenied`.
    struct FailsAfter<'a> {
        ok_bytes: &'a [u8],
    }

    impl Read for FailsAfter<'_> {
        fn read(&mut self, out: &mut [u8]) -> io::Result<usize> {
            if self.ok_bytes.is_empty() {
                return Err(io::Error::from(ErrorKind::PermissionDenied));
            }
            let n = out.len().min(self.ok_bytes.len());
            out[..n].copy_from_slice(&self.ok_bytes[..n]);
            self.ok_bytes = &self.ok_bytes[n..];
            Ok(n)
        }
    }

    fn sample_data(len: usize) -> Vec<u8> {
        (0..len).map(|i| (i * 31 % 251) as u8).collect()
    }

    fn pool(threads: usize) -> rayon::ThreadPool {
        rayon::ThreadPoolBuilder::new()
            .num_threads(threads)
            .build()
            .unwrap()
    }

    fn ms(millis: u64) -> Duration {
        Duration::from_millis(millis)
    }

    #[test]
    fn hex_is_lowercase_and_zero_padded() {
        assert_eq!(to_hex(&[]), "");
        assert_eq!(to_hex(&[0x00]), "00");
        assert_eq!(to_hex(&[0x00, 0x0f, 0xab, 0xff]), "000fabff");
        assert_eq!(to_hex(&[0x12; 32]).len(), 64);
    }

    #[test]
    fn hashes_known_vectors() {
        assert_eq!(to_hex(&hash_bytes(b"")), EMPTY_SHA256);
        assert_eq!(to_hex(&hash_bytes(b"abc")), ABC_SHA256);
        assert_eq!(
            to_hex(&hash_bytes(
                b"abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq"
            )),
            "248d6a61d20638b8e5c026930c3e6039a33ce45964ff2167f6ecedd419db06c1"
        );
        assert_eq!(
            to_hex(&hash_bytes(&vec![b'a'; 1_000_000])),
            "cdc76e5c9914fb9281a1c7e284d73e67f1809a48a497200e046d39ccc7112cd0"
        );
    }

    #[test]
    fn hash_reader_matches_hash_bytes_for_any_buffer_size() {
        let data = sample_data(10_000);
        for buf_len in [1, 7, 64, 4096, BUF_SIZE] {
            let mut buf = vec![0u8; buf_len];
            assert_eq!(
                hash_reader(data.as_slice(), &mut buf).unwrap(),
                hash_bytes(&data),
                "buffer of {buf_len} bytes"
            );
        }
    }

    #[test]
    fn hash_reader_of_empty_stream() {
        let mut buf = [0u8; 16];
        assert_eq!(
            to_hex(&hash_reader(io::empty(), &mut buf).unwrap()),
            EMPTY_SHA256
        );
    }

    #[test]
    fn hash_reader_ignores_stale_buffer_contents() {
        let mut buf = [0xee_u8; 16];
        assert_eq!(
            to_hex(&hash_reader(&b"abc"[..], &mut buf).unwrap()),
            ABC_SHA256
        );
    }

    #[test]
    fn hash_reader_handles_short_reads_and_retries_interrupted() {
        let data = sample_data(1000);
        let reader = Stuttering {
            data: &data,
            step: 3,
            interrupt_next: true,
        };
        let mut buf = [0u8; 64];
        assert_eq!(hash_reader(reader, &mut buf).unwrap(), hash_bytes(&data));
    }

    #[test]
    fn hash_reader_returns_other_errors() {
        let mut buf = [0u8; 4];
        let err = hash_reader(
            FailsAfter {
                ok_bytes: b"abcdef",
            },
            &mut buf,
        )
        .unwrap_err();
        assert_eq!(err.kind(), ErrorKind::PermissionDenied);
    }

    #[test]
    #[should_panic(expected = "buffer must not be empty")]
    fn hash_reader_rejects_an_empty_buffer() {
        let _ = hash_reader(&b"abc"[..], &mut []);
    }

    #[test]
    fn chunk_checksums_hash_each_piece_in_order() {
        let data = sample_data(10);
        assert_eq!(
            chunk_checksums(&data, 4),
            vec![
                hash_bytes(&data[0..4]),
                hash_bytes(&data[4..8]),
                hash_bytes(&data[8..10]),
            ]
        );
        assert_eq!(chunk_checksums(&data, 10), vec![hash_bytes(&data)]);
        assert_eq!(chunk_checksums(&data, 100), vec![hash_bytes(&data)]);
        assert_eq!(to_hex(&chunk_checksums(b"abcabc", 3)[1]), ABC_SHA256);
    }

    #[test]
    fn chunk_checksums_of_empty_data() {
        assert!(chunk_checksums(&[], 4).is_empty());
        assert!(par_chunk_checksums(&[], 4).is_empty());
    }

    #[test]
    fn chunk_checksums_differ_from_the_whole_checksum() {
        let data = sample_data(100);
        assert_ne!(chunk_checksums(&data, 50)[0], hash_bytes(&data));
    }

    #[test]
    fn par_chunk_checksums_match_sequential_in_any_pool() {
        let data = sample_data(100_000);
        for chunk_size in [1_000, 4_096, 99_999, 100_000, 1_000_000] {
            let expected = chunk_checksums(&data, chunk_size);
            assert_eq!(par_chunk_checksums(&data, chunk_size), expected);
            for threads in [1, 2, 4] {
                assert_eq!(
                    pool(threads).install(|| par_chunk_checksums(&data, chunk_size)),
                    expected,
                    "chunk size {chunk_size}, {threads} threads"
                );
            }
        }
    }

    #[test]
    fn default_threads_is_available_parallelism() {
        let expected = std::thread::available_parallelism().map_or(1, |n| n.get());
        assert_eq!(default_threads(), expected);
        assert!(default_threads() >= 1);
    }

    #[test]
    fn timing_of_odd_sample_count() {
        assert_eq!(
            Timing::from_samples(vec![ms(5), ms(1), ms(3)]),
            Timing {
                min: ms(1),
                median: ms(3)
            }
        );
    }

    #[test]
    fn timing_of_even_sample_count_uses_upper_middle() {
        assert_eq!(
            Timing::from_samples(vec![ms(4), ms(1), ms(2), ms(3)]),
            Timing {
                min: ms(1),
                median: ms(3)
            }
        );
    }

    #[test]
    fn timing_of_single_sample() {
        assert_eq!(
            Timing::from_samples(vec![ms(7)]),
            Timing {
                min: ms(7),
                median: ms(7)
            }
        );
    }

    #[test]
    #[should_panic(expected = "no samples")]
    fn timing_rejects_no_samples() {
        Timing::from_samples(Vec::new());
    }

    #[test]
    fn measure_warms_up_once_then_runs_n_times() {
        let mut calls = 0;
        let timing = measure(5, || {
            calls += 1;
            calls
        });
        assert_eq!(calls, 6);
        assert!(timing.min <= timing.median);
    }

    #[test]
    #[should_panic(expected = "runs must be at least 1")]
    fn measure_rejects_zero_runs() {
        measure(0, || -> u8 { panic!("f must not be called") });
    }
}
