use std::fs;
use std::io::ErrorKind;
use std::path::PathBuf;

use parallelism::{
    BUF_SIZE, Checksum, FileHash, hash_bytes, hash_file, hash_files_parallel,
    hash_files_parallel_alloc, hash_files_sequential, to_hex,
};

const EMPTY_SHA256: &str = "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855";
const ABC_SHA256: &str = "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad";

/// A fresh directory under the system temp dir, removed on drop — also when a test panics.
struct TempDir(PathBuf);

impl TempDir {
    fn new(test_name: &str) -> TempDir {
        let path =
            std::env::temp_dir().join(format!("lesson17-{}-{test_name}", std::process::id()));
        let _ = fs::remove_dir_all(&path);
        fs::create_dir_all(&path).unwrap();
        TempDir(path)
    }

    fn file(&self, name: &str, contents: &[u8]) -> PathBuf {
        let path = self.0.join(name);
        fs::write(&path, contents).unwrap();
        path
    }

    fn missing(&self, name: &str) -> PathBuf {
        self.0.join(name)
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn sample_data(len: usize, seed: u8) -> Vec<u8> {
    (0..len)
        .map(|i| (i % 251) as u8 ^ seed.wrapping_mul(37))
        .collect()
}

/// `io::Error` implements neither `PartialEq` nor `Clone`, so tests compare error kinds.
type Comparable = Vec<(PathBuf, Result<Checksum, ErrorKind>)>;

fn comparable(results: Vec<FileHash>) -> Comparable {
    results
        .into_iter()
        .map(|(path, result)| (path, result.map_err(|e| e.kind())))
        .collect()
}

/// A large file first (it finishes last), many small files of uneven sizes, a missing file in
/// the middle, and a duplicate path at the end.
fn mixed_fixture(dir: &TempDir) -> (Vec<PathBuf>, Comparable) {
    let mut paths = Vec::new();
    let mut expected = Vec::new();
    let mut add = |name: String, data: Vec<u8>| {
        let path = dir.file(&name, &data);
        paths.push(path.clone());
        expected.push((path, Ok(hash_bytes(&data))));
    };

    add("big.bin".to_string(), sample_data(2 * 1024 * 1024 + 3, 0));
    let sizes = [
        0,
        1,
        100,
        BUF_SIZE - 1,
        BUF_SIZE,
        BUF_SIZE + 1,
        3 * BUF_SIZE + 17,
    ];
    for i in 0..40 {
        add(
            format!("file-{i:02}.bin"),
            sample_data(sizes[i % sizes.len()], i as u8),
        );
    }

    let missing = dir.missing("missing.bin");
    paths.insert(20, missing.clone());
    expected.insert(20, (missing, Err(ErrorKind::NotFound)));

    paths.push(paths[1].clone());
    expected.push(expected[1].clone());
    (paths, expected)
}

#[test]
fn hash_file_matches_known_vectors() {
    let dir = TempDir::new("known-vectors");
    let mut buf = vec![0u8; BUF_SIZE];
    let abc = dir.file("abc.txt", b"abc");
    let empty = dir.file("empty.txt", b"");

    assert_eq!(to_hex(&hash_file(&abc, &mut buf).unwrap()), ABC_SHA256);
    assert_eq!(to_hex(&hash_file(&empty, &mut buf).unwrap()), EMPTY_SHA256);
}

#[test]
fn hash_file_streams_files_larger_than_the_buffer() {
    let dir = TempDir::new("streams");
    let data = sample_data(3 * BUF_SIZE + 17, 1);
    let path = dir.file("large.bin", &data);

    for buf_len in [1000, BUF_SIZE] {
        let mut buf = vec![0u8; buf_len];
        assert_eq!(hash_file(&path, &mut buf).unwrap(), hash_bytes(&data));
    }
}

#[test]
fn hash_file_reports_a_missing_file() {
    let dir = TempDir::new("missing");
    let mut buf = vec![0u8; BUF_SIZE];
    let err = hash_file(&dir.missing("nope.txt"), &mut buf).unwrap_err();
    assert_eq!(err.kind(), ErrorKind::NotFound);
}

#[test]
fn hash_file_of_a_directory_is_an_error() {
    let dir = TempDir::new("directory");
    let mut buf = vec![0u8; BUF_SIZE];
    assert!(hash_file(&dir.0, &mut buf).is_err());
}

#[test]
fn sequential_returns_one_entry_per_path_in_order() {
    let dir = TempDir::new("sequential");
    let (paths, expected) = mixed_fixture(&dir);
    assert_eq!(comparable(hash_files_sequential(&paths)), expected);
}

#[test]
fn sequential_of_no_paths_is_empty() {
    assert!(hash_files_sequential(&[]).is_empty());
}

#[test]
fn parallel_matches_sequential_for_any_thread_count() {
    let dir = TempDir::new("parallel");
    let (paths, expected) = mixed_fixture(&dir);
    assert_eq!(comparable(hash_files_sequential(&paths)), expected);

    for threads in [1, 2, 3, 8, 0] {
        assert_eq!(
            comparable(hash_files_parallel(&paths, threads).unwrap()),
            expected,
            "hash_files_parallel, {threads} threads"
        );
    }
}

#[test]
fn parallel_alloc_matches_sequential_for_any_thread_count() {
    let dir = TempDir::new("parallel-alloc");
    let (paths, expected) = mixed_fixture(&dir);

    for threads in [1, 2, 3, 8, 0] {
        assert_eq!(
            comparable(hash_files_parallel_alloc(&paths, threads).unwrap()),
            expected,
            "hash_files_parallel_alloc, {threads} threads"
        );
    }
}

#[test]
fn parallel_of_no_paths_is_empty() {
    assert!(hash_files_parallel(&[], 4).unwrap().is_empty());
    assert!(hash_files_parallel_alloc(&[], 4).unwrap().is_empty());
}
