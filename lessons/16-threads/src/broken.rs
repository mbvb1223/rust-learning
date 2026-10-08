//! Compiled only with `--features broken`, and it doesn't compile yet: each function has one
//! thread-safety error. Fix them one at a time without changing any signature or test, then
//! make the tests below pass. Read the whole compiler message before changing code.

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;
use std::thread;

/// Sum and maximum of `numbers`, computed on two threads that share the one list without
/// copying it. The maximum is `None` for an empty list.
pub fn sum_and_max(numbers: Vec<u64>) -> (u64, Option<u64>) {
    let shared = Rc::new(numbers);

    let for_sum = Rc::clone(&shared);
    let sum = thread::spawn(move || for_sum.iter().sum::<u64>());

    let for_max = Rc::clone(&shared);
    let max = thread::spawn(move || for_max.iter().copied().max());

    (sum.join().unwrap(), max.join().unwrap())
}

/// Starts `n` threads; thread `i` pushes `i` into one shared log.
/// Returns the log sorted ascending, i.e. `0..n`.
pub fn collect_ids(n: usize) -> Vec<usize> {
    let log = Arc::new(RefCell::new(Vec::new()));

    let mut handles = Vec::new();
    for i in 0..n {
        let log = Arc::clone(&log);
        handles.push(thread::spawn(move || log.borrow_mut().push(i)));
    }
    for handle in handles {
        handle.join().unwrap();
    }

    let mut ids = log.take();
    ids.sort_unstable();
    ids
}

/// Length in bytes of the longest line in `text` (as split by `str::lines`), found by two
/// threads that each scan half of the lines. 0 for empty text.
pub fn longest_line(text: &str) -> usize {
    let lines: Vec<&str> = text.lines().collect();
    let (first, second) = lines.split_at(lines.len() / 2);

    let a = thread::spawn(|| first.iter().map(|line| line.len()).max().unwrap_or(0));
    let b = thread::spawn(|| second.iter().map(|line| line.len()).max().unwrap_or(0));

    a.join().unwrap().max(b.join().unwrap())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sums_and_maxes_on_two_threads() {
        assert_eq!(sum_and_max(vec![3, 9, 4]), (16, Some(9)));
        assert_eq!(sum_and_max(Vec::new()), (0, None));
    }

    #[test]
    fn collects_every_thread_id() {
        assert_eq!(collect_ids(8), (0..8).collect::<Vec<_>>());
        assert!(collect_ids(0).is_empty());
    }

    #[test]
    fn finds_the_longest_line() {
        assert_eq!(longest_line("a\nccc\nbb"), 3);
        assert_eq!(longest_line("one line"), 8);
        assert_eq!(longest_line(""), 0);
    }
}
