
# 🔍 semcode-search

[![Rust](https://img.shields.io/badge/rust-1.75%2B-blue.svg)](https://www.rust-lang.org)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Version](https://img.shields.io/badge/version-2.0.0-brightgreen.svg)](https://github.com/lecodev-26/semcode-search/releases)
[![Crates.io](https://img.shields.io/badge/crates.io-semcode--search-orange.svg)](https://crates.io/crates/semcode-search)
[![Termux](https://img.shields.io/badge/Termux-compatible-brightgreen.svg)](https://termux.com)

> **Fast semantic code search CLI with TF-IDF ranking, caching, and advanced filtering.**

---

## 📦 Disponible en crates.io

El crate está publicado oficialmente en [crates.io](https://crates.io/crates/semcode-search).

# Instalación directa desde crates.io
```bash
cargo install semcode-search
```
Una vez instalado:

```bash
semcode-search --version
semcode-search --help
```
## ✨ Features

🔍 Text search with color highlighting

🧠 Semantic search using TF-IDF ranking

💾 Intelligent cache for instant searches

📁 Advanced filtering by extension, size, and glob patterns

📄 Filename search

⚙️ Global configuration

🏷️ Search aliases (save, list, remove, run)

🎮 Interactive mode to navigate results

⚡ Parallel indexing with rayon

📊 Stats - Project statistics (v2.0.0)

🕐 Search history - Last 100 searches (v2.0.0)

⚡ Multi-threaded search - Use all CPU cores (v2.0.0)

👁️ Watch mode - Auto-reindex on changes (v2.0.0)

🌍 Multi-platform - Binaries for Linux, macOS, Windows (v2.0.0)

📱 Termux compatible (Android)

📚 Public API for use as a library

## 🚀 Installation

# From crates.io
```bash
cargo install semcode-search
```

# From GitHub
```bash
git clone https://github.com/lecodev-26/semcode-search
```
```bash
cd semcode-search
```
```bash
cargo build --release
```

# Windows
.\target\release\semcode-search.exe --version

# Linux/macOS
./target/release/semcode-search --version

## 📖 Usage
Búsqueda básica

# Indexar un proyecto
```bash
semcode-search index --path .
```

# Buscar por texto
```bash
semcode-search search --query "fn main" --path .
```

# Búsqueda semántica (TF-IDF)
```bash
semcode-search search --query "authentication middleware" --path . --semantic
```

# Buscar por nombre de archivo
```bash
semcode-search search --file "main.rs" --path .
```

# Búsqueda con verbosa
```bash
semcode-search search --query "error" --path . --verbose
```
## Estadísticas (v2.0.0)
```bash
semcode-search stats
```
## Historial de búsquedas (v2.0.0)

# Ver últimas 20 búsquedas
```
semcode-search history list
```

# Ver últimas 50 búsquedas
```bash
semcode-search history list --limit 50
```

# Repetir la última búsqueda
```bash
semcode-search search --query "!!" --path .
```

# Limpiar historial
```bash
semcode-search history clear
```
Watch mode (v2.0.0)

# Observar cambios y reindexar automáticamente
```bash
semcode-search watch --path .
```

# Con intervalo personalizado (en segundos)
```bash
semcode-search watch --path . --interval 5
```
## Alias
```bash
semcode-search alias save find-main "main" -- --ext rs --path .
semcode-search alias list
semcode-search alias run find-main
semcode-search alias remove find-main
```
## Configuración
```bash
semcode-search init
```
## 🏗️ Architecture
```text
┌─────────────────────────────────────────────────────┐
│                    CLI (clap)                       │
├─────────────────────────────────────────────────────┤
│  Commands: init, alias, index, search, stats,       │
│            history, watch                           │
└─────────────────────┬───────────────────────────────┘
                      │
┌─────────────────────▼───────────────────────────────┐
│                 Core Library                        │
├─────────────────────────────────────────────────────┤
│  • SearchEngine (public API)                       │
│  • TF-IDF ranking                                  │
│  • Parallel indexing & search (rayon)              │
│  • Cache management                                │
│  • History management                              │
│  • Watch mode                                      │
└─────────────────────────────────────────────────────┘
```
## 🗺️ Roadmap
```marckdown
Versión	Features	Estado
v0.1.0	Basic search with colors	✅
v0.2.0	Filters, ignore directories, exact search	✅
v0.3.0	Cache - instant searches	✅
v0.4.0	Semantic search with TF-IDF	✅
v0.5.0	Filename search, occurrence counter, summary	✅
v0.6.0	Extension indexing, size filtering	✅
v0.7.0	Glob pattern ignore, compressed file search	✅
v0.8.0	Global config, aliases, interactive mode	✅
v0.9.0	Refactoring, parallel indexing	✅
v1.0.0	Stable release with public API	✅
v2.0.0	Stats, history, multi-thread, watch, multiplatform	✅
v3.0.0	IA real, servidor, TUI, plugins, gitignore	🚧 Próximamente
```
## 📁 Supported extensions
Rust (.rs)

Python (.py)

JavaScript/TypeScript (.js, .ts)

Go (.go)

Java (.java)

C/C++ (.c, .cpp, .h)

And more: .toml, .json, .yaml, .md, .sh, .bash, .css, .html, .xml, .sql, .rb, .php, .swift, .kt

## 🛠️ Development

# Clone
```bash
git clone https://github.com/lecodev-26/semcode-search
```
```bash
cd semcode-search
```

# Build
```bash
cargo build
```

# Build optimized
```bash
cargo build --release
```

# Run tests
```bash
cargo test
```

# Run benchmarks
```bash
cargo bench
```
## 📄 License
MIT

## 👤 Author
([@lecodev-26](https://github.com/lecodev-26))
