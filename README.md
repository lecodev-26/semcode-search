# 🔍 semcode-search

[![Rust](https://img.shields.io/badge/rust-1.75%2B-blue.svg)](https://www.rust-lang.org)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Version](https://img.shields.io/badge/version-3.0.0-brightgreen.svg)](https://github.com/lecodev-26/semcode-search/releases)
[![Crates.io](https://img.shields.io/badge/crates.io-semcode--search-orange.svg)](https://crates.io/crates/semcode-search)
[![Termux](https://img.shields.io/badge/Termux-compatible-brightgreen.svg)](https://termux.com)
[![GUI](https://img.shields.io/badge/GUI-Windows-blue.svg)](https://github.com/lecodev-26/semcode-search/releases)

> **Fast semantic code search CLI + Desktop GUI with AI embeddings, caching, and advanced filtering.**

---

## 📦 Disponible en crates.io

El crate está publicado oficialmente en [crates.io](https://crates.io/crates/semcode-search).

```bash
cargo install semcode-search
```
Una vez instalado:
```bash
semcode-search --version
```
```bash
semcode-search --help
```
## 🖥️ GUI de escritorio (v3.0.0)
Aplicación nativa para Windows con interfaz gráfica moderna.

Descarga: [Releases v3.0.0](https://github.com/lecodev-26/semcode-search/releases/tag/v3.0.0).

📦 semcode-search-gui_3.0.0_x64_en-US.msi (4.7 MB) — Instalador oficial

📦 semcode-search-gui_3.0.0_x64-setup.exe (3.17 MB) — Instalador NSIS (más rápido)

Características:

🎨 Interfaz moderna con tema oscuro/claro

🌍 Cambio de idioma (Español / English)

🔍 Búsqueda con preview de código

📊 Estadísticas visuales

📜 Historial de búsquedas

🏷️ Sistema de alias

⚙️ Configuración global

## ✨ Features

### 🆕 Novedades v3.0.0
- 🖥️ **GUI de escritorio** con Tauri (Windows)
- 🧠 **IA real con embeddings** (fastembed, all-MiniLM-L6-v2)
- 🌐 **Servidor HTTP con API REST** (axum)
- 🎨 **Interfaz TUI** (ratatui)
- 🌍 **Sistema i18n** (Español / English)
- 🔌 **Plugins para editores** (VSCode + Neovim)
- 📁 **`.gitignore` mejorado** (soporta `.git_exclude`, `.parents`)

### 📦 CLI
- 🔍 **Búsqueda por texto** con resaltado en color
- 🧠 **Búsqueda semántica** con TF-IDF
- 💾 **Caché inteligente** para búsquedas instantáneas
- 📁 **Filtros avanzados** por extensión, tamaño y patrones glob
- 📄 **Búsqueda por nombre** de archivo
- ⚙️ **Configuración global**
- 🏷️ **Alias de búsquedas** (save, list, remove, run)
- 🎮 **Modo interactivo** para navegar resultados
- ⚡ **Indexado paralelo** con rayon
- 📊 **Estadísticas** del proyecto
- 🕐 **Historial de búsquedas** (últimas 100)
- ⚡ **Búsqueda multi-hilo** (todos los núcleos)
- 👁️ **Watch mode** (reindexado automático)
- 📱 **Compatible con Termux** (Android)
- 📚 **API pública** para usar como librería

---

## 🚀 Installation

### Desde crates.io (CLI)

```bash
cargo install semcode-search
```
## Desde GitHub
```bash
git clone https://github.com/lecodev-26/semcode-search
cd semcode-search
cargo build --release
```
Una vez compilado:

```bash
# Windows
.\target\release\semcode-search.exe --version
```
```bash
# Linux/macOS
./target/release/semcode-search --version
```
## Con todas las features (IA + servidor + TUI)
```bash
cargo build --release --features "ai server tui"
```
GUI de escritorio
Descarga el instalador desde Releases v3.0.0.

## 📖 Usage (CLI)

### Búsqueda básica

```bash
# Indexar un proyecto
semcode-search index --path .

# Buscar por texto
semcode-search search --query "fn main" --path .

# Búsqueda semántica (TF-IDF)
semcode-search search --query "authentication middleware" --path . --semantic

# Buscar por nombre de archivo
semcode-search search --file "main.rs" --path .

# Búsqueda con verbose
semcode-search search --query "error" --path . --verbose
```
## IA real con embeddings (v3.0.0)
```bash
# Indexar con IA (genera embeddings, descarga modelo ~90MB)
semcode-search index --path . --ai

# Buscar por significado (IA real)
semcode-search search --query "funcion que valida emails" --path . --ai
```
## Estadísticas (v2.0.0)
```bash
semcode-search stats
```

Historial de búsquedas (v2.0.0)
```bash
# Ver últimas 20 búsquedas
semcode-search history list

# Ver últimas 50 búsquedas
semcode-search history list --limit 50

# Repetir la última búsqueda
semcode-search search --query "!!" --path .

# Limpiar historial
semcode-search history clear
```
## Watch mode (v2.0.0)
```bash
# Observar cambios y reindexar automáticamente
semcode-search watch --path .

# Con intervalo personalizado (en segundos)
semcode-search watch --path . --interval 5
Alias
bash
semcode-search alias save find-main "main" -- --ext rs --path .
semcode-search alias list
semcode-search alias run find-main
semcode-search alias remove find-main
```
## Configuración
```bash
semcode-search init
```
## Servidor HTTP con API REST (v3.0.0)
```bash
# Arrancar servidor en http://127.0.0.1:8080
semcode-search serve
```

# Puerto personalizado
semcode-search serve --port 9000
Endpoints disponibles:

GET / — Información del servidor

GET /health — Estado del servidor

GET /stats — Estadísticas del proyecto

GET /search?q=query — Búsqueda por texto

## Interfaz TUI (v3.0.0)
```bash
# Abrir interfaz gráfica en terminal
semcode-search tui --path .
```
## 🖥️ Usage (GUI)

La GUI de escritorio ofrece una experiencia visual completa.

### Inicio rápido

1. **Descarga** el instalador `.msi` o `.exe` desde [Releases v3.0.0](https://github.com/lecodev-26/semcode-search/releases/tag/v3.0.0).
2. **Ejecuta** el instalador.
3. **Abre** Semcode Search desde el menú de inicio.
4. **Pulsa "Indexar"** y selecciona la carpeta de tu proyecto.
5. **Escribe una búsqueda** en lenguaje natural.
6. **Haz clic** en cualquier resultado para ver el código completo.

### Funcionalidades

| Función | Descripción |
|---------|-------------|
| 🔍 **Búsqueda** | Texto normal o con IA (embeddings) |
| 📊 **Stats** | Estadísticas visuales del proyecto |
| 📜 **Historial** | Últimas 100 búsquedas con click para repetir |
| 🏷️ **Alias** | Búsquedas guardadas con nombre |
| 📁 **Proyectos** | Lista de proyectos recientes |
| ⚙️ **Configuración** | Ajusta idioma, tema y preferencias |
| 🌍 **i18n** | Cambia entre Español e English |
| 🎨 **Tema** | Modo claro / oscuro |

### Atajos de teclado

| Atajo | Acción |
|-------|--------|
| `Ctrl + K` | Enfocar búsqueda |
| `Enter` | Ejecutar búsqueda |
| `↑ / ↓` | Navegar resultados |
| `Esc` | Cerrar modal / preview |

---

## 🏗️ Architecture

### Vista general

```text
┌──────────────────────────────────────────────────────────┐
│                    Interfaces                            │
├──────────────┬────────────────┬──────────────────────────┤
│   CLI        │      TUI       │        GUI               │
│  (clap)      │   (ratatui)    │      (Tauri)             │
└──────┬───────┴────────┬───────┴────────────┬─────────────┘
       │                │                    │
       └────────────────┼────────────────────┘
                        │
┌───────────────────────▼──────────────────────────────────┐
│                    Core Library                          │
├──────────────────────────────────────────────────────────┤
│  • SearchEngine (public API)                             │
│  • Indexado paralelo con rayon                           │
│  • TF-IDF ranking + IA real (embeddings)                 │
│  • Cache con LMDB / JSON                                 │
│  • Parser con tree-sitter                                │
│  • Sistema i18n (es/en)                                  │
└───────────────────────┬──────────────────────────────────┘
                        │
┌───────────────────────▼──────────────────────────────────┐
│                    Servicios                             │
├──────────────────────────────────────────────────────────┤
│  • Servidor HTTP (axum)                                  │
│  • Watch mode (reindexado automático)                    │
│  • Configuración global (~/.config/semcode-search/)      │
│  • Historial y alias                                     │
└──────────────────────────────────────────────────────────┘
```
## Estructura del proyecto
```marckdown
semcode-search/
├── src/                          # CLI + Core
│   ├── cli/                      # Comandos y argumentos
│   ├── core/                     # Motor de búsqueda
│   ├── cache/                    # Gestión de caché
│   ├── embeddings/               # IA real (fastembed)
│   ├── history/                  # Historial de búsquedas
│   ├── i18n/                     # Internacionalización
│   ├── server/                   # Servidor HTTP
│   ├── tui/                      # Interfaz TUI
│   └── watch/                    # Watch mode
├── gui/                          # GUI con Tauri
│   ├── src/                      # Frontend (HTML/CSS/JS)
│   └── src-tauri/                # Backend Rust
├── plugins/                      # Plugins para editores
│   ├── vscode/                   # Extensión VSCode
│   └── neovim/                   # Plugin Neovim
├── benches/                      # Benchmarks
├── tests/                        # Tests de integración
└── models/                       # Modelos IA (gitignored)
```
## 🗺️ Roadmap

| Versión | Features | Estado |
|---------|----------|--------|
| **v0.1.0** | Basic search with colors | ✅ |
| **v0.2.0** | Filters, ignore directories, exact search | ✅ |
| **v0.3.0** | Cache - instant searches | ✅ |
| **v0.4.0** | Semantic search with TF-IDF | ✅ |
| **v0.5.0** | Filename search, occurrence counter, summary | ✅ |
| **v0.6.0** | Extension indexing, size filtering | ✅ |
| **v0.7.0** | Glob pattern ignore, compressed file search | ✅ |
| **v0.8.0** | Global config, aliases, interactive mode | ✅ |
| **v0.9.0** | Refactoring, parallel indexing | ✅ |
| **v1.0.0** | Stable release with public API | ✅ |
| **v2.0.0** | Stats, history, multi-thread, watch, multiplatform | ✅ |
| **v3.0.0** | GUI, IA real, server, TUI, plugins, i18n | ✅ |
| **v3.0.1** | Fix CLI accents, more languages | 🚧 Próximamente |
| **v4.0.0** | Plugins marketplace, GUI for Linux/macOS | 🚧 Futuro |

---

## 📁 Supported extensions

- Rust (`.rs`)
- Python (`.py`)
- JavaScript/TypeScript (`.js`, `.ts`)
- Go (`.go`)
- Java (`.java`)
- C/C++ (`.c`, `.cpp`, `.h`)
- HTML (`.html`)
- CSS (`.css`)
- Markdown (`.md`)
- TOML (`.toml`)
- JSON (`.json`)
- YAML (`.yml`, `.yaml`)
- Y más: `.sh`, `.bash`, `.xml`, `.sql`, `.rb`, `.php`, `.swift`, `.kt`

---

## 🛠️ Development

```bash
# Clonar
git clone https://github.com/lecodev-26/semcode-search
cd semcode-search

# Build (CLI)
cargo build

# Build optimizado
cargo build --release

# Build con todas las features
cargo build --release --features "ai server tui"

# Tests
cargo test --all

# Benchmarks
cargo bench

# Formato
cargo fmt

# Clippy
cargo clippy --all-targets --all-features -- -D warnings

# GUI (necesita Tauri CLI)
cd gui/src-tauri
cargo tauri dev     # Modo desarrollo
cargo tauri build   # Compilar instalador
```
## 🔌 Plugins
VSCode
Copia la carpeta plugins/vscode/ a:

```text
%USERPROFILE%\.vscode\extensions\semcode-search-vscode
```
Luego usa Ctrl + Shift + F para buscar.

## Neovim
Añade a tu init.lua:

```lua
require('semcode.semcode')
```
Y usa:

<leader>ss — Buscar

<leader>sa — Buscar con IA

<leader>si — Indexar

<leader>st — Stats

## 📄 License
MIT — ver LICENSE para más detalles.

## 👤 Author

**Manuel** ([@lecodev-26](https://github.com/lecodev-26))

- 🐙 **GitHub:** [@lecodev-26](https://github.com/lecodev-26)
- 📦 **Crates.io:** [semcode-search](https://crates.io/crates/semcode-search)

---

## ⭐ Support

Si este proyecto te ha sido útil, dale una ⭐ en GitHub. ¡Ayuda mucho!

[![GitHub stars](https://img.shields.io/github/stars/lecodev-26/semcode-search?style=social)](https://github.com/lecodev-26/semcode-search/stargazers)

---

[⬆ Back to top](#-semcode-search)
