# Lesson 06 — Collections and iteration

Count word frequencies and rank the top N, first with loops, then with iterators: implement every `todo!()` in `src/lib.rs` until the tests pass.

## Run

Inside the container:

```bash
cd /workspace/lessons/06-collections
cargo test
cargo fmt --check
cargo clippy --all-targets -- -D warnings
```

Until a function is implemented, its parameters show `unused variable` warnings. Don't run `cargo fix`.

## Notes

### One PHP array, several Rust types

| PHP | Rust | Iteration order |
|---|---|---|
| list `[1, 2, 3]` | `Vec<T>` | index order |
| `['ann' => 31]` | `HashMap<K, V>` | **unspecified** |
| `$seen[$x] = true` used as a set | `HashSet<T>` | **unspecified** |
| array kept `ksort`ed | `BTreeMap<K, V>` / `BTreeSet<T>` | sorted by key |
| array in insertion order | `IndexMap` (crate `indexmap`) | insertion order |

`Vec` is in the prelude; the others are imported from `std::collections`.

```rust
use std::collections::HashMap;

let mut words = vec!["a".to_string()];   // vec! ≈ array literal; one element type
words.push("b".to_string());             // $words[] = 'b'
let second = &words[1];                  // panics if out of bounds; words.get(1) returns Option<&String>

let mut ages: HashMap<String, u32> = HashMap::new();
ages.insert("ann".to_string(), 31);
ages.get("ann");                         // Option<&u32>: no "undefined array key" warning, no null
```

- **The big gotcha:** PHP arrays remember insertion order. `HashMap`/`HashSet` iteration order is unspecified: each new map gets random hash keys (protection against HashDoS), so the order can differ between runs and between two maps with the same content. Never let output or tests depend on it. Sort the entries, or pick `BTreeMap` (sorted) or `IndexMap` (insertion order, like PHP).
- The tests compare whole maps and sets with `assert_eq!`. `HashMap`/`HashSet` equality ignores order. Every function that returns a `Vec` defines its order, and the tests check that order exactly.
- A `HashMap<String, _>` or `HashSet<String>` can be queried with a `&str`: `ages.get("ann")`, `set.contains(word.as_str())`. No `String` allocation needed for a lookup.

### Entry API

```rust
*counts.entry(word).or_insert(0) += 1;   // $counts[$word] = ($counts[$word] ?? 0) + 1;
```

- `entry(key)` looks the key up once. `or_insert(0)` inserts if missing and returns `&mut usize` pointing into the map; `*` writes through it.
- `or_default()` inserts `Default::default()` (`0`, empty `Vec`, …). `or_insert_with(|| …)` is lazy.
- `entry` takes the key **by value**, even when it already exists. Fine when you own the word anyway, as here. If you only hold a `&str`, `entry(word.to_string())` allocates on every call; `get_mut` first avoids that for existing keys.

### Three ways to iterate, in ownership terms

| Call | `for` sugar | `Vec<T>` yields | `HashMap<K, V>` yields | Collection afterwards |
|---|---|---|---|---|
| `.iter()` | `for x in &v` | `&T` | `(&K, &V)` | unchanged |
| `.iter_mut()` | `for x in &mut v` | `&mut T` | `(&K, &mut V)` | changed in place |
| `.into_iter()` | `for x in v` | `T` | `(K, V)` | **moved**, unusable |

- `iter` ≈ `foreach ($a as $x)`. `iter_mut` ≈ `foreach ($a as &$x)`, minus the classic PHP bug: the borrow ends with the loop, so there is no leftover reference to overwrite the last element later.
- `into_iter` has no PHP equivalent. The loop owns each element, so you can move it elsewhere (a `String` into another map) without cloning. The source variable is gone; using it is a compile error (`borrow of moved value`).
- `for x in v` on a `Vec` is `into_iter`. Write `for x in &v` when you need `v` afterwards.
- `into_iter` on a reference borrows: `for (w, c) in freqs` with `freqs: &HashMap` yields `(&String, &usize)` and leaves the map intact. Only an owned collection is consumed.
- Map keys are never mutable while iterating (it would break hashing). `values_mut()` yields only `&mut V`; `retain(|k, v| …)` keeps only the entries it returns `true` for, in place.

### Closures

```rust
let min = 3;
let long_enough = |word: &str| word.chars().count() >= min; // borrows `min`
long_enough("rust");                                         // true

let mut total = 0;
let mut count_it = |_word: &str| total += 1;                 // mutably borrows `total`
```

| PHP | Rust |
|---|---|
| `fn($w) => strlen($w) >= $min` (captures a copy) | `\|w\| w.len() >= min` (borrows by default) |
| `function () use (&$total) { $total++; }` | `\|\| total += 1` (`&mut total`, inferred) |
| `function () use ($list) { … }` (copy) | `move \|\| …` (takes ownership) |

- Rust infers per variable whether a closure needs `&`, `&mut` or ownership. `move` forces ownership; you need it when the closure must outlive the variables it captures (returned closures, threads, async: lessons 11 and 16). Unlike PHP's copy, `move` takes the value away: a captured non-`Copy` variable such as a `String` or `Vec` is unusable after the closure is created. Clone it first if you still need it.
- Accepting a closure: `fn f(keep: impl Fn(&str) -> bool)` ≈ a `callable` parameter, but the signature is checked at compile time. `Fn` only reads its captures, `FnMut` may mutate them, `FnOnce` may consume them. Lesson 09 covers `impl Trait`.

### Iterator adapters

```rust
let lengths: Vec<usize> = words
    .iter()                       // yields &String
    .filter(|w| !w.is_empty())    // filter's closure gets &Item, here &&String
    .map(|w| w.chars().count())
    .collect();                   // the annotated type picks the collection
```

- **Lazy.** `map`/`filter` do nothing until a consumer runs the chain: `collect`, `count`, `sum`, `fold`, a `for` loop. PHP's `array_map`/`array_filter` build a whole array per step; a Rust chain is one pass with no intermediate collections.
- `collect` builds whatever the target type asks for: `Vec`, `HashSet`, `HashMap` (from `(K, V)` pairs), `String`. Annotate the variable (`let s: HashSet<_> = …`) or use the turbofish `.collect::<Vec<_>>()`.
- `filter_map(f)` maps and drops the `None`s. A function whose signature fits can be passed by name: `.filter_map(normalize_word)`.
- Also useful: `take(n)`, `enumerate()`, `rev()`, `any`/`all`, `fold(init, |acc, x| …)`.
- Closure parameters are patterns: `.map(|(word, count)| …)` destructures a pair. `filter` adds one `&`, so on `(&String, &usize)` items write `|&(word, &count)|` or `|(word, count)| **count > 1`. Edition 2024 rejects `|(word, &count)|` there: "cannot explicitly dereference within an implicitly-borrowing pattern".
- A `for` loop with `entry` is idiomatic for counting. Use whichever form reads better; Part B is practice in the iterator form.

### Sorting

| Method | Orders by | Stable |
|---|---|---|
| `v.sort()` | the element's `Ord` | yes |
| `v.sort_by(\|a, b\| …)` | a closure returning `Ordering` | yes |
| `v.sort_by_key(\|x\| key)` | the `Ord` of an extracted key | yes |
| `v.sort_unstable()`, `_by`, `_by_key` | same three | no; usually faster, no allocation |

```rust
use std::cmp::Reverse;

pairs.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0))); // count desc, then word asc
pairs.sort_by_key(|&(_, count)| Reverse(count));                 // count desc only
```

- `a.cmp(&b)` is PHP's `$a <=> $b`, returning `Ordering::Less`/`Equal`/`Greater` instead of `-1`/`0`/`1`. `then_with` replaces `?:` chaining in a `usort` callback. Swapping `a` and `b` reverses an order.
- **Stable** means equal elements keep their original relative order (PHP's sort is stable since 8.0). It only matters when the comparison has ties. Count-then-word over unique words has none, so `sort_unstable_by` gives the same result there.
- Sorting is in place on a slice or `Vec` and returns `()`. A `HashMap` can't be sorted: collect its entries into a `Vec` first.
- `sort_by_key` can't return a borrow of the element: `v.sort_by_key(|p| &p.0)` fails with "lifetime may not live long enough". Use `sort_by`, or clone the key.
- `String` order compares bytes: `"Zoo" < "apple"`, `"zoo" < "été"`, `"10" < "9"`. PHP's `sort()` compares numeric strings as numbers; Rust never does.
- `f64` isn't `Ord` (because of `NaN`): `v.sort_by(|a, b| a.total_cmp(b))`.

## Exercise

Word rules (from `normalize_word`): split with `split_whitespace`, trim ASCII punctuation from both ends of each token, lowercase, skip tokens that end up empty. Ranking: count descending, then word ascending.

Part A — loops:

1. `normalize_word`: `trim_matches` with a closure, then `to_lowercase`. `trim_matches(char::is_ascii_punctuation)` doesn't compile: that method takes `&self`, and the pattern must be `FnMut(char) -> bool`.
2. `word_frequencies`: `for token in text.split_whitespace()`, `let Some(word) = … else { continue };`, then the entry API.
3. `unique_words` and `first_repeated_word`: `HashSet::insert` returns `false` if the value was already present. It takes the `String` by value: if you still need the word afterwards, check `contains` first or clone.
4. `top_n`: push borrowed `(&str, usize)` pairs into a `Vec` in a `for` loop over `freqs`, `sort_by` with the tie rule, `truncate(n)`, then turn only those pairs into owned `String`s.

Part B — the same contracts with iterators, no `for`/`while`/`loop`:

5. `word_frequencies_iter`: `split_whitespace().filter_map(normalize_word)`, then `fold` into a `HashMap`.
6. `unique_words_iter`: one chain ending in `collect`.
7. `top_n_iter`: `iter().map(…).collect()` into a `Vec`, sort it, then `into_iter().take(n).map(…).collect()`.

Part C — the other iteration flavours, sorting, closures:

8. `merge_counts`: `for (word, count) in other` consumes `other`; each `word` moves straight into `total.entry(word)`.
9. `mask_words`: `iter_mut()`, then assign through the `&mut String`: `*word = "*".repeat(…)`. Count `chars`, not bytes.
10. `sort_by_length`: `sort_by_key`. Again `chars`, not `len()`.
11. `words_where`: call `keep(word, count)` inside `filter`, then `map` to an owned `String`, `collect`, `sort`.

Optional extension: write `group_by_count(freqs: &HashMap<String, usize>) -> BTreeMap<usize, Vec<String>>`, with each group's words sorted. Use `entry(count).or_default().push(…)`. Test it, and notice that iterating the `BTreeMap` yields counts in ascending order without any sorting step.

## Done when

- `cargo test`, `cargo fmt --check`, and `cargo clippy --all-targets -- -D warnings` pass.
- You can explain:
  - why no test depends on `HashMap` iteration order, and which type gives you PHP's insertion order
  - what `iter`, `iter_mut` and `into_iter` yield, and what is left of the collection after each
  - why `merge_counts` takes `other` by value but `total` by `&mut`
  - why `top_n` can borrow `&str` while sorting and clone only the `n` words it returns
  - why `sort_by_length` must be stable but `top_n` doesn't care
