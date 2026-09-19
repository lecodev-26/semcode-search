//! Tipos fundamentales del dominio.
//!
//! Aquí viven los enums y structs que usan todos los demás crates:
//! lenguaje, tipo de símbolo, visibilidad, hash, etc.

use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// Lenguaje de programación detectado.
///
/// Se usa para elegir el parser adecuado (Tree-sitter) y para filtrar
/// búsquedas por lenguaje.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Language {
    /// Rust (`.rs`).
    Rust,
    /// Python (`.py`, `.pyi`).
    Python,
    /// JavaScript (`.js`, `.mjs`, `.cjs`, `.jsx`).
    JavaScript,
    /// TypeScript (`.ts`, `.tsx`).
    TypeScript,
    /// Go (`.go`).
    Go,
    /// C (`.c`, `.h`).
    C,
    /// C++ (`.cpp`, `.cc`, `.cxx`, `.hpp`, `.hxx`).
    Cpp,
    /// Java (`.java`).
    Java,
    /// Ruby (`.rb`).
    Ruby,
    /// JSON (`.json`).
    Json,
    /// TOML (`.toml`).
    Toml,
    /// YAML (`.yml`, `.yaml`).
    Yaml,
    /// Markdown (`.md`, `.markdown`).
    Markdown,
    /// Bash / shell (`.sh`, `.bash`, `.zsh`).
    Bash,
    /// HTML (`.html`, `.htm`).
    Html,
    /// CSS / SCSS (`.css`, `.scss`).
    Css,
    /// SQL (`.sql`).
    Sql,
    /// PHP (`.php`).
    Php,
    /// Swift (`.swift`).
    Swift,
    /// Kotlin (`.kt`, `.kts`).
    Kotlin,
    /// Makefile.
    Make,
    /// Dockerfile.
    Dockerfile,
    /// Lenguaje no detectado o no soportado.
    Unknown,
}

impl Language {
    /// Detecta el lenguaje a partir de la extensión del archivo (sin punto).
    ///
    /// Devuelve `None` si la extensión no se reconoce. La comparación
    /// ignora mayúsculas/minúsculas.
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_ascii_lowercase().as_str() {
            "rs" => Some(Language::Rust),
            "py" | "pyi" => Some(Language::Python),
            "js" | "mjs" | "cjs" | "jsx" => Some(Language::JavaScript),
            "ts" | "tsx" => Some(Language::TypeScript),
            "go" => Some(Language::Go),
            "c" | "h" => Some(Language::C),
            "cpp" | "cc" | "cxx" | "hpp" | "hxx" => Some(Language::Cpp),
            "java" => Some(Language::Java),
            "rb" => Some(Language::Ruby),
            "json" => Some(Language::Json),
            "toml" => Some(Language::Toml),
            "yml" | "yaml" => Some(Language::Yaml),
            "md" | "markdown" => Some(Language::Markdown),
            "sh" | "bash" | "zsh" => Some(Language::Bash),
            "html" | "htm" => Some(Language::Html),
            "css" | "scss" => Some(Language::Css),
            "sql" => Some(Language::Sql),
            "php" => Some(Language::Php),
            "swift" => Some(Language::Swift),
            "kt" | "kts" => Some(Language::Kotlin),
            _ => None,
        }
    }

    /// ¿Este lenguaje tiene soporte Tree-sitter en la v4.0.0?
    ///
    /// Los lenguajes sin parser se indexan como texto plano (chunks de
    /// ventanas), sin extracción de símbolos.
    pub fn has_parser(&self) -> bool {
        matches!(
            self,
            Language::Rust
                | Language::Python
                | Language::JavaScript
                | Language::TypeScript
                | Language::Go
                | Language::Json
        )
    }

    /// Nombre canónico en minúsculas.
    pub fn as_str(&self) -> &'static str {
        match self {
            Language::Rust => "rust",
            Language::Python => "python",
            Language::JavaScript => "javascript",
            Language::TypeScript => "typescript",
            Language::Go => "go",
            Language::C => "c",
            Language::Cpp => "cpp",
            Language::Java => "java",
            Language::Ruby => "ruby",
            Language::Json => "json",
            Language::Toml => "toml",
            Language::Yaml => "yaml",
            Language::Markdown => "markdown",
            Language::Bash => "bash",
            Language::Html => "html",
            Language::Css => "css",
            Language::Sql => "sql",
            Language::Php => "php",
            Language::Swift => "swift",
            Language::Kotlin => "kotlin",
            Language::Make => "make",
            Language::Dockerfile => "dockerfile",
            Language::Unknown => "unknown",
        }
    }
}

impl fmt::Display for Language {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for Language {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        Ok(Language::from_extension(s).unwrap_or(Language::Unknown))
    }
}

/// Tipo de símbolo extraído del AST.
///
/// Cada lenguaje mapea sus construcciones a una de estas variantes
/// (por ejemplo, `function_item` de Rust → `Function`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SymbolKind {
    /// Función libre.
    Function,
    /// Método (función asociada a un tipo o impl).
    Method,
    /// Estructura (struct, record).
    Struct,
    /// Clase.
    Class,
    /// Interfaz.
    Interface,
    /// Trait (Rust) o equivalente.
    Trait,
    /// Enum.
    Enum,
    /// Variante de un enum.
    EnumVariant,
    /// Constante.
    Constant,
    /// Variable estática.
    StaticVar,
    /// Alias de tipo.
    TypeAlias,
    /// Módulo o namespace.
    Module,
    /// Import / use / require.
    Import,
    /// Macro.
    Macro,
    /// Test (función anotada como test).
    Test,
}

impl SymbolKind {
    /// Nombre canónico en minúsculas (con `_` para variantes compuestas).
    pub fn as_str(&self) -> &'static str {
        match self {
            SymbolKind::Function => "function",
            SymbolKind::Method => "method",
            SymbolKind::Struct => "struct",
            SymbolKind::Class => "class",
            SymbolKind::Interface => "interface",
            SymbolKind::Trait => "trait",
            SymbolKind::Enum => "enum",
            SymbolKind::EnumVariant => "enum_variant",
            SymbolKind::Constant => "constant",
            SymbolKind::StaticVar => "static_var",
            SymbolKind::TypeAlias => "type_alias",
            SymbolKind::Module => "module",
            SymbolKind::Import => "import",
            SymbolKind::Macro => "macro",
            SymbolKind::Test => "test",
        }
    }
}

impl fmt::Display for SymbolKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Visibilidad de un símbolo.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Visibility {
    /// `pub` (público).
    Public,
    /// Privado (sin modificador).
    Private,
    /// `protected` (Java, C++, etc.).
    Protected,
    /// `pub(crate)` (visible dentro del crate).
    Crate,
    /// `pub(super)` / package-private (visible dentro del paquete).
    Package,
    /// Visibilidad no determinada por el parser.
    Unknown,
}

impl Default for Visibility {
    fn default() -> Self {
        Visibility::Unknown
    }
}

/// Hash BLAKE3 de 32 bytes.
///
/// Se usa para detectar cambios en archivos, símbolos y chunks sin tener que
/// recalcular embeddings ni volver a indexar.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Blake3Hash(pub [u8; 32]);

impl Blake3Hash {
    /// Calcula el hash de un slice de bytes.
    pub fn from_bytes(data: &[u8]) -> Self {
        let hash = blake3::hash(data);
        Blake3Hash(*hash.as_bytes())
    }

    /// Calcula el hash de un string.
    pub fn from_str(s: &str) -> Self {
        Self::from_bytes(s.as_bytes())
    }

    /// Devuelve el hash en hexadecimal (64 caracteres).
    pub fn to_hex(&self) -> String {
        self.0.iter().map(|b| format!("{:02x}", b)).collect()
    }
}

impl fmt::Display for Blake3Hash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.to_hex())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn language_from_extension() {
        assert_eq!(Language::from_extension("rs"), Some(Language::Rust));
        assert_eq!(Language::from_extension("RS"), Some(Language::Rust));
        assert_eq!(Language::from_extension("py"), Some(Language::Python));
        assert_eq!(Language::from_extension("tsx"), Some(Language::TypeScript));
        assert_eq!(Language::from_extension("xyz"), None);
    }

    #[test]
    fn language_has_parser() {
        assert!(Language::Rust.has_parser());
        assert!(Language::Python.has_parser());
        assert!(!Language::Html.has_parser());
        assert!(!Language::Unknown.has_parser());
    }

    #[test]
    fn blake3_hash_deterministic() {
        let h1 = Blake3Hash::from_str("hello");
        let h2 = Blake3Hash::from_str("hello");
        let h3 = Blake3Hash::from_str("world");
        assert_eq!(h1, h2);
        assert_ne!(h1, h3);
        assert_eq!(h1.to_hex().len(), 64);
    }

    #[test]
    fn symbol_kind_as_str() {
        assert_eq!(SymbolKind::Function.as_str(), "function");
        assert_eq!(SymbolKind::Struct.as_str(), "struct");
        assert_eq!(SymbolKind::EnumVariant.as_str(), "enum_variant");
    }
}
