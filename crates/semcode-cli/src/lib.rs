//! # semcode-cli
//!
//! Interfaz de línea de comandos de semcode-search.
//!
//! ## Comandos
//!
//! - `init` — Inicializa un proyecto.
//! - `index` — Indexa un proyecto.
//! - `search` — Busca en el índice.
//! - `stats` — Muestra estadísticas.
//! - `serve` — Servidor HTTP (FASE 9).
//! - `mcp` — Servidor MCP (FASE 8).
//! - `watch` — Reindexado automático (FASE 10).
//!
//! ## Ejemplo
//!
//! ```bash
//! semcode-search init
//! semcode-search index --path .
//! semcode-search search "validate email"
//! semcode-search stats
//! ```

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod cli;
pub mod commands;

use clap::Parser;
use cli::{Cli, Command};
use semcode_core::error::{Error, Result};
use semcode_core::types::Language;

use tracing_subscriber::EnvFilter;

/// Punto de entrada de la CLI.
///
/// Parsea los argumentos y ejecuta el comando correspondiente.
pub fn run() -> Result<()> {
    let cli = Cli::parse();

    // Inicializar tracing (logs)
    init_tracing(cli.verbose);

    // Despachar
    match cli.command {
        Command::Init { path } => commands::init::run(&path),

        Command::Index { path, force, ai } => commands::index::run(&path, force, ai),

        Command::Search {
            query,
            path,
            mode,
            limit,
            deep,
            json,
            lang,
            case_sensitive,
        } => {
            let search_mode = mode.map(|m| m.to_search_mode());
            let language = lang.and_then(|s| Language::from_extension(&s));
            let opts = commands::search::SearchOpts {
                path: &path,
                query: &query,
                mode: search_mode,
                limit,
                deep,
                json,
                lang: language,
                case_sensitive,
            };
            commands::search::run(opts)
        }

        Command::Stats { path, json } => commands::stats::run(&path, json),

        Command::Serve { .. } => Err(Error::Other(
            "el comando `serve` estará disponible en la FASE 9".to_string(),
        )),

        Command::Mcp { .. } => Err(Error::Other(
            "el comando `mcp` estará disponible en la FASE 8".to_string(),
        )),

        Command::Watch { .. } => Err(Error::Other(
            "el comando `watch` estará disponible en la FASE 10".to_string(),
        )),
    }
}

/// Inicializa el sistema de logging.
fn init_tracing(verbose: bool) {
    let level = if verbose { "debug" } else { "info" };
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(format!("semcode={},semcode_search={}", level, level)));

    let _ = tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(false)
        .try_init();
}
