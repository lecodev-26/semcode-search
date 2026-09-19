//! Constantes y utilidades del esquema SQLite.
//!
//! La base de datos vive en `<repo>/.semcode/index.db`. Aquí definimos:
//!
//! - Los PRAGMAs que se aplican en cada conexión.
//! - La versión actual del esquema (para migraciones).
//! - Helpers para configurar conexiones.

use rusqlite::Connection;
use semcode_core::error::{Error, Result};

/// Versión actual del esquema.
///
/// Cada vez que se añade una migración, este número sube.
pub const CURRENT_SCHEMA_VERSION: u32 = 1;

/// Nombre del directorio de datos dentro del repo.
pub const SEMCODE_DIR: &str = ".semcode";

/// Nombre del archivo de base de datos.
pub const DB_FILENAME: &str = "index.db";

/// Ruta relativa del directorio de datos dentro del repo.
pub const SEMCODE_DIR_PATH: &str = ".semcode";

/// PRAGMAs que se aplican en cada conexión.
///
/// - `journal_mode = WAL`: permite lectores y un escritor concurrentes.
/// - `synchronous = NORMAL`: balance seguridad/velocidad con WAL.
/// - `foreign_keys = ON`: integridad referencial activa.
/// - `temp_store = MEMORY`: temporales en RAM.
/// - `mmap_size`: mapea 256 MB del archivo para lecturas rápidas.
/// - `cache_size`: 64 MB de cache en memoria.
/// - `busy_timeout`: espera hasta 5s si la DB está bloqueada.
/// - `auto_vacuum = INCREMENTAL`: permite recuperar espacio.
pub const PRAGMAS: &str = r#"
PRAGMA journal_mode = WAL;
PRAGMA synchronous = NORMAL;
PRAGMA foreign_keys = ON;
PRAGMA temp_store = MEMORY;
PRAGMA mmap_size = 268435456;
PRAGMA cache_size = -64000;
PRAGMA busy_timeout = 5000;
PRAGMA auto_vacuum = INCREMENTAL;
"#;

/// Aplica los PRAGMAs a una conexión recién abierta.
///
/// Se llama desde `Index::open` y desde el `ConnectionCustomizer` del pool
/// para que **todas** las conexiones tengan la misma configuración.
pub fn apply_pragmas(conn: &Connection) -> Result<()> {
    conn.execute_batch(PRAGMAS)
        .map_err(|e| Error::index(format!("aplicar PRAGMAs: {}", e)))?;
    Ok(())
}

/// Devuelve la versión actual del esquema guardada en la DB.
///
/// Si la tabla `schema_version` no existe (DB vacía), devuelve `0`.
pub fn current_db_version(conn: &Connection) -> Result<u32> {
    // Si la tabla no existe, SQLite devuelve error. Lo tratamos como "0".
    let result = conn.query_row(
        "SELECT COALESCE(MAX(version), 0) FROM schema_version",
        [],
        |row| row.get::<_, u32>(0),
    );

    match result {
        Ok(v) => Ok(v),
        Err(rusqlite::Error::SqliteFailure(_, Some(msg)))
            if msg.contains("no such table") =>
        {
            Ok(0)
        }
        Err(e) => Err(Error::index(format!("leer versión de esquema: {}", e))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pragmas_apply_cleanly() {
        let conn = Connection::open_in_memory().expect("conexión en memoria");
        // No debe fallar al aplicar los PRAGMAs (aunque WAL no aplique en memoria)
        apply_pragmas(&conn).expect("aplicar PRAGMAs");
        // Verificar que la conexión sigue operativa
        let n: i64 = conn.query_row("SELECT 1", [], |r| r.get(0)).unwrap();
        assert_eq!(n, 1);
    }

    #[test]
    fn empty_db_has_version_zero() {
        let conn = Connection::open_in_memory().unwrap();
        let v = current_db_version(&conn).unwrap();
        assert_eq!(v, 0);
    }

    #[test]
    fn schema_version_constant_is_positive() {
        assert!(CURRENT_SCHEMA_VERSION >= 1);
    }
}
