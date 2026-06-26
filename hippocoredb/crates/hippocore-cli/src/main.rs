//! `hippocore` — CLI binary. Delegates most subcommands to [`hippocore::cli`];
//! handles `studio` and `serve` locally since they require extra crates.

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
    } else if first == "serve" {
        run_serve(&args)
    } else {
        hippocore::cli::run()
    }
}

fn run_serve(args: &[String]) -> ExitCode {
    let data_dir = parse_db_arg(args);
    let port: u16 = parse_flag_u16(args, "--port").unwrap_or(8080);
    let api_key = parse_flag_str(args, "--api-key")
        .or_else(|| std::env::var("HIPPOCORE_API_KEY").ok())
        .unwrap_or_default();

    if api_key.is_empty() {
        eprintln!("error: API key required. Use --api-key <key> or set HIPPOCORE_API_KEY.");
        return ExitCode::FAILURE;
    }

    let cfg = hippocore_server::ServerConfig {
        data_dir,
        port,
        api_key,
    };

    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();

    eprintln!("hippocore serve: listening on 0.0.0.0:{port}");
    rt.block_on(async move {
        if let Err(e) = hippocore_server::serve(cfg).await {
            eprintln!("server error: {e}");
            std::process::exit(1);
        }
    });
    ExitCode::SUCCESS
}

/// Parse `--db <path>` from argv, defaulting to `./hippocore-data`.
fn parse_db_arg(args: &[String]) -> PathBuf {
    if let Some(s) = parse_flag_str(args, "--db") {
        return PathBuf::from(s);
    }
    PathBuf::from("./hippocore-data")
}

fn parse_flag_str(args: &[String], flag: &str) -> Option<String> {
    let prefix = format!("{flag}=");
    let mut it = args.iter().peekable();
    while let Some(arg) = it.next() {
        if arg == flag {
            return it.next().cloned();
        } else if let Some(val) = arg.strip_prefix(&prefix) {
            return Some(val.to_owned());
        }
    }
    None
}

fn parse_flag_u16(args: &[String], flag: &str) -> Option<u16> {
    parse_flag_str(args, flag)?.parse().ok()
}
