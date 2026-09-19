//! Operaciones sobre la tabla `symbols`.

use semcode_core::error::{Error, Result};
use semcode_core::traits::ParsedSymbol;
use semcode_core::types::{Language, SymbolKind, Visibility};

use rusqlite::{params, Connection};

/// Símbolo persistido (con ID asignado).
#[derive(Debug, Clone, PartialEq)]
pub struct StoredSymbol {
    /// ID interno.
    pub id: i64,

    /// ID del archivo al que pertenece.
    pub file_id: i64,

    /// ID del símbolo padre (si es método anidado).
    pub parent_id: Option<i64>,

    /// Nombre simple.
    pub name: String,

    /// Nombre cualificado.
    pub qualified_name: String,

    /// Tipo del símbolo.
    pub kind: SymbolKind,

    /// Lenguaje.
    pub language: Language,

    /// Línea inicial.
    pub start_line: usize,

    /// Línea final.
    pub end_line: usize,

    /// Firma (si existe).
    pub signature: Option<String>,

    /// Comentario de doc (si existe).
    pub doc_comment: Option<String>,

    /// Visibilidad.
    pub visibility: Visibility,

    /// ¿Es async?
    pub is_async: bool,

    /// ¿Es unsafe?
    pub is_unsafe: bool,

    /// ¿Es exportado/público?
    pub is_exported: bool,
}

/// Inserta un lote de símbolos para un archivo.
///
/// Reemplaza los símbolos existentes del archivo. Se hace dentro de una
/// transacción para eficiencia.
pub fn insert_symbols(
    conn: &mut Connection,
    file_id: i64,
    language: Language,
    symbols: &[ParsedSymbol],
) -> Result<Vec<i64>> {
    let tx = conn
        .transaction()
        .map_err(|e| Error::index(format!("iniciar transacción symbols: {}", e)))?;

    // Borrar símbolos antiguos del archivo
    tx.execute("DELETE FROM symbols WHERE file_id = ?1", [file_id])
        .map_err(|e| Error::index(format!("borrar símbolos antiguos: {}", e)))?;

    let mut ids = Vec::with_capacity(symbols.len());

    for sym in symbols {
        tx.execute(
            "INSERT INTO symbols (
                file_id, parent_id, name, qualified_name, kind, language,
                start_line, end_line, start_byte, end_byte,
                signature, doc_comment, visibility,
                is_async, is_unsafe, is_exported
             ) VALUES (
                ?1, NULL, ?2, ?3, ?4, ?5,
                ?6, ?7, ?8, ?9,
                ?10, ?11, ?12,
                ?13, ?14, ?15
             )",
            params![
                file_id,
                sym.name,
                sym.qualified_name,
                sym.kind.as_str(),
                language.as_str(),
                sym.start_line as i64,
                sym.end_line as i64,
                sym.start_byte as i64,
                sym.end_byte as i64,
                sym.signature,
                sym.doc_comment,
                "unknown", // visibility por ahora
                0i32,      // is_async
                0i32,      // is_unsafe
                0i32,      // is_exported
            ],
        )
        .map_err(|e| Error::index(format!("insertar símbolo {}: {}", sym.name, e)))?;

        ids.push(tx.last_insert_rowid());
    }

    tx.commit()
        .map_err(|e| Error::index(format!("commit symbols: {}", e)))?;

    Ok(ids)
}

/// Cuenta los símbolos de un archivo.
pub fn count_by_file(conn: &Connection, file_id: i64) -> Result<usize> {
    let n: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM symbols WHERE file_id = ?1",
            [file_id],
            |r| r.get(0),
        )
        .map_err(|e| Error::index(format!("contar símbolos: {}", e)))?;
    Ok(n as usize)
}

/// Busca símbolos por nombre (match parcial).
pub fn find_by_name(conn: &Connection, name: &str, limit: usize) -> Result<Vec<StoredSymbol>> {
    let pattern = format!("%{}%", name);

    let mut stmt = conn
        .prepare(
            "SELECT id, file_id, parent_id, name, qualified_name, kind, language,
                    start_line, end_line, signature, doc_comment, visibility,
                    is_async, is_unsafe, is_exported
             FROM symbols
             WHERE name LIKE ?1 OR qualified_name LIKE ?1
             ORDER BY
                CASE WHEN name = ?2 THEN 0
                     WHEN name LIKE ?3 THEN 1
                     ELSE 2
                END,
                name
             LIMIT ?4",
        )
        .map_err(|e| Error::index(format!("preparar find_by_name: {}", e)))?;

    let prefix = format!("{}%", name);
    let rows = stmt
        .query_map(params![pattern, name, prefix, limit as i64], row_to_symbol)
        .map_err(|e| Error::index(format!("find_by_name: {}", e)))?;

    let mut out = Vec::new();
    for r in rows {
        out.push(r.map_err(|e| Error::index(format!("leer símbolo: {}", e)))?);
    }
    Ok(out)
}

/// Lista todos los símbolos de un archivo.
pub fn list_by_file(conn: &Connection, file_id: i64) -> Result<Vec<StoredSymbol>> {
    let mut stmt = conn
        .prepare(
            "SELECT id, file_id, parent_id, name, qualified_name, kind, language,
                    start_line, end_line, signature, doc_comment, visibility,
                    is_async, is_unsafe, is_exported
             FROM symbols
             WHERE file_id = ?1
             ORDER BY start_line",
        )
        .map_err(|e| Error::index(format!("preparar list_by_file: {}", e)))?;

    let rows = stmt
        .query_map([file_id], row_to_symbol)
        .map_err(|e| Error::index(format!("list_by_file: {}", e)))?;

    let mut out = Vec::new();
    for r in rows {
        out.push(r.map_err(|e| Error::index(format!("leer símbolo: {}", e)))?);
    }
    Ok(out)
}

/// Convierte una fila SQL en `StoredSymbol`.
fn row_to_symbol(row: &rusqlite::Row<'_>) -> rusqlite::Result<StoredSymbol> {
    let kind_str: String = row.get(5)?;
    let kind = parse_symbol_kind(&kind_str);

    let lang_str: String = row.get(6)?;
    let language = Language::from_extension(&lang_str).unwrap_or(Language::Unknown);

    let vis_str: String = row.get(11)?;
    let visibility = parse_visibility(&vis_str);

    Ok(StoredSymbol {
        id: row.get(0)?,
        file_id: row.get(1)?,
        parent_id: row.get(2)?,
        name: row.get(3)?,
        qualified_name: row.get(4)?,
        kind,
        language,
        start_line: row.get::<_, i64>(7)? as usize,
        end_line: row.get::<_, i64>(8)? as usize,
        signature: row.get(9)?,
        doc_comment: row.get(10)?,
        visibility,
        is_async: row.get::<_, i32>(12)? != 0,
        is_unsafe: row.get::<_, i32>(13)? != 0,
        is_exported: row.get::<_, i32>(14)? != 0,
    })
}

/// Parsea un string a `SymbolKind` con fallback a `Function`.
fn parse_symbol_kind(s: &str) -> SymbolKind {
    match s {
        "function" => SymbolKind::Function,
        "method" => SymbolKind::Method,
        "struct" => SymbolKind::Struct,
        "class" => SymbolKind::Class,
        "interface" => SymbolKind::Interface,
        "trait" => SymbolKind::Trait,
        "enum" => SymbolKind::Enum,
        "enum_variant" => SymbolKind::EnumVariant,
        "constant" => SymbolKind::Constant,
        "static_var" => SymbolKind::StaticVar,
        "type_alias" => SymbolKind::TypeAlias,
        "module" => SymbolKind::Module,
        "import" => SymbolKind::Import,
        "macro" => SymbolKind::Macro,
        "test" => SymbolKind::Test,
        _ => SymbolKind::Function,
    }
}

/// Parsea un string a `Visibility`.
fn parse_visibility(s: &str) -> Visibility {
    match s {
        "public" => Visibility::Public,
        "private" => Visibility::Private,
        "protected" => Visibility::Protected,
        "crate" => Visibility::Crate,
        "package" => Visibility::Package,
        _ => Visibility::Unknown,
    }
}
