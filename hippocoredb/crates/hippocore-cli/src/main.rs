//! `hippocore` — thin binary wrapper over [`hippocore::cli`].

use std::process::ExitCode;

fn main() -> ExitCode {
    hippocore::cli::run()
}
