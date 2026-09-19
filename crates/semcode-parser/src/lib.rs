//! # semcode-parser
//!
//! Parser de código fuente basado en Tree-sitter.
//!
//! Extrae símbolos (funciones, structs, clases, métodos, imports, ...) del
//! AST y los devuelve en un formato común ([`ParsedSymbol`]) que el resto
//! de crates consume.
//!
//! ## Lenguajes soportados (v4.0.0)
//!
//! - Rust
//! - Python
//! - JavaScript
//! - TypeScript
//! - Go
//! - JSON
//!
//! ## Ejemplo
//!
//! ```
//! use semcode_parser::CodeParser;
//! use semcode_core::types::Language;
//!
//! let mut parser = CodeParser::new().unwrap();
//! let src = "pub fn hello() -> &'static str { \"hi\" }";
//! let symbols = parser.parse(src, Language::Rust).unwrap();
//!
//! assert!(symbols.iter().any(|s| s.name == "hello"));
//! ```

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod extractor;
pub mod language;
pub mod parser;

pub use language::detect;
pub use parser::CodeParser;
