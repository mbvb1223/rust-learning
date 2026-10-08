//! Lesson 15 — directory size scanner. Replace every `todo!()` until `cargo test` passes.

use std::collections::BTreeMap;
use std::fmt;
use std::io::{self, BufRead, Write};
use std::path::{Path, PathBuf};

/// Bytes examined by [`looks_binary`]: 8 KiB.
pub const SNIFF_LEN: usize = 8 * 1024;

/// Human-readable size with 1024-based units.
///
/// - Below 1024: whole bytes — `"0 B"`, `"1023 B"`.
/// - Otherwise: the largest of `KiB`, `MiB`, `GiB`, `TiB`, `PiB`, `EiB` that keeps the value
///   at least 1, with exactly one decimal as `{:.1}` formats it — `1536` → `"1.5 KiB"`.
/// - Never `1024.0`: if the value would display as `1024.0` in a unit, use the next unit —
///   `1_048_575` → `"1.0 MiB"`, not `"1024.0 KiB"`.
pub fn format_size(bytes: u64) -> String {
    todo!()
}

/// The file name's extension as [`Path::extension`] defines it, lowercased.
///
/// `""` when there is none: `"Makefile"`, `".gitignore"` (a leading dot doesn't start an
/// extension) and `"file."` (empty extension). `"archive.tar.gz"` → `"gz"`.
/// A non-UTF-8 extension is converted lossily.
pub fn extension_key(path: &Path) -> String {
    todo!()
}

/// `true` if the first [`SNIFF_LEN`] bytes of `reader` contain a NUL byte (`0x00`).
///
/// - Examine at most `SNIFF_LEN` bytes and stop reading there: the reader may be endless,
///   and a NUL after that point doesn't count.
/// - A reader may hand out fewer bytes than are available (short reads). Keep reading until
///   `SNIFF_LEN` bytes were examined or the reader reaches end of input.
/// - Empty input is not binary. Read errors are returned.
pub fn looks_binary<R: BufRead>(reader: R) -> io::Result<bool> {
    todo!()
}

/// A regular file found by [`scan`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileEntry {
    pub path: PathBuf,
    pub size: u64,
}

/// The `n` largest files, largest `size` first. Equal sizes are ordered by `path`
/// ascending (`PathBuf`'s `Ord`). Fewer than `n` files → all of them.
pub fn top_files(files: Vec<FileEntry>, n: usize) -> Vec<FileEntry> {
    todo!()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScanOptions {
    /// How many files [`ScanReport::largest`] keeps.
    pub top: usize,
    /// Sniff every regular file with [`looks_binary`] and fill [`ScanReport::binary_files`].
    pub detect_binary: bool,
}

/// Count and total size of the regular files sharing one [`extension_key`].
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ExtStats {
    pub files: u64,
    pub bytes: u64,
}

/// A failure on one entry below the root. See the error policy on [`scan`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntryError {
    pub path: PathBuf,
    pub kind: io::ErrorKind,
    /// The `io::Error`'s `Display` text.
    pub message: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ScanReport {
    /// The `root` argument, unchanged.
    pub root: PathBuf,
    /// Regular files.
    pub files: u64,
    /// Directories below the root, including ones that couldn't be listed. The root isn't counted.
    pub dirs: u64,
    /// Every symlink below the root, sorted by path.
    pub symlinks: Vec<PathBuf>,
    /// Entries that are not a regular file, directory or symlink: sockets, FIFOs, devices.
    pub other: u64,
    /// Sum of regular file sizes (`Metadata::len`, the apparent size).
    pub total_bytes: u64,
    /// With `detect_binary`: `Some(n)`, `n` being the files [`looks_binary`] flagged. Without: `None`.
    pub binary_files: Option<u64>,
    /// [`top_files`] over all regular files, with [`ScanOptions::top`].
    pub largest: Vec<FileEntry>,
    /// Regular files grouped by [`extension_key`].
    pub by_extension: BTreeMap<String, ExtStats>,
    /// Per-entry failures, sorted by path.
    pub errors: Vec<EntryError>,
}

/// Fatal [`scan`] errors: problems with the root itself.
#[derive(Debug)]
pub enum ScanError {
    /// `root` couldn't be inspected or opened for listing (missing, permission denied, …).
    Root { path: PathBuf, source: io::Error },
    /// `root` exists but isn't a directory, even after following symlinks.
    NotADirectory(PathBuf),
}

impl fmt::Display for ScanError {
    /// `cannot read {path}: {source}` or `not a directory: {path}`, paths via `Path::display`.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        todo!()
    }
}

impl std::error::Error for ScanError {
    /// The `io::Error` for `Root`, `None` for `NotADirectory`.
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        todo!()
    }
}

/// Recursively summarizes the directory `root` with `std::fs::read_dir`.
///
/// **Root.** Inspected with `fs::metadata`, which follows symlinks, so a symlink to a
/// directory is a valid root. These are the only errors returned as `Err`:
/// - [`ScanError::Root`] if `root` can't be inspected or opened for listing;
/// - [`ScanError::NotADirectory`] if it isn't a directory.
///
/// **Entries below the root** are inspected without following symlinks
/// (`fs::symlink_metadata`, `DirEntry::metadata` or `DirEntry::file_type`):
/// - regular file → counted in `files`, `total_bytes`, `by_extension` and `largest`. When
///   `detect_binary` is set, opened and passed to [`looks_binary`] through a `BufReader`;
/// - directory → counted in `dirs`, then scanned;
/// - symlink → listed in `symlinks` and never followed: its target is not counted, opened or
///   descended into, whether it is a file, a directory, or missing;
/// - anything else → counted in `other`, never opened.
///
/// Every path in the report is `root` joined with the entry's path below it — what
/// `DirEntry::path` returns. Hidden entries are included.
///
/// **Error policy.** A failure on one entry doesn't stop the scan: it is recorded in
/// `errors` with the path it concerns, and scanning continues with the next entry.
/// - A subdirectory that can't be listed, or an error while iterating any directory →
///   that directory's path. The subdirectory still counts in `dirs`.
/// - Metadata that can't be read → the entry's path; the entry isn't counted anywhere else.
/// - A file that can't be opened or sniffed → the file's path. The file still counts
///   everywhere except `binary_files`.
pub fn scan(root: &Path, opts: &ScanOptions) -> Result<ScanReport, ScanError> {
    todo!()
}

/// Writes `report` as text. Every line ends in `\n`. First the summary:
///
/// ```text
/// Root: {root}
/// Files: {files}
/// Directories: {dirs}
/// Symlinks (not followed): {number of symlinks}
/// Other: {other}
/// Total: {format_size(total_bytes)} ({total_bytes} bytes)
/// ```
///
/// with `Files: {files} ({n} binary)` instead when `binary_files` is `Some(n)`.
///
/// Then each non-empty section in this order, preceded by one empty line:
/// - `Largest files:`, then per entry `format!("  {:>10}  {}", format_size(size), path)`;
/// - `By extension:`, then per key in map order
///   `format!("  {:<10} {:>6}  {:>10}", key, files, format_size(bytes))`, with `(none)` as key for `""`;
/// - `Errors:`, then per error `format!("  {}: {}", path, message)`.
///
/// Paths are printed with `Path::display`. Returns the first write error.
pub fn write_report(report: &ScanReport, out: &mut impl Write) -> io::Result<()> {
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::{BufReader, Read};
    use tempfile::TempDir;

    fn tempdir() -> TempDir {
        tempfile::tempdir().expect("create temp dir")
    }

    /// Creates `rel` (and missing parent directories) under `root`, filled with `len` bytes of `b'x'`.
    fn write_file(root: &Path, rel: &str, len: usize) -> PathBuf {
        let path = root.join(rel);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, vec![b'x'; len]).unwrap();
        path
    }

    fn options(top: usize, detect_binary: bool) -> ScanOptions {
        ScanOptions { top, detect_binary }
    }

    fn entry(path: &str, size: u64) -> FileEntry {
        FileEntry {
            path: PathBuf::from(path),
            size,
        }
    }

    fn ext(files: u64, bytes: u64) -> ExtStats {
        ExtStats { files, bytes }
    }

    // --- format_size

    #[test]
    fn formats_whole_bytes_below_one_kib() {
        assert_eq!(format_size(0), "0 B");
        assert_eq!(format_size(1), "1 B");
        assert_eq!(format_size(1023), "1023 B");
    }

    #[test]
    fn formats_larger_sizes_with_one_decimal() {
        assert_eq!(format_size(1024), "1.0 KiB");
        assert_eq!(format_size(1536), "1.5 KiB");
        assert_eq!(format_size(10 * 1024 * 1024), "10.0 MiB");
        assert_eq!(
            format_size(5 * 1024 * 1024 * 1024 + 512 * 1024 * 1024),
            "5.5 GiB"
        );
        assert_eq!(format_size(u64::MAX), "16.0 EiB");
    }

    #[test]
    fn moves_to_the_next_unit_instead_of_showing_1024() {
        assert_eq!(format_size(1_048_524), "1023.9 KiB");
        assert_eq!(format_size(1_048_575), "1.0 MiB");
        assert_eq!(format_size(1024u64.pow(4) - 1), "1.0 TiB");
    }

    // --- extension_key

    #[test]
    fn extension_key_is_the_lowercased_last_extension() {
        assert_eq!(extension_key(Path::new("photo.JPG")), "jpg");
        assert_eq!(extension_key(Path::new("archive.tar.gz")), "gz");
        assert_eq!(extension_key(Path::new("src/notes.md")), "md");
        assert_eq!(extension_key(Path::new(".config.toml")), "toml");
    }

    #[test]
    fn extension_key_is_empty_without_an_extension() {
        assert_eq!(extension_key(Path::new("Makefile")), "");
        assert_eq!(extension_key(Path::new(".gitignore")), "");
        assert_eq!(extension_key(Path::new("file.")), "");
        assert_eq!(extension_key(Path::new("v1.2/README")), "");
    }

    // --- looks_binary

    #[test]
    fn text_is_not_binary() {
        assert!(!looks_binary(&b"hello\nworld\n"[..]).unwrap());
        assert!(!looks_binary("héllo ✓".as_bytes()).unwrap());
        assert!(!looks_binary(&b""[..]).unwrap());
    }

    #[test]
    fn a_nul_byte_means_binary() {
        assert!(looks_binary(&b"PK\x03\x04\0\0"[..]).unwrap());
        assert!(looks_binary(&b"\0"[..]).unwrap());
    }

    #[test]
    fn only_the_first_8_kib_count() {
        let mut data = vec![b'a'; SNIFF_LEN + 100];
        data[SNIFF_LEN - 1] = 0;
        assert!(looks_binary(&data[..]).unwrap());

        let mut data = vec![b'a'; SNIFF_LEN + 100];
        data[SNIFF_LEN] = 0;
        assert!(!looks_binary(&data[..]).unwrap());
    }

    #[test]
    fn stops_reading_after_8_kib() {
        let endless = BufReader::new(io::repeat(b'a'));
        assert!(!looks_binary(endless).unwrap());
    }

    #[test]
    fn keeps_reading_after_short_reads() {
        let mut data = [b'a'; 100];
        data[99] = 0;
        assert!(looks_binary(BufReader::with_capacity(1, &data[..])).unwrap());

        let chained = (&b"abc"[..]).chain(&b"def\0"[..]);
        assert!(looks_binary(chained).unwrap());
    }

    #[test]
    fn returns_read_errors() {
        struct Broken;
        impl Read for Broken {
            fn read(&mut self, _buf: &mut [u8]) -> io::Result<usize> {
                Err(io::Error::other("disk on fire"))
            }
        }

        let err = looks_binary(BufReader::new(Broken)).unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::Other);
    }

    // --- top_files

    #[test]
    fn top_files_puts_the_largest_first() {
        let files = vec![entry("a", 10), entry("b", 30), entry("c", 20)];
        assert_eq!(top_files(files, 2), vec![entry("b", 30), entry("c", 20)]);
    }

    #[test]
    fn top_files_orders_equal_sizes_by_path() {
        let files = vec![
            entry("z.txt", 5),
            entry("sub/b.txt", 5),
            entry("a.txt", 5),
            entry("big", 9),
        ];
        assert_eq!(
            top_files(files, 3),
            vec![entry("big", 9), entry("a.txt", 5), entry("sub/b.txt", 5)]
        );
    }

    #[test]
    fn top_files_handles_small_inputs() {
        assert_eq!(top_files(vec![entry("a", 1)], 5), vec![entry("a", 1)]);
        assert_eq!(top_files(vec![entry("a", 1)], 0), vec![]);
        assert_eq!(top_files(vec![], 3), vec![]);
    }

    // --- ScanError

    #[test]
    fn scan_error_messages_and_sources() {
        use std::error::Error;

        let root = ScanError::Root {
            path: PathBuf::from("/nope"),
            source: io::Error::from(io::ErrorKind::NotFound),
        };
        let expected = format!(
            "cannot read /nope: {}",
            io::Error::from(io::ErrorKind::NotFound)
        );
        assert_eq!(root.to_string(), expected);
        assert!(root.source().is_some());

        let not_dir = ScanError::NotADirectory(PathBuf::from("/etc/hosts"));
        assert_eq!(not_dir.to_string(), "not a directory: /etc/hosts");
        assert!(not_dir.source().is_none());
    }

    // --- scan

    #[test]
    fn scans_an_empty_directory() {
        let dir = tempdir();
        let report = scan(dir.path(), &options(10, false)).unwrap();
        assert_eq!(
            report,
            ScanReport {
                root: dir.path().to_path_buf(),
                ..ScanReport::default()
            }
        );
    }

    #[test]
    fn counts_files_directories_and_bytes_recursively() {
        let dir = tempdir();
        let root = dir.path();
        write_file(root, "a.txt", 10);
        write_file(root, ".hidden", 20);
        write_file(root, "sub/b.rs", 30);
        write_file(root, "sub/deeper/c", 40);
        fs::create_dir(root.join("empty")).unwrap();

        let report = scan(root, &options(10, false)).unwrap();
        assert_eq!(report.root, root);
        assert_eq!(report.files, 4);
        assert_eq!(report.dirs, 3);
        assert_eq!(report.total_bytes, 100);
        assert_eq!(report.other, 0);
        assert_eq!(report.binary_files, None);
        assert!(report.symlinks.is_empty());
        assert!(report.errors.is_empty());
    }

    #[test]
    fn groups_files_by_extension() {
        let dir = tempdir();
        let root = dir.path();
        write_file(root, "a.txt", 10);
        write_file(root, "sub/B.TXT", 5);
        write_file(root, "main.rs", 7);
        write_file(root, "Makefile", 3);
        write_file(root, ".gitignore", 1);

        let report = scan(root, &options(10, false)).unwrap();
        let expected = BTreeMap::from([
            (String::new(), ext(2, 4)),
            ("rs".to_string(), ext(1, 7)),
            ("txt".to_string(), ext(2, 15)),
        ]);
        assert_eq!(report.by_extension, expected);
    }

    #[test]
    fn keeps_the_largest_files_with_full_paths() {
        let dir = tempdir();
        let root = dir.path();
        let small = write_file(root, "small", 1);
        let big = write_file(root, "nested/dir/big.bin", 500);
        let tie_b = write_file(root, "b.log", 50);
        let tie_a = write_file(root, "nested/a.log", 50);

        let report = scan(root, &options(3, false)).unwrap();
        let expected = vec![
            FileEntry {
                path: big,
                size: 500,
            },
            FileEntry {
                path: tie_b,
                size: 50,
            },
            FileEntry {
                path: tie_a,
                size: 50,
            },
        ];
        assert_eq!(report.largest, expected);
        assert!(!report.largest.iter().any(|f| f.path == small));

        let none = scan(root, &options(0, false)).unwrap();
        assert!(none.largest.is_empty());
        assert_eq!(none.files, 4);
    }

    #[test]
    fn counts_binary_files_only_when_asked() {
        let dir = tempdir();
        let root = dir.path();
        write_file(root, "text.txt", 100);
        write_file(root, "empty", 0);
        fs::write(root.join("image.png"), b"\x89PNG\r\n\x1a\n\0\0\0\rIHDR").unwrap();

        let report = scan(root, &options(10, true)).unwrap();
        assert_eq!(report.binary_files, Some(1));

        let report = scan(root, &options(10, false)).unwrap();
        assert_eq!(report.binary_files, None);
    }

    #[test]
    fn missing_root_is_fatal() {
        let dir = tempdir();
        let missing = dir.path().join("missing");

        let err = scan(&missing, &options(10, false)).unwrap_err();
        let ScanError::Root { path, source } = err else {
            panic!("expected ScanError::Root, got {err:?}");
        };
        assert_eq!(path, missing);
        assert_eq!(source.kind(), io::ErrorKind::NotFound);
    }

    #[test]
    fn file_root_is_not_a_directory() {
        let dir = tempdir();
        let file = write_file(dir.path(), "file.txt", 3);

        let err = scan(&file, &options(10, false)).unwrap_err();
        assert!(
            matches!(&err, ScanError::NotADirectory(path) if *path == file),
            "got {err:?}"
        );
    }

    #[cfg(unix)]
    #[test]
    fn symlinks_are_listed_but_never_followed() {
        use std::os::unix::fs::symlink;

        let outside = tempdir();
        write_file(outside.path(), "huge.bin", 4096);

        let dir = tempdir();
        let root = dir.path();
        let real = write_file(root, "real.txt", 5);
        symlink(&real, root.join("to_file")).unwrap();
        symlink(outside.path(), root.join("to_dir")).unwrap();
        symlink(root.join("nowhere"), root.join("dangling")).unwrap();

        let report = scan(root, &options(10, true)).unwrap();
        assert_eq!(report.files, 1);
        assert_eq!(report.dirs, 0);
        assert_eq!(report.total_bytes, 5);
        assert_eq!(report.binary_files, Some(0));
        assert_eq!(
            report.symlinks,
            vec![
                root.join("dangling"),
                root.join("to_dir"),
                root.join("to_file")
            ]
        );
        assert!(report.errors.is_empty());
    }

    #[cfg(unix)]
    #[test]
    fn symlink_cycles_do_not_loop() {
        use std::os::unix::fs::symlink;

        let dir = tempdir();
        let root = dir.path();
        write_file(root, "sub/file", 8);
        symlink("..", root.join("sub/parent")).unwrap();
        symlink(root, root.join("self")).unwrap();

        let report = scan(root, &options(10, false)).unwrap();
        assert_eq!(report.files, 1);
        assert_eq!(report.dirs, 1);
        assert_eq!(
            report.symlinks,
            vec![root.join("self"), root.join("sub/parent")]
        );
    }

    #[cfg(unix)]
    #[test]
    fn a_symlinked_root_is_followed() {
        use std::os::unix::fs::symlink;

        let dir = tempdir();
        write_file(dir.path(), "real/data.csv", 12);
        let link = dir.path().join("link");
        symlink(dir.path().join("real"), &link).unwrap();

        let report = scan(&link, &options(10, false)).unwrap();
        assert_eq!(report.root, link);
        assert_eq!(report.files, 1);
        assert_eq!(
            report.largest,
            vec![FileEntry {
                path: link.join("data.csv"),
                size: 12,
            }]
        );
    }

    #[cfg(unix)]
    #[test]
    fn special_files_count_as_other() {
        use std::os::unix::net::UnixListener;

        let dir = tempdir();
        write_file(dir.path(), "a.txt", 2);
        drop(UnixListener::bind(dir.path().join("app.sock")).unwrap());

        let report = scan(dir.path(), &options(10, true)).unwrap();
        assert_eq!(report.other, 1);
        assert_eq!(report.files, 1);
        assert_eq!(report.total_bytes, 2);
        assert!(report.errors.is_empty());
    }

    #[cfg(unix)]
    #[test]
    fn an_unreadable_directory_is_recorded_and_the_scan_continues() {
        use std::os::unix::fs::PermissionsExt;

        let dir = tempdir();
        let root = dir.path();
        write_file(root, "ok.txt", 3);
        write_file(root, "locked/secret.txt", 1000);
        let locked = root.join("locked");
        fs::set_permissions(&locked, fs::Permissions::from_mode(0o000)).unwrap();

        if fs::read_dir(&locked).is_ok() {
            // Root ignores permission bits, and Docker runs as root. Nothing to test here.
            fs::set_permissions(&locked, fs::Permissions::from_mode(0o755)).unwrap();
            eprintln!("skipped: permission bits are not enforced for this user");
            return;
        }

        let result = scan(root, &options(10, false));
        fs::set_permissions(&locked, fs::Permissions::from_mode(0o755)).unwrap();
        let report = result.unwrap();

        assert_eq!(report.files, 1);
        assert_eq!(report.dirs, 1);
        assert_eq!(report.total_bytes, 3);
        assert_eq!(report.errors.len(), 1);
        assert_eq!(report.errors[0].path, locked);
        assert_eq!(report.errors[0].kind, io::ErrorKind::PermissionDenied);
        assert!(!report.errors[0].message.is_empty());
    }

    // --- write_report

    fn render(report: &ScanReport) -> String {
        let mut out = Vec::new();
        write_report(report, &mut out).unwrap();
        String::from_utf8(out).unwrap()
    }

    #[test]
    fn writes_every_section() {
        let report = ScanReport {
            root: PathBuf::from("data"),
            files: 3,
            dirs: 1,
            symlinks: vec![PathBuf::from("data/link")],
            other: 0,
            total_bytes: 2560,
            binary_files: Some(1),
            largest: vec![entry("data/sub/b.bin", 2048), entry("data/a.txt", 500)],
            by_extension: BTreeMap::from([
                (String::new(), ext(1, 12)),
                ("bin".to_string(), ext(1, 2048)),
                ("txt".to_string(), ext(1, 500)),
            ]),
            errors: vec![EntryError {
                path: PathBuf::from("data/locked"),
                kind: io::ErrorKind::PermissionDenied,
                message: "Permission denied (os error 13)".to_string(),
            }],
        };

        let expected = concat!(
            "Root: data\n",
            "Files: 3 (1 binary)\n",
            "Directories: 1\n",
            "Symlinks (not followed): 1\n",
            "Other: 0\n",
            "Total: 2.5 KiB (2560 bytes)\n",
            "\n",
            "Largest files:\n",
            "     2.0 KiB  data/sub/b.bin\n",
            "       500 B  data/a.txt\n",
            "\n",
            "By extension:\n",
            "  (none)          1        12 B\n",
            "  bin             1     2.0 KiB\n",
            "  txt             1       500 B\n",
            "\n",
            "Errors:\n",
            "  data/locked: Permission denied (os error 13)\n",
        );
        assert_eq!(render(&report), expected);
    }

    #[test]
    fn omits_empty_sections() {
        let report = ScanReport {
            root: PathBuf::from("."),
            ..ScanReport::default()
        };

        let expected = concat!(
            "Root: .\n",
            "Files: 0\n",
            "Directories: 0\n",
            "Symlinks (not followed): 0\n",
            "Other: 0\n",
            "Total: 0 B (0 bytes)\n",
        );
        assert_eq!(render(&report), expected);
    }

    #[test]
    fn returns_write_errors() {
        struct Full;
        impl Write for Full {
            fn write(&mut self, _buf: &[u8]) -> io::Result<usize> {
                Err(io::Error::from(io::ErrorKind::StorageFull))
            }
            fn flush(&mut self) -> io::Result<()> {
                Ok(())
            }
        }

        let err = write_report(&ScanReport::default(), &mut Full).unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::StorageFull);
    }
}
