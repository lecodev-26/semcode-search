//! Definición de los argumentos y comandos de la CLI.
//!
//! Usa `clap` con derive para generar automáticamente el parser, la ayuda
//! y los mensajes de error.

use clap::{Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

/// semcode-search — Motor de búsqueda semántica de código.
#[derive(Debug, Parser)]
#[command(
    name = "semcode-search",
    version,
    about = "Motor de búsqueda semántica de código (local-first)",
    long_about = None,
    propagate_version = true
)]
pub struct Cli {
    /// Comando a ejecutar.
    #[command(subcommand)]
    pub command: Command,

    /// Modo verbose (más logs).
    #[arg(long, global = true, short = 'v')]
    pub verbose: bool,

    /// Archivo de configuración alternativo.
    #[arg(long, global = true)]
    pub config: Option<PathBuf>,
}

/// Comandos disponibles.
#[derive(Debug, Subcommand)]
pub enum Command {
    /// Inicializa un proyecto (crea `.semcode/`).
    Init {
        /// Ruta del proyecto. Por defecto, el directorio actual.
        #[arg(default_value = ".")]
        path: PathBuf,
    },

    /// Indexa un proyecto.
    Index {
        /// Ruta del proyecto.
        #[arg(long, default_value = ".")]
        path: PathBuf,

        /// Forzar reindexado completo (ignora cache).
        #[arg(long)]
        force: bool,

        /// Generar embeddings con IA (más lento la primera vez).
        #[arg(long)]
        ai: bool,
    },

    /// Busca en el índice.
    Search {
        /// Consulta de búsqueda.
        query: String,

        /// Ruta del proyecto.
        #[arg(long, default_value = ".")]
        path: PathBuf,

        /// Modo de búsqueda.
        #[arg(long, value_enum)]
        mode: Option<SearchModeArg>,

        /// Número máximo de resultados.
        #[arg(long, short = 'n', default_value_t = 10)]
        limit: usize,

        /// Activar reranker (feature `rerank`).
        #[arg(long)]
        deep: bool,

        /// Salida en JSON.
        #[arg(long)]
        json: bool,

        /// Filtrar por lenguaje.
        #[arg(long)]
        lang: Option<String>,

        /// Búsqueda case-sensitive (solo Exact).
        #[arg(long)]
        case_sensitive: bool,
    },

    /// Muestra estadísticas del proyecto indexado.
    Stats {
        /// Ruta del proyecto.
        #[arg(long, default_value = ".")]
        path: PathBuf,

        /// Salida en JSON.
        #[arg(long)]
        json: bool,
    },

    /// Arranca el servidor HTTP (feature `server`).
    Serve {
        /// Ruta del proyecto.
        #[arg(long, default_value = ".")]
        path: PathBuf,

        /// Host de escucha.
        #[arg(long, default_value = "127.0.0.1")]
        host: String,

        /// Puerto.
        #[arg(long, default_value_t = 8080)]
        port: u16,
    },

    /// Arranca el servidor MCP (feature `mcp`).
    Mcp {
        /// Ruta del proyecto.
        #[arg(long, default_value = ".")]
        path: PathBuf,

        /// Transporte HTTP (si no, usa stdio).
        #[arg(long)]
        http: Option<String>,
    },

    /// Modo watch: reindexa al guardar archivos.
    Watch {
        /// Ruta del proyecto.
        #[arg(long, default_value = ".")]
        path: PathBuf,
    },
}

/// Modo de búsqueda (para clap).
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum SearchModeArg {
    /// Búsqueda exacta (literal/regex).
    Exact,
    /// Búsqueda léxica BM25.
    Bm25,
    /// Búsqueda semántica.
    Semantic,
    /// Búsqueda estructural.
    Symbol,
    /// Búsqueda híbrida (default).
    Hybrid,
}

impl SearchModeArg {
    /// Convierte al `SearchMode` del motor.
    pub fn to_search_mode(self) -> semcode_search::SearchMode {
        match self {
            SearchModeArg::Exact => semcode_search::SearchMode::Exact,
            SearchModeArg::Bm25 => semcode_search::SearchMode::Bm25,
            SearchModeArg::Semantic => semcode_search::SearchMode::Semantic,
            SearchModeArg::Symbol => semcode_search::SearchMode::Symbol,
            SearchModeArg::Hybrid => semcode_search::SearchMode::Hybrid,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    #[test]
    fn cli_verify() {
        // Verifica que la definición de clap es válida
        Cli::command().debug_assert();
    }

    #[test]
    fn search_mode_arg_to_search_mode() {
        assert_eq!(
            SearchModeArg::Exact.to_search_mode(),
            semcode_search::SearchMode::Exact
        );
        assert_eq!(
            SearchModeArg::Hybrid.to_search_mode(),
            semcode_search::SearchMode::Hybrid
        );
    }

    #[test]
    fn parse_search_command() {
        let cli = Cli::try_parse_from([
            "semcode-search",
            "search",
            "validate email",
            "--limit",
            "5",
            "--mode",
            "semantic",
        ])
        .unwrap();

        match cli.command {
            Command::Search {
                query, limit, mode, ..
            } => {
                assert_eq!(query, "validate email");
                assert_eq!(limit, 5);
                assert_eq!(mode, Some(SearchModeArg::Semantic));
            }
            _ => panic!("esperaba Search"),
        }
    }

    #[test]
    fn parse_init_default_path() {
        let cli = Cli::try_parse_from(["semcode-search", "init"]).unwrap();
        match cli.command {
            Command::Init { path } => {
                assert_eq!(path, PathBuf::from("."));
            }
            _ => panic!("esperaba Init"),
        }
    }

    #[test]
    fn parse_index_with_flags() {
        let cli = Cli::try_parse_from([
            "semcode-search",
            "index",
            "--path",
            "/tmp/test",
            "--force",
            "--ai",
        ])
        .unwrap();

        match cli.command {
            Command::Index { path, force, ai } => {
                assert_eq!(path, PathBuf::from("/tmp/test"));
                assert!(force);
                assert!(ai);
            }
            _ => panic!("esperaba Index"),
        }
    }

    #[test]
    fn parse_serve_default_port() {
        let cli = Cli::try_parse_from(["semcode-search", "serve"]).unwrap();
        match cli.command {
            Command::Serve { host, port, .. } => {
                assert_eq!(host, "127.0.0.1");
                assert_eq!(port, 8080);
            }
            _ => panic!("esperaba Serve"),
        }
    }
}
