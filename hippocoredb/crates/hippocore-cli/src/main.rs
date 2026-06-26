//! `hippocore` — CLI binary. Delegates most subcommands to [`hippocore::cli`];
//! handles `studio` locally since it requires ratatui/crossterm.

mod tui;

use std::path::PathBuf;
use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().collect();
    let first = args.get(1).map(String::as_str).unwrap_or("");

    if first == "studio" {
        let db = parse_db_arg(&args);
        match tui::run(&db) {
            Ok(()) => ExitCode::SUCCESS,
            Err(e) => {
                eprintln!("studio error: {e}");
                ExitCode::FAILURE
            }
        }
    } else {
        hippocore::cli::run()
    }
}

/// Parse `--db <path>` from argv, defaulting to `./hippocore-data`.
fn parse_db_arg(args: &[String]) -> PathBuf {
    let mut it = args.iter().peekable();
    while let Some(arg) = it.next() {
        if arg == "--db" {
            if let Some(p) = it.next() {
                return PathBuf::from(p);
            }
        } else if let Some(val) = arg.strip_prefix("--db=") {
            return PathBuf::from(val);
        }
    }
    PathBuf::from("./hippocore-data")
}
