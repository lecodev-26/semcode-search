//! Comando `stats`: muestra estadísticas del proyecto indexado.

use colored::Colorize;
use semcode_core::error::{Error, Result};
use semcode_index::Index;

use rusqlite::OptionalExtension;
use serde::Serialize;

use std::path::Path;

/// Estadísticas del repositorio.
#[derive(Debug, Clone, Serialize)]
pub struct Stats {
    /// Ruta del proyecto.
    pub path: String,
    /// Número de archivos indexados.
    pub files: usize,
    /// Número de símbolos extraídos.
    pub symbols: usize,
    /// Número de chunks.
    pub chunks: usize,
    /// Número de embeddings generados.
    pub embeddings: usize,
    /// Tamaño total de la base de datos (bytes).
    pub db_size: u64,
}

/// Ejecuta el comando `stats`.
pub fn run(path: &Path, json: bool) -> Result<()> {
    let canonical = path.canonicalize().map_err(|e| {
        Error::index(format!("no se pudo acceder a {}: {}", path.display(), e))
    })?;

    let mut index = Index::open(&canonical)?;
    index.migrate()?;

    let stats = collect_stats(&index, &canonical)?;

    if json {
        let s = serde_json::to_string_pretty(&stats)
            .map_err(|e| Error::index(format!("serializar stats: {}", e)))?;
        println!("{}", s);
    } else {
        print_pretty(&stats);
    }

    Ok(())
}

/// Recoge las estadísticas de la DB.
fn collect_stats(index: &Index, path: &Path) -> Result<Stats> {
    let conn = index.conn()?;
    let repo_id = index.repo_id();

    let files = count_query(&conn, "SELECT COUNT(*) FROM files WHERE repo_id = ?1", repo_id)?;
    let symbols = count_query(
        &conn,
        "SELECT COUNT(*) FROM symbols s JOIN files f ON f.id = s.file_id WHERE f.repo_id = ?1",
        repo_id,
    )?;
    let chunks = count_query(
        &conn,
        "SELECT COUNT(*) FROM chunks c JOIN files f ON f.id = c.file_id WHERE f.repo_id = ?1",
        repo_id,
    )?;
    let embeddings = count_query_no_param(&conn, "SELECT COUNT(*) FROM embeddings")?;

    // Tamaño del archivo index.db
    let db_path = path.join(".semcode/index.db");
    let db_size = std::fs::metadata(&db_path).map(|m| m.len()).unwrap_or(0);

    Ok(Stats {
        path: path.display().to_string(),
        files,
        symbols,
        chunks,
        embeddings,
        db_size,
    })
}

/// Ejecuta un COUNT con un solo parámetro i64.
fn count_query(
    conn: &rusqlite::Connection,
    sql: &str,
    param: i64,
) -> Result<usize> {
    let n: i64 = conn
        .query_row(sql, [param], |r| r.get(0))
        .optional()
        .map_err(|e| Error::index(format!("query: {}", e)))?
        .unwrap_or(0);
    Ok(n as usize)
}

/// Ejecuta un COUNT sin parámetros.
fn count_query_no_param(
    conn: &rusqlite::Connection,
    sql: &str,
) -> Result<usize> {
    let n: i64 = conn
        .query_row(sql, [], |r| r.get(0))
        .optional()
        .map_err(|e| Error::index(format!("query: {}", e)))?
        .unwrap_or(0);
    Ok(n as usize)
}

/// Imprime las estadísticas por terminal.
fn print_pretty(s: &Stats) {
    println!("{} Estadísticas de {}", "→".bright_cyan(), s.path.bright_white());
    println!();
    println!("  {} {}", "Archivos:".dimmed(), s.files.to_string().bright_white());
    println!("  {} {}", "Símbolos:".dimmed(), s.symbols.to_string().bright_white());
    println!("  {} {}", "Chunks:".dimmed(), s.chunks.to_string().bright_white());
    println!(
        "  {} {}",
        "Embeddings:".dimmed(),
        s.embeddings.to_string().bright_white()
    );
    println!(
        "  {} {}",
        "Tamaño DB:".dimmed(),
        format_bytes(s.db_size).bright_white()
    );
}

/// Formatea bytes a un string legible (KB/MB/GB).
fn format_bytes(b: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if b >= GB {
        format!("{:.2} GB", b as f64 / GB as f64)
    } else if b >= MB {
        format!("{:.2} MB", b as f64 / MB as f64)
    } else if b >= KB {
        format!("{:.2} KB", b as f64 / KB as f64)
    } else {
        format!("{} B", b)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn format_bytes_basic() {
        assert_eq!(format_bytes(512), "512 B");
        assert_eq!(format_bytes(1024), "1.00 KB");
        assert_eq!(format_bytes(1024 * 1024), "1.00 MB");
        assert_eq!(format_bytes(1024 * 1024 * 1024), "1.00 GB");
    }

    #[test]
    fn stats_on_empty_project() {
        let dir = tempdir().unwrap();
        let mut index = Index::open(dir.path()).unwrap();
        index.migrate().unwrap();
        let stats = collect_stats(&index, dir.path()).unwrap();
        assert_eq!(stats.files, 0);
        assert_eq!(stats.symbols, 0);
        assert_eq!(stats.chunks, 0);
    }
}
