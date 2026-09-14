//! Mdulo History - Historial de bsquedas

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HistoryEntry {
    pub query: String,
    pub timestamp: String,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct HistoryData {
    pub entries: Vec<HistoryEntry>,
}

pub struct History;

impl History {
    fn get_path() -> PathBuf {
        if let Some(config_dir) = dirs::config_dir() {
            config_dir.join("semcode-search").join("history.json")
        } else {
            PathBuf::from(".semcode-search-history.json")
        }
    }

    fn load() -> anyhow::Result<HistoryData> {
        let path = Self::get_path();
        if path.exists() {
            let content = fs::read_to_string(&path)?;
            let data: HistoryData = serde_json::from_str(&content)?;
            Ok(data)
        } else {
            Ok(HistoryData::default())
        }
    }

    fn save(data: &HistoryData) -> anyhow::Result<()> {
        let path = Self::get_path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let content = serde_json::to_string_pretty(data)?;
        fs::write(path, content)?;
        Ok(())
    }

    /// Aadir una bsqueda al historial
    pub fn add(query: &str) -> anyhow::Result<()> {
        let mut data = Self::load()?;

        // Evitar duplicados consecutivos
        if let Some(last) = data.entries.last() {
            if last.query == query {
                return Ok(());
            }
        }

        data.entries.push(HistoryEntry {
            query: query.to_string(),
            timestamp: chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
        });

        // Mantener solo las ltimas 100 entradas
        if data.entries.len() > 100 {
            let start = data.entries.len() - 100;
            data.entries = data.entries[start..].to_vec();
        }

        Self::save(&data)?;
        Ok(())
    }

    /// Listar todas las bsquedas (ms recientes primero)
    pub fn list() -> anyhow::Result<Vec<HistoryEntry>> {
        let data = Self::load()?;
        let mut entries = data.entries;
        entries.reverse(); // Ms recientes primero
        Ok(entries)
    }

    /// Obtener la ltima bsqueda
    pub fn last() -> anyhow::Result<Option<HistoryEntry>> {
        let data = Self::load()?;
        Ok(data.entries.last().cloned())
    }

    /// Limpiar el historial
    pub fn clear() -> anyhow::Result<()> {
        let empty = HistoryData::default();
        Self::save(&empty)?;
        Ok(())
    }
}
