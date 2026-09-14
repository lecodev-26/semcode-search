//! Módulo Watch - Observa cambios y reindexa automáticamente

use colored::*;
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::{Duration, SystemTime};

pub struct Watcher {
    path: String,
    ignore_dirs: Vec<String>,
    interval: u64,
    // Guarda: ruta del archivo -> (tamaño, última modificación)
    file_states: HashMap<PathBuf, (u64, SystemTime)>,
}

impl Watcher {
    pub fn new(path: &str, ignore_dirs: Vec<String>, interval: u64) -> Self {
        Self {
            path: path.to_string(),
            ignore_dirs,
            interval,
            file_states: HashMap::new(),
        }
    }

    /// Escanear todos los archivos y devolver su estado actual
    fn scan_files(&self) -> HashMap<PathBuf, (u64, SystemTime)> {
        use ignore::WalkBuilder;

        let mut states = HashMap::new();

        let walker = WalkBuilder::new(&self.path)
            .git_ignore(true)
            .follow_links(false)
            .build();

        for result in walker {
            let entry = match result {
                Ok(e) => e,
                Err(_) => continue,
            };

            let p = entry.path();

            // Ignorar directorios
            if entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false) {
                if let Some(name) = p.file_name().and_then(|n| n.to_str()) {
                    if self.ignore_dirs.contains(&name.to_string()) {
                        continue;
                    }
                }
                continue;
            }

            // Solo archivos
            if !entry.file_type().map(|ft| ft.is_file()).unwrap_or(false) {
                continue;
            }

            // Verificar extensión soportada
            if let Some(ext) = p.extension().and_then(|e| e.to_str()) {
                let exts = [
                    "rs", "py", "js", "ts", "go", "java", "c", "cpp", "h", "toml", "json", "txt",
                    "md", "sh", "bash", "yaml", "yml", "css", "html", "xml", "sql", "rb", "php",
                    "swift", "kt",
                ];
                if !exts.contains(&ext) {
                    continue;
                }

                if let Ok(metadata) = std::fs::metadata(p) {
                    let size = metadata.len();
                    let modified = metadata.modified().unwrap_or(SystemTime::UNIX_EPOCH);
                    states.insert(p.to_path_buf(), (size, modified));
                }
            }
        }

        states
    }

    /// Detectar qué archivos han cambiado
    fn detect_changes(
        &self,
        old_states: &HashMap<PathBuf, (u64, SystemTime)>,
        new_states: &HashMap<PathBuf, (u64, SystemTime)>,
    ) -> Vec<PathBuf> {
        let mut changes = Vec::new();

        // Archivos nuevos o modificados
        for (path, (size, modified)) in new_states {
            match old_states.get(path) {
                Some((old_size, old_modified)) => {
                    if size != old_size || modified != old_modified {
                        changes.push(path.clone());
                    }
                }
                None => {
                    changes.push(path.clone());
                }
            }
        }

        // Archivos eliminados
        for path in old_states.keys() {
            if !new_states.contains_key(path) {
                changes.push(path.clone());
            }
        }

        changes
    }

    /// Ejecutar el watcher
    pub fn run(&mut self) -> anyhow::Result<()> {
        println!(
            "{} Observando cambios en: {}",
            "👁️".cyan(),
            self.path.green()
        );
        println!("{} Intervalo: {} segundos", "⏱️".cyan(), self.interval);

        // Indexación inicial
        println!("\n{} Indexando por primera vez...", "📁".blue());
        let ignore_refs: Vec<&str> = self.ignore_dirs.iter().map(|s| s.as_str()).collect();
        crate::core::index_files(&self.path, ignore_refs, None, None, false)?;

        // Guardar estado inicial
        self.file_states = self.scan_files();
        println!(
            "\n{} Observando {} archivos... (Ctrl+C para salir)",
            "🔄".cyan(),
            self.file_states.len()
        );

        let interval = Duration::from_secs(self.interval);

        loop {
            std::thread::sleep(interval);

            let new_states = self.scan_files();
            let changes = self.detect_changes(&self.file_states, &new_states);

            if !changes.is_empty() {
                let timestamp = chrono::Local::now().format("%H:%M:%S");
                println!(
                    "\n[{}] {} {} cambios detectados",
                    timestamp.to_string().dimmed(),
                    "⚠️".yellow(),
                    changes.len()
                );

                for change in changes.iter().take(5) {
                    if let Some(name) = change.file_name().and_then(|n| n.to_str()) {
                        println!("  {} {}", "→".dimmed(), name.dimmed());
                    }
                }
                if changes.len() > 5 {
                    println!(
                        "  {} ...y {} más",
                        "→".dimmed(),
                        (changes.len() - 5).to_string().dimmed()
                    );
                }

                println!("\n{} Reindexando...", "📁".blue());
                let ignore_refs: Vec<&str> = self.ignore_dirs.iter().map(|s| s.as_str()).collect();
                match crate::core::index_files(&self.path, ignore_refs, None, None, false) {
                    Ok(_) => {
                        let timestamp = chrono::Local::now().format("%H:%M:%S");
                        println!(
                            "[{}] {} Reindexado completo\n",
                            timestamp.to_string().dimmed(),
                            "✅".green()
                        );
                    }
                    Err(e) => {
                        println!("{} Error al reindexar: {}\n", "❌".red(), e);
                    }
                }

                self.file_states = new_states;
            }
        }
    }
}
