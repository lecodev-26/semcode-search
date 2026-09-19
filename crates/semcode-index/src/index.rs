//! Índice principal.
//!
//! El [`Index`] es el punto de entrada a la base de datos. Gestiona:
//!
//! - El pool de conexiones (r2d2).
//! - La aplicación de PRAGMAs en cada conexión.
//! - El sistema de migraciones.
//! - El `repo_id` asociado.
//!
//! ## Ejemplo
//!
//! ```no_run
//! use semcode_index::Index;
//!
//! let mut index = Index::open(".").unwrap();
//! index.migrate().unwrap();
//! ```

use crate::migrations;
use crate::schema::{apply_pragmas, DB_FILENAME, SEMCODE_DIR};
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::Connection;
use semcode_core::error::{Error, Result};

use std::path::{Path, PathBuf};

/// Índice de un repositorio.
///
/// Mantiene un pool de conexiones a `.semcode/index.db`. Es `Send + Sync`
/// por lo que se puede compartir entre hilos (con `Arc<Index>`).
pub struct Index {
    pool: Pool<SqliteConnectionManager>,
    repo_path: PathBuf,
    repo_id: i64,
}

impl Index {
    /// Abre el índice de un repositorio, creando el directorio si no existe.
    ///
    /// Si la base de datos no existe, la crea vacía. **No** aplica
    /// migraciones automáticamente: hay que llamar a [`Index::migrate`].
    pub fn open<P: AsRef<Path>>(repo_path: P) -> Result<Self> {
        let repo_path = repo_path.as_ref().canonicalize().map_err(|e| {
            Error::index(format!(
                "canonicalizar ruta {:?}: {}",
                repo_path.as_ref(),
                e
            ))
        })?;

        // Crear .semcode/ si no existe
        let semcode_dir = repo_path.join(SEMCODE_DIR);
        std::fs::create_dir_all(&semcode_dir).map_err(|e| {
            Error::index(format!("crear {}: {}", semcode_dir.display(), e))
        })?;

        let db_path = semcode_dir.join(DB_FILENAME);

        let manager = SqliteConnectionManager::file(&db_path).with_init(|c| {
            apply_pragmas(c)
                .map_err(|e| rusqlite::Error::SqliteFailure(
                    rusqlite::ffi::Error::new(rusqlite::ffi::SQLITE_ERROR),
                    Some(e.to_string()),
                ))
        });

        let pool = Pool::builder()
            .max_size((num_cpus::get() as u32) * 2)
            .build(manager)
            .map_err(|e| Error::index(format!("crear pool de conexiones: {}", e)))?;

        Ok(Self {
            pool,
            repo_path,
            repo_id: 0, // se asigna en migrate()
        })
    }

    /// Devuelve una conexión del pool.
    pub fn conn(&self) -> Result<r2d2::PooledConnection<SqliteConnectionManager>> {
        self.pool
            .get()
            .map_err(|e| Error::index(format!("obtener conexión del pool: {}", e)))
    }

    /// Ruta raíz del repositorio.
    pub fn repo_path(&self) -> &Path {
        &self.repo_path
    }

    /// ID del repositorio en la base de datos.
    pub fn repo_id(&self) -> i64 {
        self.repo_id
    }

    /// Aplica las migraciones pendientes y asegura el repositorio.
    ///
    /// Después de llamar a este método, el índice está listo para usarse.
    pub fn migrate(&mut self) -> Result<()> {
        let mut conn = self.conn()?;

        // Aplicar migraciones
        migrations::apply_all(&mut conn)?;

        // Asegurar que el repo existe en la tabla
        self.repo_id = self.upsert_repo(&conn)?;

        tracing::info!(
            "índice listo: repo_id={} path={}",
            self.repo_id,
            self.repo_path.display()
        );

        Ok(())
    }

    /// Inserta o actualiza el repositorio y devuelve su ID.
    fn upsert_repo(&self, conn: &Connection) -> Result<i64> {
        let path_str = self.repo_path.to_string_lossy().to_string();
        let name = self
            .repo_path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string();

        conn.execute(
            "INSERT INTO repositories (path, name, indexed_at)
             VALUES (?1, ?2, datetime('now'))
             ON CONFLICT(path) DO UPDATE SET
                name = excluded.name,
                indexed_at = excluded.indexed_at",
            rusqlite::params![path_str, name],
        )
        .map_err(|e| Error::index(format!("upsert repositorio: {}", e)))?;

        let id: i64 = conn
            .query_row(
                "SELECT id FROM repositories WHERE path = ?1",
                [&path_str],
                |r| r.get(0),
            )
            .map_err(|e| Error::index(format!("leer repo_id: {}", e)))?;

        Ok(id)
    }
}
