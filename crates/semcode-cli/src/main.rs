//! Punto de entrada de la CLI de semcode-search.
//!
//! Este binario delega en `semcode_cli::run()`.

use std::process::ExitCode;

fn main() -> ExitCode {
    match semcode_cli::run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("error: {}", e);
            ExitCode::FAILURE
        }
    }
}
