# 🔍 semcode-search

[![Rust](https://img.shields.io/badge/rust-1.75%2B-blue.svg)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)
[![Latest Release](https://img.shields.io/github/v/release/lecodev-26/semcode-search)](https://github.com/lecodev-26/semcode-search/releases)
[![CI](https://github.com/lecodev-26/semcode-search/actions/workflows/ci.yml/badge.svg)](https://github.com/lecodev-26/semcode-search/actions/workflows/ci.yml)
[![Crates.io](https://img.shields.io/crates/v/semcode-search.svg)](https://crates.io/crates/semcode-search)
[![Downloads](https://img.shields.io/crates/d/semcode-search.svg)](https://crates.io/crates/semcode-search)

> **Search your codebase by meaning, not only by exact text.**

`semcode-search` is a local-first code search tool written in Rust. It combines traditional text search, TF-IDF semantic ranking and optional local AI embeddings, with CLI, TUI, desktop GUI, REST API and editor integrations.

## ✨ Why semcode-search?

When a codebase gets large, exact text search is often not enough. You may remember what a piece of code does without remembering the exact symbol or wording.

```bash
semcode-search search \
  --query "where are user permissions validated?" \
  --path . \
  --ai
```

The goal is to help developers find relevant code from natural-language intent while keeping the core workflow local and scriptable.

## 🚀 Install

### From crates.io

```bash
cargo install semcode-search
```

### From source

```bash
git clone https://github.com/lecodev-26/semcode-search
cd semcode-search
cargo build --release
```

## 🔎 Quick start

```bash
semcode-search index --path .
semcode-search search --query "fn main" --path .
semcode-search search --query "authentication middleware" --path . --semantic
semcode-search index --path . --ai
semcode-search search --query "function that validates email addresses" --path . --ai
```

## 🧠 Search modes

| Mode | Purpose |
|---|---|
| Text search | Fast exact/substring-oriented search |
| TF-IDF | Semantic ranking without an external AI service |
| AI embeddings | Natural-language similarity using local embeddings |
| Filename search | Locate files by filename |
| Watch mode | Reindex automatically as files change |

AI embeddings are optional. The project is designed to remain useful without requiring a remote AI API.

## 🖥️ Interfaces

### CLI
The primary command-line interface.

### TUI

```bash
semcode-search tui --path .
```

### Desktop GUI

The v3.x desktop application uses Tauri and provides search, code preview, indexing, statistics, history, aliases, configuration and Spanish/English UI.

Download installers from [GitHub Releases](https://github.com/lecodev-26/semcode-search/releases).

### REST API

```bash
semcode-search serve
```

Default address:

```text
http://127.0.0.1:8080
```

Endpoints include:

```text
GET /
GET /health
GET /stats
GET /search?q=query
```

For network-exposed deployments, review [SECURITY.md](SECURITY.md).

### Editor integrations

- VS Code
- Neovim

See [`plugins/`](plugins/).

## 📦 Features

- 🔍 text search with highlighted results
- 🧠 TF-IDF semantic ranking
- 🤖 optional local embeddings with `fastembed`
- 💾 caching
- 📁 extension, size and glob filters
- 📄 filename search
- ⚡ parallel indexing with Rayon
- ⚡ multithreaded search
- 👁️ watch mode
- 📊 project statistics
- 🕐 search history
- 🏷️ saved aliases
- 🌐 REST API with Axum
- 🎨 TUI with Ratatui
- 🖥️ desktop GUI with Tauri
- 🌍 Spanish/English interface
- 🔌 VS Code and Neovim integrations
- 📚 public Rust library API
- 📱 Termux-compatible CLI

## 🏗️ Architecture

```text
                    Interfaces
       ┌──────────────┬──────────────┬──────────────┐
       │     CLI      │     TUI      │      GUI     │
       │    clap      │   ratatui    │    Tauri     │
       └──────────────┴──────┬───────┴──────────────┘
                             │
                      Core Library
       ┌─────────────────────┴─────────────────────┐
       │ SearchEngine / public API                 │
       │ indexing · ranking · cache · embeddings   │
       │ tree-sitter · filters · i18n              │
       └─────────────────────┬─────────────────────┘
                             │
                         Services
       ┌─────────────────────┴─────────────────────┐
       │ REST API · watch mode · config · history │
       │ aliases · project metadata                │
       └───────────────────────────────────────────┘
```

## 🌍 Supported source files

Rust, Python, JavaScript/TypeScript, Go, Java, C/C++, HTML/CSS, Markdown, TOML, JSON, YAML, Shell, SQL, Ruby, PHP, Swift, Kotlin and more.

## 🧪 Development

```bash
cargo build
cargo build --release
cargo test --all
cargo fmt -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo bench
```

Optional features:

```bash
cargo build --release --features "ai server tui"
```

## 🤝 Contributing

Bug reports, documentation improvements and pull requests are welcome.

Before opening a PR:

```bash
cargo fmt -- --check
cargo test --all
cargo clippy --all-targets --all-features -- -D warnings
```

Read [CONTRIBUTING.md](CONTRIBUTING.md) and [SECURITY.md](SECURITY.md).

## 🔐 Security

Do not publish sensitive vulnerability details in public issues. See [SECURITY.md](SECURITY.md).

## 🗺️ Roadmap

### Completed

- [x] stable CLI and public library API
- [x] caching and filtering
- [x] TF-IDF semantic ranking
- [x] parallel indexing
- [x] statistics and history
- [x] watch mode
- [x] multiplatform CLI support
- [x] desktop GUI
- [x] local AI embeddings
- [x] REST API
- [x] TUI
- [x] VS Code / Neovim integrations
- [x] internationalization

### Next

- [ ] improve public API documentation
- [ ] expand integration and regression tests
- [ ] improve Linux/macOS desktop packaging
- [ ] expand editor integrations
- [ ] improve benchmark coverage and published performance data
- [ ] continue security and dependency hardening

## 📄 License

MIT. See [LICENSE](LICENSE).

## 🔗 Links

- [GitHub](https://github.com/lecodev-26/semcode-search)
- [Crates.io](https://crates.io/crates/semcode-search)
- [Documentation](https://docs.rs/semcode-search)
- [Releases](https://github.com/lecodev-26/semcode-search/releases)
