//! Comando `init`: inicializa un proyecto.
//!
//! Crea el directorio `.semcode/` con una base de datos vacía y las
//! migraciones aplicadas.

use colored::Colorize;
use semcode_core::error::Result;
use semcode_index::Index;

use std::path::Path;

/// Ejecuta el comando `init`.
pub fn run(path: &Path) -> Result<()> {
    let canonical = path.canonicalize().map_err(|e| {
        semcode_core::error::Error::index(format!(
            "no se pudo acceder a {}: {}",
            path.display(),
            e
        ))
    })?;

    println!(
        "{} Inicializando proyecto en {}",
        "→".bright_cyan(),
        canonical.display().to_string().bright_white()
    );

    let mut index = Index::open(&canonical)?;
    index.migrate()?;

    println!(
        "{} Proyecto inicializado correctamente",
        "✓".bright_green()
    );
    println!(
        "  {} {}",
        "Directorio:".dimmed(),
        canonical.join(".semcode").display()
    );
    println!(
        "  {} {}",
        "Próximo paso:".dimmed(),
        format!("semcode-search index --path {}", canonical.display()).bright_white()
    );

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn init_creates_semcode_dir() {
        let dir = tempdir().unwrap();
        run(dir.path()).unwrap();
        assert!(dir.path().join(".semcode").exists());
        assert!(dir.path().join(".semcode/index.db").exists());
    }

    #[test]
    fn init_is_idempotent() {
        let dir = tempdir().unwrap();
        run(dir.path()).unwrap();
        run(dir.path()).unwrap(); // segunda vez no debe fallar
        assert!(dir.path().join(".semcode/index.db").exists());
    }
}
