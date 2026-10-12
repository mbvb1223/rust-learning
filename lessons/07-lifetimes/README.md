# Lesson 07 — Practical lifetimes

Return slices borrowed from the input instead of copies in `src/lib.rs`, then fix five lifetime errors in `src/broken.rs`.

## Run

Inside the container:

```bash
cd /workspace/lessons/07-lifetimes
cargo test
cargo test --features broken
cargo fmt --check
cargo clippy --all-targets --features broken -- -D warnings
```

`src/broken.rs` only compiles with `--features broken`, so plain `cargo test` works while it's still broken. Don't run `cargo fix`.

## Notes

### What a lifetime is

A borrow like `&text[0..5]` points into `text` without copying it. Its **lifetime** is the stretch of code where the borrow is valid: it must end before `text` is dropped.

```rust
let text = String::from("hello world");
let hello = &text[0..5]; // no copy: points into `text`
println!("{hello}");     // hello
```

PHP's `substr()` returns a copy, so it never has this problem.

Lifetimes are checked at compile time only. Annotations never keep data alive; they only say which input a borrow comes from.

### Borrowed return values

A function can return a piece of its input:

```rust
fn first_line(text: &str) -> &str {
    text.lines().next().unwrap_or("")
}

let text = String::from("one\ntwo");
println!("{}", first_line(&text)); // one
```

- `trim`, `trim_matches`, `split_whitespace`, `split_once`, and `&text[a..b]` all return a `&str` pointing into the original string.
- String literals like `""` are `&'static str`: they live for the whole program, so they fit any lifetime.

### Lifetime elision

Every reference has a lifetime, but you only write it when the compiler can't work it out:

1. Each reference parameter gets its own lifetime.
2. If there is exactly one input lifetime, the output gets it.
3. If a parameter is `&self` or `&mut self`, the output gets its lifetime.

```rust
fn longest_word(text: &str) -> Option<&str> // ok: one input (rule 2)
fn longer_of(a: &str, b: &str) -> &str       // error[E0106]: from `a` or from `b`?
```

`&[&str]` holds two references, so it counts as two input lifetimes.

### `'a` links inputs to the output

```rust
fn longer_of<'a>(a: &'a str, b: &'a str) -> &'a str
```

Read it as "the result is valid only while both `a` and `b` are". So the shorter-lived argument decides:

```rust
let outer = String::from("long-lived");
let result;
{
    let inner = String::from("short");
    result = longer_of(&outer, &inner);
}                     // `inner` dropped here
println!("{result}"); // error[E0597]: `inner` does not live long enough
```

At runtime `result` is `outer`, but the compiler checks the signature, not the value.

Tie the output only to the inputs it borrows from:

```rust
fn text_after<'a>(text: &'a str, marker: &str) -> Option<&'a str>
```

Here `marker` can be dropped while the result is still in use. A PHP signature never says which argument a return value comes from; a Rust signature has to.

### Structs that borrow

A struct that holds a reference needs a lifetime parameter:

```rust
pub struct Highlight<'a> {
    text: &'a str,
    start: usize,
    end: usize,
}

impl<'a> Highlight<'a> {
    pub fn text(&self) -> &str { … }       // rule 3: valid while the Highlight is
    pub fn matched(&self) -> &'a str { … } // valid while the original text is
}
```

- A `Highlight<'a>` can't outlive the text it borrows.
- `impl<'a>` declares `'a` once for every method in the block.
- Slicing `self.text` gives a `&'a str`, even through `&self`. Return `&'a str` when callers need the slice after the `Highlight` is gone.

Borrow in short-lived views such as search hits and parsers. Use owned `String` fields in data you store.

### Owned return values

Return `String` when the result is new text that isn't in any input:

```rust
fn shout(word: &str) -> String {
    word.to_uppercase()
}

println!("{}", shout("hi")); // HI
```

- A piece of an input → `&str`.
- New text (lowercased, joined, formatted) → `String`.

Returning a reference to a local fails with E0515: the local is dropped when the function returns. `'static` doesn't fix it; it's for literals, not data built at runtime.

### Error codes you'll meet

- E0106: missing lifetime; the signature doesn't say which input the output borrows from
- E0515: returned a reference to a local variable
- E0597: a value is dropped while a borrow of it is still used
- E0499: two `&mut` borrows at once, often because elision tied a result to `&mut self`

## Exercise

Part A — borrowed results (`src/lib.rs`):

1. `longest_word`: `split_whitespace`, `trim_matches(|c: char| c.is_ascii_punctuation())`, skip empty tokens. Compare `chars().count()` and replace the best only on `>` (`max_by_key` keeps the *last* tie).
2. `longer_of`: one `if`/`else` expression.
3. `text_after`: `split_once(marker)`, then `trim`.
4. `longest_word_across`: `longest_word` on each text, combined with `longer_of` (its tie rule keeps the earlier word).

Part B — owned results:

5. `normalize_word`: trim punctuation, then `to_lowercase()`.

Part C — `Highlight<'a>`:

6. `find`: return `None` for an empty needle first; `str::find` gives the byte `start`, and `end` is `start + needle.len()`.
7. `start`, `text`, `matched`, `before`, `after`: slice `self.text` with byte ranges.
8. `find_all`: `str::match_indices` yields non-overlapping `(offset, &str)` pairs.
9. `render`: `format!` with `before`, `matched`, and `after`.

Part D — `src/broken.rs`:

10. Run `cargo test --features broken` and fix each case without changing its documented behaviour. Case 3's error appears only after case 1 is fixed.
11. Fill in the table:

| Case | Error | Your fix, and why it is correct |
|---|---|---|
| 1 `shorter` | E0106 | |
| 2 `first_word` | E0515 | |
| 3 `shorter_name_len` | E0597 | |
| 4 `parse_setting` | E0515 | |
| 5 `words` | E0499 | |

<details>
<summary>Check your fixes</summary>

1. `shorter`: declare `<'a>` on both inputs and the output, because either may be returned.
2. `first_word`: slice `text` directly; the copy `owned` is dropped at `return`.
3. `shorter_name_len`: declare `full` in the outer scope so it lives as long as `result`.
4. `parse_setting`: the lowercase text is new, so `Setting` must own it: `String` fields, no `<'a>`.
5. `words`: make `next_word` return `Option<&'a str>`, so words borrow the text, not the cursor.

</details>

Optional: change `Highlight::matched` to return `&str`, see which test stops compiling and why, then revert.

## Done when

- `cargo test`, `cargo test --features broken`, `cargo fmt --check`, and `cargo clippy --all-targets --features broken -- -D warnings` pass.
- The Part D table is filled in.
- You can explain:
  - which elision rule gives `longest_word` its output lifetime, and why `longer_of` needs `'a`
  - why `text_after` puts `'a` on `text` but not on `marker`
  - why no lifetime annotation could fix broken cases 2, 3, or 4
