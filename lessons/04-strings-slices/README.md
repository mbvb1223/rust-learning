# Lesson 04 — Strings and slices

Build text and slice helpers that handle empty input and non-ASCII text in `src/lib.rs`.

## Run

Inside the container:

```bash
cd /workspace/lessons/04-strings-slices
cargo test
cargo fmt --check
cargo clippy --all-targets -- -D warnings
```

Don't run `cargo fix`. `byte_slicing_inside_a_char_panics` passes from the start: it shows the panic your truncate functions must avoid.

## Notes

### `String` and `&str`

`String` owns its text and can grow. `&str` is a borrowed view of text that someone else owns.

```rust
let literal: &str = "hi";                // a string literal is a &str
let owned: String = literal.to_string(); // a copy you own; also String::from("hi")
let view: &str = &owned;                 // borrows, no copy
println!("{literal} {owned} {view}");    // hi hi hi
```

PHP has one string type; Rust separates owning text (`String`) from viewing it (`&str`).

Return `&str` when the result is a piece of the input (`first_word`), and `String` when you build new text (`title_case`).

### Take `&str` parameters

A `&str` parameter accepts literals, `String`s, and sub-slices. A `&String` parameter accepts only `&String`.

```rust
fn shout(s: &str) -> String {
    s.to_uppercase()
}

let owned = String::from("hey");
println!("{}", shout("hi"));        // HI
println!("{}", shout(&owned));      // HEY
println!("{}", shout(&owned[1..])); // EY
```

Clippy's `ptr_arg` lint rejects `&String` and `&Vec<T>` parameters.

### Bytes and chars

Strings are UTF-8, so one `char` takes 1 to 4 bytes. `len()` counts bytes like PHP's `strlen`; `chars().count()` counts chars like `mb_strlen`.

```rust
let s = "héllo";
println!("{}", s.len());           // 6
println!("{}", s.chars().count()); // 5
```

`char_indices()` gives each char with its byte offset:

```rust
for (i, c) in "héllo".char_indices() {
    print!("{i}:{c} "); // 0:h 1:é 3:l 4:l 5:o
}
```

### No indexing by position

`s[0]` doesn't compile, because byte 0 isn't always a whole char. Ask for chars instead:

```rust
let s = "héllo";
// let c = s[0];                    // error[E0277]: the type `str` cannot be indexed by `{integer}`
println!("{:?}", s.chars().next()); // Some('h')
println!("{:?}", s.chars().nth(1)); // Some('é')
```

### Slicing by byte range

`&s[a..b]` takes part of a string by **byte** offsets. Cutting inside a char panics; `get` returns `None` instead.

```rust
let s = "héllo";
println!("{}", &s[0..3]);                 // hé
// &s[0..2];                              // panics: byte index 2 is not a char boundary
println!("{:?}", s.get(0..2));            // None
println!("{}", s.is_char_boundary(2));    // false
println!("{}", s.floor_char_boundary(2)); // 1
```

PHP's `substr("héllo", 0, 2)` silently returns half of `é`. Rust never lets a `&str` hold broken UTF-8.

### A char is not always one symbol

What a reader sees as one symbol can be several `char`s:

```rust
println!("{}", "👍🏽".chars().count());     // 2: thumbs up + skin tone
println!("{}", "e\u{301}".chars().count()); // 2: 'e' + accent, shown as "é"
```

This lesson counts `char`s; the tests show where that differs from what you see.

### Upper and lower case

On a `&str`, `to_uppercase` returns a new `String`. On a `char` it returns an iterator, because one char can become several:

```rust
println!("{}", "straße".to_uppercase()); // STRASSE

let mut out = String::new();
out.extend('ß'.to_uppercase());
println!("{out}");                       // SS
```

`to_lowercase` works the same way.

### Common `&str` methods

```rust
let s = " key = a=b ";
println!("[{}]", s.trim());          // [key = a=b]
println!("{:?}", s.split_once('=')); // Some((" key ", " a=b "))
println!("{:?}", s.find('='));       // Some(5)
println!("{}", s.contains("a="));    // true
println!("{}", s.starts_with(' '));  // true
```

`trim` and `split_once` return views into `s`, not copies. `find` returns a byte offset, like `strpos`.

`split_whitespace` yields the words, skipping any run of whitespace:

```rust
for word in "  hello   big world ".split_whitespace() {
    print!("[{word}]"); // [hello][big][world]
}
```

### Building a `String`

Start with `String::new()`, then append a `char` with `push` or a `&str` with `push_str`. `format!` works like `sprintf`.

```rust
let mut out = String::new();
out.push('h');
out.push_str("ello");
let line = format!("{out}, {}!", "world");
println!("{line}"); // hello, world!
```

### Slices

`&[T]` is to `Vec<T>` what `&str` is to `String`: a borrowed view of some elements.

```rust
let v = vec![3, 1, 4, 1, 5];
let s: &[i32] = &v[1..4];
println!("{s:?}");           // [1, 4, 1]
println!("{:?}", s.first()); // Some(1)
println!("{:?}", s.get(10)); // None
// s[10];                    // panics: index out of bounds
```

A `&[i32]` parameter accepts `&vec`, `&array`, and `&vec[1..3]`; `&Vec<i32>` accepts only the first.

### Searching a slice

`position` finds the first index where a condition holds; `rposition` searches from the end. `windows(n)` yields each run of `n` neighbours.

```rust
let v = [3, 1, 4, 1, 5];
println!("{:?}", v.iter().position(|&x| x == 1));  // Some(1)
println!("{:?}", v.iter().rposition(|&x| x == 1)); // Some(3)

for pair in v.windows(2) {
    print!("{pair:?} "); // [3, 1] [1, 4] [4, 1] [1, 5]
}
```

`|&x| x == 1` is a closure, like PHP's `fn($x) => $x === 1` (lesson 06).

### Slice patterns

`match` can check a slice's shape. `..` skips any number of elements.

```rust
fn ends(values: &[i32]) -> String {
    match values {
        [] => "none".to_string(),
        [only] => format!("just {only}"),
        [first, .., last] => format!("{first} to {last}"),
    }
}

println!("{}", ends(&[4, 8, 15])); // 4 to 15
```

### Returning a borrow

`fn first_word(s: &str) -> &str` returns a view into `s`, so the result can't outlive `s`:

```rust
let word;
{
    let owned = String::from("hello world");
    word = first_word(&owned);
} // owned is dropped here
println!("{word}"); // error[E0597]: `owned` does not live long enough
```

Return a `String` when the result must outlive its input.

## Exercise

A `for` loop works for every step; iterator adapters come in lesson 06.

Part A — read strings without copying:

1. `byte_len`, `char_count`.
2. `first_word`: `trim_start()`, then `find(char::is_whitespace)` gives the byte offset where the word ends. Then compare with `s.split_whitespace().next()`.
3. `truncate_chars`: `s.char_indices().nth(max)` is the byte offset of the first char to drop, or `None` when nothing needs cutting.
4. `truncate_bytes`: clamp `max_bytes` to `s.len()`, then step back while `!s.is_char_boundary(end)`. Then compare with `floor_char_boundary`.
5. `parse_pair`: `s.split_once('=')?` returns `None` early; `trim()` both parts and reject an empty key.

Part B — build new strings:

6. `reverse_chars`: push each char of `s.chars().rev()` into a `String`.
7. `title_case`: for each word of `split_whitespace()`, `let mut chars = word.chars();` then `chars.next()` is the first char and `chars.as_str()` the rest. Single spaces between words only.
8. `is_palindrome`: collect the lowercased alphanumeric chars into a `Vec<char>`, then compare position `i` with `len - 1 - i`.

Part C — slices:

9. `sum`, `max_adjacent_sum`: `i64::from(x)` widens; loop over `values.windows(2)` keeping `let mut best: Option<i64> = None`.
10. `trim_zeros`: `position` and `rposition` find the first and last non-zero, then `&values[start..=end]`. `&[]` is a valid empty slice.
11. `summarize`: one `match` with slice patterns, `format!` in each arm.

Optional: add `ellipsize(s: &str, max: usize) -> String` that keeps at most `max` chars, ending in `…` when it cuts; decide what `max == 0` returns and write the tests first.

## Done when

- `cargo test`, `cargo fmt --check`, and `cargo clippy --all-targets -- -D warnings` pass.
- You can explain:
  - why `first_word` returns `&str` but `title_case` returns `String`
  - why `&s[0..2]` panics on `"héllo"`, and how `truncate_chars` and `truncate_bytes` avoid it
  - why parameters take `&str` and `&[i32]` instead of `&String` and `&Vec<i32>`
