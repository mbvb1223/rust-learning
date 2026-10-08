//! Fenced code block detection. Private to `markdown`: declared there with plain `mod fence;`.

/// `true` if `line`, after leading spaces and tabs, starts with three backticks or three
/// tildes. Anything may follow them, such as a language name.
pub(super) fn is_fence(line: &str) -> bool {
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_backtick_and_tilde_fences() {
        for line in ["```", "```rust", "~~~", "~~~~ text", "  ```", "\t~~~"] {
            assert!(is_fence(line), "{line:?}");
        }
    }

    #[test]
    fn rejects_lines_that_are_not_fences() {
        todo!(
            "assert that `is_fence` is false for lines its doc comment excludes, e.g. two backticks only, text before the fence, mixed backticks and tildes, an empty line"
        )
    }
}
