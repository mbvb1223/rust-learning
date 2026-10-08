use std::time::Duration;

use async_basics::{
    TimerReport, create_then_wait, run_blocking, run_concurrent, run_sequential, run_spawned,
    spawn_then_wait, timer,
};
use tokio::time::Instant;

#[tokio::main]
async fn main() {
    let start = Instant::now();
    let (a, b) = run_sequential(("a", 100), ("b", 200)).await;
    print_timeline("run_sequential", start, &[a, b]);

    let start = Instant::now();
    let (a, b) = run_concurrent(("a", 100), ("b", 200)).await;
    print_timeline("run_concurrent (join!)", start, &[a, b]);

    let start = Instant::now();
    let (a, b) = run_spawned(("a", 100), ("b", 200))
        .await
        .expect("timer task panicked");
    print_timeline("run_spawned (tokio::spawn)", start, &[a, b]);

    let start = Instant::now();
    let lazy = create_then_wait("lazy", 100, 50).await;
    print_timeline("create_then_wait, awaited after 50 ms", start, &[lazy]);

    let start = Instant::now();
    let eager = spawn_then_wait("eager", 100, 50)
        .await
        .expect("timer task panicked");
    print_timeline("spawn_then_wait, awaited after 50 ms", start, &[eager]);

    let start = Instant::now();
    let (a, b) = tokio::join!(blocking_timer("a", 100), blocking_timer("b", 200));
    print_timeline("join! with std::thread::sleep (the bug)", start, &[a, b]);

    let start = Instant::now();
    let (job, tick) = tokio::join!(
        run_blocking(|| blocking_job("job", 200)),
        timer("tick", 100)
    );
    let job = job.expect("blocking job panicked");
    print_timeline("run_blocking job + async timer", start, &[job, tick]);
}

// The bug on purpose: async code that never yields. It holds the thread polling this task for
// `ms`, so nothing else in this task runs meanwhile.
async fn blocking_timer(name: &str, ms: u64) -> TimerReport {
    blocking_job(name, ms)
}

fn blocking_job(name: &str, ms: u64) -> TimerReport {
    let started_at = Instant::now();
    std::thread::sleep(Duration::from_millis(ms));
    TimerReport {
        name: name.to_owned(),
        started_at,
        finished_at: Instant::now(),
    }
}

fn print_timeline(title: &str, start: Instant, reports: &[TimerReport]) {
    println!("{title}: {} ms total", start.elapsed().as_millis());
    for report in reports {
        println!(
            "  {:<6} {:>4} ms → {:>4} ms",
            report.name,
            report.started_at.duration_since(start).as_millis(),
            report.finished_at.duration_since(start).as_millis(),
        );
    }
    println!();
}
