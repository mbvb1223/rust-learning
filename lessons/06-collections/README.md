# Lesson 06 — Collections and iteration

Count word frequencies and rank the top N, first with loops, then with iterators: implement every `todo!()` in `src/lib.rs`.

## Run

Inside the container:

```bash
cd /workspace/lessons/06-collections
cargo test
cargo fmt --check
cargo clippy --all-targets -- -D warnings
```

Don't run `cargo fix`.

## Notes

### Vec

A `Vec<T>` is a growable list where every element has the same type, like a PHP list:

```rust
let mut words = vec!["a".to_string()];
words.push("b".to_string());    // $words[] = 'b'
println!("{}", words[1]);       // b
println!("{:?}", words.get(5)); // None
```

`words[5]` would panic; `get` returns an `Option` instead. `v.truncate(n)` keeps only the first `n` items.

### HashMap

A `HashMap<K, V>` maps keys to values, like a PHP array with string keys:

```rust
use std::collections::HashMap;

let mut ages: HashMap<String, u32> = HashMap::new();
ages.insert("ann".to_string(), 31);
println!("{:?}", ages.get("ann")); // Some(31)
println!("{:?}", ages.get("bob")); // None
```

A map with `String` keys can be looked up with a plain `&str`.

### HashSet

A `HashSet<T>` holds unique values, like `$seen[$x] = true` in PHP. `insert` returns `false` if the value was already there:

```rust
use std::collections::HashSet;

let mut seen = HashSet::new();
println!("{}", seen.insert("go")); // true
println!("{}", seen.insert("go")); // false
println!("{}", seen.len());        // 1
```

### Iteration order

PHP arrays keep insertion order. `HashMap` and `HashSet` don't: their order can change from one run to the next.

- Never let output or tests depend on `HashMap` order; sort the entries first.
- `BTreeMap` / `BTreeSet` iterate sorted by key.
- `IndexMap` (crate `indexmap`) keeps insertion order, like PHP.

Comparing two maps with `==` ignores order, so the tests can `assert_eq!` whole maps.

### Entry API

`entry(key)` looks a key up once, then lets you insert or update it:

```rust
let mut counts: HashMap<String, usize> = HashMap::new();
for word in ["a", "b", "a"] {
    *counts.entry(word.to_string()).or_insert(0) += 1;
}
println!("{}", counts["a"]); // 2
```

PHP: `$counts[$word] = ($counts[$word] ?? 0) + 1;`. `or_insert` returns a `&mut` to the value; `*` writes through it. `or_default()` inserts `0`, an empty `Vec`, and so on.

### iter, iter_mut, into_iter

- `v.iter()` or `for x in &v`: yields `&T`; `v` is unchanged.
- `v.iter_mut()` or `for x in &mut v`: yields `&mut T`; change items in place.
- `v.into_iter()` or `for x in v`: yields `T`; `v` is moved and unusable afterwards.

```rust
let mut v = vec![1, 2, 3];
for x in v.iter_mut() {
    *x *= 10;
}
println!("{v:?}"); // [10, 20, 30]
```

On a `HashMap` they yield `(&K, &V)`, `(&K, &mut V)`, and `(K, V)`. With `into_iter` you own each key, so you can move it into another map without cloning.

### Closures

A closure is an inline function that can use the variables around it, like PHP's `fn($x) => …`:

```rust
let min = 3;
let long_enough = |word: &str| word.chars().count() >= min;
println!("{}", long_enough("rust")); // true
println!("{}", long_enough("go"));   // false
```

To accept a closure as a parameter, use `impl Fn(…) -> …`, a typed `callable`:

```rust
fn count_where(words: &[&str], keep: impl Fn(&str) -> bool) -> usize {
    words.iter().filter(|w| keep(w)).count()
}

println!("{}", count_where(&["a", "bb", "ccc"], |w| w.len() > 1)); // 2
```

### Iterator adapters

Adapters such as `filter` and `map` build a chain; a consumer such as `collect` runs it in one pass:

```rust
let words = ["Rust", "", "go"];
let lengths: Vec<usize> = words
    .iter()
    .filter(|w| !w.is_empty())
    .map(|w| w.len())
    .collect();
println!("{lengths:?}"); // [4, 2]
```

- Nothing runs until a consumer: `collect`, `count`, `sum`, `fold`, or a `for` loop.
- `collect` builds the type you annotate: `Vec`, `HashSet`, `HashMap` (from pairs), `String`.
- `filter_map(f)` maps and drops the `None`s. A function can be passed by name: `.filter_map(normalize_word)`.
- `take(n)` keeps the first `n` items.

### fold

`fold(start, |acc, item| …)` carries one value through the whole iterator, like PHP's `array_reduce`:

```rust
let total = [1, 2, 3].iter().fold(0, |sum, x| sum + x);
println!("{total}"); // 6
```

The accumulator can be any value, even a `HashMap`: take it as `mut map`, update it, and return it.

### Patterns in closure parameters

`filter` passes each item by reference. Map items are `(&K, &V)` pairs, so destructure them with `&(…)`:

```rust
let freqs = HashMap::from([("a".to_string(), 1), ("b".to_string(), 3)]);
let common: Vec<&String> = freqs
    .iter()
    .filter(|&(_, &count)| count > 1)
    .map(|(word, _)| word)
    .collect();
println!("{common:?}"); // ["b"]
```

### Sorting

`sort()` sorts in place by natural order. `sort_by` takes a compare closure, `sort_by_key` a key:

```rust
let mut pairs = vec![("b", 2), ("a", 2), ("c", 5)];
pairs.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0))); // count desc, then word asc
println!("{pairs:?}"); // [("c", 5), ("a", 2), ("b", 2)]

let mut words = vec!["ccc", "a", "bb"];
words.sort_by_key(|w| w.len());
println!("{words:?}"); // ["a", "bb", "ccc"]
```

- `a.cmp(&b)` is PHP's `$a <=> $b`. Swap `a` and `b` to reverse the order.
- `sort`, `sort_by`, and `sort_by_key` are **stable**: equal items keep their original order. `sort_unstable*` may reorder them.
- A `HashMap` can't be sorted; collect its entries into a `Vec` first.
- Strings compare by bytes: `"Zoo" < "apple" < "zoo" < "été"`.

## Exercise

Word rules (from `normalize_word`): split with `split_whitespace`, trim ASCII punctuation from both ends of each token, lowercase, skip tokens that end up empty. Ranking: count descending, then word ascending.

Part A — loops:

1. `normalize_word`: `trim_matches` with a closure `|c: char| …` (passing `char::is_ascii_punctuation` by name doesn't compile), then `to_lowercase`.
2. `word_frequencies`: `for token in text.split_whitespace()`, `let Some(word) = … else { continue };`, then the entry API.
3. `unique_words` and `first_repeated_word`: `HashSet::insert` returns `false` for a repeat; it takes the `String` by value.
4. `top_n`: push borrowed `(&str, usize)` pairs into a `Vec`, `sort_by` with the ranking rule, `truncate(n)`, then turn only those into owned `String`s.

Part B — the same contracts with iterators, no `for`/`while`/`loop`:

5. `word_frequencies_iter`: `split_whitespace().filter_map(normalize_word)`, then `fold` into a `HashMap`.
6. `unique_words_iter`: one chain ending in `collect`.
7. `top_n_iter`: `iter().map(…).collect()` into a `Vec`, sort it, then `into_iter().take(n).map(…).collect()`.

Part C — the other iteration flavours, sorting, closures:

8. `merge_counts`: `for (word, count) in other` consumes `other`; move each `word` into `total.entry(word)`.
9. `mask_words`: `iter_mut()`, then `*word = "*".repeat(…)`; count `chars`, not bytes.
10. `sort_by_length`: `sort_by_key` on `chars().count()`, not `len()`.
11. `words_where`: call `keep(word, count)` inside `filter`, `map` to an owned `String`, `collect`, `sort`.

Optional: write `group_by_count(freqs) -> BTreeMap<usize, Vec<String>>` with `entry(count).or_default().push(…)`, and notice the counts come out sorted.

## Done when

- `cargo test`, `cargo fmt --check`, and `cargo clippy --all-targets -- -D warnings` pass.
- You can explain:
  - why no test depends on `HashMap` iteration order, and which type keeps PHP's insertion order
  - what `iter`, `iter_mut`, and `into_iter` yield, and what is left of the collection after each
  - why `sort_by_length` must be stable but `top_n` doesn't care
