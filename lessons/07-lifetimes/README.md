# Lesson 07 — Practical lifetimes

Return slices borrowed from the input instead of copies: implement every `todo!()` in `src/lib.rs` until the tests pass, then fix the compile errors in `src/broken.rs`.

## Run

From the repository root:

```bash
docker compose run --rm -w /workspace/lessons/07-lifetimes rust cargo test
docker compose run --rm -w /workspace/lessons/07-lifetimes rust cargo test --features broken
docker compose run --rm -w /workspace/lessons/07-lifetimes rust cargo fmt --check
docker compose run --rm -w /workspace/lessons/07-lifetimes rust cargo clippy --all-targets --features broken -- -D warnings
```

`src/broken.rs` is compiled only with `--features broken` (`[features] broken = []` in `Cargo.toml`, `#[cfg(feature = "broken")]` in `lib.rs`), so plain `cargo test` works while it is still broken.

Until a function is implemented, its parameters show `unused variable` warnings. Don't run `cargo fix`; it renames them to `_name`.

## Notes

### What a lifetime is

PHP's `substr($text, 0, 5)` copies. PHP strings are refcounted, so nothing you hold can point at freed memory.

Rust's `&text[0..5]` is a pointer and a length into `text`'s buffer: no copy, no refcount. The owner frees the buffer when it goes out of scope (lesson 03), so the compiler must prove that every borrow is gone before that. A **lifetime** is the stretch of code where a borrow is valid.

- Lifetimes exist only at compile time. They never change what a program does, only whether it compiles.
- **Annotations don't keep data alive.** They describe how input and output borrows relate, so the compiler can check each caller. Where PHP's refcount would keep a string alive, Rust rejects the program.
- PHP's `&$var` references alias a variable. They have no lifetimes and are a different concept.

### Borrowed return values

```rust
fn first_line(text: &str) -> &str {
    text.lines().next().unwrap_or("")
}
```

The result points into the caller's string, so the caller can use it only while that string is alive.

Slicing a borrow gives a borrow of the **same data**: `trim`, `trim_matches`, `split_whitespace`, `split_once`, `strip_prefix`, and `&text[a..b]` return `&str` tied to the original string. That's why a function can return them. String literals such as `""` are `&'static str`: they live for the whole program, so they fit any lifetime.

### The three elision rules

Every reference in a signature has a lifetime. You write it only when these rules can't fill it in:

1. Each elided lifetime in the parameters becomes its own lifetime parameter.
2. If there is exactly one input lifetime, it is assigned to every elided output lifetime.
3. If a parameter is `&self` or `&mut self`, its lifetime is assigned to every elided output lifetime.

If an output lifetime is still unknown, the result is E0106.

| You write | The compiler reads | Rules |
|---|---|---|
| `fn longest_word(text: &str) -> Option<&str>` | `fn longest_word<'t>(text: &'t str) -> Option<&'t str>` | 1, 2 |
| `fn text(&self) -> &str` in `impl<'a> Highlight<'a>` | `fn text<'s>(&'s self) -> &'s str` | 1, 3 |
| `fn longer_of(a: &str, b: &str) -> &str` | two input lifetimes, no `self` | E0106 |
| `fn make() -> &str` | no input lifetime at all | E0106 |

`&[&str]` contains two references, so it has two input lifetimes. For `fn make() -> &str`, the compiler suggests `&'static str` or `String`; for data created at runtime, `String` is the answer.

### `'a` describes a relationship

```rust
fn longer_of<'a>(a: &'a str, b: &'a str) -> &'a str
```

Read it as "the result is valid only while both `a` and `b` are". At each call, `'a` is a region where both arguments are borrowed, so in practice it is limited by the shorter-lived one:

```rust
let outer = String::from("long-lived");
let result;
{
    let inner = String::from("short");
    result = longer_of(&outer, &inner);
}                       // `inner` dropped here
println!("{result}");   // E0597: `inner` does not live long enough
```

At runtime `result` is `outer`, but the compiler checks signatures, not values: the signature says the result may be `inner`.

Tie an output only to the inputs it borrows from:

```rust
fn text_after<'a>(text: &'a str, marker: &str) -> Option<&'a str>
```

`marker` gets its own elided lifetime, so callers can pass a temporary and keep the result. A PHP signature never says which argument a return value comes from; a Rust signature has to.

### Structs that borrow

```rust
pub struct Highlight<'a> {
    text: &'a str,
    start: usize,
    end: usize,
}

impl<'a> Highlight<'a> {
    pub fn text(&self) -> &str { … }       // rule 3: borrowed from the Highlight
    pub fn matched(&self) -> &'a str { … } // borrowed from the original text
}
```

- `Highlight<'a>` means "a `Highlight` that borrows something for `'a`". It can't outlive that something.
- `impl<'a> Highlight<'a>` declares `'a` once for every method in the block.
- Rule 3 is a safe default for getters, but it can be too short. Write `&'a str` when callers need the slice after the struct is dropped or while they mutate it.
- `self.text` is a `&'a str`, so slicing it gives a `&'a str`, even though you reached it through `&self`.
- In a return type, write `Highlight<'_>` rather than `Highlight`. `'_` means "elided, but visible". The bare form compiles, but the `mismatched_lifetime_syntaxes` lint warns that it hides a borrow.

Borrow in short-lived views: search hits, parsers, cursors, iterators over a buffer. Own (`String`) in anything stored, returned from a loader, or kept after its input is gone. That covers most domain structs, such as Stage 2's `Task`. Start with owned fields, as you would in PHP, and borrow when a view is the point.

### Owned return values

Return `String` when the result contains text that isn't in any input:

```rust
fn normalize_word(word: &str) -> String // "Hello," → "hello": new bytes
```

| The result is… | Return |
|---|---|
| a piece of an input | `&str` with that input's lifetime |
| new text (lowercased, joined, formatted) | `String` |
| sometimes a piece, sometimes new | `String` for now; `Cow<'a, str>` is listed under Advanced topics |

Returning a reference to a local fails with E0515: the local is dropped when the function returns. `'static` doesn't help, because it is for literals and statics, not for data created at runtime. Converting borrowed to owned is one call: `longest_word(text).map(normalize_word)`.

### Reading the errors

| Code | Message | Meaning |
|---|---|---|
| E0106 | missing lifetime specifier | The signature doesn't say which input an output borrows from |
| E0515 | cannot return value referencing local variable | The result points at something the function owns and drops |
| E0597 | borrowed value does not live long enough | An owner is dropped while a borrow of it is used later |
| E0499 | cannot borrow as mutable more than once at a time | A borrow you hold keeps a `&mut` borrow alive, often because elision tied a result to `&mut self` |

E0106 is a signature error. Borrow-check errors in other functions are reported alongside it, but errors in code that *calls* the broken signature appear only after the signature is fixed. Fix from the top and re-run.

## Exercise

Part A — borrowed results:

1. `longest_word`: `split_whitespace`, then `trim_matches(|c: char| c.is_ascii_punctuation())`, then skip empty tokens. Compare `chars().count()`, keep the best in an `Option<&str>`, and replace it only on `>`. `Iterator::max_by_key` returns the *last* maximum, which breaks the tie rule.
2. `longer_of`: one `if`/`else` expression.
3. `text_after`: `str::split_once`, then `trim`. Because `marker` has no `'a`, the test `text_after_result_does_not_borrow_the_marker` compiles.
4. `longest_word_across`: call `longest_word` on each text and combine with `longer_of`, whose tie rule keeps the earlier word. `'a` is on the items, not on the slice, so the result outlives the `Vec` in `longest_word_across_result_outlives_the_list`.

Part B — owned results:

5. `normalize_word`: trim punctuation, then `to_lowercase()`, which allocates the `String`.

Part C — `Highlight<'a>`:

6. `find`: `str::find` returns a byte offset; `end` is `start + needle.len()`. Handle the empty needle first: `"abc".find("")` is `Some(0)`.
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

| Case | Error | Cause | Fix |
|---|---|---|---|
| 1 `shorter` | E0106 | Two borrowed inputs, a borrowed output, no `self`: no elision rule applies | Declare `<'a>` and use it on both inputs and the output, because either may be returned |
| 2 `first_word` | E0515 | The result borrows `owned`, a copy that is dropped at `return` | Slice `text` directly. Subslices of `text` live as long as `text`; the copy was never needed |
| 3 `shorter_name_len` | E0597 | `'a` must cover both arguments, but `full` is dropped at the inner `}` while `result` is still used | Make `full` live as long as `result` is used: declare it in the outer scope, or finish with `result` inside the block |
| 4 `parse_setting` | E0515 | Lowercasing creates new text owned by the function; the returned `Setting` borrows it | Make `Setting` own its data: `String` fields, `.to_string()`, no `<'a>`. Borrowing from `line` can't work because the lowercase text isn't in `line` |
| 5 `words` | E0499 | Rule 3 ties each word from `next_word(&mut self) -> Option<&str>` to the `&mut` borrow of `cursor`; holding a word keeps `cursor` mutably borrowed | Return `Option<&'a str>`: words borrow the text, not the cursor. The method itself compiled; its signature promised callers too little |

</details>

Optional extensions:

- Change `Highlight::matched` to return `&str`. Read which test stops compiling and why, then revert.
- Implement `Iterator` for the fixed `Cursor<'a>` with `type Item = &'a str` (traits are lesson 09), then reduce `words` to `Cursor::new(text).collect()`.

## Done when

- `cargo test`, `cargo test --features broken`, `cargo fmt --check`, and `cargo clippy --all-targets --features broken -- -D warnings` pass.
- The Part D table is filled in.
- You can explain:
  - which elision rule gives `longest_word` its output lifetime, and why `longer_of` needs `'a`
  - why `text_after` puts `'a` on `text` but not on `marker`
  - why `matched` returns `&'a str` while `text` can rely on elision
  - why `normalize_word` returns `String`, and why `'static` wouldn't fix a `&str` version
  - each fix in `src/broken.rs`, and why no lifetime annotation could fix cases 2, 3, or 4
