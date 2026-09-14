//! Backend de la GUI de semcode-search

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use semcode_search::Cache;

/// Estado de la aplicación
pub struct AppState {
    pub current_path: Mutex<String>,
}

// ============================================================
// TIPOS DE DATOS
// ============================================================

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SearchResultJson {
    pub path: String,
    pub matches: usize,
    pub size: u64,
    pub preview: String,
    pub score: Option<f32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchResponse {
    pub total: usize,
    pub results: Vec<SearchResultJson>,
    pub time_ms: u128,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IndexResponse {
    pub total_files: usize,
    pub total_size: u64,
    pub success: bool,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StatsResponse {
    pub total_files: usize,
    pub total_lines: usize,
    pub total_size: u64,
    pub languages: Vec<LanguageInfo>,
    pub has_embeddings: bool,
    pub last_indexed: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LanguageInfo {
    pub name: String,
    pub files: usize,
    pub size: u64,
    pub percentage: f32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RecentProject {
    pub path: String,
    pub name: String,
    pub added_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HistoryEntry {
    pub query: String,
    pub timestamp: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AliasEntry {
    pub name: String,
    pub query: String,
    pub params: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ConfigJson {
    pub default_ext: Vec<String>,
    pub default_ignore: Vec<String>,
    pub default_ignore_pattern: String,
    pub max_size: String,
    pub verbose: bool,
    pub interactive: bool,
}

impl Default for ConfigJson {
    fn default() -> Self {
        Self {
            default_ext: vec!["rs".to_string(), "md".to_string(), "toml".to_string()],
            default_ignore: vec![
                ".git".to_string(),
                "target".to_string(),
                "node_modules".to_string(),
            ],
            default_ignore_pattern: "*.log".to_string(),
            max_size: "1MB".to_string(),
            verbose: true,
            interactive: false,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct SearchFilters {
    pub extensions: Option<Vec<String>>,
    pub max_size: Option<u64>,
    pub min_size: Option<u64>,
    pub exact: Option<bool>,
    pub ignore_case: Option<bool>,
    pub regex: Option<String>,
    pub filename_only: Option<bool>,
    // Campos que el frontend envía pero mapeamos a otros
    pub exclude: Option<Vec<String>>,
    pub min_score: Option<f32>,
    pub case_sensitive: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SystemInfo {
    pub os: String,
    pub arch: String,
    pub num_cpus: usize,
    pub version: String,
    pub executable_path: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AiStatus {
    pub enabled: bool,
    pub model_available: bool,
    pub model_name: String,
    pub dimensions: usize,
    pub message: String,
}

// ============================================================
// COMANDOS BÁSICOS
// ============================================================

#[tauri::command]
fn search(query: String, path: String, limit: Option<usize>) -> Result<SearchResponse, String> {
    use std::time::Instant;
    let start = Instant::now();

    let cache_path = if path.is_empty() {
        PathBuf::from(".semantic-index.json")
    } else {
        Path::new(&path).join(".semantic-index.json")
    };

    if !cache_path.exists() {
        return Err(format!(
            "No se encontró caché en {}. Ejecuta 'index' primero.",
            cache_path.display()
        ));
    }

    let cache = Cache::load(&cache_path).map_err(|e| format!("Error al cargar caché: {}", e))?;

    let query_lower = query.to_lowercase();
    let limit = limit.unwrap_or(50);

    let mut results: Vec<SearchResultJson> = cache
        .entries
        .iter()
        .filter_map(|(p, e)| {
            let content_lower = e.content.to_lowercase();
            if content_lower.contains(&query_lower) {
                let matches = content_lower.matches(&query_lower).count();
                Some(SearchResultJson {
                    path: p.display().to_string(),
                    matches,
                    size: e.size,
                    preview: e.content.lines().take(10).collect::<Vec<_>>().join("\n"),
                    score: None,
                })
            } else {
                None
            }
        })
        .collect();

    results.sort_by_key(|r| std::cmp::Reverse(r.matches));
    results.truncate(limit);

    let _ = add_to_history_internal(&query);

    Ok(SearchResponse {
        total: results.len(),
        results,
        time_ms: start.elapsed().as_millis(),
    })
}

#[tauri::command]
fn index_project(path: String) -> Result<IndexResponse, String> {
    let cache_path = Path::new(&path).join(".semantic-index.json");

    let result = semcode_search::core::index_files(
        &path,
        vec![".git", "target", "node_modules", "dist", "build"],
        None,
        None,
        true,
    );

    match result {
        Ok(_) => {
            if let Ok(cache) = Cache::load(&cache_path) {
                let total_files = cache.entries.len();
                let total_size: u64 = cache.entries.values().map(|e| e.size).sum();
                let _ = add_project_to_recent(&path);

                Ok(IndexResponse {
                    total_files,
                    total_size,
                    success: true,
                    message: format!("Indexados {} archivos", total_files),
                })
            } else {
                Err("No se pudo leer la caché después de indexar".to_string())
            }
        }
        Err(e) => Err(format!("Error al indexar: {}", e)),
    }
}

#[tauri::command]
fn get_stats(path: String) -> Result<StatsResponse, String> {
    let cache_path = if path.is_empty() {
        PathBuf::from(".semantic-index.json")
    } else {
        Path::new(&path).join(".semantic-index.json")
    };

    if !cache_path.exists() {
        return Err("No se encontró caché. Ejecuta 'index' primero.".to_string());
    }

    let cache = Cache::load(&cache_path).map_err(|e| format!("Error al cargar caché: {}", e))?;

    let total_files = cache.entries.len();
    let total_size: u64 = cache.entries.values().map(|e| e.size).sum();
    let total_lines: usize = cache
        .entries
        .values()
        .map(|e| e.content.lines().count())
        .sum();

    let mut lang_map: HashMap<String, (usize, u64)> = HashMap::new();
    for entry in cache.entries.values() {
        if let Some(ext) = entry.path.extension().and_then(|e| e.to_str()) {
            let counter = lang_map.entry(ext.to_string()).or_insert((0, 0));
            counter.0 += 1;
            counter.1 += entry.size;
        }
    }

    let mut languages: Vec<LanguageInfo> = lang_map
        .into_iter()
        .map(|(name, (files, size))| LanguageInfo {
            name,
            files,
            size,
            percentage: if total_files > 0 {
                (files as f32 / total_files as f32) * 100.0
            } else {
                0.0
            },
        })
        .collect();
    languages.sort_by_key(|l| std::cmp::Reverse(l.files));

    Ok(StatsResponse {
        total_files,
        total_lines,
        total_size,
        languages,
        has_embeddings: cache.has_embeddings,
        last_indexed: cache.updated.clone(),
    })
}

#[tauri::command]
fn read_file(path: String) -> Result<String, String> {
    std::fs::read_to_string(&path).map_err(|e| format!("Error al leer archivo: {}", e))
}

// ============================================================
// BLOQUE 1: i18n
// ============================================================

#[tauri::command]
fn get_current_language() -> String {
    use semcode_search::i18n::current_language;
    match current_language() {
        semcode_search::i18n::Language::Spanish => "es".to_string(),
        semcode_search::i18n::Language::English => "en".to_string(),
    }
}

#[tauri::command]
fn set_language(lang: String) -> Result<String, String> {
    match lang.as_str() {
        "es" | "en" => Ok(format!("Idioma cambiado a: {}", lang)),
        _ => Err(format!("Idioma no soportado: {}", lang)),
    }
}

#[tauri::command]
fn get_available_languages() -> Vec<serde_json::Value> {
    use serde_json::json;
    vec![
        json!({"code": "es", "name": "Español", "flag": "ES"}),
        json!({"code": "en", "name": "English", "flag": "EN"}),
    ]
}

#[tauri::command]
fn get_translations(lang: String) -> Result<serde_json::Value, String> {
    use semcode_search::i18n::{get_all_translations, Language};

    let language = match lang.as_str() {
        "es" => Language::Spanish,
        "en" => Language::English,
        _ => return Err(format!("Idioma no soportado: {}", lang)),
    };

    let map = get_all_translations(language);

    serde_json::to_value(map).map_err(|e| format!("Error al serializar: {}", e))
}

// ============================================================
// BLOQUE 2: RECIENTES
// ============================================================

fn get_recent_projects_path() -> PathBuf {
    if let Some(config_dir) = dirs::config_dir() {
        let dir = config_dir.join("semcode-search");
        let _ = std::fs::create_dir_all(&dir);
        dir.join("recent_projects.json")
    } else {
        PathBuf::from(".semcode-search-recent.json")
    }
}

fn load_recent_projects() -> Vec<RecentProject> {
    let path = get_recent_projects_path();
    if path.exists() {
        if let Ok(content) = std::fs::read_to_string(&path) {
            if let Ok(list) = serde_json::from_str::<Vec<RecentProject>>(&content) {
                return list;
            }
        }
    }
    Vec::new()
}

fn save_recent_projects(projects: &[RecentProject]) -> Result<(), String> {
    let path = get_recent_projects_path();
    let content = serde_json::to_string_pretty(projects)
        .map_err(|e| format!("Error al serializar: {}", e))?;
    std::fs::write(&path, content).map_err(|e| format!("Error al guardar: {}", e))
}

fn add_project_to_recent(path: &str) -> Result<(), String> {
    let mut projects = load_recent_projects();
    projects.retain(|p| p.path != path);

    let name = Path::new(path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or(path)
        .to_string();

    projects.insert(
        0,
        RecentProject {
            path: path.to_string(),
            name,
            added_at: chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
        },
    );

    projects.truncate(10);
    save_recent_projects(&projects)
}

#[tauri::command]
fn list_recent_projects() -> Vec<RecentProject> {
    load_recent_projects()
}

#[tauri::command]
fn add_recent_project(path: String) -> Result<String, String> {
    add_project_to_recent(&path)?;
    Ok(format!("Proyecto añadido: {}", path))
}

#[tauri::command]
fn remove_recent_project(path: String) -> Result<String, String> {
    let mut projects = load_recent_projects();
    let original_len = projects.len();
    projects.retain(|p| p.path != path);

    if projects.len() == original_len {
        return Err(format!("Proyecto no encontrado: {}", path));
    }

    save_recent_projects(&projects)?;
    Ok(format!("Proyecto eliminado: {}", path))
}

#[tauri::command]
fn clear_recent_projects() -> Result<String, String> {
    save_recent_projects(&[])?;
    Ok("Lista de recientes limpiada".to_string())
}

// ============================================================
// BLOQUE 3: HISTORIAL
// ============================================================

fn get_history_path() -> PathBuf {
    if let Some(config_dir) = dirs::config_dir() {
        let dir = config_dir.join("semcode-search");
        let _ = std::fs::create_dir_all(&dir);
        dir.join("history.json")
    } else {
        PathBuf::from(".semcode-search-history.json")
    }
}

fn load_history() -> Vec<HistoryEntry> {
    let path = get_history_path();
    if path.exists() {
        if let Ok(content) = std::fs::read_to_string(&path) {
            if let Ok(list) = serde_json::from_str::<Vec<HistoryEntry>>(&content) {
                return list;
            }
        }
    }
    Vec::new()
}

fn save_history(history: &[HistoryEntry]) -> Result<(), String> {
    let path = get_history_path();
    let content = serde_json::to_string_pretty(history)
        .map_err(|e| format!("Error al serializar: {}", e))?;
    std::fs::write(&path, content).map_err(|e| format!("Error al guardar: {}", e))
}

fn add_to_history_internal(query: &str) -> Result<(), String> {
    let mut history = load_history();

    if let Some(last) = history.last() {
        if last.query == query {
            return Ok(());
        }
    }

    history.push(HistoryEntry {
        query: query.to_string(),
        timestamp: chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
    });

    if history.len() > 100 {
        let start = history.len() - 100;
        history = history[start..].to_vec();
    }

    save_history(&history)
}

#[tauri::command]
fn get_history(limit: Option<usize>) -> Vec<HistoryEntry> {
    let mut history = load_history();
    history.reverse();
    let limit = limit.unwrap_or(50);
    history.truncate(limit);
    history
}

#[tauri::command]
fn clear_history() -> Result<String, String> {
    save_history(&[])?;
    Ok("Historial limpiado".to_string())
}

#[tauri::command]
fn get_last_search() -> Option<HistoryEntry> {
    let history = load_history();
    history.last().cloned()
}

#[tauri::command]
fn add_to_history(query: String) -> Result<String, String> {
    add_to_history_internal(&query)?;
    Ok(format!("Añadido al historial: {}", query))
}

// ============================================================
// BLOQUE 4: ALIAS
// ============================================================

fn get_config_path() -> PathBuf {
    if let Some(config_dir) = dirs::config_dir() {
        let dir = config_dir.join("semcode-search");
        let _ = std::fs::create_dir_all(&dir);
        dir.join("config.toml")
    } else {
        PathBuf::from(".semcode-search-config.toml")
    }
}

#[derive(Debug, Serialize, Deserialize, Default)]
struct ConfigToml {
    default_ext: Option<Vec<String>>,
    default_ignore: Option<Vec<String>>,
    default_ignore_pattern: Option<String>,
    max_size: Option<String>,
    verbose: Option<bool>,
    interactive: Option<bool>,
    aliases: Option<HashMap<String, AliasToml>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct AliasToml {
    query: String,
    params: Vec<String>,
}

fn load_aliases_from_config() -> HashMap<String, AliasToml> {
    let path = get_config_path();
    if path.exists() {
        if let Ok(content) = std::fs::read_to_string(&path) {
            if let Ok(config) = toml::from_str::<ConfigToml>(&content) {
                return config.aliases.unwrap_or_default();
            }
        }
    }
    HashMap::new()
}

fn save_aliases_to_config(aliases: &HashMap<String, AliasToml>) -> Result<(), String> {
    let path = get_config_path();

    let mut config = if path.exists() {
        std::fs::read_to_string(&path)
            .ok()
            .and_then(|c| toml::from_str::<ConfigToml>(&c).ok())
            .unwrap_or_default()
    } else {
        ConfigToml::default()
    };

    config.aliases = Some(aliases.clone());

    let content =
        toml::to_string_pretty(&config).map_err(|e| format!("Error al serializar: {}", e))?;

    std::fs::write(&path, content).map_err(|e| format!("Error al guardar: {}", e))
}

#[tauri::command]
fn list_aliases() -> Vec<AliasEntry> {
    let aliases = load_aliases_from_config();
    aliases
        .into_iter()
        .map(|(name, entry)| AliasEntry {
            name,
            query: entry.query,
            params: entry.params,
        })
        .collect()
}

/// save_alias acepta `params` como `Option<Vec<String>>` para que el JS pueda enviar {} o []
#[tauri::command]
fn save_alias(
    name: String,
    query: String,
    params: Option<Vec<String>>,
) -> Result<String, String> {
    let mut aliases = load_aliases_from_config();
    aliases.insert(
        name.clone(),
        AliasToml {
            query,
            params: params.unwrap_or_default(),
        },
    );
    save_aliases_to_config(&aliases)?;
    Ok(format!("Alias guardado: {}", name))
}

#[tauri::command]
fn delete_alias(name: String) -> Result<String, String> {
    let mut aliases = load_aliases_from_config();
    if aliases.remove(&name).is_none() {
        return Err(format!("Alias no encontrado: {}", name));
    }
    save_aliases_to_config(&aliases)?;
    Ok(format!("Alias eliminado: {}", name))
}

#[tauri::command]
fn run_alias(name: String) -> Result<AliasEntry, String> {
    let aliases = load_aliases_from_config();
    let entry = aliases
        .get(&name)
        .ok_or_else(|| format!("Alias no encontrado: {}", name))?;

    Ok(AliasEntry {
        name,
        query: entry.query.clone(),
        params: entry.params.clone(),
    })
}

// ============================================================
// BLOQUE 5: CONFIGURACIÓN
// ============================================================

#[tauri::command]
fn get_config() -> ConfigJson {
    let path = get_config_path();
    if path.exists() {
        if let Ok(content) = std::fs::read_to_string(&path) {
            if let Ok(config) = toml::from_str::<ConfigToml>(&content) {
                return ConfigJson {
                    default_ext: config.default_ext.unwrap_or_default(),
                    default_ignore: config.default_ignore.unwrap_or_default(),
                    default_ignore_pattern: config.default_ignore_pattern.unwrap_or_default(),
                    max_size: config.max_size.unwrap_or_default(),
                    verbose: config.verbose.unwrap_or(true),
                    interactive: config.interactive.unwrap_or(false),
                };
            }
        }
    }
    ConfigJson::default()
}

#[tauri::command]
fn save_config(config: ConfigJson) -> Result<String, String> {
    let path = get_config_path();

    let toml_config = ConfigToml {
        default_ext: Some(config.default_ext),
        default_ignore: Some(config.default_ignore),
        default_ignore_pattern: Some(config.default_ignore_pattern),
        max_size: Some(config.max_size),
        verbose: Some(config.verbose),
        interactive: Some(config.interactive),
        aliases: Some(load_aliases_from_config()),
    };

    let content =
        toml::to_string_pretty(&toml_config).map_err(|e| format!("Error al serializar: {}", e))?;

    std::fs::write(&path, content).map_err(|e| format!("Error al guardar: {}", e))?;
    Ok("Configuración guardada".to_string())
}

#[tauri::command]
fn reset_config() -> Result<ConfigJson, String> {
    let default = ConfigJson::default();
    save_config(default.clone())?;
    Ok(default)
}

#[tauri::command]
fn export_config(output_path: String) -> Result<String, String> {
    let config = get_config();
    let json = serde_json::to_string_pretty(&config)
        .map_err(|e| format!("Error al serializar: {}", e))?;
    std::fs::write(&output_path, json).map_err(|e| format!("Error al escribir: {}", e))?;
    Ok(format!("Configuración exportada a: {}", output_path))
}

#[tauri::command]
fn import_config(input_path: String) -> Result<ConfigJson, String> {
    let content = std::fs::read_to_string(&input_path)
        .map_err(|e| format!("Error al leer archivo: {}", e))?;

    let config: ConfigJson =
        serde_json::from_str(&content).map_err(|e| format!("Error al parsear JSON: {}", e))?;

    save_config(config.clone())?;
    Ok(config)
}

// ============================================================
// BLOQUE 6: BÚSQUEDAS AVANZADAS + EXPORTACIÓN
// ============================================================

#[tauri::command]
fn search_advanced(
    query: String,
    path: String,
    filters: SearchFilters,
    limit: Option<usize>,
) -> Result<SearchResponse, String> {
    use std::time::Instant;
    let start = Instant::now();

    let cache_path = if path.is_empty() {
        PathBuf::from(".semantic-index.json")
    } else {
        Path::new(&path).join(".semantic-index.json")
    };

    if !cache_path.exists() {
        return Err(format!("No se encontró caché en {}", cache_path.display()));
    }

    let cache = Cache::load(&cache_path).map_err(|e| format!("Error al cargar caché: {}", e))?;

    let limit = limit.unwrap_or(50);

    let use_regex = filters.regex.is_some();
    let regex_compiled = if let Some(ref regex_str) = filters.regex {
        if !regex_str.is_empty() {
            Some(regex::Regex::new(regex_str).map_err(|e| format!("Regex inválido: {}", e))?)
        } else {
            None
        }
    } else {
        None
    };

    let ignore_case = filters.ignore_case.unwrap_or(!filters.case_sensitive.unwrap_or(false));

    let query_for_search = if ignore_case {
        query.to_lowercase()
    } else {
        query.clone()
    };

    let filename_only = filters.filename_only.unwrap_or(false);

    let mut results: Vec<SearchResultJson> = cache
        .entries
        .iter()
        .filter_map(|(p, e)| {
            // Filtro por extensiones
            if let Some(ref exts) = filters.extensions {
                if !exts.is_empty() {
                    let file_ext = p
                        .extension()
                        .and_then(|x| x.to_str())
                        .unwrap_or("")
                        .to_string();
                    if !exts.iter().any(|ex| ex == &file_ext) {
                        return None;
                    }
                }
            }

            // Filtro por exclusión (busca en el path)
            if let Some(ref excludes) = filters.exclude {
                let path_str = p.display().to_string();
                if excludes.iter().any(|ex| path_str.contains(ex)) {
                    return None;
                }
            }

            // Filtro por tamaño máximo
            if let Some(max_size) = filters.max_size {
                if e.size > max_size {
                    return None;
                }
            }

            // Filtro por tamaño mínimo
            if let Some(min_size) = filters.min_size {
                if e.size < min_size {
                    return None;
                }
            }

            // Filtrar solo por nombre de archivo
            if filename_only {
                let filename = p.file_name().and_then(|n| n.to_str()).unwrap_or("");
                let filename_to_check = if ignore_case {
                    filename.to_lowercase()
                } else {
                    filename.to_string()
                };

                if filename_to_check.contains(&query_for_search) {
                    return Some(SearchResultJson {
                        path: p.display().to_string(),
                        matches: 1,
                        size: e.size,
                        preview: e.content.lines().take(10).collect::<Vec<_>>().join("\n"),
                        score: None,
                    });
                }
                return None;
            }

            let content_to_search = if ignore_case {
                e.content.to_lowercase()
            } else {
                e.content.clone()
            };

            if use_regex {
                if let Some(ref regex) = regex_compiled {
                    let matches: Vec<_> = regex.find_iter(&e.content).collect();
                    if !matches.is_empty() {
                        return Some(SearchResultJson {
                            path: p.display().to_string(),
                            matches: matches.len(),
                            size: e.size,
                            preview: e.content.lines().take(10).collect::<Vec<_>>().join("\n"),
                            score: None,
                        });
                    }
                }
                None
            } else {
                let found = if filters.exact.unwrap_or(false) {
                    content_to_search
                        .split(|c: char| !c.is_alphanumeric() && c != '_')
                        .any(|w| w == query_for_search)
                } else {
                    content_to_search.contains(&query_for_search)
                };

                if found {
                    let matches = if filters.exact.unwrap_or(false) {
                        content_to_search
                            .split(|c: char| !c.is_alphanumeric() && c != '_')
                            .filter(|w| *w == query_for_search)
                            .count()
                    } else {
                        content_to_search.matches(&query_for_search).count()
                    };

                    Some(SearchResultJson {
                        path: p.display().to_string(),
                        matches,
                        size: e.size,
                        preview: e.content.lines().take(10).collect::<Vec<_>>().join("\n"),
                        score: None,
                    })
                } else {
                    None
                }
            }
        })
        .collect();

    results.sort_by_key(|r| std::cmp::Reverse(r.matches));
    results.truncate(limit);

    let _ = add_to_history_internal(&query);

    Ok(SearchResponse {
        total: results.len(),
        results,
        time_ms: start.elapsed().as_millis(),
    })
}

#[tauri::command]
fn search_by_filename(
    pattern: String,
    path: String,
    ignore_case: Option<bool>,
) -> Result<SearchResponse, String> {
    use std::time::Instant;
    let start = Instant::now();

    let cache_path = if path.is_empty() {
        PathBuf::from(".semantic-index.json")
    } else {
        Path::new(&path).join(".semantic-index.json")
    };

    if !cache_path.exists() {
        return Err(format!("No se encontró caché en {}", cache_path.display()));
    }

    let cache = Cache::load(&cache_path).map_err(|e| format!("Error al cargar caché: {}", e))?;

    let pattern_lower = if ignore_case.unwrap_or(true) {
        pattern.to_lowercase()
    } else {
        pattern.clone()
    };

    let mut results: Vec<SearchResultJson> = cache
        .entries
        .iter()
        .filter_map(|(p, e)| {
            let filename = p.file_name().and_then(|n| n.to_str()).unwrap_or("");
            let filename_to_check = if ignore_case.unwrap_or(true) {
                filename.to_lowercase()
            } else {
                filename.to_string()
            };

            if filename_to_check.contains(&pattern_lower) {
                Some(SearchResultJson {
                    path: p.display().to_string(),
                    matches: 1,
                    size: e.size,
                    preview: e.content.lines().take(10).collect::<Vec<_>>().join("\n"),
                    score: None,
                })
            } else {
                None
            }
        })
        .collect();

    results.sort_by(|a, b| a.path.cmp(&b.path));

    Ok(SearchResponse {
        total: results.len(),
        results,
        time_ms: start.elapsed().as_millis(),
    })
}

#[tauri::command]
fn export_results_json(
    results: Vec<SearchResultJson>,
    output_path: String,
) -> Result<String, String> {
    let json = serde_json::to_string_pretty(&results)
        .map_err(|e| format!("Error al serializar: {}", e))?;
    std::fs::write(&output_path, json).map_err(|e| format!("Error al escribir: {}", e))?;
    Ok(format!(
        "{} resultados exportados a: {}",
        results.len(),
        output_path
    ))
}

#[tauri::command]
fn export_results_csv(
    results: Vec<SearchResultJson>,
    output_path: String,
) -> Result<String, String> {
    let mut csv = String::from("path,matches,size_bytes\n");
    for r in &results {
        let path_escaped = r.path.replace('"', "\"\"");
        csv.push_str(&format!("\"{}\",{},{}\n", path_escaped, r.matches, r.size));
    }
    std::fs::write(&output_path, csv).map_err(|e| format!("Error al escribir: {}", e))?;
    Ok(format!(
        "{} resultados exportados a: {}",
        results.len(),
        output_path
    ))
}

#[tauri::command]
fn export_index(path: String, output_path: String) -> Result<String, String> {
    let cache_path = if path.is_empty() {
        PathBuf::from(".semantic-index.json")
    } else {
        Path::new(&path).join(".semantic-index.json")
    };

    if !cache_path.exists() {
        return Err("No se encontró caché".to_string());
    }

    let cache = Cache::load(&cache_path).map_err(|e| format!("Error al cargar caché: {}", e))?;

    let json = serde_json::to_string(&cache).map_err(|e| format!("Error al serializar: {}", e))?;
    std::fs::write(&output_path, json).map_err(|e| format!("Error al escribir: {}", e))?;

    Ok(format!(
        "Índice exportado ({} archivos) a: {}",
        cache.entries.len(),
        output_path
    ))
}

#[tauri::command]
fn import_index(input_path: String, target_path: String) -> Result<String, String> {
    let content = std::fs::read_to_string(&input_path)
        .map_err(|e| format!("Error al leer archivo: {}", e))?;

    let cache: Cache =
        serde_json::from_str(&content).map_err(|e| format!("Error al parsear JSON: {}", e))?;

    let cache_path = Path::new(&target_path).join(".semantic-index.json");
    cache
        .save(&cache_path)
        .map_err(|e| format!("Error al guardar caché: {}", e))?;

    Ok(format!(
        "Índice importado ({} archivos) a: {}",
        cache.entries.len(),
        cache_path.display()
    ))
}

// ============================================================
// BLOQUE 7: UTILIDADES
// ============================================================

#[tauri::command]
fn copy_to_clipboard(text: String) -> Result<String, String> {
    #[cfg(target_os = "windows")]
    {
        use std::process::Command;
        let escaped = text.replace('\'', "''");
        let output = Command::new("powershell")
            .args(&["-Command", &format!("Set-Clipboard -Value '{}'", escaped)])
            .output()
            .map_err(|e| format!("Error al copiar: {}", e))?;

        if !output.status.success() {
            return Err("Error al copiar al portapapeles".to_string());
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        let _ = text;
        return Err("Copiar al portapapeles solo está soportado en Windows por ahora".to_string());
    }

    Ok("Copiado al portapapeles".to_string())
}

#[tauri::command]
fn open_in_editor(path: String) -> Result<String, String> {
    #[cfg(target_os = "windows")]
    {
        use std::process::Command;
        Command::new("cmd")
            .args(&["/C", "code", &path])
            .spawn()
            .map_err(|e| format!("Error al abrir VSCode: {}", e))?;
    }

    #[cfg(target_os = "macos")]
    {
        use std::process::Command;
        Command::new("open")
            .args(&["-a", "Visual Studio Code", &path])
            .spawn()
            .map_err(|e| format!("Error al abrir VSCode: {}", e))?;
    }

    #[cfg(target_os = "linux")]
    {
        use std::process::Command;
        Command::new("code")
            .arg(&path)
            .spawn()
            .map_err(|e| format!("Error al abrir VSCode: {}", e))?;
    }

    Ok(format!("Abierto en VSCode: {}", path))
}

#[tauri::command]
fn open_in_explorer(path: String) -> Result<String, String> {
    let folder = Path::new(&path)
        .parent()
        .ok_or_else(|| "No se pudo obtener la carpeta padre".to_string())?
        .to_string_lossy()
        .to_string();

    #[cfg(target_os = "windows")]
    {
        use std::process::Command;
        Command::new("explorer")
            .arg(&folder)
            .spawn()
            .map_err(|e| format!("Error al abrir explorador: {}", e))?;
    }

    #[cfg(target_os = "macos")]
    {
        use std::process::Command;
        Command::new("open")
            .arg(&folder)
            .spawn()
            .map_err(|e| format!("Error al abrir Finder: {}", e))?;
    }

    #[cfg(target_os = "linux")]
    {
        use std::process::Command;
        Command::new("xdg-open")
            .arg(&folder)
            .spawn()
            .map_err(|e| format!("Error al abrir explorador: {}", e))?;
    }

    Ok(format!("Abierto en explorador: {}", folder))
}

#[tauri::command]
fn open_in_browser(url: String) -> Result<String, String> {
    #[cfg(target_os = "windows")]
    {
        use std::process::Command;
        Command::new("cmd")
            .args(&["/C", "start", "", &url])
            .spawn()
            .map_err(|e| format!("Error al abrir navegador: {}", e))?;
    }

    #[cfg(target_os = "macos")]
    {
        use std::process::Command;
        Command::new("open")
            .arg(&url)
            .spawn()
            .map_err(|e| format!("Error al abrir navegador: {}", e))?;
    }

    #[cfg(target_os = "linux")]
    {
        use std::process::Command;
        Command::new("xdg-open")
            .arg(&url)
            .spawn()
            .map_err(|e| format!("Error al abrir navegador: {}", e))?;
    }

    Ok(format!("Abierto en navegador: {}", url))
}

#[tauri::command]
fn get_system_info() -> SystemInfo {
    SystemInfo {
        os: std::env::consts::OS.to_string(),
        arch: std::env::consts::ARCH.to_string(),
        num_cpus: num_cpus::get(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        executable_path: std::env::current_exe()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|_| "desconocido".to_string()),
    }
}

#[tauri::command]
fn get_executable_path() -> String {
    std::env::current_exe()
        .map(|p| p.display().to_string())
        .unwrap_or_else(|_| "desconocido".to_string())
}

#[tauri::command]
fn get_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

// ============================================================
// BLOQUE 8: IA Y PROGRESO
// ============================================================

#[tauri::command]
fn get_ai_status() -> AiStatus {
    #[cfg(feature = "ai")]
    {
        AiStatus {
            enabled: true,
            model_available: true,
            model_name: "all-MiniLM-L6-v2".to_string(),
            dimensions: 384,
            message: "IA habilitada. Modelo disponible.".to_string(),
        }
    }

    #[cfg(not(feature = "ai"))]
    {
        AiStatus {
            enabled: false,
            model_available: false,
            model_name: "".to_string(),
            dimensions: 0,
            message: "IA no habilitada. Compila con --features ai".to_string(),
        }
    }
}

#[tauri::command]
fn check_ai_model() -> bool {
    #[cfg(feature = "ai")]
    {
        true
    }

    #[cfg(not(feature = "ai"))]
    {
        false
    }
}

#[tauri::command]
fn search_with_ai(
    query: String,
    path: String,
    limit: Option<usize>,
) -> Result<SearchResponse, String> {
    #[cfg(feature = "ai")]
    {
        use std::time::Instant;
        let start = Instant::now();

        let cache_path = if path.is_empty() {
            PathBuf::from(".semantic-index.json")
        } else {
            Path::new(&path).join(".semantic-index.json")
        };

        if !cache_path.exists() {
            return Err(format!("No se encontró caché en {}", cache_path.display()));
        }

        let cache =
            Cache::load(&cache_path).map_err(|e| format!("Error al cargar caché: {}", e))?;

        if !cache.has_embeddings {
            return Err(
                "La caché no tiene embeddings. Ejecuta 'index_with_ai' primero.".to_string(),
            );
        }

        let embedder = semcode_search::Embedder::new()
            .map_err(|e| format!("Error al cargar modelo: {}", e))?;

        let query_embedding = embedder
            .embed(&query)
            .map_err(|e| format!("Error al generar embedding: {}", e))?;

        let mut scored: Vec<(SearchResultJson, f32)> = cache
            .entries
            .iter()
            .filter_map(|(p, e)| {
                let emb = e.embedding.as_ref()?;
                let similarity =
                    semcode_search::embeddings::cosine_similarity(&query_embedding, emb);

                if similarity < 0.3 {
                    return None;
                }

                Some((
                    SearchResultJson {
                        path: p.display().to_string(),
                        matches: 0,
                        size: e.size,
                        preview: e.content.lines().take(10).collect::<Vec<_>>().join("\n"),
                        score: Some(similarity),
                    },
                    similarity,
                ))
            })
            .collect();

        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        let limit = limit.unwrap_or(50);
        scored.truncate(limit);

        let results: Vec<SearchResultJson> = scored.into_iter().map(|(r, _)| r).collect();

        let _ = add_to_history_internal(&query);

        Ok(SearchResponse {
            total: results.len(),
            results,
            time_ms: start.elapsed().as_millis(),
        })
    }

    #[cfg(not(feature = "ai"))]
    {
        let _ = (query, path, limit);
        Err("La búsqueda con IA requiere compilar con --features ai".to_string())
    }
}

#[tauri::command]
fn index_with_ai(path: String) -> Result<IndexResponse, String> {
    #[cfg(feature = "ai")]
    {
        let cache_path = Path::new(&path).join(".semantic-index.json");

        let result = semcode_search::core::index_files_with_ai(
            &path,
            vec![".git", "target", "node_modules", "dist", "build"],
            None,
            None,
            true,
        );

        match result {
            Ok(_) => {
                if let Ok(cache) = Cache::load(&cache_path) {
                    let total_files = cache.entries.len();
                    let total_size: u64 = cache.entries.values().map(|e| e.size).sum();
                    let _ = add_project_to_recent(&path);

                    Ok(IndexResponse {
                        total_files,
                        total_size,
                        success: true,
                        message: format!("Indexados {} archivos con IA", total_files),
                    })
                } else {
                    Err("No se pudo leer la caché después de indexar".to_string())
                }
            }
            Err(e) => Err(format!("Error al indexar con IA: {}", e)),
        }
    }

    #[cfg(not(feature = "ai"))]
    {
        let _ = path;
        Err("El indexado con IA requiere compilar con --features ai".to_string())
    }
}

// ============================================================
// PUNTO DE ENTRADA
// ============================================================

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_log::Builder::default().build())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .manage(AppState {
            current_path: Mutex::new(".".to_string()),
        })
        .invoke_handler(tauri::generate_handler![
            // Básicos
            search,
            index_project,
            get_stats,
            read_file,
            // i18n
            get_current_language,
            set_language,
            get_available_languages,
            get_translations,
            // Recientes
            list_recent_projects,
            add_recent_project,
            remove_recent_project,
            clear_recent_projects,
            // Historial
            get_history,
            clear_history,
            get_last_search,
            add_to_history,
            // Alias
            list_aliases,
            save_alias,
            delete_alias,
            run_alias,
            // Configuración
            get_config,
            save_config,
            reset_config,
            export_config,
            import_config,
            // Búsquedas avanzadas + Exportación
            search_advanced,
            search_by_filename,
            export_results_json,
            export_results_csv,
            export_index,
            import_index,
            // Utilidades
            copy_to_clipboard,
            open_in_editor,
            open_in_explorer,
            open_in_browser,
            get_system_info,
            get_executable_path,
            get_version,
            // IA y Progreso
            get_ai_status,
            check_ai_model,
            search_with_ai,
            index_with_ai
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}