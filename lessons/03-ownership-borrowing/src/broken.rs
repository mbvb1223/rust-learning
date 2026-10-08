//! Part A — every function below has exactly one borrow-checker error.
//!
//! See all of them with `cargo check --features broken`. Fix them one at a time and keep each
//! doc-comment contract. A signature may change when that is the right fix; the tests may not.

/// Returns the numbers sorted ascending: `[3, 1, 2]` → `[1, 2, 3]`.
pub fn sorted(numbers: Vec<i32>) -> Vec<i32> {
    numbers.sort();
    numbers
}

/// Uppercases every name and also returns how many names there were:
/// `["ada", "bob"]` → `(["ADA", "BOB"], 2)`.
pub fn shout_all(names: Vec<String>) -> (Vec<String>, usize) {
    let mut shouted = Vec::new();
    for name in names {
        shouted.push(name.to_uppercase());
    }
    (shouted, names.len())
}

pub struct Customer {
    pub name: String,
    pub email: String,
}

/// The customer's name. The caller keeps `customer` and can still use it afterwards.
pub fn customer_name(customer: &Customer) -> String {
    customer.name
}

/// Trims surrounding whitespace and lowercases: `"  Ada LOVELACE "` → `"ada lovelace"`.
pub fn normalize(name: &str) -> &str {
    let normalized = name.trim().to_lowercase();
    &normalized
}

/// Appends `entry` to the end of `log` and returns the number of characters (not bytes) in
/// `entry`: `"héllo"` → 5.
pub fn record(log: &mut Vec<String>, entry: String) -> usize {
    let text = entry.as_str();
    log.push(entry);
    text.chars().count()
}

/// Appends a doubled copy of every number after the originals:
/// `[1, 2, 3]` → `[1, 2, 3, 2, 4, 6]`. An empty vector stays empty.
pub fn append_doubles(numbers: &mut Vec<i32>) {
    for n in numbers.iter() {
        numbers.push(n * 2);
    }
}

/// Moves `amount` from `balances[from]` to `balances[to]`.
/// `from == to` leaves every balance unchanged. Panics if either index is out of bounds.
pub fn transfer(balances: &mut [i64], from: usize, to: usize, amount: i64) {
    let source = &mut balances[from];
    let target = &mut balances[to];
    *source -= amount;
    *target += amount;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sorted_sorts_ascending() {
        assert_eq!(sorted(vec![3, 1, 2]), [1, 2, 3]);
        assert_eq!(sorted(vec![5, -1, 5, 0]), [-1, 0, 5, 5]);
        assert!(sorted(Vec::new()).is_empty());
    }

    #[test]
    fn shout_all_uppercases_and_counts() {
        let names = vec!["ada".to_string(), "bob".to_string()];
        assert_eq!(
            shout_all(names),
            (vec!["ADA".to_string(), "BOB".to_string()], 2)
        );
        assert_eq!(shout_all(Vec::new()), (Vec::new(), 0));
    }

    #[test]
    fn customer_name_leaves_the_customer_usable() {
        let customer = Customer {
            name: "Ada".to_string(),
            email: "ada@example.com".to_string(),
        };
        assert_eq!(customer_name(&customer), "Ada");
        assert_eq!(customer.name, "Ada");
        assert_eq!(customer.email, "ada@example.com");
    }

    #[test]
    fn normalize_trims_and_lowercases() {
        assert_eq!(normalize("  Ada LOVELACE "), "ada lovelace");
        assert_eq!(normalize("bob"), "bob");
        assert_eq!(normalize("   "), "");
    }

    #[test]
    fn record_appends_and_counts_chars() {
        let mut log = vec!["start".to_string()];
        assert_eq!(record(&mut log, "héllo".to_string()), 5);
        assert_eq!(record(&mut log, String::new()), 0);
        assert_eq!(log, ["start", "héllo", ""]);
    }

    #[test]
    fn append_doubles_appends_after_originals() {
        let mut numbers = vec![1, 2, 3];
        append_doubles(&mut numbers);
        assert_eq!(numbers, [1, 2, 3, 2, 4, 6]);

        let mut empty: Vec<i32> = Vec::new();
        append_doubles(&mut empty);
        assert!(empty.is_empty());
    }

    #[test]
    fn transfer_moves_an_amount() {
        let mut balances = [100, 50, 0];
        transfer(&mut balances, 0, 2, 30);
        assert_eq!(balances, [70, 50, 30]);
        transfer(&mut balances, 2, 1, 5);
        assert_eq!(balances, [70, 55, 25]);
        transfer(&mut balances, 1, 1, 10);
        assert_eq!(balances, [70, 55, 25]);
    }

    #[test]
    #[should_panic]
    fn transfer_panics_on_a_bad_index() {
        let mut balances = [100, 50];
        transfer(&mut balances, 0, 2, 30);
    }
}
