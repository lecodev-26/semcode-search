//! Comando `search`: busca en el índice.
//!
//! Construye el motor de búsqueda, ejecuta la consulta en el modo
//! indicado y muestra los resultados por terminal (o JSON).

use colored::Colorize;
use semcode_core::error::{Error, Result};
use semcode_core::types::Language;
use semcode_embeddings::Provider;
use semcode_index::Index;
use semcode_search::{Filters, Query, SearchEngine, SearchMode, SearchResult};

use std::path::Path;
use std::sync::Arc;
use std::time::Instant;

/// Opciones del comando search.
pub struct SearchOpts<'a> {
    /// Ruta del proyecto.
    pub path: &'a Path,
    /// Consulta.
    pub query: &'a str,
    /// Modo de búsqueda (None = híbrida).
    pub mode: Option<SearchMode>,
    /// Número máximo de resultados.
    pub limit: usize,
    /// Activar reranker (feature `rerank`, aún no implementado).
    pub deep: bool,
    /// Salida en JSON.
    pub json: bool,
    /// Filtrar por lenguaje.
    pub lang: Option<Language>,
    /// Búsqueda case-sensitive.
    pub case_sensitive: bool,
}

/// Ejecuta el comando `search`.
pub fn run(opts: SearchOpts<'_>) -> Result<()> {
    let started = Instant::now();

    let canonical = opts.path.canonicalize().map_err(|e| {
        Error::search(format!("no se pudo acceder a {}: {}", opts.path.display(), e))
    })?;

    // Abrir índice
    let mut index = Index::open(&canonical)?;
    index.migrate()?;

    // Construir embedder y motor
    let embedder = Arc::new(Provider::fastembed()?);
    let engine = SearchEngine::new(Arc::new(index), embedder);

    // Construir query
    let mode = opts.mode.unwrap_or(SearchMode::Hybrid);
    let mut q = Query::new(opts.query.to_string(), mode, opts.limit);
    q.filters = Filters {
        language: opts.lang,
        case_sensitive: opts.case_sensitive,
        deep: opts.deep,
        ..Default::default()
    };

    // Ejecutar
    let results = engine.search(q)?;
    let elapsed = started.elapsed();

    // Salida
    if opts.json {
        print_json(&results)?;
    } else {
        print_pretty(&results, opts.query, elapsed.as_secs_f64())?;
    }

    Ok(())
}

/// Imprime resultados en JSON.
fn print_json(results: &[SearchResult]) -> Result<()> {
    let json = serde_json::to_string_pretty(results)
        .map_err(|e| Error::search(format!("serializar JSON: {}", e)))?;
    println!("{}", json);
    Ok(())
}

/// Imprime resultados por terminal con colores.
fn print_pretty(results: &[SearchResult], query: &str, elapsed: f64) -> Result<()> {
    if results.is_empty() {
        println!(
            "{} No se encontraron resultados para {}",
            "✗".bright_red(),
            format!("\"{}\"", query).bright_white()
        );
        return Ok(());
    }

    println!(
        "{} {} resultados para {} ({:.2}s)",
        "✓".bright_green(),
        results.len().to_string().bright_white(),
        format!("\"{}\"", query).bright_white(),
        elapsed
    );
    println!();

    for (i, r) in results.iter().enumerate() {
        let idx = format!("{}.", i + 1).dimmed();
        let path = r.path.bright_white();
        let lines = format!("{}:{}", r.start_line, r.end_line).dimmed();
        let score = format!("{:.4}", r.score).yellow();

        // Cabecera del resultado
        println!("{} {} {} [{}]", idx, path, lines, score);

        // Motores que lo encontraron
        if !r.sources.is_empty() {
            let sources: Vec<String> = r
                .sources
                .iter()
                .map(|s| s.as_str().to_string())
                .collect();
            println!("   {} {}", "fuentes:".dimmed(), sources.join(", ").cyan());
        }

        // Símbolo
        if let Some(sym) = &r.symbol {
            println!(
                "   {} {} {}",
                "símbolo:".dimmed(),
                sym.name.bright_magenta(),
                format!("({})", sym.kind.as_str()).dimmed()
            );
        }

        // Snippet (indentado)
        for line in r.snippet.lines() {
            println!("   {}", line.dimmed());
        }
        println!();
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn search_opts_construction() {
        let opts = SearchOpts {
            path: Path::new("."),
            query: "test",
            mode: Some(SearchMode::Hybrid),
            limit: 10,
            deep: false,
            json: false,
            lang: Some(Language::Rust),
            case_sensitive: false,
        };
        assert_eq!(opts.query, "test");
        assert_eq!(opts.limit, 10);
    }
}
