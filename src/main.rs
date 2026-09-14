//! # semcode-search - Punto de entrada de la aplicaciÃ³n

use clap::Parser;
use colored::*;
use semcode_search::cli::{AliasAction, Cli, Commands, HistoryAction};
use semcode_search::core::{search_files, SearchConfigInternal};
use semcode_search::history::History;
use semcode_search::i18n::t;
use semcode_search::watch::Watcher;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;

// ===== ConfiguraciÃ³n =====

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Config {
    pub default_ext: Option<Vec<String>>,
    pub default_ignore: Option<Vec<String>>,
    pub default_ignore_pattern: Option<String>,
    pub max_size: Option<String>,
    pub verbose: Option<bool>,
    pub interactive: Option<bool>,
    pub aliases: HashMap<String, AliasEntry>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AliasEntry {
    pub query: String,
    pub params: Vec<String>,
}

pub fn get_config_path() -> PathBuf {
    if let Some(config_dir) = dirs::config_dir() {
        config_dir.join("semcode-search").join("config.toml")
    } else {
        PathBuf::from(".semcode-search-config.toml")
    }
}

pub fn load_config() -> anyhow::Result<Config> {
    let config_path = get_config_path();
    if config_path.exists() {
        let content = fs::read_to_string(&config_path)?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
    } else {
        Ok(Config::default())
    }
}

pub fn save_config(config: &Config) -> anyhow::Result<()> {
    let config_path = get_config_path();
    if let Some(parent) = config_path.parent() {
        fs::create_dir_all(parent)?;
    }
    let content = toml::to_string_pretty(config)?;
    fs::write(config_path, content)?;
    Ok(())
}

type AppliedConfig = (
    Option<Vec<String>>,
    Vec<String>,
    bool,
    bool,
    Option<String>,
    Option<String>,
);

fn apply_config_to_search(
    ext: Option<String>,
    ignore: String,
    verbose: bool,
    interactive: bool,
    max_size: Option<String>,
    ignore_pattern: Option<String>,
    config: &Config,
) -> AppliedConfig {
    let mut ext_vec = ext
        .as_ref()
        .map(|e| e.split(',').map(|s| s.to_string()).collect());
    let mut ignore_vec: Vec<String> = ignore.split(',').map(|s| s.to_string()).collect();
    let mut new_verbose = verbose;
    let mut new_interactive = interactive;
    let mut new_max_size = max_size;
    let mut new_ignore_pattern = ignore_pattern;

    if let Some(ref default_ext) = config.default_ext {
        if ext_vec.is_none() {
            ext_vec = Some(default_ext.clone());
        }
    }
    if let Some(ref default_ignore) = config.default_ignore {
        ignore_vec = default_ignore.clone();
    }
    if config.verbose.unwrap_or(false) && !verbose {
        new_verbose = true;
    }
    if config.interactive.unwrap_or(false) && !interactive {
        new_interactive = true;
    }
    if new_max_size.is_none() {
        new_max_size = config.max_size.clone();
    }
    if new_ignore_pattern.is_none() {
        new_ignore_pattern = config.default_ignore_pattern.clone();
    }
    (
        ext_vec,
        ignore_vec,
        new_verbose,
        new_interactive,
        new_max_size,
        new_ignore_pattern,
    )
}

/// Muestra un mensaje de bienvenida cuando no se pasan argumentos
fn show_welcome() {
    let tr = t();

    println!();
    println!("{}", "â•".repeat(60).cyan());
    println!("  {}", tr.welcome_title.bold().cyan());
    println!("  {}", format!("v{}", env!("CARGO_PKG_VERSION")).green());
    println!("{}", "â•".repeat(60).cyan());
    println!();
    println!("  {}", tr.welcome_message);
    println!();
    println!("{}", "â”€".repeat(60).dimmed());
    println!();
    println!("  {}:", tr.more_info.green());
    println!("    {}  semcode-search --help", "â†’".dimmed());
    println!();
    println!("  {}:", tr.languages.green());
    println!("    {}  semcode-search index --path .", "â†’".dimmed());
    println!(
        "    {}  semcode-search search --query \"fn\" --path .",
        "â†’".dimmed()
    );
    println!("    {}  semcode-search stats", "â†’".dimmed());
    println!("    {}  semcode-search watch --path .", "â†’".dimmed());
    println!();
    println!("{}", "â”€".repeat(60).dimmed());
    println!();
    println!("  ðŸ”— https://github.com/lecodev-26/semcode-search");
    println!();
    println!("{}", "â•".repeat(60).cyan());
    println!();
    print!("  {} ", tr.press_enter.yellow());
    io::stdout().flush().unwrap();
    let mut input = String::new();
    let _ = io::stdin().read_line(&mut input);
}

fn main() -> anyhow::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() <= 1 {
        show_welcome();
        return Ok(());
    }

    let cli = Cli::parse();
    let config = load_config().unwrap_or_default();
    let tr = t();

    match cli.command {
        Commands::Init { force } => {
            let config_path = get_config_path();
            if config_path.exists() && !force {
                println!("{}", tr.error_config_exists.yellow());
                return Ok(());
            }
            let default_config = Config {
                default_ext: Some(vec!["rs".to_string(), "md".to_string(), "toml".to_string()]),
                default_ignore: Some(vec![
                    ".git".to_string(),
                    "target".to_string(),
                    "node_modules".to_string(),
                ]),
                default_ignore_pattern: Some("*.log".to_string()),
                max_size: Some("1MB".to_string()),
                verbose: Some(true),
                interactive: Some(false),
                aliases: HashMap::new(),
            };
            save_config(&default_config)?;
            println!("{} {}", tr.config_created.green(), config_path.display());
            println!("{}", tr.config_hint.cyan());
        }

        Commands::Alias(action) => {
            let mut config = load_config()?;
            match action {
                AliasAction::Save { name, query, params } => {
                    config.aliases.insert(name.clone(), AliasEntry { query, params });
                    save_config(&config)?;
                    println!("{}: {}", tr.alias_saved.green(), name);
                }
                AliasAction::List => {
                    if config.aliases.is_empty() {
                        println!("{}", tr.alias_empty.yellow());
                    } else {
                        println!("{}:", tr.alias_list.blue());
                        for (name, entry) in &config.aliases {
                            println!(
                                "  {}: {} {}",
                                name.green(),
                                entry.query,
                                entry.params.join(" ")
                            );
                        }
                    }
                }
                AliasAction::Remove { name } => {
                    if config.aliases.remove(&name).is_some() {
                        save_config(&config)?;
                        println!("{}: {}", tr.alias_removed.green(), name);
                    } else {
                        println!("{}: {}", tr.alias_not_found.yellow(), name);
                    }
                }
                AliasAction::Run { name } => {
                    if let Some(entry) = config.aliases.get(&name) {
                        let params_str = entry.params.join(" ");
                        println!("{}: {}", tr.alias_running.cyan(), name);
                        println!("  query: {}", entry.query);
                        println!("  params: {}", params_str);
                        println!(
                            "ðŸ’¡ Para ejecutar manualmente: semcode-search search --query \"{}\" {}",
                            entry.query, params_str
                        );
                    } else {
                        println!("{}: {}", tr.alias_not_found.yellow(), name);
                    }
                }
            }
        }

        Commands::Stats => {
            show_stats()?;
        }

        Commands::History(action) => match action {
            HistoryAction::List { limit } => {
                show_history(limit)?;
            }
            HistoryAction::Clear => {
                History::clear()?;
                println!("{}", tr.history_cleared.green());
            }
            HistoryAction::Last => {
                if let Some(entry) = History::last()? {
                    println!("{}:", tr.last_search.cyan());
                    println!("  Query: {}", entry.query.green());
                    println!("  Fecha: {}", entry.timestamp.dimmed());
                    println!(
                        "\nðŸ’¡ Para repetir: semcode-search search --query \"{}\"",
                        entry.query
                    );
                } else {
                    println!("{}", tr.history_empty.yellow());
                }
            }
        },

        Commands::Index {
            path,
            ignore,
            force,
            ext,
            ignore_pattern,
            ai,
        } => {
            let ignore_dirs: Vec<&str> = ignore.split(',').collect();
            let ext_filter = ext.as_ref().map(|e| e.split(',').collect());

            #[cfg(feature = "ai")]
            {
                if ai {
                    semcode_search::core::index_files_with_ai(
                        &path,
                        ignore_dirs,
                        ext_filter,
                        ignore_pattern.as_deref(),
                        force,
                    )?;
                    return Ok(());
                }
            }

            #[cfg(not(feature = "ai"))]
            {
                if ai {
                    println!(
                        "{} El flag --ai requiere compilar con: cargo build --release --features ai",
                        "âš ï¸".yellow()
                    );
                    return Ok(());
                }
            }

            semcode_search::core::index_files(
                &path,
                ignore_dirs,
                ext_filter,
                ignore_pattern.as_deref(),
                force,
            )?;
        }

        Commands::Watch {
            path,
            ignore,
            interval,
        } => {
            let ignore_dirs: Vec<String> = ignore.split(',').map(|s| s.to_string()).collect();
            let mut watcher = Watcher::new(&path, ignore_dirs, interval);
            watcher.run()?;
        }

        Commands::Serve { port, host } => {
            #[cfg(feature = "server")]
            {
                let rt = tokio::runtime::Runtime::new()?;
                rt.block_on(async { semcode_search::run_server(&host, port).await })?;
            }
            #[cfg(not(feature = "server"))]
            {
                let _ = (port, host);
                println!(
                    "{} El comando 'serve' requiere compilar con: cargo build --release --features server",
                    "âš ï¸".yellow()
                );
            }
        }

        Commands::Tui { path } => {
            #[cfg(feature = "tui")]
            {
                semcode_search::run_tui(&path)?;
            }
            #[cfg(not(feature = "tui"))]
            {
                let _ = path;
                println!(
                    "{} El comando 'tui' requiere compilar con: cargo build --release --features tui",
                    "âš ï¸".yellow()
                );
            }
        }

        Commands::Search {
            query,
            path,
            ext,
            ignore,
            exact,
            ignore_case,
            verbose,
            no_cache,
            update: _,
            semantic,
            ai,
            file,
            summary,
            max_size,
            ignore_pattern,
            extract,
            interactive,
            alias,
        } => {
            let (query, ext_vec, ignore_vec, verbose, interactive, max_size, ignore_pattern) =
                if let Some(alias_name) = alias {
                    let config = load_config()?;
                    if let Some(entry) = config.aliases.get(&alias_name) {
                        let q = entry.query.clone();
                        let params = entry.params.clone();
                        let mut new_ext = ext;
                        let mut new_ignore = ignore;
                        let mut new_verbose = verbose;
                        let mut new_interactive = interactive;
                        let mut new_max_size = max_size;
                        let mut new_ignore_pattern = ignore_pattern;
                        for param in &params {
                            if param.starts_with("--ext") || param.starts_with("-e") {
                                if let Some(ext_value) = params
                                    .iter()
                                    .find(|p| p.starts_with("--ext") || p.starts_with("-e"))
                                    .and_then(|p| p.split('=').nth(1))
                                {
                                    new_ext = Some(ext_value.to_string());
                                }
                            }
                            if param == "--verbose" || param == "-v" {
                                new_verbose = true;
                            }
                            if param == "--interactive" {
                                new_interactive = true;
                            }
                            if param.starts_with("--max-size") {
                                if let Some(size_value) = params
                                    .iter()
                                    .find(|p| p.starts_with("--max-size"))
                                    .and_then(|p| p.split('=').nth(1))
                                {
                                    new_max_size = Some(size_value.to_string());
                                }
                            }
                            if param.starts_with("--ignore-pattern") {
                                if let Some(pattern_value) = params
                                    .iter()
                                    .find(|p| p.starts_with("--ignore-pattern"))
                                    .and_then(|p| p.split('=').nth(1))
                                {
                                    new_ignore_pattern = Some(pattern_value.to_string());
                                }
                            }
                            if param.starts_with("--ignore") || param.starts_with("-i") {
                                if let Some(ignore_value) = params
                                    .iter()
                                    .find(|p| p.starts_with("--ignore") || p.starts_with("-i"))
                                    .and_then(|p| p.split('=').nth(1))
                                {
                                    new_ignore = ignore_value.to_string();
                                }
                            }
                        }
                        let (ext_vec, ignore_vec, v, i, max, pat) = apply_config_to_search(
                            new_ext,
                            new_ignore,
                            new_verbose,
                            new_interactive,
                            new_max_size,
                            new_ignore_pattern,
                            &config,
                        );
                        (Some(q), ext_vec, ignore_vec, v, i, max, pat)
                    } else {
                        println!("{}: {}", tr.alias_not_found.yellow(), alias_name);
                        return Ok(());
                    }
                } else if let Some(q) = query {
                    let (ext_vec, ignore_vec, v, i, max, pat) = apply_config_to_search(
                        ext,
                        ignore,
                        verbose,
                        interactive,
                        max_size,
                        ignore_pattern,
                        &config,
                    );
                    (Some(q), ext_vec, ignore_vec, v, i, max, pat)
                } else {
                    println!("{}", tr.error_no_query.yellow());
                    return Ok(());
                };

            let query_str = match query {
                Some(q) => q,
                None => {
                    println!("âš ï¸ Error: no se pudo obtener la query.");
                    return Ok(());
                }
            };

            let final_query = if query_str == "!!" {
                if let Some(last) = History::last()? {
                    println!("ðŸ”„ Repitiendo Ãºltima bÃºsqueda: '{}'", last.query);
                    last.query
                } else {
                    println!("âš ï¸ No hay bÃºsquedas anteriores.");
                    return Ok(());
                }
            } else {
                query_str
            };

            let _ = History::add(&final_query);

            let search_config = SearchConfigInternal {
                query: final_query,
                path,
                ext: ext_vec,
                ignore: ignore_vec,
                exact,
                ignore_case,
                verbose,
                no_cache,
                semantic,
                ai,
                file,
                summary,
                max_size,
                ignore_pattern,
                extract,
                interactive,
            };

            #[cfg(feature = "ai")]
            {
                if ai {
                    semcode_search::core::search_files_with_ai(search_config)?;
                    return Ok(());
                }
            }

            #[cfg(not(feature = "ai"))]
            {
                if ai {
                    println!(
                        "{} El flag --ai requiere compilar con: cargo build --release --features ai",
                        "âš ï¸".yellow()
                    );
                    return Ok(());
                }
            }

            search_files(search_config)?;
        }
    }

    Ok(())
}

// ===== FunciÃ³n para mostrar estadÃ­sticas =====

fn show_stats() -> anyhow::Result<()> {
    use semcode_search::Cache;
    use std::path::Path;

    let tr = t();
    let cache_path = Path::new(".semantic-index.json");

    if !cache_path.exists() {
        println!("{}", tr.error_cache_not_found.yellow());
        return Ok(());
    }

    let cache = Cache::load(cache_path)?;

    let total_files = cache.entries.len();
    let total_size: u64 = cache.entries.values().map(|e| e.size).sum();
    let total_lines: usize = cache
        .entries
        .values()
        .map(|e| e.content.lines().count())
        .sum();

    let mut extensions: std::collections::HashMap<String, (usize, u64)> =
        std::collections::HashMap::new();
    for entry in cache.entries.values() {
        if let Some(ext) = entry.path.extension().and_then(|e| e.to_str()) {
            let counter = extensions.entry(ext.to_string()).or_insert((0, 0));
            counter.0 += 1;
            counter.1 += entry.size;
        }
    }

    println!("\n{}", tr.stats_title.bold());
    println!("{}", "â”€".repeat(50).dimmed());
    println!(
        "  {}: {}",
        tr.files_indexed.cyan(),
        total_files.to_string().green()
    );
    println!(
        "  {}: {}",
        tr.total_lines.cyan(),
        total_lines.to_string().green()
    );
    println!(
        "  {}: {}",
        tr.total_size.cyan(),
        format_size_bytes(total_size).green()
    );
    println!("\n  {}:", tr.languages.cyan());

    let mut sorted_exts: Vec<_> = extensions.iter().collect();
    sorted_exts.sort_by_key(|a| std::cmp::Reverse(a.1 .0));

    let total_for_pct = total_files as f32;
    for (ext, (count, size)) in sorted_exts {
        let pct = (*count as f32 / total_for_pct) * 100.0;
        println!(
            "    â”œâ”€â”€ {}: {} archivos ({:.0}%) [{}]",
            ext.green(),
            count.to_string().yellow(),
            pct,
            format_size_bytes(*size).dimmed()
        );
    }

    println!("\n  {}: {}", tr.last_indexed.cyan(), cache.updated.dimmed());

    if cache.has_embeddings {
        let with_emb = cache.count_with_embeddings();
        println!(
            "  ðŸ§  Con embeddings: {}/{} archivos",
            with_emb.to_string().green(),
            total_files.to_string().dimmed()
        );
    }

    println!("{}", "â”€".repeat(50).dimmed());

    Ok(())
}

// ===== FunciÃ³n para mostrar historial =====

fn show_history(limit: usize) -> anyhow::Result<()> {
    let tr = t();
    let history = History::list()?;

    if history.is_empty() {
        println!("{}", tr.history_empty.yellow());
        return Ok(());
    }

    println!("\n{}", tr.history_title.bold());
    println!("{}", "â”€".repeat(50).dimmed());

    for (i, entry) in history.iter().take(limit).enumerate() {
        println!(
            "  {}. {} {}",
            (i + 1).to_string().yellow(),
            entry.query.green(),
            format!("({})", entry.timestamp).dimmed()
        );
    }

    println!("{}", "â”€".repeat(50).dimmed());
    println!(
        "  {}: {} bÃºsquedas",
        tr.history_total.cyan(),
        history.len().to_string().green()
    );

    Ok(())
}

fn format_size_bytes(bytes: u64) -> String {
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