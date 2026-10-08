//! Integration tests: a separate crate that uses the library the way any dependent crate would.
//! Only `pub` items are reachable, through `modules_testing::…` paths.

mod common;

use std::{fs, io};

use modules_testing::{generate_toc, markdown, toc};

// Returning `Result` lets a test use `?`; an `Err` fails the test.
#[test]
fn generates_the_toc_of_the_fixture() -> Result<(), io::Error> {
    let text = fs::read_to_string(common::fixture("sample.md"))?;
    assert_eq!(generate_toc(&text, 6), common::SAMPLE_TOC);
    assert_eq!(generate_toc(&text, 2), common::SAMPLE_TOC_MAX_2);
    Ok(())
}

#[test]
fn re_export_and_full_path_name_the_same_type() {
    let heading: modules_testing::Heading = markdown::parse_heading("## Install").unwrap();
    let headings: Vec<markdown::Heading> = vec![heading];
    assert_eq!(toc::render(&headings, 6), "- [Install](#install)\n");
}

#[test]
fn text_without_headings_gives_an_empty_toc() {
    assert_eq!(generate_toc("Just a paragraph.\n", 6), "");
}
