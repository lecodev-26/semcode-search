//! Tipos de error y Result para todo el proyecto.
//!
//! Centralizamos los errores en `Error` para que todos los crates usen el
//! mismo tipo de retorno y se puedan componer fácilmente con `?`.

use thiserror::Error;

/// Error principal de semcode-search.
///
/// Cada variante corresponde a una capa del sistema. Los crates superiores
/// pueden envolver errores de crates inferiores usando `#[from]`.
#[derive(Debug, Error)]
pub enum Error {
    /// Errores de entrada/salida (archivos, red).
    #[error("error de E/S: {0}")]
    Io(#[from] std::io::Error),

    /// Errores del parser de código.
    #[error("error de parseo: {0}")]
    Parse(String),

    /// Errores del sistema de indexación / base de datos.
    #[error("error de índice: {0}")]
    Index(String),

    /// Errores del sistema de embeddings.
    #[error("error de embeddings: {0}")]
    Embedding(String),

    /// Errores de búsqueda.
    #[error("error de búsqueda: {0}")]
    Search(String),

    /// Un recurso no fue encontrado.
    #[error("no encontrado: {0}")]
    NotFound(String),

    /// Una entrada del usuario es inválida.
    #[error("entrada inválida: {0}")]
    InvalidInput(String),

    /// Un lenguaje no está soportado.
    #[error("lenguaje no soportado: {0}")]
    UnsupportedLanguage(String),

    /// Error genérico con mensaje.
    #[error("{0}")]
    Other(String),
}

/// Alias de `Result` con nuestro `Error`.
pub type Result<T> = std::result::Result<T, Error>;

/// Extensiones útiles sobre `Result`.
impl Error {
    /// Crea un error de parseo desde cualquier cosa convertible a String.
    pub fn parse<S: Into<String>>(msg: S) -> Self {
        Error::Parse(msg.into())
    }

    /// Crea un error de índice desde cualquier cosa convertible a String.
    pub fn index<S: Into<String>>(msg: S) -> Self {
        Error::Index(msg.into())
    }

    /// Crea un error de búsqueda desde cualquier cosa convertible a String.
    pub fn search<S: Into<String>>(msg: S) -> Self {
        Error::Search(msg.into())
    }

    /// Crea un error "no encontrado" desde cualquier cosa convertible a String.
    pub fn not_found<S: Into<String>>(msg: S) -> Self {
        Error::NotFound(msg.into())
    }

    /// Crea un error de entrada inválida desde cualquier cosa convertible a String.
    pub fn invalid<S: Into<String>>(msg: S) -> Self {
        Error::InvalidInput(msg.into())
    }
}
