```
███████╗███████╗██████╗ ██████╗ ██╗████████╗
██╔════╝██╔════╝██╔══██╗██╔══██╗██║╚══██╔══╝
█████╗  ███████╗██████╔╝██████╔╝██║   ██║   
██╔══╝  ╚════██║██╔═══╝ ██╔══██╗██║   ██║   
███████╗███████║██║     ██║  ██║██║   ██║   
╚══════╝╚══════╝╚═╝     ╚═╝  ╚═╝╚═╝   ╚═╝   
```

# ESPRIT
### The Autonomous AI Workspace & Operating Layer for Developers

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/Rust-1.80%2B-orange.svg)](https://www.rust-lang.org/)
[![Platform](https://img.shields.io/badge/Platform-macOS%20%7C%20Linux%20%7C%20Windows-lightgrey.svg)]()
[![Inference](https://img.shields.io/badge/Inference-Embedded%20llama.cpp%20(Metal%2FCPU)-brightgreen.svg)]()
[![Dependencies](https://img.shields.io/badge/Runtime%20Deps-Zero%20(Self--Contained)-success.svg)]()

---

## 🌟 What is Esprit?

**Esprit** is a high-performance, **zero-dependency AI workspace and operating layer** engineered in Rust. It turns any codebase or directory tree into an interactive, AI-indexed knowledge base with instant sub-millisecond keyword lookup, semantic vector retrieval, conversational memory, file organization, and autonomous agent workflows.

### Why Esprit is Different:
* **Zero Runtime Dependencies**: No Ollama daemon required, no Python, no Docker, no external services. Everything is embedded into a single static native binary.
* **Hardware-Accelerated Inference**: Uses native `llama.cpp` Rust bindings with automatic Apple Silicon Metal acceleration on macOS and optimized multi-threaded AVX2/NEON CPU execution on Linux and Windows.
* **Hybrid Search Engine**: Combines BM25 full-text indexing ([Tantivy](https://github.com/quickwit-oss/tantivy)) with semantic vector embeddings and cosine nearest-neighbour search.
* **Incremental & Git-Aware**: Automatically honors `.gitignore`, skips unchanged files using file `mtime` cache, and watches directories in real-time with debounced notify channels.
* **Robust Local Storage**: SQLite in WAL (Write-Ahead Logging) mode ensures fast, zero-lock concurrent reads/writes across watcher threads and CLI queries.
* **Polished Terminal UX**: Fast interactive spinners, progress bars with ETA, syntax-colored output, and structured JSON outputs for automation.

---

## 🚀 Installation

### 1. One-Line Install (macOS & Linux)
Install the latest pre-compiled release binary directly to your terminal:
```bash
curl -fsSL https://raw.githubusercontent.com/krtvyasingh/Esprit/main/scripts/install.sh | sh
```

### 2. Homebrew (macOS & Linux)
```bash
brew tap krtvyasingh/esprit https://github.com/krtvyasingh/Esprit
brew install esprit
```

### 3. Pre-Built Standalone Binaries
Download directly from [GitHub Releases](https://github.com/krtvyasingh/Esprit/releases):

| Platform | Architecture | Binary Package |
| :--- | :--- | :--- |
| **macOS** | Apple Silicon (M1/M2/M3/M4) | `esprit-aarch64-apple-darwin.tar.gz` |
| **macOS** | Intel (x86_64) | `esprit-x86_64-apple-darwin.tar.gz` |
| **Linux** | x86_64 (Static musl) | `esprit-x86_64-unknown-linux-musl.tar.gz` |
| **Linux** | ARM64 (aarch64 musl) | `esprit-aarch64-unknown-linux-musl.tar.gz` |
| **Windows** | x86_64 | `esprit-x86_64-pc-windows-msvc.zip` |

### 4. Build From Source
Requires standard Rust toolchain:
```bash
# Clone the repository
git clone https://github.com/krtvyasingh/Esprit.git
cd Esprit

# Build optimized release binary
cargo build --release -p esprit-cli

# Symlink or copy to PATH
cp target/release/esprit /usr/local/bin/
```

---

## ⚡ Quickstart (First Run)

### Step 1: Initialize & Download Base Model
Download the compact, high-efficiency base model (~390 MB) into the local cache:
```bash
esprit init
```
*(Optional: include `--with-embeddings` to also download the Nomic Embed model for semantic vector search).*

### Step 2: Check System Health
Verify system environment, developer tools, and storage metrics:
```bash
esprit doctor
```

### Step 3: Index Your Project
Index all source files in your current repository (respects `.gitignore`):
```bash
esprit index .
esprit rebuild
```

### Step 4: Ask Questions with AI
Query your codebase using natural language:
```bash
esprit ask "How is authentication handled in this project?" --sources
```

---

## 📖 Command Line Reference

```
Usage: esprit [OPTIONS] <COMMAND>

Commands:
  init          Download default models and initialize Esprit
  doctor        Check system health, developer tools, and index metrics
  ask           Ask the local AI questions grounded in your project
  search        Search indexed code using full-text BM25 or regex
  index         Index a directory tree into the database
  rebuild       Rebuild full-text Tantivy search index
  db            List all indexed files and size breakdowns
  index-stats   Display total indexed files, storage size, and languages
  watch         Real-time debounced filesystem watcher & live indexer
  model         Manage local GGUF models (list, pull, remove)
  agent         Run specialized agents (chat, code, search)
  workflow      Execute developer workflows (explain, review, search)
  duplicates    Find duplicate files by SHA-256 content hashing
  organize      Sort files into extension-based folders (with --dry-run)
  stats         Scan folder metrics, file counts, and extension distribution
  memory-stats  Inspect conversation memory statistics
  memory-clear  Erase conversation history
  version       Print version banner and build details
```

### Examples

#### 1. Codebase RAG & Question Answering
```bash
# Ask with source file attribution
esprit ask "Where is the database connection pool initialized?" --sources

# Ask code agent for refactoring ideas
esprit agent code "How can I reduce clone allocations in the indexer module?"

# Explain a complex module
esprit workflow explain "crates/esprit-index/src/indexer.rs"
```

#### 2. Model Management
```bash
# List available & installed models
esprit model list

# Pull a higher capability model (e.g. Qwen3 1.7B)
esprit model pull qwen3:1.7b

# Switch default active model
esprit config --set-model qwen3:1.7b
```

#### 3. Real-Time Indexing & Watching
```bash
# Keep index continually synchronized as you edit code
esprit watch .
```

#### 4. Filesystem Utilities
```bash
# Analyze file extension breakdown
esprit stats .

# Preview grouping files into extension directories
esprit organize ./downloads --dry-run

# Detect duplicate files via cryptographic hashing
esprit duplicates ./assets
```

---

## 🏗️ Architecture & Crates

Esprit is organized as a clean, modular Cargo workspace:

```
Esprit/
├── apps/
│   └── esprit-cli/          # High-performance CLI interface & commands
├── crates/
│   ├── esprit-ai/           # Embedded llama.cpp inference engine (Metal/CPU)
│   ├── esprit-models/       # GGUF model registry, downloader, and resume logic
│   ├── esprit-embeddings/   # Native vector embeddings extraction
│   ├── esprit-index/        # Tantivy full-text indexer + SQLite file registry
│   ├── esprit-search/       # BM25 & regex search engine
│   ├── esprit-rag/          # Hybrid retrieval-augmented generation engine
│   ├── esprit-memory/       # Multi-session conversational memory (SQLite WAL)
│   ├── esprit-vectors/      # Vector database & cosine similarity search
│   ├── esprit-platform/     # Cross-platform diagnostics & debounced watcher
│   ├── esprit-filesystem/   # SHA-256 hashing, duplicate scanner, organizer
│   ├── esprit-config/       # TOML configuration loader and settings manager
│   ├── esprit-agents/       # Autonomous specialized agent roles
│   ├── esprit-workflows/    # Higher-level code review & search workflows
│   ├── esprit-plugins/      # Extensible plugin registration & dispatch trait
│   ├── esprit-telemetry/    # Structured tracing subscriber & log subscriber
│   ├── esprit-storage/      # Safe ProjectDirs path resolver & WAL connections
│   └── esprit-core/         # Core error types, versioning, and banner
├── packaging/
│   └── homebrew/            # Homebrew formula for macOS / Linux
├── scripts/
│   └── install.sh           # One-line terminal installer
└── Cargo.toml               # Workspace configuration with release profiles
```

---

## ⚙️ Configuration

Esprit stores configuration at standard OS locations:
* **macOS**: `~/Library/Application Support/dev.esprit.esprit/config.toml`
* **Linux**: `~/.config/esprit/config.toml`
* **Windows**: `%APPDATA%\esprit\esprit\config.toml`

### Configuration File Format (`config.toml`)
```toml
ai_model = "qwen3:0.6b"
ollama_url = "http://127.0.0.1:11434"
threads = 8
color = true
context_chars_per_file = 3500
max_context_files = 8
```

### Environment Overrides
Environment variables take precedence over config files:
* `ESPRIT_MODEL`: Select active model (`qwen3:0.6b`, `qwen3:1.7b`, etc.)
* `RUST_LOG`: Set logging verbosity (`debug`, `info`, `warn`, `error`)

---

## 🔮 Roadmap: Top 20 Advanced Features

1. **🧠 Code Memory Graph**: Petgraph-based AST call-graph indexing for structural code traversal.
2. **⚡ System Daemon Service**: Native background service (`launchd`/`systemd`) for continuous instant re-embedding.
3. **🔀 Branching Chat Sessions**: Tree-structured SQLite conversation branching and session management.
4. **📊 Ratatui Terminal Dashboard**: Rich TUI overview of code health, test status, and indexing metrics.
5. **🌊 Streaming Token Rendering**: Real-time token streaming with inline syntax highlighting.
6. **🔌 WASM Plugin Sandbox**: Safe community plugins running under `wasmtime` runtime isolation.
7. **🎯 Semantic Git Diff**: Natural language summaries of commits and pull request diffs.
8. **🕵️ Offline Code Reviewer**: In-terminal security, performance, and style inspections.
9. **📦 Dependency Intelligence**: AI-powered dependency graph analysis and vulnerability auditing.
10. **✏️ Refactoring Planner**: Step-by-step structural refactoring guides grounded in code.
11. **🧪 Test Gap Finder**: Identification of untested public functions and edge-case branches.
12. **🔐 Secret & Key Scanner**: Local high-entropy credential and token detector.
13. **💬 Inline Code Annotator**: Automated docstring and parameter documentation generator.
14. **🌐 Multi-Model Router**: Intelligent task-based routing across fast and deep models.
15. **📝 Developer Diary**: Git-linked conceptual session notes with semantic search.
16. **🔭 Cross-Project Search**: Multi-workspace unified index and search federation.
17. **🏎️ Benchmark Impact Analysis**: Predictive test and benchmark triage on staged diffs.
18. **🗺️ Visual Architecture Map**: Automated Mermaid and ASCII architecture diagram generation.
19. **🤝 Split-Pane Pair Programmer**: Live reactive assistant watching git staging status.
20. **📤 Project Intelligence Exporter**: Markdown onboarding summary report generation.

---

## 🤝 Contributing

Contributions are welcome!
```bash
# Run unit & integration tests
cargo test --workspace

# Check formatting
cargo fmt --all -- --check

# Check lints
cargo clippy --workspace -- -D warnings
```

---

## 📄 License

Esprit is open source software released under the [MIT License](LICENSE).
