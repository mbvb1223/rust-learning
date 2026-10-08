//! Compile errors on purpose: `cargo test --features broken` fails until you fix them.
//! Fix each function without changing its signature or the value it returns.

use std::rc::Rc;
use std::time::Duration;

/// Spawns a task that shares a list through two reference-counted handles, sleeps 10 ms, and
/// returns the list's length: `3`.
pub async fn shared_len_after_sleep() -> usize {
    let handle = tokio::spawn(async {
        let list = Rc::new(vec![1, 2, 3]);
        let shared = Rc::clone(&list);
        tokio::time::sleep(Duration::from_millis(10)).await;
        shared.len()
    });
    handle.await.expect("task panicked")
}

/// Spawns a task that greets a name created in this function: `"hello, Ferris"`.
pub async fn greeting() -> String {
    let name = String::from("Ferris");
    let handle = tokio::spawn(async { format!("hello, {name}") });
    handle.await.expect("task panicked")
}

/// Spawns one task per name, each returning that name's length in chars, and returns the sum.
/// No names: `0`.
pub async fn total_chars(names: &[String]) -> usize {
    let mut handles = Vec::new();
    for name in names {
        handles.push(tokio::spawn(async { name.chars().count() }));
    }
    let mut total = 0;
    for handle in handles {
        total += handle.await.expect("task panicked");
    }
    total
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test(start_paused = true)]
    async fn shared_len_after_sleep_returns_three() {
        assert_eq!(shared_len_after_sleep().await, 3);
    }

    #[tokio::test]
    async fn greeting_greets_ferris() {
        assert_eq!(greeting().await, "hello, Ferris");
    }

    #[tokio::test]
    async fn total_chars_counts_chars_not_bytes() {
        let names = vec![String::from("Ana"), String::from("Zoë")];
        assert_eq!(total_chars(&names).await, 6);
        assert_eq!(total_chars(&[]).await, 0);
    }
}
