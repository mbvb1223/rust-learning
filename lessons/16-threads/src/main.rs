use std::path::PathBuf;

use threads::{count_file, run_pool, summarize};

fn main() {
    // A lazy iterator: run_pool pulls paths only as fast as the workers take them.
    let paths = std::env::args_os().skip(1).map(PathBuf::from);
    let reports = run_pool(paths, 4, 8, count_file);

    for report in &reports {
        let path = report.path.display();
        match report.outcome {
            Ok(counts) => println!(
                "{:>7} lines {:>9} bytes  {path}",
                counts.lines, counts.bytes
            ),
            Err(kind) => println!("error: {kind}  {path}"),
        }
    }

    let total = summarize(&reports);
    println!(
        "{} files, {} failed, {} lines, {} bytes",
        total.files, total.failed, total.lines, total.bytes
    );
}
