//! Módulo Core - Lógica principal del buscador
// Submódulos privados
mod engine;
mod error;
mod types;

// Re-exportar API pública
pub use engine::SearchEngine;
pub use error::{Result, SearchError};
pub use types::{SearchConfig, SearchResult};

// ===== Funciones auxiliares y lógica principal =====

use colored::*;
use glob::Pattern;
use ignore::WalkBuilder;
use rayon::prelude::*;
use regex::Regex;
use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Instant;

// ===== Funciones Auxiliares =====

pub fn get_word_vector(text: &str) -> HashMap<String, u32> {
    let mut word_count = HashMap::new();
    for word in text.split_whitespace() {
        let word = word.to_lowercase();
        let word = word.trim_matches(|c: char| !c.is_alphanumeric());
        if word.len() > 2 {
            *word_count.entry(word.to_string()).or_insert(0) += 1;
        }
    }
    word_count
}

pub fn cosine_similarity(vec1: &HashMap<String, u32>, vec2: &HashMap<String, u32>) -> f32 {
    let mut dot_product = 0.0;
    let mut norm1 = 0.0;
    let mut norm2 = 0.0;

    for (word, count1) in vec1 {
        if let Some(count2) = vec2.get(word) {
            dot_product += (*count1 as f32) * (*count2 as f32);
        }
        norm1 += (*count1 as f32) * (*count1 as f32);
    }

    for count2 in vec2.values() {
        norm2 += (*count2 as f32) * (*count2 as f32);
    }

    if norm1 == 0.0 || norm2 == 0.0 {
        return 0.0;
    }

    dot_product / (norm1.sqrt() * norm2.sqrt())
}

pub fn parse_size(size_str: &str) -> anyhow::Result<u64> {
    let size = byte_unit::Byte::parse_str(size_str, true)
        .map_err(|e| anyhow::anyhow!("Error parsing size: {}", e))?;
    Ok(size.as_u64())
}

pub fn format_size(bytes: u64) -> String {
    if bytes < 1024 {
        format!("{} B", bytes)
    } else if bytes < 1024 * 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else if bytes < 1024 * 1024 * 1024 {
        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
    } else {
        format!("{:.1} GB", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
    }
}

pub fn matches_pattern(path: &Path, pattern: &str) -> bool {
    if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
        if let Ok(pattern) = Pattern::new(pattern) {
            return pattern.matches(file_name);
        }
    }
    false
}

pub fn extract_archive_content(path: &Path) -> anyhow::Result<Option<String>> {
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
    if ext == "zip" {
        if let Ok(content) = fs::read_to_string(path) {
            return Ok(Some(content));
        }
    }
    Ok(None)
}

// ===== Estructura interna =====

#[derive(Debug, Clone)]
pub struct InternalSearchResult {
    pub path: PathBuf,
    pub content: String,
    pub matches: usize,
    pub size: u64,
    pub line_start: Option<usize>,
    pub line_end: Option<usize>,
    pub highlight_regex: Option<Regex>,
}

// ===== Modo Interactivo =====

pub fn interactive_mode(results: &[InternalSearchResult]) -> anyhow::Result<()> {
    use std::io::{self, Write};

    if results.is_empty() {
        println!("{} No hay resultados para mostrar.", "⚠️".yellow());
        return Ok(());
    }

    let total = results.len();
    let mut idx = 0;

    loop {
        clear_screen();
        println!(
            "{} {} de {} (Presiona Enter para avanzar, q para salir)",
            "📖".cyan(),
            idx + 1,
            total
        );

        let result = &results[idx];
        println!("\n{}", result.path.display().to_string().green().bold());
        println!("  Coincidencias: {}", result.matches);
        println!("  Tamaño: {}", format_size(result.size));

        let lines: Vec<&str> = result.content.lines().collect();
        let start = result.line_start.unwrap_or(0).saturating_sub(3);
        let end = result.line_end.unwrap_or(lines.len()).saturating_add(3);

        for (i, line) in lines.iter().enumerate().take(end).skip(start) {
            let line_num = format!("{}:", i + 1).yellow();
            let display_line = if let Some(ref re) = result.highlight_regex {
                re.replace_all(line, |caps: &regex::Captures| {
                    caps[0].to_string().red().to_string()
                })
                .to_string()
            } else {
                line.to_string()
            };
            println!("  {} {}", line_num, display_line);
        }

        println!("\n---");
        print!("[Enter] siguiente  [q] salir ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim();

        if input == "q" || input == "Q" {
            break;
        }

        idx = (idx + 1) % total;
        if idx == 0 {
            println!(
                "\n{} Has llegado al final. Volviendo al principio...",
                "🔄".yellow()
            );
            std::thread::sleep(std::time::Duration::from_millis(500));
        }
    }

    Ok(())
}

fn clear_screen() {
    print!("\x1B[2J\x1B[1;1H");
    std::io::stdout().flush().unwrap();
}

// ===== Configuración interna para búsqueda =====

pub struct SearchConfigInternal {
    pub query: String,
    pub path: String,
    pub ext: Option<Vec<String>>,
    pub ignore: Vec<String>,
    pub exact: bool,
    pub ignore_case: bool,
    pub verbose: bool,
    pub no_cache: bool,
    pub semantic: bool,
    pub file: Option<String>,
    pub summary: bool,
    pub max_size: Option<String>,
    pub ignore_pattern: Option<String>,
    pub extract: bool,
    pub interactive: bool,
}

// ===== Indexado =====

pub fn index_files(
    path: &str,
    ignore_dirs: Vec<&str>,
    ext_filter: Option<Vec<&str>>,
    ignore_pattern: Option<&str>,
    force: bool,
) -> anyhow::Result<()> {
    use crate::cache::Cache;

    let cache_path = Path::new(".semantic-index.json");
    if force && cache_path.exists() {
        fs::remove_file(cache_path)?;
        println!("{} Caché eliminada.", "🗑️".yellow());
    }

    println!("{} Indexando: {}", "📁".green(), path);

    let mut count = 0;
    let mut total_size = 0u64;
    let mut cache = Cache::new();

    let walker = WalkBuilder::new(path)
        .git_ignore(true)
        .follow_links(false)
        .build();

    let files: Vec<PathBuf> = walker
        .filter_map(|result| {
            let entry = result.ok()?;
            let p = entry.path();

            if entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false) {
                if let Some(name) = p.file_name().and_then(|n| n.to_str()) {
                    if ignore_dirs.contains(&name) {
                        return None;
                    }
                }
            }

            if entry.file_type().map(|ft| ft.is_file()).unwrap_or(false) {
                if let Some(pattern) = ignore_pattern {
                    if matches_pattern(p, pattern) {
                        return None;
                    }
                }

                if let Some(ext_str) = p.extension().and_then(|e| e.to_str()) {
                    let should_include = if let Some(ref exts) = ext_filter {
                        exts.contains(&ext_str)
                    } else {
                        true
                    };

                    if should_include {
                        let exts = [
                            "rs", "py", "js", "ts", "go", "java", "c", "cpp", "h", "toml", "json",
                            "txt", "md", "sh", "bash", "yaml", "yml", "css", "html", "xml", "sql",
                            "rb", "php", "swift", "kt", "zip",
                        ];
                        if exts.contains(&ext_str) {
                            return Some(p.to_path_buf());
                        }
                    }
                }
            }
            None
        })
        .collect();

    let chunks: Vec<_> = files
        .par_iter()
        .filter_map(|p| {
            if let Ok(content) = fs::read_to_string(p) {
                if let Ok(metadata) = fs::metadata(p) {
                    let file_size = metadata.len();
                    let modified = metadata
                        .modified()
                        .map(|t| t.duration_since(std::time::UNIX_EPOCH).unwrap().as_secs())
                        .unwrap_or(0);

                    let words: Vec<String> = content
                        .split_whitespace()
                        .map(|w| w.to_lowercase())
                        .filter(|w| w.len() > 2)
                        .collect();

                    return Some((p.clone(), content, modified, words, file_size));
                }
            }
            None
        })
        .collect();

    for (p, content, modified, words, file_size) in chunks {
        count += 1;
        total_size += file_size;
        cache.entries.insert(
            p.clone(),
            crate::cache::CacheEntry {
                path: p,
                content,
                modified,
                words,
                size: file_size,
            },
        );
    }

    cache.updated = chrono::Local::now().to_string();
    cache.save(cache_path)?;

    println!("{} Indexados {} archivos.", "✅".green(), count);
    if let Some(ref exts) = ext_filter {
        println!("{} Filtro por extensiones: {:?}", "📋".blue(), exts);
    }
    if let Some(ref pattern) = ignore_pattern {
        println!("{} Ignorando patrón: {}", "🚫".blue(), pattern);
    }
    println!("{} Tamaño total: {}", "💾".blue(), format_size(total_size));
    println!("{} Caché guardada en .semantic-index.json", "💾".green());

    Ok(())
}

// ===== Búsqueda (AHORA MULTI-HILO) =====

pub fn search_files(config: SearchConfigInternal) -> anyhow::Result<Vec<super::SearchResult>> {
    use crate::cache::Cache;

    let start_time = Instant::now();

    let cache_path = Path::new(".semantic-index.json");

    let use_cache = !config.no_cache && cache_path.exists();
    let cache = if use_cache {
        match Cache::load(cache_path) {
            Ok(c) => c,
            Err(_) => {
                println!("{} Caché corrupta. Ejecute 'index' primero.", "⚠️".yellow());
                return Ok(Vec::new());
            }
        }
    } else {
        println!(
            "{} No se encontró caché. Ejecute 'index' primero.",
            "⚠️".yellow()
        );
        return Ok(Vec::new());
    };

    let max_size_bytes = if let Some(size_str) = &config.max_size {
        Some(parse_size(size_str)?)
    } else {
        None
    };

    let ext_filter: Option<Vec<&str>> = config
        .ext
        .as_ref()
        .map(|e| e.iter().map(|s| s.as_str()).collect());

    // Búsqueda por nombre de archivo
    if let Some(file_pattern) = &config.file {
        let found: Vec<PathBuf> = cache
            .entries
            .keys()
            .filter(|p| {
                p.file_name()
                    .and_then(|n| n.to_str())
                    .map(|n| {
                        if config.ignore_case {
                            n.to_lowercase().contains(&file_pattern.to_lowercase())
                        } else {
                            n.contains(file_pattern)
                        }
                    })
                    .unwrap_or(false)
            })
            .cloned()
            .collect();

        let mut results = Vec::new();
        for p in found {
            if let Some(entry) = cache.entries.get(&p) {
                results.push(super::SearchResult {
                    path: p,
                    content: entry.content.clone(),
                    matches: 0,
                    size: entry.size,
                    line_start: None,
                    line_end: None,
                    score: None,
                    keywords: Vec::new(),
                });
            }
        }
        return Ok(results);
    }

    if config.semantic {
        println!("{} Búsqueda SEMÁNTICA (TF-IDF)", "🧠".cyan());
    } else {
        println!("{} Búsqueda por TEXTO (multi-hilo ⚡)", "🔍".cyan());
    }
    println!("  Query: '{}'", config.query);

    if let Some(max_size) = max_size_bytes {
        println!("  {} Máximo tamaño: {}", "📏".blue(), format_size(max_size));
    }
    if let Some(ref pattern) = config.ignore_pattern {
        println!("  {} Ignorando patrón: {}", "🚫".blue(), pattern);
    }
    if config.extract {
        println!(
            "  {} Buscando en archivos comprimidos (experimental)",
            "📦".blue()
        );
    }
    if config.interactive {
        println!("  {} Modo interactivo activado", "🎮".blue());
    }

    let total_archivos = cache.entries.len();

    if config.verbose {
        println!("{} Revisando {} archivos...", "📄".blue(), total_archivos);
    }

    let query_words = if config.semantic {
        Some(get_word_vector(&config.query))
    } else {
        None
    };

    let query_regex = if !config.semantic {
        Some(if config.ignore_case {
            Regex::new(&format!(r"(?i){}", regex::escape(&config.query)))?
        } else if config.exact {
            Regex::new(&format!(r"\b{}\b", regex::escape(&config.query)))?
        } else {
            Regex::new(&regex::escape(&config.query))?
        })
    } else {
        None
    };

    // 👇 NUEVO: Procesar archivos en paralelo con Rayon
    let encontrados = Arc::new(AtomicUsize::new(0));
    let total_ocurrencias = Arc::new(AtomicUsize::new(0));

    let query_regex_ref = query_regex.as_ref();
    let query_words_ref = query_words.as_ref();
    let ext_filter_ref = ext_filter.as_ref();
    let ignore_pattern_ref = config.ignore_pattern.as_ref();
    let query_ref = config.query.as_str();
    let ignore_case = config.ignore_case;
    let exact = config.exact;
    let extract = config.extract;
    let interactive = config.interactive;
    let semantic = config.semantic;

    let mut search_results: Vec<super::SearchResult> = cache
        .entries
        .par_iter()
        .filter_map(|(p, entry)| {
            // Filtro de ignore_pattern
            if let Some(pattern) = ignore_pattern_ref {
                if matches_pattern(p, pattern) {
                    return None;
                }
            }

            // Filtro de tamaño
            if let Some(max_size) = max_size_bytes {
                if entry.size > max_size {
                    return None;
                }
            }

            // Filtro de extensión
            let should_include = if let Some(exts) = ext_filter_ref {
                p.extension()
                    .and_then(|e| e.to_str())
                    .map(|e| exts.contains(&e))
                    .unwrap_or(false)
            } else {
                true
            };

            if !should_include {
                return None;
            }

            // Contenido a buscar (con extracción si aplica)
            let content_to_search = if extract {
                if let Ok(Some(extracted)) = extract_archive_content(p) {
                    extracted
                } else {
                    entry.content.clone()
                }
            } else {
                entry.content.clone()
            };

            // Búsqueda semántica
            if semantic {
                if let Some(q_vec) = query_words_ref {
                    let entry_vec = get_word_vector(&content_to_search);
                    let similarity = cosine_similarity(q_vec, &entry_vec);

                    if similarity > 0.15 {
                        encontrados.fetch_add(1, Ordering::SeqCst);
                        total_ocurrencias.fetch_add(1, Ordering::SeqCst);

                        let ocurrencias = entry_vec.values().sum::<u32>();
                        return Some(super::SearchResult {
                            path: p.to_path_buf(),
                            content: content_to_search.clone(),
                            matches: ocurrencias as usize,
                            size: entry.size,
                            line_start: None,
                            line_end: None,
                            score: Some(similarity),
                            keywords: entry_vec.keys().cloned().collect(),
                        });
                    }
                }
                None
            } else {
                // Búsqueda por texto
                if let Some(re) = query_regex_ref {
                    let matches: Vec<_> = re.find_iter(&content_to_search).collect();
                    if !matches.is_empty() {
                        encontrados.fetch_add(1, Ordering::SeqCst);
                        total_ocurrencias.fetch_add(matches.len(), Ordering::SeqCst);

                        let first_match = matches.first().map(|m| m.start());
                        let last_match = matches.last().map(|m| m.end());
                        let (line_start, line_end) =
                            if let (Some(start), Some(end)) = (first_match, last_match) {
                                let content_before = &content_to_search[..start];
                                let line_start = content_before.lines().count();
                                let content_until = &content_to_search[..end];
                                let line_end = content_until.lines().count();
                                (Some(line_start), Some(line_end))
                            } else {
                                (None, None)
                            };

                        Some(super::SearchResult {
                            path: p.to_path_buf(),
                            content: content_to_search.clone(),
                            matches: matches.len(),
                            size: entry.size,
                            line_start,
                            line_end,
                            score: None,
                            keywords: Vec::new(),
                        })
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
        })
        .collect();

    // Ordenar resultados por cantidad de coincidencias (descendente)
    search_results.sort_by_key(|a| std::cmp::Reverse(a.matches));

    // Mostrar resultados si NO es modo interactivo
    if !interactive {
        for result in &search_results {
            if semantic {
                println!(
                    "\n{} [Similitud: {:.2}%] ({} palabras clave) [{}]",
                    result.path.display().to_string().green(),
                    result.score.unwrap_or(0.0) * 100.0,
                    result.matches,
                    format_size(result.size).dimmed()
                );
                let preview: String = result
                    .content
                    .lines()
                    .take(3)
                    .collect::<Vec<_>>()
                    .join("\n");
                println!("  {}", preview);
            } else {
                println!(
                    "\n{} ({} coincidencias) [{}]",
                    result.path.display().to_string().green(),
                    result.matches,
                    format_size(result.size).dimmed()
                );

                if let Some(re) = query_regex_ref {
                    let lineas: Vec<String> = result
                        .content
                        .lines()
                        .enumerate()
                        .filter_map(|(num, line)| {
                            if re.is_match(line) {
                                let line_num = format!("{}:", num + 1).yellow();
                                let highlighted = if ignore_case {
                                    let re_ignore =
                                        Regex::new(&format!(r"(?i){}", regex::escape(query_ref)))
                                            .unwrap();
                                    re_ignore
                                        .replace_all(line, |caps: &regex::Captures| {
                                            caps[0].to_string().red().to_string()
                                        })
                                        .to_string()
                                } else if exact {
                                    let re_exact =
                                        Regex::new(&format!(r"\b{}\b", regex::escape(query_ref)))
                                            .unwrap();
                                    re_exact
                                        .replace_all(line, |caps: &regex::Captures| {
                                            caps[0].to_string().red().to_string()
                                        })
                                        .to_string()
                                } else {
                                    line.replace(query_ref, &query_ref.red().to_string())
                                };
                                Some(format!("  {} {}", line_num, highlighted))
                            } else {
                                None
                            }
                        })
                        .collect();

                    if !lineas.is_empty() {
                        println!("{}", lineas.join("\n"));
                    }
                }
            }
        }
    }

    if config.verbose {
        println!();
    }

    let total_encontrados = encontrados.load(Ordering::SeqCst);
    let total_matches = total_ocurrencias.load(Ordering::SeqCst);
    let elapsed = start_time.elapsed();

    if config.interactive && !search_results.is_empty() {
        let internal_results: Vec<InternalSearchResult> = search_results
            .iter()
            .map(|r| InternalSearchResult {
                path: r.path.clone(),
                content: r.content.clone(),
                matches: r.matches,
                size: r.size,
                line_start: r.line_start,
                line_end: r.line_end,
                highlight_regex: query_regex.clone(),
            })
            .collect();
        interactive_mode(&internal_results)?;
    } else if !config.interactive && total_encontrados == 0 {
        println!("{} No se encontraron coincidencias.", "⚠️".yellow());
    } else if !config.interactive {
        if config.summary {
            println!("\n{} {}", "📊".blue(), "Resumen:".bold());
            println!(
                "  {} Archivos encontrados: {}",
                "•".cyan(),
                total_encontrados
            );
            println!("  {} Coincidencias totales: {}", "•".cyan(), total_matches);
            println!("  {} Tiempo: {:.2}s", "•".cyan(), elapsed.as_secs_f32());
            println!(
                "  {} Núcleos usados: {}",
                "⚡".cyan(),
                rayon::current_num_threads()
            );
        } else {
            println!(
                "\n{} Encontrados {} archivos.",
                "✅".green(),
                total_encontrados
            );
        }
    }

    Ok(search_results)
}
