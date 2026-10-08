# Lesson 04 — Strings and slices

Build text and slice helpers that survive empty input and non-ASCII text: implement every `todo!()` in `src/lib.rs` until the tests pass.

## Run

Inside the container:

```bash
cd /workspace/lessons/04-strings-slices
cargo test
cargo fmt --check
cargo clippy --all-targets -- -D warnings
```

Until a function is implemented, its parameters show `unused variable` warnings. Don't run `cargo fix`.

`byte_slicing_inside_a_char_panics` passes from the start: it demonstrates the panic your truncate functions must avoid.

## Notes

### `String` vs `&str`

| | `String` | `&str` |
|---|---|---|
| Owns its bytes | yes, a heap buffer | no, borrows someone else's |
| Can grow | yes (`push`, `push_str`) | no |
| Made of | pointer + length + capacity | pointer + length |
| Comes from | `String::from`, `.to_string()`, `format!`, `collect()` | literals, `&owned`, `&owned[a..b]`, `trim()`, `split…` |

```rust
let literal: &'static str = "hi";        // stored in the binary, valid for the whole program
let owned: String = literal.to_string(); // heap copy you own; also String::from / .to_owned()
let view: &str = &owned;                 // borrow, no copy; also owned.as_str()
let part: &str = &owned[0..1];           // sub-slice by BYTE range
```

PHP has one string type: a refcounted, copy-on-write byte buffer. Rust separates owning a buffer (`String`) from viewing one (`&str`). `trim`, `split_whitespace`, and `split_once` return views into the original; PHP's `trim()` and `explode()` build new strings.

Rule of thumb: return `&str` when the result is a piece of the input (`first_word`), `String` when you create new text (`title_case`).

### Borrowed parameters: `&str`, not `&String`

```rust
fn shout(s: &str) -> String { s.to_uppercase() }

shout("literal");     // &'static str
shout(&owned);        // &String coerces to &str (deref coercion)
shout(&owned[1..]);   // a sub-slice
```

With `fn shout(s: &String)`, the first and third calls don't compile. Likewise a `&[i32]` parameter accepts `&vec`, `&array`, and `&vec[1..3]`; `&Vec<i32>` accepts only the first. Clippy's `ptr_arg` lint flags `&String` and `&Vec<T>` parameters, so `-D warnings` rejects them.

Take `String` by value only when the function keeps it, for example in a struct field; the caller hands over ownership (lesson 03).

### UTF-8: bytes vs `char`

A `str` is always valid UTF-8. A `char` is one Unicode scalar value: 4 bytes as a standalone value, 1–4 bytes inside a string.

| Text | `s.len()` | `s.chars().count()` | PHP `strlen` / `mb_strlen` |
|---|---|---|---|
| `"hello"` | 5 | 5 | 5 / 5 |
| `"héllo"` | 6 | 5 | 6 / 5 |
| `"日本語"` | 9 | 3 | 9 / 3 |
| `"🦀"` | 4 | 1 | 4 / 1 |

- `len()` counts **bytes** in O(1). `chars().count()` decodes the whole string, O(n), like `mb_strlen`.
- `s[0]` doesn't compile (`the type str cannot be indexed by {integer}`): byte 0 isn't necessarily a character. Use `s.chars().next()`, `s.chars().nth(i)` (O(n)), or `s.as_bytes()[i]` when you want a byte.
- `s.chars()` yields `char`s, `s.bytes()` yields `u8`s, and `s.char_indices()` yields `(byte_offset, char)`, the offsets you may slice at.

PHP strings are byte arrays with no encoding attached: `substr("héllo", 0, 2)` returns `"h\xC3"`, half of `é`, silently. Rust never lets a `&str` hold invalid UTF-8:

```rust
let s = String::from("héllo");
&s[0..2];                 // panics: end byte index 2 is not a char boundary; it is inside 'é' (bytes 1..3 of string)
s.get(0..2);              // None, the non-panicking version
s.is_char_boundary(2);    // false
s.floor_char_boundary(2); // 1, the nearest boundary at or below 2
```

Slice ranges are always **byte** offsets. Take them from `char_indices()`, `find()`, `len()`, or a boundary check, never from a character count.

### What a reader sees: grapheme clusters

A `char` is not "one symbol on screen":

```rust
"👍🏽".chars().count();      // 2: thumbs up + skin-tone modifier, 8 bytes
"e\u{301}".chars().count(); // 2: 'e' + combining acute accent, renders as "é"
```

Counting or reversing user-perceived characters needs grapheme segmentation: the `unicode-segmentation` crate (PHP: `grapheme_strlen` from `intl`). It's out of scope; this lesson works with `char`s, and the tests show where that differs from what you see.

### Case mapping

```rust
'ß'.to_uppercase();          // an iterator yielding 'S', 'S'
"straße".to_uppercase();     // "STRASSE": a new String, possibly longer
"ÉTÉ".to_ascii_lowercase();  // "ÉtÉ": ASCII letters only
```

`char::to_uppercase` returns an iterator because one char can map to several. Append it with `out.extend(c.to_uppercase())`. PHP: `mb_strtoupper` vs `strtoupper`, which is ASCII-only since PHP 8.2.

### Common `&str` methods

| PHP | Rust | Returns |
|---|---|---|
| `trim($s)` | `s.trim()` | `&str` view |
| `explode(',', $s)` | `s.split(',')` | iterator of `&str` |
| `preg_split('/\s+/', trim($s))` | `s.split_whitespace()` | iterator of `&str` |
| `explode('=', $s, 2)` | `s.split_once('=')` | `Option<(&str, &str)>` |
| `strpos($s, 'x')` | `s.find('x')` | `Option<usize>`, a byte offset |
| `str_contains`, `str_starts_with` | `contains`, `starts_with` | `bool` |
| `implode(' ', $parts)` | `parts.join(" ")` | `String` |
| `$a . $b` | `format!("{a}{b}")` | `String` |
| `$a .= $b` | `a.push_str(b)` | `()`: grows `a` in place |

Build text in a `String::new()` with `push(char)` and `push_str(&str)`, or `collect::<String>()` from an iterator of `char`s. `a + &b` also works, but it moves `a` (lesson 03).

### Slices: `&[T]`

`&[T]` is to `Vec<T>` what `&str` is to `String`: a pointer + length view into elements owned by a `Vec`, an array, or another slice.

```rust
let v = vec![3, 1, 4, 1, 5];
let s: &[i32] = &v[1..4];   // [1, 4, 1], no copy
s.first();                  // Some(&1)
s.get(10);                  // None
s[10];                      // panics: index out of bounds
v.windows(2);               // [3, 1], [1, 4], [4, 1], [1, 5]
v.chunks(2);                // [3, 1], [4, 1], [5]
v.iter().position(|&x| x == 1); // Some(1); rposition searches from the end
```

`|&x| x == 1` is a closure, like PHP's `fn($x) => $x === 1`. `iter()` yields `&i32`, and the `&x` pattern copies the `i32` out. Lesson 06 covers closures.

- PHP's `$arr[10]` on a missing key gives `null` plus a warning. `s[10]` panics; use `get` when the index may be missing.
- PHP arrays are ordered hash maps. `Vec<T>` and `[T]` are contiguous lists indexed by `usize` only. `HashMap` comes in lesson 06.

Slice patterns match on shape:

```rust
match values {
    [] => …,
    [only] => …,
    [first, second] => …,
    [first, .., last] => …, // `..` skips any number of elements; `rest @ ..` binds them
}
```

### Returning a borrow

`fn first_word(s: &str) -> &str` ties the result's lifetime to `s`: callers must treat it as borrowed from `s`, so it can't outlive `s`. The compiler infers that link (lifetime elision, lesson 07) and enforces it at every call site:

```rust
let word;
{
    let owned = String::from("hello world");
    word = first_word(&owned);
} // `owned` is dropped here
println!("{word}"); // error[E0597]: `owned` does not live long enough
```

PHP's refcount would keep the string alive. Rust rejects the program instead; return a `String` when the result must outlive its input.

## Exercise

Implement in this order. Iterator adapters (`map`, `filter`, `collect`) are covered in lesson 06; a `for` loop works for everything below.

Part A — read strings without copying:

1. `byte_len`, `char_count`.
2. `first_word`: `trim_start()`, then `find(char::is_whitespace)` gives the byte offset where the word ends. Then compare with `s.split_whitespace().next()`.
3. `truncate_chars`: `s.char_indices().nth(max)` is the byte offset of the first char to drop, or `None` when nothing needs cutting.
4. `truncate_bytes`: clamp `max_bytes` to `s.len()`, then step back while `!s.is_char_boundary(end)`. Afterwards, compare with `floor_char_boundary`.
5. `parse_pair`: `s.split_once('=')?` returns `None` early when there is no `=` (`?` on `Option`, lesson 02). `trim()` both parts and reject an empty key.

Part B — build new strings:

6. `reverse_chars`: push each char of `s.chars().rev()` into a `String`.
7. `title_case`: loop over `split_whitespace()`. For each word, `let mut chars = word.chars();` then `chars.next()` is the first char and `chars.as_str()` is the rest. Put a space between words, not after the last.
8. `is_palindrome`: collect the lowercased alphanumeric chars into a `Vec<char>`, then compare position `i` with `len - 1 - i`.

Part C — slices:

9. `sum`, `max_adjacent_sum`: `i64::from(x)` widens losslessly. Each item of `values.windows(2)` is a two-element `&[i32]`; keep the best total so far in a `let mut best: Option<i64> = None`.
10. `trim_zeros`: find the first non-zero index with `position` and the last with `rposition`, then return `&values[start..=end]`. `&[]` is a valid empty slice.
11. `summarize`: a single `match` with slice patterns, `format!` in each arm.

Optional extension: add `ellipsize(s: &str, max: usize) -> String` that returns `s` unchanged if it has at most `max` chars, otherwise the first `max - 1` chars followed by `…` (one char). Decide what `max == 0` returns and write the tests first. It allocates even when nothing is cut; `Cow<str>` (ROADMAP advanced topics) avoids that.

## Done when

- `cargo test`, `cargo fmt --check`, and `cargo clippy --all-targets -- -D warnings` pass.
- You can explain:
  - why `first_word` returns `&str` but `title_case` returns `String`
  - why `&s[0..2]` panics on `"héllo"`, and how `truncate_chars` and `truncate_bytes` avoid it
  - why parameters take `&str` and `&[i32]` instead of `&String` and `&Vec<i32>`
  - why `"👍🏽"` is 8 bytes and 2 chars but shows as one symbol
