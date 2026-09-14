//! Módulo CLI - Comandos y argumentos

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "semcode-search")]
#[command(version = "2.0.0")]
#[command(about = "🔍 Fast semantic code search with TF-IDF, caching, and advanced filtering")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug, Clone)]
pub enum Commands {
    /// Inicializar configuración
    Init {
        /// Forzar sobreescritura
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
        /// Ruta a indexar
        #[arg(short, long, default_value = ".")]
        path: String,

        /// Carpetas a ignorar (comma-separated)
        #[arg(short, long, default_value = ".git,target,node_modules,dist,build")]
        ignore: String,

        /// Forzar re-indexado
        #[arg(short, long)]
        force: bool,

        /// Filtrar por extensiones (comma-separated)
        #[arg(short = 'e', long)]
        ext: Option<String>,

        /// Ignorar archivos por patrón glob
        #[arg(long)]
        ignore_pattern: Option<String>,
    },

    /// Observar cambios y reindexar automáticamente
    Watch {
        /// Ruta a observar
        #[arg(short, long, default_value = ".")]
        path: String,

        /// Carpetas a ignorar (comma-separated)
        #[arg(short, long, default_value = ".git,target,node_modules,dist,build")]
        ignore: String,

        /// Intervalo en segundos entre comprobaciones
        #[arg(short, long, default_value_t = 3)]
        interval: u64,
    },

    /// Buscar en archivos
    Search {
        /// Término de búsqueda (usa !! para repetir la última)
        #[arg(short, long)]
        query: Option<String>,

        /// Ruta a buscar
        #[arg(short, long, default_value = ".")]
        path: String,

        /// Filtrar por extensiones (comma-separated)
        #[arg(short = 'e', long)]
        ext: Option<String>,

        /// Carpetas a ignorar (comma-separated)
        #[arg(
            short = 'i',
            long,
            default_value = ".git,target,node_modules,dist,build"
        )]
        ignore: String,

        /// Búsqueda exacta (palabra completa)
        #[arg(long)]
        exact: bool,

        /// Ignorar mayúsculas/minúsculas
        #[arg(long)]
        ignore_case: bool,

        /// Modo verboso (muestra progreso)
        #[arg(short, long)]
        verbose: bool,

        /// Ignorar caché
        #[arg(long)]
        no_cache: bool,

        /// Actualizar caché antes de buscar
        #[arg(long)]
        update: bool,

        /// Búsqueda semántica (TF-IDF)
        #[arg(long)]
        semantic: bool,

        /// Buscar por nombre de archivo
        #[arg(short = 'f', long)]
        file: Option<String>,

        /// Mostrar resumen al final
        #[arg(long, default_value_t = true)]
        summary: bool,

        /// Tamaño máximo de archivo (ej: 1MB, 500KB)
        #[arg(long)]
        max_size: Option<String>,

        /// Ignorar archivos por patrón glob
        #[arg(long)]
        ignore_pattern: Option<String>,

        /// Buscar en archivos comprimidos (experimental)
        #[arg(long)]
        extract: bool,

        /// Modo interactivo para navegar resultados
        #[arg(long)]
        interactive: bool,

        /// Usar un alias guardado
        #[arg(long)]
        alias: Option<String>,
    },
}

#[derive(Subcommand, Debug, Clone)]
pub enum AliasAction {
    /// Guardar una búsqueda como alias
    Save {
        /// Nombre del alias
        name: String,

        /// Query de búsqueda
        query: String,

        /// Parámetros adicionales (ej: --ext rs --path src)
        params: Vec<String>,
    },

    /// Listar todos los alias guardados
    List,

    /// Eliminar un alias
    Remove {
        /// Nombre del alias
        name: String,
    },

    /// Ejecutar un alias
    Run {
        /// Nombre del alias
        name: String,
    },
}

#[derive(Subcommand, Debug, Clone)]
pub enum HistoryAction {
    /// Listar las últimas búsquedas
    List {
        /// Número de búsquedas a mostrar (por defecto: 20)
        #[arg(short, long, default_value_t = 20)]
        limit: usize,
    },

    /// Limpiar el historial
    Clear,

    /// Repetir la última búsqueda
    Last,
}
