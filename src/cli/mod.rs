//! Módulo CLI - Comandos y argumentos

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "semcode-search")]
#[command(version = "3.0.0")]
#[command(about = "Fast semantic code search with TF-IDF, caching, and advanced filtering")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug, Clone)]
pub enum Commands {
    /// Inicializar configuración
    Init {
        #[arg(short, long)]
        force: bool,
    },

    /// Gestionar alias para búsquedas
    #[command(subcommand)]
    Alias(AliasAction),

    /// Mostrar estadísticas del proyecto indexado
    Stats,

    /// Gestionar historial de búsquedas
    #[command(subcommand)]
    History(HistoryAction),

    /// Indexar archivos y guardar caché
    Index {
        #[arg(short, long, default_value = ".")]
        path: String,

        #[arg(
            short,
            long,
            default_value = ".git,target,node_modules,dist,build,.fastembed_cache,.cache"
        )]
        ignore: String,

        #[arg(short, long)]
        force: bool,

        #[arg(short = 'e', long)]
        ext: Option<String>,

        #[arg(long)]
        ignore_pattern: Option<String>,

        #[arg(long)]
        ai: bool,
    },

    /// Observar cambios y reindexar automáticamente
    Watch {
        #[arg(short, long, default_value = ".")]
        path: String,

        #[arg(
            short,
            long,
            default_value = ".git,target,node_modules,dist,build,.fastembed_cache,.cache"
        )]
        ignore: String,

        #[arg(short, long, default_value_t = 3)]
        interval: u64,
    },

    /// Iniciar servidor HTTP con API REST (requiere --features server)
    Serve {
        /// Puerto donde escuchar (por defecto: 8080)
        #[arg(short, long, default_value_t = 8080)]
        port: u16,

        /// Host donde escuchar (por defecto: 127.0.0.1)
        #[arg(long, default_value = "127.0.0.1")]
        host: String,
    },

    /// Interfaz gráfica en terminal (TUI) (requiere --features tui)
    Tui {
        /// Ruta del proyecto a buscar
        #[arg(short, long, default_value = ".")]
        path: String,
    },

    /// Buscar en archivos
    Search {
        #[arg(short, long)]
        query: Option<String>,

        #[arg(short, long, default_value = ".")]
        path: String,

        #[arg(short = 'e', long)]
        ext: Option<String>,

        #[arg(
            short = 'i',
            long,
            default_value = ".git,target,node_modules,dist,build"
        )]
        ignore: String,

        #[arg(long)]
        exact: bool,

        #[arg(long)]
        ignore_case: bool,

        #[arg(short, long)]
        verbose: bool,

        #[arg(long)]
        no_cache: bool,

        #[arg(long)]
        update: bool,

        #[arg(long)]
        semantic: bool,

        #[arg(long)]
        ai: bool,

        #[arg(short = 'f', long)]
        file: Option<String>,

        #[arg(long, default_value_t = true)]
        summary: bool,

        #[arg(long)]
        max_size: Option<String>,

        #[arg(long)]
        ignore_pattern: Option<String>,

        #[arg(long)]
        extract: bool,

        #[arg(long)]
        interactive: bool,

        #[arg(long)]
        alias: Option<String>,
    },
}

#[derive(Subcommand, Debug, Clone)]
pub enum AliasAction {
    Save {
        name: String,
        query: String,
        params: Vec<String>,
    },
    List,
    Remove {
        name: String,
    },
    Run {
        name: String,
    },
}

#[derive(Subcommand, Debug, Clone)]
pub enum HistoryAction {
    List {
        #[arg(short, long, default_value_t = 20)]
        limit: usize,
    },
    Clear,
    Last,
}
