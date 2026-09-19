//! Sistema de migraciones versionadas.
//!
//! Cada migración es un archivo `.sql` embebido con `include_str!`. Se
//! aplican en orden, una sola vez, y se registran en la tabla
//! `schema_version`.
//!
//! ## Reglas
//!
//! - **Nunca editar** una migración ya aplicada.
//! - Cada migración nueva sube `CURRENT_SCHEMA_VERSION` en `schema.rs`.
//! - Si una migración falla, se hace rollback de esa migración (transacción).

use crate::schema::current_db_version;
use rusqlite::Connection;
use semcode_core::error::{Error, Result};

/// Una migración concreta.
pub struct Migration {
    /// Número de versión (correlativo, empezando en 1).
    pub version: u32,
    /// Descripción corta.
    pub description: &'static str,
    /// Contenido SQL.
    pub sql: &'static str,
}

/// Lista completa de migraciones, en orden.
pub const MIGRATIONS: &[Migration] = &[Migration {
    version: 1,
    description: "esquema inicial: repos, files, symbols, chunks, embeddings, fts",
    sql: include_str!("001_initial.sql"),
}];

/// Aplica todas las migraciones pendientes.
///
/// Lee la versión actual de la DB, y aplica las migraciones con versión
/// mayor. Cada migración se aplica dentro de una transacción.
pub fn apply_all(conn: &mut Connection) -> Result<()> {
    let current = current_db_version(conn)?;

    for migration in MIGRATIONS.iter().filter(|m| m.version > current) {
        apply_one(conn, migration)?;
    }

    Ok(())
}

/// Aplica una migración concreta dentro de una transacción.
fn apply_one(conn: &mut Connection, migration: &Migration) -> Result<()> {
    let tx = conn
        .transaction()
        .map_err(|e| Error::index(format!("iniciar transacción: {}", e)))?;

    tx.execute_batch(migration.sql)
        .map_err(|e| {
            Error::index(format!(
                "aplicar migración v{} ({}): {}",
                migration.version, migration.description, e
            ))
        })?;

    tx.execute(
        "INSERT INTO schema_version(version) VALUES (?)",
        [migration.version],
    )
    .map_err(|e| Error::index(format!("registrar migración v{}: {}", migration.version, e)))?;

    tx.commit()
        .map_err(|e| Error::index(format!("commit migración v{}: {}", migration.version, e)))?;

    tracing::info!(
        "migración v{} aplicada: {}",
        migration.version,
        migration.description
    );

    Ok(())
}

/// Devuelve el número de migraciones disponibles.
pub fn count() -> usize {
    MIGRATIONS.len()
}

/// Devuelve la versión más alta disponible en el código.
pub fn latest_version() -> u32 {
    MIGRATIONS.iter().map(|m| m.version).max().unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn migrations_are_ordered() {
        let mut last = 0;
        for m in MIGRATIONS {
            assert!(
                m.version > last,
                "las migraciones deben estar ordenadas y sin huecos: v{} después de v{}",
                m.version,
                last
            );
            last = m.version;
        }
    }

    #[test]
    fn latest_version_matches_count() {
        assert_eq!(latest_version() as usize, count());
    }
}
