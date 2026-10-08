//! Shared test helpers. The path is `tests/common/mod.rs`, not `tests/common.rs`: Cargo compiles
//! every `.rs` file directly in `tests/` as its own test crate.

use std::path::PathBuf;

/// The table of contents of `fixtures/sample.md` with the default `--max-level` of 6.
pub const SAMPLE_TOC: &str = "\
- [Rust notes](#rust-notes)
  - [Install](#install)
  - [Usage](#usage)
    - [Options](#options)
      - [Deep detail](#deep-detail)
  - [Usage](#usage-1)
  - [FAQ: Why `Option<T>`?](#faq-why-optiont)
";

/// The table of contents of `fixtures/sample.md` with `--max-level 2`.
pub const SAMPLE_TOC_MAX_2: &str = "\
- [Rust notes](#rust-notes)
  - [Install](#install)
  - [Usage](#usage)
  - [Usage](#usage-1)
  - [FAQ: Why `Option<T>`?](#faq-why-optiont)
";

pub fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name)
}
