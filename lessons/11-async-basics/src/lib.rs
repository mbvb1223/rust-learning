//! Lesson 11 — enough async to start. Replace every `todo!()` until `cargo test` passes,
//! then fix `src/broken.rs` until `cargo test --features broken` passes.

use tokio::task::JoinError;
use tokio::time::Instant;

#[cfg(feature = "broken")]
pub mod broken;

/// What one timer did. The instants come from `tokio::time::Instant`, so tests with a paused
/// clock see virtual time.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TimerReport {
    pub name: String,
    /// When the timer's body began running (its first poll), not when its future was created.
    pub started_at: Instant,
    pub finished_at: Instant,
}

/// Reads the clock, sleeps `ms` milliseconds with `tokio::time::sleep`, reads the clock again,
/// and returns both readings under an owned copy of `name`.
/// `ms == 0` finishes without the clock moving.
pub async fn timer(name: &str, ms: u64) -> TimerReport {
    todo!()
}

/// Awaits `timer(first)` to completion, then `timer(second)`.
/// `second` starts when `first` finishes, so the total time is the sum of both.
/// Returns the reports in argument order.
pub async fn run_sequential(first: (&str, u64), second: (&str, u64)) -> (TimerReport, TimerReport) {
    todo!()
}

/// Runs both timers concurrently inside the current task with `tokio::join!`.
/// Both start at once, so the total time is the longer timer.
/// Returns the reports in argument order, whichever finishes first.
pub async fn run_concurrent(first: (&str, u64), second: (&str, u64)) -> (TimerReport, TimerReport) {
    todo!()
}

/// Creates the `timer(name, ms)` future, sleeps `wait_ms`, and only then awaits the timer.
/// The timer starts when it is awaited, `wait_ms` after the call began;
/// the total time is `wait_ms + ms`.
pub async fn create_then_wait(name: &str, ms: u64, wait_ms: u64) -> TimerReport {
    todo!()
}

/// Spawns each timer as its own task with `tokio::spawn`, then awaits both `JoinHandle`s.
/// Both start at once, so the total time is the longer timer.
/// Returns the reports in argument order. `Err` if either task panicked.
///
/// The names are borrowed, but a spawned task must be `'static`: give each task an owned copy.
pub async fn run_spawned(
    first: (&str, u64),
    second: (&str, u64),
) -> Result<(TimerReport, TimerReport), JoinError> {
    todo!()
}

/// Spawns `timer(name, ms)` with `tokio::spawn`, sleeps `wait_ms`, then awaits the `JoinHandle`.
/// The task starts without being awaited, so the timer starts at once and finishes after `ms`.
/// Returns after the longer of `ms` and `wait_ms`. `Err` if the task panicked.
pub async fn spawn_then_wait(name: &str, ms: u64, wait_ms: u64) -> Result<TimerReport, JoinError> {
    todo!()
}

/// Spawns one task per `(name, ms)` pair, all before awaiting any of them, then awaits every
/// handle. Returns the reports in input order, not completion order; the total time is the
/// longest timer. An empty input returns an empty `Vec` at once. `Err` if any task panicked.
pub async fn run_many(timers: Vec<(String, u64)>) -> Result<Vec<TimerReport>, JoinError> {
    todo!()
}

/// Runs `timer(name, ms)` under a `limit_ms` deadline with `tokio::time::timeout`.
/// `ms < limit_ms`: `Some(report)`, returned when the timer finishes.
/// `ms > limit_ms`: `None`, returned at the deadline; the timer future is dropped mid-sleep, which
/// cancels it. `ms == limit_ms` is a race between two equal deadlines; don't rely on its outcome.
pub async fn with_timeout(name: &str, ms: u64, limit_ms: u64) -> Option<TimerReport> {
    todo!()
}

/// Runs blocking `work` on Tokio's blocking thread pool with `tokio::task::spawn_blocking`, so it
/// never runs on the caller's thread, and returns its result. `Err` if `work` panicked.
pub async fn run_blocking<F, T>(work: F) -> Result<T, JoinError>
where
    F: FnOnce() -> T + Send + 'static,
    T: Send + 'static,
{
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    // Every `start_paused` test runs on a virtual clock: when all tasks are waiting, Tokio jumps
    // straight to the next timer. Elapsed times are exact and the tests finish instantly.

    fn ms(millis: u64) -> Duration {
        Duration::from_millis(millis)
    }

    /// `(started, finished)` in milliseconds after `t0`.
    fn span(t0: Instant, report: &TimerReport) -> (u128, u128) {
        (
            report.started_at.duration_since(t0).as_millis(),
            report.finished_at.duration_since(t0).as_millis(),
        )
    }

    #[tokio::test(start_paused = true)]
    async fn timer_sleeps_for_the_given_time() {
        let t0 = Instant::now();
        let report = timer("a", 100).await;
        assert_eq!(report.name, "a");
        assert_eq!(span(t0, &report), (0, 100));
        assert_eq!(t0.elapsed(), ms(100));
    }

    #[tokio::test(start_paused = true)]
    async fn zero_ms_timer_finishes_at_once() {
        let t0 = Instant::now();
        let report = timer("now", 0).await;
        assert_eq!(span(t0, &report), (0, 0));
        assert_eq!(t0.elapsed(), ms(0));
    }

    #[tokio::test(start_paused = true)]
    async fn a_future_does_nothing_until_awaited() {
        let t0 = Instant::now();
        let future = timer("lazy", 100);
        tokio::time::sleep(ms(50)).await;
        let report = future.await;
        assert_eq!(span(t0, &report), (50, 150));
    }

    #[tokio::test(start_paused = true)]
    async fn sequential_runs_one_after_the_other() {
        let t0 = Instant::now();
        let (a, b) = run_sequential(("a", 100), ("b", 200)).await;
        assert_eq!((a.name.as_str(), b.name.as_str()), ("a", "b"));
        assert_eq!(span(t0, &a), (0, 100));
        assert_eq!(span(t0, &b), (100, 300));
        assert_eq!(t0.elapsed(), ms(300));
    }

    #[tokio::test(start_paused = true)]
    async fn sequential_starts_the_second_after_a_longer_first() {
        let t0 = Instant::now();
        let (a, b) = run_sequential(("a", 200), ("b", 100)).await;
        assert_eq!(span(t0, &a), (0, 200));
        assert_eq!(span(t0, &b), (200, 300));
    }

    #[tokio::test(start_paused = true)]
    async fn concurrent_starts_both_at_once() {
        let t0 = Instant::now();
        let (a, b) = run_concurrent(("a", 100), ("b", 200)).await;
        assert_eq!(span(t0, &a), (0, 100));
        assert_eq!(span(t0, &b), (0, 200));
        assert_eq!(t0.elapsed(), ms(200));
    }

    #[tokio::test(start_paused = true)]
    async fn concurrent_keeps_argument_order_not_finish_order() {
        let t0 = Instant::now();
        let (a, b) = run_concurrent(("a", 200), ("b", 100)).await;
        assert_eq!((a.name.as_str(), b.name.as_str()), ("a", "b"));
        assert_eq!(span(t0, &a), (0, 200));
        assert_eq!(span(t0, &b), (0, 100));
    }

    #[tokio::test(start_paused = true)]
    async fn create_then_wait_starts_the_timer_late() {
        let t0 = Instant::now();
        let report = create_then_wait("lazy", 100, 50).await;
        assert_eq!(report.name, "lazy");
        assert_eq!(span(t0, &report), (50, 150));
        assert_eq!(t0.elapsed(), ms(150));
    }

    #[tokio::test(start_paused = true)]
    async fn spawned_tasks_run_concurrently() {
        let t0 = Instant::now();
        let (a, b) = run_spawned(("a", 100), ("b", 200)).await.unwrap();
        assert_eq!(span(t0, &a), (0, 100));
        assert_eq!(span(t0, &b), (0, 200));
        assert_eq!(t0.elapsed(), ms(200));
    }

    #[tokio::test(start_paused = true)]
    async fn spawned_accepts_borrowed_names_and_keeps_argument_order() {
        let first = String::from("first");
        let second = String::from("second");
        let t0 = Instant::now();
        let (a, b) = run_spawned((&first, 200), (&second, 100)).await.unwrap();
        assert_eq!((a.name.as_str(), b.name.as_str()), ("first", "second"));
        assert_eq!(span(t0, &a), (0, 200));
        assert_eq!(span(t0, &b), (0, 100));
    }

    #[tokio::test(start_paused = true)]
    async fn spawn_then_wait_starts_the_timer_at_once() {
        let t0 = Instant::now();
        let report = spawn_then_wait("eager", 100, 50).await.unwrap();
        assert_eq!(report.name, "eager");
        assert_eq!(span(t0, &report), (0, 100));
        assert_eq!(t0.elapsed(), ms(100));
    }

    #[tokio::test(start_paused = true)]
    async fn spawn_then_wait_returns_after_a_longer_wait() {
        let t0 = Instant::now();
        let report = spawn_then_wait("eager", 100, 300).await.unwrap();
        assert_eq!(span(t0, &report), (0, 100));
        assert_eq!(t0.elapsed(), ms(300));
    }

    #[tokio::test(start_paused = true)]
    async fn run_many_starts_every_task_at_once() {
        let timers = vec![
            (String::from("a"), 300),
            (String::from("b"), 100),
            (String::from("c"), 200),
        ];
        let t0 = Instant::now();
        let reports = run_many(timers).await.unwrap();
        let names: Vec<&str> = reports.iter().map(|r| r.name.as_str()).collect();
        assert_eq!(names, ["a", "b", "c"]);
        let spans: Vec<_> = reports.iter().map(|r| span(t0, r)).collect();
        assert_eq!(spans, [(0, 300), (0, 100), (0, 200)]);
        assert_eq!(t0.elapsed(), ms(300));
    }

    #[tokio::test(start_paused = true)]
    async fn run_many_with_no_timers_returns_at_once() {
        let t0 = Instant::now();
        assert!(run_many(Vec::new()).await.unwrap().is_empty());
        assert_eq!(t0.elapsed(), ms(0));
    }

    #[tokio::test(start_paused = true)]
    async fn with_timeout_returns_a_report_in_time() {
        let t0 = Instant::now();
        let report = with_timeout("fast", 100, 200).await.unwrap();
        assert_eq!(report.name, "fast");
        assert_eq!(span(t0, &report), (0, 100));
        assert_eq!(t0.elapsed(), ms(100));
    }

    #[tokio::test(start_paused = true)]
    async fn with_timeout_gives_up_at_the_deadline() {
        let t0 = Instant::now();
        assert_eq!(with_timeout("slow", 300, 100).await, None);
        assert_eq!(t0.elapsed(), ms(100));
    }

    #[tokio::test]
    async fn run_blocking_returns_the_result_of_work() {
        let text = String::from("owned by the closure");
        let len = run_blocking(move || text.len()).await;
        assert_eq!(len.unwrap(), 20);
    }

    #[tokio::test]
    async fn run_blocking_runs_work_on_another_thread() {
        let caller = std::thread::current().id();
        let worker = run_blocking(|| std::thread::current().id()).await.unwrap();
        assert_ne!(caller, worker);
    }

    #[tokio::test]
    async fn run_blocking_reports_a_panic_as_an_error() {
        let result = run_blocking(|| -> u32 { panic!("expected panic from a blocking job") }).await;
        assert!(result.unwrap_err().is_panic());
    }
}
