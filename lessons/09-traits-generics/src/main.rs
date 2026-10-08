use std::process::ExitCode;

use traits_generics::{FormatKind, Report, export_as};

fn main() -> ExitCode {
    let kind = match std::env::args().nth(1) {
        None => FormatKind::default(),
        Some(arg) => match arg.parse::<FormatKind>() {
            Ok(kind) => kind,
            Err(error) => {
                eprintln!("error: {error}");
                return ExitCode::FAILURE;
            }
        },
    };

    let report = Report::new("Fruit sales")
        .with_row(("apples", 3))
        .with_row(("bananas, ripe", 12))
        .with_row(("cherries", 120));
    let export = export_as(kind, &report, "fruit-sales");

    println!("{report}");
    println!("--- {}", export.file_name);
    print!("{}", export.contents);
    ExitCode::SUCCESS
}
