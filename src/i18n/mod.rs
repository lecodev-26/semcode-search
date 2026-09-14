//! Módulo i18n - Sistema de internacionalización
//!
//! Detecta automáticamente el idioma del sistema y proporciona
//! los textos traducidos.

use std::env;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Language {
    Spanish,
    English,
}

impl Language {
    /// Detecta el idioma del sistema operativo
    pub fn detect() -> Self {
        // Intentar detectar desde la variable de entorno LANG (Linux/macOS)
        if let Ok(lang) = env::var("LANG") {
            let lang = lang.to_lowercase();
            if lang.starts_with("es") {
                return Language::Spanish;
            }
            if lang.starts_with("en") {
                return Language::English;
            }
        }

        // Intentar detectar desde la variable de entorno LC_ALL
        if let Ok(lang) = env::var("LC_ALL") {
            let lang = lang.to_lowercase();
            if lang.starts_with("es") {
                return Language::Spanish;
            }
            if lang.starts_with("en") {
                return Language::English;
            }
        }

        // En Windows, comprobar el idioma del sistema
        #[cfg(target_os = "windows")]
        {
            if let Ok(lang) = env::var("USERPROFILE") {
                let _ = lang;
            }
            // Por defecto, si no se detecta, español (por el target español del proyecto)
            return Language::Spanish;
        }

        // Por defecto, inglés
        #[cfg(not(target_os = "windows"))]
        {
            Language::English
        }
    }

    /// Devuelve los textos del idioma actual
    pub fn t(&self) -> Translations {
        match self {
            Language::Spanish => Translations::spanish(),
            Language::English => Translations::english(),
        }
    }
}

/// Todos los textos traducidos
pub struct Translations {
    // Mensajes generales
    pub welcome_title: &'static str,
    pub welcome_message: &'static str,
    pub press_enter: &'static str,
    pub more_info: &'static str,

    // Comandos
    pub cmd_init: &'static str,
    pub cmd_index: &'static str,
    pub cmd_search: &'static str,
    pub cmd_stats: &'static str,
    pub cmd_history: &'static str,
    pub cmd_alias: &'static str,
    pub cmd_watch: &'static str,

    // Index
    pub indexing: &'static str,
    pub indexed_files: &'static str,
    pub total_size: &'static str,
    pub cache_saved: &'static str,
    pub cache_cleared: &'static str,
    pub ignore_pattern: &'static str,
    pub ext_filter: &'static str,

    // Búsqueda
    pub search_text: &'static str,
    pub search_semantic: &'static str,
    pub search_multithread: &'static str,
    pub query_label: &'static str,
    pub checking_files: &'static str,
    pub progress: &'static str,
    pub no_matches: &'static str,

    // Resumen
    pub summary: &'static str,
    pub files_found: &'static str,
    pub total_matches: &'static str,
    pub time: &'static str,
    pub cores_used: &'static str,

    // Stats
    pub stats_title: &'static str,
    pub files_indexed: &'static str,
    pub total_lines: &'static str,
    pub languages: &'static str,
    pub last_indexed: &'static str,

    // Historial
    pub history_title: &'static str,
    pub history_empty: &'static str,
    pub history_cleared: &'static str,
    pub history_total: &'static str,
    pub last_search: &'static str,

    // Alias
    pub alias_saved: &'static str,
    pub alias_list: &'static str,
    pub alias_empty: &'static str,
    pub alias_removed: &'static str,
    pub alias_not_found: &'static str,
    pub alias_running: &'static str,

    // Errores
    pub error_cache_not_found: &'static str,
    pub error_cache_corrupted: &'static str,
    pub error_no_query: &'static str,
    pub error_config_exists: &'static str,

    // Config
    pub config_created: &'static str,
    pub config_hint: &'static str,

    // Watch
    pub watch_starting: &'static str,
    pub watch_interval: &'static str,
    pub watch_initial_index: &'static str,
    pub watch_observing: &'static str,
    pub watch_changes: &'static str,
    pub watch_reindexing: &'static str,
    pub watch_done: &'static str,
}

impl Translations {
    pub fn spanish() -> Self {
        Self {
            // General
            welcome_title: "🔍 semcode-search",
            welcome_message: "¡Bienvenido! Esta es una herramienta de línea de comandos.",
            press_enter: "Pulsa ENTER para salir...",
            more_info: "Más información:",

            // Comandos
            cmd_init: "Inicializar configuración",
            cmd_index: "Indexar un proyecto",
            cmd_search: "Buscar en archivos",
            cmd_stats: "Ver estadísticas",
            cmd_history: "Ver historial de búsquedas",
            cmd_alias: "Gestionar alias",
            cmd_watch: "Observar cambios y reindexar",

            // Index
            indexing: "📁 Indexando",
            indexed_files: "✅ Indexados",
            total_size: "💾 Tamaño total",
            cache_saved: "💾 Caché guardada en",
            cache_cleared: "🗑️ Caché eliminada",
            ignore_pattern: "🚫 Ignorando patrón",
            ext_filter: "📋 Filtro por extensiones",

            // Búsqueda
            search_text: "🔍 Búsqueda por TEXTO",
            search_semantic: "🧠 Búsqueda SEMÁNTICA (TF-IDF)",
            search_multithread: "🔍 Búsqueda por TEXTO (multi-hilo ⚡)",
            query_label: "Query",
            checking_files: "📄 Revisando",
            progress: "Progreso",
            no_matches: "⚠️ No se encontraron coincidencias",

            // Resumen
            summary: "📊 Resumen",
            files_found: "Archivos encontrados",
            total_matches: "Coincidencias totales",
            time: "Tiempo",
            cores_used: "Núcleos usados",

            // Stats
            stats_title: "📊 Estadísticas de semcode-search",
            files_indexed: "📁 Archivos indexados",
            total_lines: "📝 Total de líneas",
            languages: "🔤 Lenguajes",
            last_indexed: "🕐 Última indexación",

            // Historial
            history_title: "🕐 Historial de búsquedas",
            history_empty: "📭 No hay búsquedas en el historial",
            history_cleared: "🗑️ Historial limpiado",
            history_total: "📊 Total",
            last_search: "🕐 Última búsqueda",

            // Alias
            alias_saved: "✅ Alias guardado",
            alias_list: "📋 Alias guardados",
            alias_empty: "📭 No hay alias guardados",
            alias_removed: "🗑️ Alias eliminado",
            alias_not_found: "⚠️ Alias no encontrado",
            alias_running: "🚀 Ejecutando alias",

            // Errores
            error_cache_not_found: "⚠️ No se encontró caché. Ejecute 'index' primero",
            error_cache_corrupted: "⚠️ Caché corrupta. Ejecute 'index' primero",
            error_no_query: "⚠️ Debes proporcionar una query con --query o un alias con --alias",
            error_config_exists: "⚠️ Configuración ya existe. Usa --force para sobreescribir",

            // Config
            config_created: "✅ Configuración creada en",
            config_hint: "💡 Puedes editarla manualmente o usar 'alias' para gestionar búsquedas",

            // Watch
            watch_starting: "👁️ Observando cambios en",
            watch_interval: "⏱️ Intervalo",
            watch_initial_index: "📁 Indexando por primera vez...",
            watch_observing: "🔄 Observando",
            watch_changes: "⚠️ cambios detectados",
            watch_reindexing: "📁 Reindexando...",
            watch_done: "✅ Reindexado completo",
        }
    }

    pub fn english() -> Self {
        Self {
            // General
            welcome_title: "🔍 semcode-search",
            welcome_message: "Welcome! This is a command-line tool.",
            press_enter: "Press ENTER to exit...",
            more_info: "More information:",

            // Commands
            cmd_init: "Initialize configuration",
            cmd_index: "Index a project",
            cmd_search: "Search in files",
            cmd_stats: "Show statistics",
            cmd_history: "Show search history",
            cmd_alias: "Manage aliases",
            cmd_watch: "Watch changes and reindex",

            // Index
            indexing: "📁 Indexing",
            indexed_files: "✅ Indexed",
            total_size: "💾 Total size",
            cache_saved: "💾 Cache saved at",
            cache_cleared: "🗑️ Cache cleared",
            ignore_pattern: "🚫 Ignoring pattern",
            ext_filter: "📋 Extension filter",

            // Search
            search_text: "🔍 TEXT search",
            search_semantic: "🧠 SEMANTIC search (TF-IDF)",
            search_multithread: "🔍 TEXT search (multi-thread ⚡)",
            query_label: "Query",
            checking_files: "📄 Checking",
            progress: "Progress",
            no_matches: "⚠️ No matches found",

            // Summary
            summary: "📊 Summary",
            files_found: "Files found",
            total_matches: "Total matches",
            time: "Time",
            cores_used: "Cores used",

            // Stats
            stats_title: "📊 semcode-search Statistics",
            files_indexed: "📁 Indexed files",
            total_lines: "📝 Total lines",
            languages: "🔤 Languages",
            last_indexed: "🕐 Last indexed",

            // History
            history_title: "🕐 Search history",
            history_empty: "📭 No searches in history",
            history_cleared: "🗑️ History cleared",
            history_total: "📊 Total",
            last_search: "🕐 Last search",

            // Alias
            alias_saved: "✅ Alias saved",
            alias_list: "📋 Saved aliases",
            alias_empty: "📭 No aliases saved",
            alias_removed: "🗑️ Alias removed",
            alias_not_found: "⚠️ Alias not found",
            alias_running: "🚀 Running alias",

            // Errors
            error_cache_not_found: "⚠️ No cache found. Run 'index' first",
            error_cache_corrupted: "⚠️ Corrupted cache. Run 'index' first",
            error_no_query: "⚠️ You must provide a query with --query or an alias with --alias",
            error_config_exists: "⚠️ Configuration already exists. Use --force to overwrite",

            // Config
            config_created: "✅ Configuration created at",
            config_hint: "💡 You can edit it manually or use 'alias' to manage searches",

            // Watch
            watch_starting: "👁️ Watching changes in",
            watch_interval: "⏱️ Interval",
            watch_initial_index: "📁 Indexing for the first time...",
            watch_observing: "🔄 Watching",
            watch_changes: "⚠️ changes detected",
            watch_reindexing: "📁 Reindexing...",
            watch_done: "✅ Reindex complete",
        }
    }
}

/// Obtiene el idioma actual
pub fn current_language() -> Language {
    Language::detect()
}

/// Traduce al idioma actual
pub fn t() -> Translations {
    current_language().t()
}
/// Devuelve todas las traducciones como un HashMap serializable a JSON
pub fn get_all_translations(lang: Language) -> std::collections::HashMap<String, String> {
    let tr = lang.t();
    let mut map = std::collections::HashMap::new();

    map.insert("welcome_title".to_string(), tr.welcome_title.to_string());
    map.insert("welcome_message".to_string(), tr.welcome_message.to_string());
    map.insert("press_enter".to_string(), tr.press_enter.to_string());
    map.insert("more_info".to_string(), tr.more_info.to_string());

    map.insert("cmd_init".to_string(), tr.cmd_init.to_string());
    map.insert("cmd_index".to_string(), tr.cmd_index.to_string());
    map.insert("cmd_search".to_string(), tr.cmd_search.to_string());
    map.insert("cmd_stats".to_string(), tr.cmd_stats.to_string());
    map.insert("cmd_history".to_string(), tr.cmd_history.to_string());
    map.insert("cmd_alias".to_string(), tr.cmd_alias.to_string());
    map.insert("cmd_watch".to_string(), tr.cmd_watch.to_string());

    map.insert("indexing".to_string(), tr.indexing.to_string());
    map.insert("indexed_files".to_string(), tr.indexed_files.to_string());
    map.insert("total_size".to_string(), tr.total_size.to_string());
    map.insert("cache_saved".to_string(), tr.cache_saved.to_string());
    map.insert("cache_cleared".to_string(), tr.cache_cleared.to_string());
    map.insert("ignore_pattern".to_string(), tr.ignore_pattern.to_string());
    map.insert("ext_filter".to_string(), tr.ext_filter.to_string());

    map.insert("search_text".to_string(), tr.search_text.to_string());
    map.insert("search_semantic".to_string(), tr.search_semantic.to_string());
    map.insert("search_multithread".to_string(), tr.search_multithread.to_string());
    map.insert("query_label".to_string(), tr.query_label.to_string());
    map.insert("checking_files".to_string(), tr.checking_files.to_string());
    map.insert("progress".to_string(), tr.progress.to_string());
    map.insert("no_matches".to_string(), tr.no_matches.to_string());

    map.insert("summary".to_string(), tr.summary.to_string());
    map.insert("files_found".to_string(), tr.files_found.to_string());
    map.insert("total_matches".to_string(), tr.total_matches.to_string());
    map.insert("time".to_string(), tr.time.to_string());
    map.insert("cores_used".to_string(), tr.cores_used.to_string());

    map.insert("stats_title".to_string(), tr.stats_title.to_string());
    map.insert("files_indexed".to_string(), tr.files_indexed.to_string());
    map.insert("total_lines".to_string(), tr.total_lines.to_string());
    map.insert("languages".to_string(), tr.languages.to_string());
    map.insert("last_indexed".to_string(), tr.last_indexed.to_string());

    map.insert("history_title".to_string(), tr.history_title.to_string());
    map.insert("history_empty".to_string(), tr.history_empty.to_string());
    map.insert("history_cleared".to_string(), tr.history_cleared.to_string());
    map.insert("history_total".to_string(), tr.history_total.to_string());
    map.insert("last_search".to_string(), tr.last_search.to_string());

    map.insert("alias_saved".to_string(), tr.alias_saved.to_string());
    map.insert("alias_list".to_string(), tr.alias_list.to_string());
    map.insert("alias_empty".to_string(), tr.alias_empty.to_string());
    map.insert("alias_removed".to_string(), tr.alias_removed.to_string());
    map.insert("alias_not_found".to_string(), tr.alias_not_found.to_string());
    map.insert("alias_running".to_string(), tr.alias_running.to_string());

    map.insert("error_cache_not_found".to_string(), tr.error_cache_not_found.to_string());
    map.insert("error_cache_corrupted".to_string(), tr.error_cache_corrupted.to_string());
    map.insert("error_no_query".to_string(), tr.error_no_query.to_string());
    map.insert("error_config_exists".to_string(), tr.error_config_exists.to_string());

    map.insert("config_created".to_string(), tr.config_created.to_string());
    map.insert("config_hint".to_string(), tr.config_hint.to_string());

    map.insert("watch_starting".to_string(), tr.watch_starting.to_string());
    map.insert("watch_interval".to_string(), tr.watch_interval.to_string());
    map.insert("watch_initial_index".to_string(), tr.watch_initial_index.to_string());
    map.insert("watch_observing".to_string(), tr.watch_observing.to_string());
    map.insert("watch_changes".to_string(), tr.watch_changes.to_string());
    map.insert("watch_reindexing".to_string(), tr.watch_reindexing.to_string());
    map.insert("watch_done".to_string(), tr.watch_done.to_string());

    map
}