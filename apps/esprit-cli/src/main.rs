pub mod agents_swarm;
pub mod cleaner_cmd;
pub mod collab;
pub mod daemon;
pub mod devops;
pub mod lsp;
pub mod ui;
pub mod updater;

use anyhow::Result;
use clap::{Parser, Subcommand};
use owo_colors::OwoColorize;
use std::time::Instant;

pub use ui::{bar, divider, elapsed, fail, info, kv, ok, panel_header, section, spinner, warn};

// ── CLI definition ────────────────────────────────────────────────────────────

#[derive(Parser)]
#[command(
    name = "esprit",
    version,
    about = "Esprit — AI workspace & operating layer",
    long_about = None,
    propagate_version = true,
)]
struct Cli {
    /// Enable verbose logging (RUST_LOG=debug).
    #[arg(long, global = true)]
    verbose: bool,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Automatically update Esprit directly from GitHub
    Update {
        /// Force update and rebuild even if already on latest commit
        #[arg(long, short)]
        force: bool,

        /// Check for updates without applying
        #[arg(long, short)]
        check: bool,
    },

    /// Show interactive welcome screen, command guide & live status
    Welcome,
    /// Check system health and tool availability.
    Doctor,

    /// Developer Diary notes linked to current git branch
    Diary {
        /// Note content to save (if empty, lists existing notes)
        note: Option<String>,
    },

    /// Terminal TUI Dashboard for metrics
    Find {
        query: String,
    },

    /// Trace a function's call graph using AI
    Trace {
        function: String,
    },

    /// Detect if a file's docstrings have drifted from its implementation
    Drift {
        file: String,
    },

    Debate {
        topic: String,
    },
    EnvScaffold,
    Docker,
    Score {
        file: String,
    },

    Dashboard,

    /// Output the architectural graph as Mermaid JS
    Graph,

    /// Output project dependencies extracted from index
    Deps,

    /// Find source files missing test files
    /// Generate a natural language commit summary from git diff
    Diff,

    /// macOS Omni-Agent: Natural Language OS Assistant
    Os {
        prompt: Vec<String>,
    },

    /// Offline Code Reviewer for staged changes
    Review,

    /// Generate missing docstrings for a file
    Annotate {
        file: String,
    },

    /// Generate a step-by-step refactoring plan
    Plan {
        goal: String,
    },

    /// Triage which tests to run based on recent changes
    Triage,

    /// Export the entire indexed project to a single Markdown onboarding guide
    Export,

    /// Start the Esprit system daemon (background watcher & real-time pair programmer)
    Daemon,

    /// Execute a WASM plugin sandbox extension
    Plugin {
        file: String,
        input: Option<String>,
    },

    TestGaps,

    /// Print Esprit version and build info.
    Version,

    /// Show and manage Esprit configuration.
    Config {
        /// Set AI model (e.g. llama3.2).
        #[arg(long)]
        set_model: Option<String>,
    },

    /// Search the indexed project (keyword or regex).
    Search {
        /// Pattern to search for.
        pattern: String,

        /// Treat pattern as a regular expression.
        #[arg(long, short)]
        regex: bool,

        /// Search across all known workspaces globally
        #[arg(long)]
        all: bool,
    },

    /// Show filesystem statistics for a folder.
    Stats {
        /// Target folder to analyse.
        folder: String,
    },

    /// Organise files in a folder into extension subdirectories.
    Organize {
        /// Folder to organise.
        folder: String,

        /// Preview changes without moving files.
        #[arg(long)]
        dry_run: bool,
    },

    /// Find duplicate files by content hash.
    Duplicates {
        /// Folder to scan.
        folder: String,
    },

    /// Index a folder into the search database.
    Index {
        /// Root folder to index.
        folder: String,
    },

    /// Rebuild the full-text search index from the current database.
    Rebuild,

    /// List all files in the index database.
    Db,

    /// Show a summary of the current index.
    IndexStats,

    /// Watch a folder for changes and keep the index up to date.
    Watch {
        /// Folder to watch.
        folder: String,
    },

    /// Ask the AI a question about your indexed project.
    Ask {
        /// Your question.
        prompt: String,

        /// Show which source files were used to answer.
        #[arg(long, short)]
        sources: bool,
    },

    /// Run a named AI agent.
    Agent {
        /// Agent name: chat | code | search
        agent: String,
        /// Prompt.
        prompt: String,
    },

    /// Run a named workflow.
    Workflow {
        /// Workflow name: explain | review | search
        workflow: String,
        /// Prompt.
        prompt: String,
    },

    /// Clear conversation memory.
    MemoryClear,

    /// Show conversation memory stats.
    MemoryStats,

    /// Compress long-term chat memory to free up context
    MemoryCompress,

    /// Download default models and set up Esprit for first use.
    Init {
        /// Also download the embedding model for semantic search.
        #[arg(long)]
        with_embeddings: bool,
    },

    /// Manage local AI models.
    Model {
        #[command(subcommand)]
        action: ModelAction,
    },

    /// Native Language Server Protocol (LSP) server over stdio for VS Code/Zed/Neovim
    Lsp,

    /// Calculate cross-file AST blast radius and downstream dependency impact
    BlastRadius {
        /// Function, struct, or file target to inspect
        target: String,
    },

    /// Analyze Cognitive and Cyclomatic complexity hotspots
    Complexity {
        /// Path to analyze (defaults to current directory)
        path: Option<String>,
    },

    /// Identify dead, orphaned, or unreferenced functions and structs
    DeadCode {
        /// Root path to analyze (defaults to current directory)
        path: Option<String>,
    },

    /// Scan codebase for high-entropy secrets and credential leaks
    ScanSecrets {
        /// Directory path to scan (defaults to current directory)
        path: Option<String>,
    },

    /// Autonomous Test-Driven Development (TDD) loop agent
    Tdd {
        /// Feature goal or requirement specification
        goal: String,
    },

    /// AppSec Red-Team vulnerability auditor agent
    RedTeam {
        /// Target path to audit (defaults to current directory)
        path: Option<String>,
    },

    /// Production incident and runtime error triager
    Triager {
        /// Error snippet, stack trace, or compiler log
        error_snippet: String,
    },

    /// Hypothetical Document Embeddings (HyDE) contextual search
    Hyde {
        /// Search query
        query: String,
    },

    /// Export vector brain knowledge base to portable JSON package
    BrainExport {
        /// Output file path (defaults to .esprit/brain.json)
        output: Option<String>,
    },

    /// Generate production CI/CD matrix GitHub Actions workflow
    CiGen {
        /// Primary language: rust | node | go | python
        lang: Option<String>,
    },

    /// Generate multi-stage distroless Dockerfile
    DockerGen {
        /// Binary application name
        app_name: Option<String>,
    },

    /// Estimate cloud infrastructure costs for compute & memory workload
    CostEstimate {
        /// Service profile name
        service: Option<String>,
        /// Memory in GB
        #[arg(long, default_value = "4")]
        memory_gb: u32,
        /// vCPU cores
        #[arg(long, default_value = "2")]
        cpu_cores: u32,
    },

    /// Generate comprehensive ONBOARDING.md team developer guide
    Onboarding,

    /// Audit and enforce production code conventions & safety
    StyleEnforce {
        /// Directory path to scan
        path: Option<String>,
    },

    /// Deep system cleaner, cache sweeper, ghost app remover & malware scanner
    Clean {
        /// Scan and display diagnostic report without deleting
        #[arg(long, short)]
        scan: bool,

        /// Clean all safe caches, app remnants, and temporary files
        #[arg(long, short)]
        all: bool,

        /// Clean developer tool caches (Xcode, Cargo, npm, pip, etc.)
        #[arg(long, short)]
        cache: bool,

        /// Clean uninstalled / deleted application leftovers
        #[arg(long, short)]
        leftovers: bool,

        /// Quarantine or clean identified temporary threat droppers
        #[arg(long)]
        threats: bool,

        /// Perform dry-run without removing files
        #[arg(long)]
        dry_run: bool,

        /// Auto-confirm cleanup without interactive confirmation
        #[arg(long, short)]
        yes: bool,

        /// Output diagnostic report as JSON
        #[arg(long)]
        json: bool,
    },
}

// ── Model sub-command ─────────────────────────────────────────────────────────

#[derive(Subcommand)]
enum ModelAction {
    /// List all known models and their installation status.
    List,
    /// Download a model by ID (e.g. `qwen3:0.6b`).
    Pull {
        /// Model ID from `esprit model list`.
        id: String,
    },
    /// Remove an installed model to free disk space.
    Remove { id: String },
}

// ── main ──────────────────────────────────────────────────────────────────────

fn main() -> Result<()> {
    let cli = Cli::parse();

    if cli.verbose {
        std::env::set_var("RUST_LOG", "esprit=debug,warn");
    }
    esprit_telemetry::init()?;

    let command = cli.command.unwrap_or(Commands::Welcome);

    // Non-intrusive background check for GitHub updates on command execution
    if !matches!(command, Commands::Update { .. }) {
        updater::notify_if_available();
    }

    match command {
        // ── update ───────────────────────────────────────────────────────────
        Commands::Update { force, check } => {
            if check {
                ui::banner();
                ui::panel_header("Update Status Check", Some("GitHub"));
                let sp = ui::spinner("Querying GitHub for latest revisions…");
                let info = updater::check_update(true);
                sp.finish_and_clear();
                match info {
                    Some(u) if u.has_update => {
                        ui::update_badge(
                            &u.current_commit,
                            &u.latest_commit,
                            Some(&u.latest_message),
                        );
                        println!(
                            "  Run {} to upgrade immediately.\n",
                            "esprit update".bold().green()
                        );
                    }
                    Some(u) => {
                        ui::ok(&format!(
                            "Esprit is up to date on commit {}.",
                            u.current_commit.cyan().bold()
                        ));
                        println!("  Latest commit: {}\n", u.latest_message.dimmed());
                    }
                    None => {
                        ui::warn("Could not reach GitHub API. Check your internet connection.\n");
                    }
                }
            } else {
                updater::execute_update(force)?;
            }
        }

        // ── welcome ──────────────────────────────────────────────────────────
        Commands::Welcome => {
            ui::banner();
            ui::panel_header("Welcome to Esprit AI", Some("Overview"));
            ui::card(
                "CORE CAPABILITIES",
                &[
                    format!(
                        "{} {:<20} {}",
                        "⚡".cyan(),
                        "esprit ask <query>".bold(),
                        "Ask questions about your entire codebase".dimmed()
                    ),
                    format!(
                        "{} {:<20} {}",
                        "🤖".magenta(),
                        "esprit os <prompt>".bold(),
                        "Offline macOS natural language assistant".dimmed()
                    ),
                    format!(
                        "{} {:<20} {}",
                        "🔍".green(),
                        "esprit find <query>".bold(),
                        "Semantic code search via vector embeddings".dimmed()
                    ),
                    format!(
                        "{} {:<20} {}",
                        "⚖️".yellow(),
                        "esprit debate <topic>".bold(),
                        "Multi-agent architecture debate (Sec vs Perf)".dimmed()
                    ),
                    format!(
                        "{} {:<20} {}",
                        "🧹".green(),
                        "esprit clean".bold(),
                        "Deep system cache, ghost app & malware cleaner".dimmed()
                    ),
                    format!(
                        "{} {:<20} {}",
                        "🩺".blue(),
                        "esprit doctor".bold(),
                        "Run full system & AI health diagnostic".dimmed()
                    ),
                    format!(
                        "{} {:<20} {}",
                        "🔄".cyan(),
                        "esprit update".bold(),
                        "Instant self-update directly from GitHub".dimmed()
                    ),
                ],
            );
            println!();
            let model_count = esprit_models::list_status()
                .unwrap_or_default()
                .iter()
                .filter(|(_, i)| *i)
                .count();
            let vector_count = esprit_vectors::count().unwrap_or(0);
            let memory_count = esprit_memory::count().unwrap_or(0);

            ui::panel_header("Live System State", Some("Local Cache"));
            ui::kv_dot(
                "Active Workspace",
                &format!("{}", std::env::current_dir().unwrap_or_default().display()),
            );
            ui::kv_dot(
                "Installed AI Models",
                &format!("{} models ready", model_count),
            );
            ui::kv_dot(
                "Indexed Code Vectors",
                &format!("{} vectors in SQLite", vector_count),
            );
            ui::kv_dot(
                "Conversation Lines",
                &format!("{} stored in diary", memory_count),
            );
            println!(
                "\n  {} Try running: {}\n",
                "→".cyan().bold(),
                "esprit doctor".bold().cyan()
            );
        }
        Commands::Os { prompt } => {
            let user_prompt = prompt.join(" ");
            let sp = spinner("Analyzing macOS intent...");
            let ai = esprit_ai::Ai::default_model().unwrap_or_else(|_| {
                fail("AI model not loaded.");
                std::process::exit(1);
            });

            let sys_prompt = format!(
                "You are an expert macOS systems assistant. The user wants to: '{}'. \
                 Write a single valid bash command to achieve this on macOS (using mdfind, osascript, find, etc.). \
                 Rules: 1. Only output the raw command. 2. No markdown, no backticks, no explanations. 3. Be precise.",
                user_prompt
            );

            let mut cmd_str = ai.ask(&sys_prompt).unwrap_or_default().trim().to_string();
            cmd_str = cmd_str.trim_matches('`').trim().to_string();
            sp.finish_and_clear();

            println!(
                "{} 🤖 Omni-Agent proposes: {}",
                "✨".cyan(),
                cmd_str.yellow().bold()
            );

            let safe_prefixes = [
                "mdfind", "find", "ls", "cat", "echo", "pwd", "date", "whoami", "grep", "rg",
            ];
            let is_safe = safe_prefixes.iter().any(|p| cmd_str.starts_with(p));

            if !is_safe {
                print!(
                    "{} ⚠️  Command may be mutating. Execute? [y/N]: ",
                    "🛡️".red()
                );
                use std::io::Write;
                std::io::stdout().flush().unwrap();
                let mut input = String::new();
                std::io::stdin().read_line(&mut input).unwrap();
                if input.trim().to_lowercase() != "y" {
                    fail("Execution aborted by user.");
                    return Ok(());
                }
            }

            let sp_exec = spinner("Executing on macOS...");
            let output = std::process::Command::new("bash")
                .arg("-c")
                .arg(&cmd_str)
                .output();
            sp_exec.finish_and_clear();

            match output {
                Ok(out) => {
                    let stdout = String::from_utf8_lossy(&out.stdout).to_string();
                    let stderr = String::from_utf8_lossy(&out.stderr).to_string();

                    if stdout.trim().is_empty() && stderr.trim().is_empty() {
                        ok("Command executed successfully with no output.");
                        return Ok(());
                    }

                    let sp_format = spinner("Formatting results...");
                    let format_prompt = format!(
                        "The user asked: '{}'. The macOS command `{}` output:
STDOUT:
{}
STDERR:
{}

Summarize this output clearly and concisely in natural English for the user.",
                        user_prompt,
                        cmd_str,
                        stdout.chars().take(3000).collect::<String>(),
                        stderr.chars().take(1000).collect::<String>()
                    );

                    let final_ans = ai.ask(&format_prompt).unwrap_or_default();
                    sp_format.finish_and_clear();

                    println!(
                        "
{}
",
                        final_ans.cyan()
                    );
                }
                Err(e) => {
                    fail(&format!("Failed to execute command: {}", e));
                }
            }
        }

        // ── version ──────────────────────────────────────────────────────────
        Commands::Version => {
            println!("{}", esprit_core::banner().cyan().bold());
        }

        // ── doctor ───────────────────────────────────────────────────────────
        Commands::Doctor => {
            ui::banner();
            let sp = ui::spinner("Gathering system & model telemetry…");
            let report = esprit_platform::doctor();
            sp.finish_and_clear();

            ui::panel_header("Host Architecture & Environment", Some("Hardware"));
            ui::kv_dot("Operating System", &report.os);
            ui::kv_dot("Kernel Version", &report.kernel);
            ui::kv_dot("Host Architecture", &report.cpu);
            ui::kv_dot(
                "CPU Core Count",
                &format!("{} logical cores", report.cpu_cores),
            );
            ui::kv_dot("System Memory", &format!("{:.1} GB RAM", report.ram_gb));

            ui::panel_header("AI Inference Engine", Some("Apple Silicon Metal"));
            #[cfg(target_os = "macos")]
            ui::kv_dot(
                "Inference Backend",
                "Embedded llama.cpp (Metal GPU Acceleration)",
            );
            #[cfg(not(target_os = "macos"))]
            ui::kv_dot(
                "Inference Backend",
                "Embedded llama.cpp (Multi-threaded CPU)",
            );

            if let Ok(models) = esprit_models::list_status() {
                let installed: Vec<_> = models.into_iter().filter(|(_, inst)| *inst).collect();
                if installed.is_empty() {
                    println!(
                        "\n  {} No models installed yet — run: {}",
                        "⚠".yellow().bold(),
                        "esprit init".bold().cyan()
                    );
                } else {
                    println!();
                    for (entry, _) in installed {
                        println!(
                            "  {} {:<16} {}",
                            "✓".green().bold(),
                            entry.id.bold(),
                            entry.display.dimmed()
                        );
                    }
                }
            }

            ui::panel_header("Developer Toolchain Status", Some("CLI Tools"));
            let chk = |ok: bool, name: &str, ver: Option<&str>| {
                if ok {
                    if let Some(v) = ver {
                        println!(
                            "  {} {:<16} {}",
                            "✓".green().bold(),
                            name.bold(),
                            v.dimmed()
                        );
                    } else {
                        println!("  {} {:<16}", "✓".green().bold(), name.bold());
                    }
                } else {
                    println!(
                        "  {} {:<16} {}",
                        "○".dimmed(),
                        name.dimmed(),
                        "not found (optional)".dimmed()
                    );
                }
            };
            chk(report.git, "git", report.git_version.as_deref());
            chk(report.rust, "rustc", report.rust_version.as_deref());
            chk(report.cargo, "cargo", None);
            chk(report.ollama, "ollama", report.ollama_version.as_deref());

            ui::panel_header("Knowledge Base & Index", Some("Tantivy / SQLite"));
            match esprit_index::index_stats() {
                Ok(s) => {
                    ui::kv_dot("Indexed Files", &s.file_count.to_string());
                    ui::kv_dot(
                        "Repository Size",
                        &format!("{:.1} MB", s.total_bytes as f64 / 1_048_576.0),
                    );
                }
                Err(_) => {
                    ui::warn("No index created yet. Run `esprit index .` to index workspace.");
                }
            }
            println!();
        }

        // ── config ───────────────────────────────────────────────────────────
        Commands::Config { set_model } => {
            let mut cfg = esprit_config::Config::load()?;

            if let Some(model) = set_model {
                cfg.set_model(&model)?;
                ok(&format!("AI model set to {}", model.bold()));
                return Ok(());
            }

            section("Esprit Configuration");
            divider();
            kv("AI Model", &cfg.ai_model);
            kv("Ollama URL", &cfg.ollama_url);
            kv("Workspace", &cfg.workspace.display().to_string());
            kv("Threads", &cfg.threads.to_string());
            kv("Color", if cfg.color { "enabled" } else { "disabled" });
            kv(
                "Context chars/file",
                &cfg.context_chars_per_file.to_string(),
            );
            kv("Max context files", &cfg.max_context_files.to_string());
            println!();
        }

        // ── search ───────────────────────────────────────────────────────────
        Commands::Search {
            pattern,
            regex: _,
            all,
        } => {
            let sp = spinner(&format!("Searching for \"{}\"…", pattern.bold()));
            let t = Instant::now();
            let results = if all {
                esprit_index::search_all_workspaces(&pattern)
            } else {
                esprit_index::search(&pattern)
            };
            sp.finish_and_clear();

            match results {
                Err(e) => {
                    fail(&format!("Search failed: {e}"));
                    warn("Run `esprit rebuild` to build the search index first.");
                }
                Ok(results) => {
                    if results.is_empty() {
                        println!(
                            "\n  {} No results for \"{}\"\n",
                            "○".dimmed(),
                            pattern.yellow()
                        );
                    } else {
                        println!(
                            "\n  {} Found {} match{} for \"{}\"  {}\n",
                            "●".cyan(),
                            results.len().to_string().bold(),
                            if results.len() == 1 { "" } else { "es" },
                            pattern.yellow(),
                            elapsed(t).dimmed(),
                        );
                        for path in &results {
                            println!("  {} {}", "│".dimmed(), path.cyan());
                        }
                        println!();
                    }
                }
            }
        }

        // ── stats ────────────────────────────────────────────────────────────
        Commands::Stats { folder } => {
            let sp = spinner(&format!("Scanning {}…", folder.bold()));
            let t = Instant::now();
            let stats = esprit_filesystem::stats::FolderStats::scan(&folder)?;
            sp.finish_and_clear();

            section(&format!("Stats — {folder}"));
            divider();
            kv("Files", &stats.files.to_string());
            kv("Directories", &stats.directories.to_string());
            kv(
                "Total size",
                &format!("{:.1} MB", stats.bytes as f64 / 1_048_576.0),
            );

            let mut exts: Vec<_> = stats.extensions.into_iter().collect();
            exts.sort_by_key(|(_, n)| std::cmp::Reverse(*n));

            println!();
            println!("  {}", "Extensions:".bold());
            for (ext, count) in exts.iter().take(15) {
                let bar_len = (count * 20 / exts[0].1).max(1) as usize;
                println!(
                    "  {:<12} {:>6}  {}",
                    ext.cyan(),
                    count.to_string().bold(),
                    "█".repeat(bar_len).green()
                );
            }
            println!("\n  Scanned in {}\n", elapsed(t).dimmed());
        }

        // ── organize ─────────────────────────────────────────────────────────
        Commands::Organize { folder, dry_run } => {
            let sp = spinner(if dry_run {
                "Previewing organisation…"
            } else {
                "Organising files…"
            });
            let ops = if dry_run {
                esprit_filesystem::organize_dry_run(&folder)?
            } else {
                esprit_filesystem::organize(&folder)?
            };
            sp.finish_and_clear();

            if ops.is_empty() {
                ok("Nothing to organise.");
            } else {
                let label = if dry_run { "Would move" } else { "Moved" };
                println!(
                    "\n  {} {} file{}\n",
                    "●".cyan(),
                    ops.len(),
                    if ops.len() == 1 { "" } else { "s" }
                );
                for op in &ops {
                    println!(
                        "  {} {} {} {}",
                        "→".dimmed(),
                        op.from
                            .file_name()
                            .unwrap_or_default()
                            .to_string_lossy()
                            .bold(),
                        "→".dimmed(),
                        op.to.display().to_string().cyan()
                    );
                }
                if dry_run {
                    println!(
                        "\n  {} Dry run — no files were moved. Remove --dry-run to apply.\n",
                        "⚠".yellow()
                    );
                } else {
                    println!("\n  {} {label} {} files.\n", "✓".green(), ops.len());
                }
            }
        }

        // ── duplicates ───────────────────────────────────────────────────────
        Commands::Duplicates { folder } => {
            let sp = spinner(&format!("Hashing files in {}…", folder.bold()));
            let t = Instant::now();
            let groups = esprit_filesystem::duplicates(&folder)?;
            sp.finish_and_clear();

            if groups.is_empty() {
                ok(&format!("No duplicates found.  {}", elapsed(t).dimmed()));
            } else {
                println!(
                    "\n  {} {} duplicate group{}  {}\n",
                    "⚠".yellow().bold(),
                    groups.len().to_string().bold(),
                    if groups.len() == 1 { "" } else { "s" },
                    elapsed(t).dimmed(),
                );
                for (i, group) in groups.iter().enumerate() {
                    println!("  Group {}", (i + 1).to_string().bold().yellow());
                    for file in group {
                        println!("    {} {}", "│".dimmed(), file.display().to_string().cyan());
                    }
                    println!();
                }
            }
        }

        // ── index ────────────────────────────────────────────────────────────
        Commands::Index { folder } => {
            let t = Instant::now();
            let sp = spinner(&format!("Indexing {}…", folder.bold()));
            let files = esprit_index::index(&folder);
            sp.finish_and_clear();

            match files {
                Err(e) => fail(&format!("Indexing failed: {e}")),
                Ok(files) => {
                    ok(&format!(
                        "Indexed {} files  {}",
                        files.len().to_string().bold(),
                        elapsed(t).dimmed()
                    ));
                    println!(
                        "  {} Run {} to build full-text search.\n",
                        "→".dimmed(),
                        "esprit rebuild".bold().cyan()
                    );
                }
            }
        }

        // ── rebuild ──────────────────────────────────────────────────────────
        Commands::Rebuild => {
            let t = Instant::now();
            let sp = spinner("Building full-text search index…");
            let res = esprit_index::rebuild_search_index();
            sp.finish_and_clear();

            match res {
                Err(e) => fail(&format!("Rebuild failed: {e}")),
                Ok(()) => ok(&format!("Search index built  {}", elapsed(t).dimmed())),
            }
        }

        // ── db ───────────────────────────────────────────────────────────────
        Commands::Db => {
            let files = esprit_index::all_files()?;
            println!(
                "\n  {} {} file{} in index\n",
                "●".cyan(),
                files.len().to_string().bold(),
                if files.len() == 1 { "" } else { "s" }
            );
            for file in &files {
                println!(
                    "  {}{:>10}  {}",
                    "│".dimmed(),
                    format_size(file.size),
                    file.path.display().to_string().cyan()
                );
            }
            println!();
        }

        // ── index-stats ──────────────────────────────────────────────────────
        Commands::IndexStats => {
            let s = esprit_index::index_stats()?;
            section("Index Summary");
            divider();
            kv("Files indexed", &s.file_count.to_string());
            kv(
                "Total size",
                &format!("{:.1} MB", s.total_bytes as f64 / 1_048_576.0),
            );
            println!();
        }

        // ── watch ────────────────────────────────────────────────────────────
        Commands::Watch { folder } => {
            println!(
                "\n  {} Watching {}  (press Ctrl-C to stop)\n",
                "👁".bold(),
                folder.bold().cyan()
            );
            esprit_platform::watch(&folder)?;
        }

        // ── ask ──────────────────────────────────────────────────────────────
        Commands::Ask { prompt, sources } => {
            // Optionally show sources first (pre-retrieval)
            if sources {
                let sp = spinner("Retrieving context…");
                let srcs = esprit_rag::source_files(&prompt).unwrap_or_default();
                sp.finish_and_clear();

                if !srcs.is_empty() {
                    println!("\n  {}", "Sources:".bold().dimmed());
                    for s in &srcs {
                        println!("  {} {}", "·".dimmed(), s.cyan());
                    }
                    println!();
                }
            }

            let sp = spinner("Thinking…");
            let t = Instant::now();
            let _ai = esprit_ai::Ai::default_model().unwrap_or_else(|_| {
                fail("AI model not loaded. Run `esprit init` first.");
                std::process::exit(1);
            });
            sp.finish_and_clear();

            println!("\n  {}\n", "─".repeat(52).dimmed());
            print!("  "); // Initial indent

            let mut current_line_len = 2;
            let result = esprit_rag::ask_stream(&prompt, |chunk| {
                for c in chunk.chars() {
                    if c == '\n' {
                        println!();
                        print!("  ");
                        current_line_len = 2;
                    } else {
                        print!("{c}");
                        current_line_len += 1;
                        if current_line_len >= 74 && c.is_whitespace() {
                            println!();
                            print!("  ");
                            current_line_len = 2;
                        }
                    }
                }
                use std::io::Write;
                let _ = std::io::stdout().flush();
            });

            match result {
                Err(e) => {
                    fail(&format!("{e}"));
                    warn("Is Ollama running?  Try: esprit model pull <id>");
                }
                Ok((_, meta)) => {
                    println!("\n\n  {}\n", "─".repeat(52).dimmed());
                    println!(
                        "  {} {} tokens  {}",
                        "⏱".dimmed(),
                        meta.tokens.to_string().dimmed(),
                        elapsed(t).dimmed()
                    );
                    println!();
                }
            }
        }

        // ── agent ────────────────────────────────────────────────────────────
        Commands::Agent { agent, prompt } => {
            use esprit_agents::Agent;
            let ag = match agent.as_str() {
                "chat" => Agent::Chat,
                "code" => Agent::Code,
                "search" => Agent::Search,
                other => anyhow::bail!("unknown agent \"{other}\" — choose: chat | code | search"),
            };

            let sp = spinner(&format!("Running {} agent…", agent.bold()));
            let t = Instant::now();
            let res = esprit_agents::run(ag, &prompt);
            sp.finish_and_clear();

            match res {
                Err(e) => fail(&e.to_string()),
                Ok(out) => {
                    println!("\n  {}\n", "─".repeat(52).dimmed());
                    for line in out.lines() {
                        println!("  {line}");
                    }
                    println!("\n  {}  {}\n", "─".repeat(52).dimmed(), elapsed(t).dimmed());
                }
            }
        }

        // ── workflow ─────────────────────────────────────────────────────────
        Commands::Workflow { workflow, prompt } => {
            let sp = spinner(&format!("Running {} workflow…", workflow.bold()));
            let t = Instant::now();
            let out = match workflow.as_str() {
                "explain" => esprit_workflows::explain(&prompt),
                "review" => esprit_workflows::code_review(&prompt),
                "search" => esprit_workflows::project_search(&prompt),
                other => anyhow::bail!(
                    "unknown workflow \"{other}\" — choose: explain | review | search"
                ),
            };
            sp.finish_and_clear();

            match out {
                Err(e) => fail(&e.to_string()),
                Ok(text) => {
                    println!("\n  {}\n", "─".repeat(52).dimmed());
                    for line in text.lines() {
                        println!("  {line}");
                    }
                    println!("\n  {}  {}\n", "─".repeat(52).dimmed(), elapsed(t).dimmed());
                }
            }
        }

        // ── memory-clear ─────────────────────────────────────────────────────
        Commands::MemoryClear => {
            esprit_memory::clear()?;
            ok("Conversation memory cleared.");
        }

        Commands::MemoryCompress => {
            let sp = spinner("Compressing memory...");
            let ai = esprit_ai::Ai::default_model().unwrap_or_else(|_| {
                fail("AI model not loaded.");
                std::process::exit(1);
            });
            let rows = esprit_memory::recall(10).unwrap_or_default();
            if rows.is_empty() {
                sp.finish_and_clear();
                ok("Memory is already empty.");
            } else {
                let mut ctx = String::new();
                for (id, txt) in rows {
                    ctx.push_str(&format!("{id}: {txt}\n"));
                }
                let prompt = format!(
                    "Summarize this chat history into a dense knowledge summary:

{ctx}"
                );
                if let Ok(summary) = ai.ask(&prompt) {
                    let _ = esprit_memory::clear();
                    let _ =
                        esprit_memory::remember("System", &format!("Compressed Memory: {summary}"));
                    sp.finish_and_clear();
                    ok("Memory compressed successfully.");
                } else {
                    sp.finish_and_clear();
                    fail("Failed to compress memory.");
                }
            }
        }

        Commands::MemoryStats => {
            let n = esprit_memory::count()?;
            section("Conversation Memory");
            divider();
            kv("Stored exchanges", &n.to_string());
            println!();
        }

        // ── diary ────────────────────────────────────────────────────────────
        Commands::Diary { note } => {
            let branch = esprit_platform::doctor::capture("git", &["branch", "--show-current"])
                .unwrap_or_else(|| "main".to_string());
            let branch = branch.trim();

            if let Some(content) = note {
                diary::add_note(branch, &content)?;
                ok(&format!("Saved note to branch '{}'", branch.bold()));
            } else {
                let notes = diary::list_notes(branch)?;
                section(&format!("Diary notes for '{}'", branch.bold()));
                divider();
                if notes.is_empty() {
                    println!("  (No notes found)");
                } else {
                    for (i, n) in notes.iter().enumerate() {
                        println!("  {}. {}", i + 1, n);
                    }
                }
                println!();
            }
        }

        // ── dashboard ────────────────────────────────────────────────────────
        Commands::Graph => {
            let graph = esprit_index::graph::build_graph()?;
            let mermaid = esprit_index::graph::to_mermaid(&graph);
            println!("{mermaid}");
        }
        Commands::Deps => {
            let graph = esprit_index::graph::build_graph()?;
            section("Project Dependencies (from index)");
            divider();
            for edge in graph.graph.edge_indices() {
                if let Some((s_idx, t_idx)) = graph.graph.edge_endpoints(edge) {
                    let src = &graph.graph[s_idx];
                    let tgt = &graph.graph[t_idx];
                    let kind = &graph.graph[edge];
                    println!("  {src} -> {tgt} ({kind})");
                }
            }
            println!();
        }
        Commands::Diff => {
            let sp = spinner("Analyzing git diff…");
            let diff =
                esprit_platform::doctor::capture("git", &["diff", "HEAD"]).unwrap_or_default();
            let ai = esprit_ai::Ai::default_model().unwrap_or_else(|_| {
                fail("AI model not loaded. Run `esprit init` first.");
                std::process::exit(1);
            });
            sp.finish_and_clear();
            if diff.trim().is_empty() {
                println!("  No changes found in git diff.");
            } else {
                let prompt =
                    format!("Summarize this git diff into a concise commit message:\n\n{diff}");
                println!("\n  {}\n", "─".repeat(52).dimmed());
                let _ = ai.ask_stream(&prompt, |c| print!("{c}"));
                println!("\n\n  {}\n", "─".repeat(52).dimmed());
            }
        }
        Commands::Review => {
            let sp = spinner("Reviewing staged changes…");
            let diff =
                esprit_platform::doctor::capture("git", &["diff", "--cached"]).unwrap_or_default();
            let ai = esprit_ai::Ai::default_model().unwrap_or_else(|_| {
                fail("AI model not loaded. Run `esprit init` first.");
                std::process::exit(1);
            });
            sp.finish_and_clear();
            if diff.trim().is_empty() {
                println!("  No staged changes to review.");
            } else {
                let prompt = format!("Perform a strict code review on these changes. Focus on security, performance, and best practices:\n\n{diff}");
                println!("\n  {}\n", "─".repeat(52).dimmed());
                let _ = ai.ask_stream(&prompt, |c| print!("{c}"));
                println!("\n\n  {}\n", "─".repeat(52).dimmed());
            }
        }
        Commands::Annotate { file } => {
            let sp = spinner(&format!("Annotating {file}…"));
            if let Ok(content) = std::fs::read_to_string(&file) {
                let ai = esprit_ai::Ai::default_model().unwrap_or_else(|_| {
                    fail("AI model not loaded. Run `esprit init` first.");
                    std::process::exit(1);
                });
                sp.finish_and_clear();
                let prompt = format!("Add missing docstrings and inline comments to this code. Output ONLY the updated code:\n\n{content}");
                println!("\n  {}\n", "─".repeat(52).dimmed());
                let _ = ai.ask_stream(&prompt, |c| print!("{c}"));
                println!("\n\n  {}\n", "─".repeat(52).dimmed());
            } else {
                sp.finish_and_clear();
                fail("Could not read file");
            }
        }
        Commands::Plan { goal } => {
            let sp = spinner("Formulating refactoring plan…");
            let ai = esprit_ai::Ai::default_model().unwrap_or_else(|_| {
                fail("AI model not loaded. Run `esprit init` first.");
                std::process::exit(1);
            });
            sp.finish_and_clear();
            let prompt = format!(
                "Create a step-by-step structural refactoring plan to achieve this goal:\n\n{goal}"
            );
            println!("\n  {}\n", "─".repeat(52).dimmed());
            let _ = ai.ask_stream(&prompt, |c| print!("{c}"));
            println!("\n\n  {}\n", "─".repeat(52).dimmed());
        }
        Commands::Triage => {
            let sp = spinner("Triaging impact…");
            let diff = esprit_platform::doctor::capture("git", &["diff", "--name-only", "HEAD"])
                .unwrap_or_default();
            let ai = esprit_ai::Ai::default_model().unwrap_or_else(|_| {
                fail("AI model not loaded. Run `esprit init` first.");
                std::process::exit(1);
            });
            sp.finish_and_clear();
            let prompt = format!("Based on these modified files, list the specific test files or testing areas that MUST be run to ensure no regressions:\n\n{diff}");
            println!("\n  {}\n", "─".repeat(52).dimmed());
            let _ = ai.ask_stream(&prompt, |c| print!("{c}"));
            println!("\n\n  {}\n", "─".repeat(52).dimmed());
        }
        Commands::Export => {
            let sp = spinner("Compiling Project Intelligence…");
            let files = esprit_index::all_files()?;
            let mut md = String::from("# Project Onboarding Guide\n\n## Indexed Files\n\n");
            for f in &files {
                md.push_str(&format!("- {}\n", f.path.display()));
            }
            std::fs::write("esprit_onboarding.md", md)?;
            sp.finish_and_clear();
            ok("Exported project intelligence to esprit_onboarding.md");
        }

        Commands::Daemon => {
            daemon::run_daemon()?;
        }

        Commands::Plugin { file, input } => {
            let sp = spinner("Executing WASM plugin…");
            let input_val = input.unwrap_or_default();
            let result = esprit_plugins::run_plugin(&file, &input_val);
            sp.finish_and_clear();
            match result {
                Ok(out) => println!("  {} {}", "⚡".cyan(), out),
                Err(e) => fail(&e.to_string()),
            }
        }

        Commands::TestGaps => {
            section("Test Gap Finder");
            divider();
            let files = esprit_index::all_files()?;
            let mut sources = Vec::new();
            let mut tests = Vec::new();
            for f in files {
                let s = f.path.to_string_lossy().to_string();
                if s.contains("test") || s.contains("spec") {
                    tests.push(s);
                } else if s.ends_with(".rs")
                    || s.ends_with(".js")
                    || s.ends_with(".ts")
                    || s.ends_with(".py")
                {
                    sources.push(s);
                }
            }
            let mut gaps = 0;
            for src in &sources {
                let name = std::path::Path::new(src)
                    .file_stem()
                    .unwrap_or_default()
                    .to_string_lossy();
                let has_test = tests.iter().any(|t| t.contains(&*name));
                if !has_test {
                    println!("  {} Missing test for: {}", "○".dimmed(), src);
                    gaps += 1;
                }
            }
            println!("\n  Found {gaps} source files without matching test files.\n");
        }

        Commands::Find { query } => {
            let sp = spinner("Semantic search...");
            let _ = esprit_ai::Ai::default_model().unwrap_or_else(|_| {
                fail("AI model not loaded. Run `esprit init` first.");
                std::process::exit(1);
            });
            let emb = esprit_embeddings::embed(&query)
                .unwrap_or_default()
                .unwrap_or_default();
            sp.finish_and_clear();
            if !emb.is_empty() {
                let results = esprit_vectors::nearest(&emb, 5).unwrap_or_default();
                section("Semantic Search Results");
                divider();
                if results.is_empty() {
                    println!("  No indexed code found. Have you modified files yet?");
                } else {
                    for (path, score) in results {
                        println!("  {} [{:.2}] {}", "○".dimmed(), score, path.cyan());
                    }
                }
            } else {
                fail("Embedding model not ready.");
            }
            println!();
        }

        Commands::Trace { function } => {
            let sp = spinner(&format!("Tracing function {}...", function.bold()));
            let ai = esprit_ai::Ai::default_model().unwrap_or_else(|_| {
                fail("AI model not loaded. Run `esprit init` first.");
                std::process::exit(1);
            });

            // 1. Ripgrep to find the function
            let rg_out = esprit_platform::doctor::capture("rg", &["-n", "-C", "10", &function])
                .unwrap_or_default();
            sp.finish_and_clear();

            if rg_out.is_empty() {
                fail("Function not found in the codebase.");
            } else {
                let prompt = format!(
                    "Analyze the following code search results for the function `{}`.
                     Identify what this function does, what other internal functions it calls, and what external dependencies it relies on.
                     Draw a textual call graph.

Code context:
{}",
                    function,
                    rg_out.chars().take(4000).collect::<String>()
                );
                section(&format!("Call Trace: {}", function));
                divider();
                let _ = ai.ask_stream(&prompt, |c| print!("{c}"));
                println!();
            }
        }

        Commands::Drift { file } => {
            let sp = spinner(&format!(
                "Checking for documentation drift in {}...",
                file.bold()
            ));
            let ai = esprit_ai::Ai::default_model().unwrap_or_else(|_| {
                fail("AI model not loaded.");
                std::process::exit(1);
            });
            if let Ok(content) = std::fs::read_to_string(&file) {
                sp.finish_and_clear();
                let prompt = format!(
                    "Identify any docstrings that NO LONGER MATCH the logic in this code.

Code:
{}",
                    content
                );
                section(&format!("Doc-Drift Report: {}", file));
                divider();
                let _ = ai.ask_stream(&prompt, |c| print!("{c}"));
                println!();
            } else {
                sp.finish_and_clear();
                fail("Could not read file.");
            }
        }

        Commands::Debate { topic } => {
            let sp = spinner("Consulting Security and Performance agents...");
            let ai = esprit_ai::Ai::default_model().unwrap_or_else(|_| {
                fail("AI model not loaded.");
                std::process::exit(1);
            });

            let sec_prompt = format!("You are a paranoid Security Engineer. Argue the security risks and defenses for this architectural topic: {topic}");
            let perf_prompt = format!("You are a ruthless Performance Engineer. Argue the performance benefits and bottlenecks for this architectural topic: {topic}");

            let sec_resp = ai.ask(&sec_prompt).unwrap_or_default();
            let perf_resp = ai.ask(&perf_prompt).unwrap_or_default();
            sp.finish_and_clear();

            section("🛡️ Security Engineer Persona");
            println!(
                "
{}
",
                sec_resp.cyan()
            );
            divider();
            section("⚡ Performance Engineer Persona");
            println!(
                "
{}
",
                perf_resp.yellow()
            );
        }

        Commands::EnvScaffold => {
            let sp = spinner("Scanning codebase for environment variables...");
            let rg_out = esprit_platform::doctor::capture(
                "rg",
                &[
                    "-o",
                    r#"(?:std::env::var|process\.env\.|"env": ")[A-Z0-9_]+"#,
                ],
            )
            .unwrap_or_default();
            sp.finish_and_clear();

            let mut vars = std::collections::HashSet::new();
            for line in rg_out.lines() {
                if let Some(v) = line.split(&['"', '(', ')', '.'][..]).next_back() {
                    let cleaned = v.trim_matches(|c: char| !c.is_alphanumeric() && c != '_');
                    if !cleaned.is_empty() && cleaned.chars().any(|c| c.is_alphabetic()) {
                        vars.insert(cleaned.to_string());
                    }
                }
            }

            if vars.is_empty() {
                ok("No environment variables detected in the codebase.");
            } else {
                let mut out = String::new();
                for v in &vars {
                    out.push_str(&format!(
                        "{v}=
"
                    ));
                }
                std::fs::write(".env.example", &out).unwrap();
                ok(&format!(
                    "Successfully scaffolded .env.example with {} variables.",
                    vars.len()
                ));
            }
        }

        Commands::Docker => {
            let sp = spinner("Analyzing project structure for Docker optimization...");
            let ai = esprit_ai::Ai::default_model().unwrap_or_else(|_| {
                fail("AI model not loaded.");
                std::process::exit(1);
            });

            let toml = std::fs::read_to_string("Cargo.toml").unwrap_or_default();
            sp.finish_and_clear();

            let prompt = format!("Write a secure, multi-stage, distroless Dockerfile for this Rust project. Only output the raw Dockerfile.

Cargo.toml:
{}", toml);
            section("🐳 Optimized Dockerfile");
            divider();
            let _ = ai.ask_stream(&prompt, |c| print!("{c}"));
            println!();
        }

        Commands::Score { file } => {
            let sp = spinner(&format!(
                "Calculating Cognitive Load Score for {}...",
                file.bold()
            ));
            let ai = esprit_ai::Ai::default_model().unwrap_or_else(|_| {
                fail("AI model not loaded.");
                std::process::exit(1);
            });

            if let Ok(content) = std::fs::read_to_string(&file) {
                sp.finish_and_clear();
                let prompt = format!("Grade this code based on Human Cognitive Load (readability, variable names, abstraction complexity) out of 100. Be ruthless. Suggest 3 improvements.

Code:
{}", content.chars().take(4000).collect::<String>());
                section(&format!("Cognitive Load Score: {}", file));
                divider();
                let _ = ai.ask_stream(&prompt, |c| print!("{c}"));
                println!();
            } else {
                sp.finish_and_clear();
                fail("Could not read file.");
            }
        }

        Commands::Dashboard => {
            dashboard::run()?;
        }

        // ── init ─────────────────────────────────────────────────────────────
        Commands::Init { with_embeddings } => {
            println!("{}", esprit_core::banner().cyan().bold());
            println!("\n  Setting up Esprit — this downloads AI models on first run.\n");

            let llm = esprit_models::default_llm();
            if esprit_models::is_installed(llm)? {
                ok(&format!("{} is already installed.", llm.display));
            } else {
                println!(
                    "  {} Downloading {} (~{} MB)…\n",
                    "⬇".cyan().bold(),
                    llm.display.bold(),
                    llm.size_bytes / 1_000_000
                );
                esprit_models::pull(llm)?;
            }

            if with_embeddings {
                let emb = esprit_models::default_embed();
                if esprit_models::is_installed(emb)? {
                    ok(&format!("{} is already installed.", emb.display));
                } else {
                    println!(
                        "\n  {} Downloading {} (~{} MB)…\n",
                        "⬇".cyan().bold(),
                        emb.display.bold(),
                        emb.size_bytes / 1_000_000
                    );
                    esprit_models::pull(emb)?;
                }
            }

            println!();
            ok("Esprit is ready.");
            println!(
                "\n  {} Try: {}\n  {} Try: {}\n",
                "→".dimmed(),
                "esprit ask \"what does this project do?\"".bold().cyan(),
                "→".dimmed(),
                "esprit doctor".bold().cyan(),
            );
        }

        // ── model ─────────────────────────────────────────────────────────────
        Commands::Model { action } => match action {
            ModelAction::List => {
                section("Available Models");
                divider();
                for (entry, installed) in esprit_models::list_status()? {
                    let status = if installed {
                        "✓ installed".green().bold().to_string()
                    } else {
                        "○ not installed".dimmed().to_string()
                    };
                    let kind = match entry.kind {
                        esprit_models::ModelKind::Llm => "LLM",
                        esprit_models::ModelKind::Embedding => "Embed",
                    };
                    println!(
                        "  {:<14} {:<8} {:<30} {}",
                        entry.id.bold(),
                        kind.dimmed(),
                        entry.display,
                        status
                    );
                }
                println!(
                    "\n  Pull a model: {}\n",
                    "esprit model pull <id>".bold().cyan()
                );
            }

            ModelAction::Pull { id } => {
                let entry = esprit_models::lookup(&id).ok_or_else(|| {
                    anyhow::anyhow!("Unknown model \"{id}\" — run `esprit model list`")
                })?;
                println!(
                    "\n  {} Downloading {}…\n",
                    "⬇".cyan().bold(),
                    entry.display.bold()
                );
                esprit_models::pull(entry)?;
                ok("Done.");
            }

            ModelAction::Remove { id } => {
                let entry = esprit_models::lookup(&id)
                    .ok_or_else(|| anyhow::anyhow!("Unknown model \"{id}\""))?;
                esprit_models::remove(entry)?;
                ok(&format!("Removed {}.", entry.display));
            }
        },

        // ── lsp ──────────────────────────────────────────────────────────────
        Commands::Lsp => {
            lsp::run_lsp_server()?;
        }

        // ── blast-radius ─────────────────────────────────────────────────────
        Commands::BlastRadius { target } => {
            let sp = spinner(&format!(
                "Calculating AST blast radius for `{}`…",
                target.bold()
            ));
            let report = esprit_codeintel::calculate_blast_radius(".", &target)?;
            sp.finish_and_clear();

            ui::panel_header("AST Blast Radius Analysis", Some(&target));
            let risk_color = match report.risk_level.as_str() {
                "CRITICAL" => report.risk_level.red().bold().to_string(),
                "HIGH" => report.risk_level.yellow().bold().to_string(),
                "MEDIUM" => report.risk_level.cyan().bold().to_string(),
                _ => report.risk_level.green().bold().to_string(),
            };

            let mut card_lines = vec![
                format!("Risk Assessment:       {}", risk_color),
                format!(
                    "Direct Dependents:     {} files",
                    report.total_impacted_files
                ),
                format!(
                    "Total Call References: {} occurrences",
                    report.total_references
                ),
            ];
            if !report.impacted_test_files.is_empty() {
                card_lines.push(format!(
                    "Impacted Test Suites:  {} test files",
                    report.impacted_test_files.len()
                ));
            }
            ui::card("IMPACT SURFACE SUMMARY", &card_lines);

            if !report.direct_dependents.is_empty() {
                println!();
                ui::section("Dependent Files");
                for dep in report.direct_dependents.iter().take(10) {
                    println!("  {} {}", "•".cyan(), dep);
                }
                if report.direct_dependents.len() > 10 {
                    println!(
                        "  {} … and {} more files",
                        "•".dimmed(),
                        report.direct_dependents.len() - 10
                    );
                }
            }

            if !report.recommended_actions.is_empty() {
                println!();
                ui::section("Recommended Next Steps");
                for act in &report.recommended_actions {
                    println!("  {} {}", "✓".green(), act);
                }
            }
            println!();
        }

        // ── complexity ───────────────────────────────────────────────────────
        Commands::Complexity { path } => {
            let target = path.unwrap_or_else(|| "crates/esprit-core/src/lib.rs".to_string());
            let sp = spinner(&format!("Analyzing complexity hotspots for `{}`…", target));
            match esprit_codeintel::analyze_complexity(&target) {
                Ok(rep) => {
                    sp.finish_and_clear();
                    ui::panel_header("Cognitive & Cyclomatic Complexity", Some(&target));
                    ui::card(
                        "COMPLEXITY METRICS",
                        &[
                            format!("Cognitive Score:       {}", rep.cognitive_score),
                            format!("Cyclomatic Score:      {}", rep.cyclomatic_score),
                            format!("Lines of Code:         {}", rep.lines_of_code),
                            format!("Rating:                {}", rep.rating.cyan().bold()),
                        ],
                    );
                    if !rep.hotspots.is_empty() {
                        println!();
                        ui::section("Complexity Hotspots");
                        for h in rep.hotspots.iter().take(5) {
                            println!("  {} Line {}: {}", "⚠".yellow(), h.line, h.reason);
                            println!("    {}", h.preview.dimmed());
                        }
                    }
                    println!();
                }
                Err(_) => {
                    sp.finish_and_clear();
                    ui::fail(&format!("Could not analyze complexity for `{}`", target));
                }
            }
        }

        // ── dead-code ────────────────────────────────────────────────────────
        Commands::DeadCode { path } => {
            let target = path.unwrap_or_else(|| ".".to_string());
            let sp = spinner(&format!(
                "Scanning for orphaned & dead symbols in `{}`…",
                target
            ));
            let findings = esprit_codeintel::find_dead_code(&target)?;
            sp.finish_and_clear();

            ui::panel_header("Dead Code & Orphan Pruning", Some(&target));
            if findings.is_empty() {
                ui::ok("No dead or unreferenced symbols detected. Clean codebase!");
            } else {
                println!(
                    "  Found {} potentially unreferenced symbols:\n",
                    findings.len().to_string().yellow().bold()
                );
                for f in findings.iter().take(15) {
                    println!(
                        "  {} {} `{}` at `{}:{}`",
                        "○".dimmed(),
                        f.kind.cyan(),
                        f.symbol.bold(),
                        f.file,
                        f.line
                    );
                }
                if findings.len() > 15 {
                    println!("  … and {} more symbols.", findings.len() - 15);
                }
            }
            println!();
        }

        // ── scan-secrets ─────────────────────────────────────────────────────
        Commands::ScanSecrets { path } => {
            let target = path.unwrap_or_else(|| ".".to_string());
            let sp = spinner(&format!(
                "Scanning for high-entropy secrets and credentials in `{}`…",
                target
            ));
            let scanner = esprit_security::SecretScanner::new();
            let findings = scanner.scan_directory(&target)?;
            sp.finish_and_clear();

            ui::panel_header("Security & Secret Leak Scanner", Some(&target));
            if findings.is_empty() {
                ui::ok("Zero leaked API keys, tokens, or credentials found.");
            } else {
                ui::warn(&format!(
                    "Detected {} potential secret leaks:",
                    findings.len()
                ));
                for f in findings.iter().take(10) {
                    let badge = match f.severity.as_str() {
                        "CRITICAL" => f.severity.red().bold().to_string(),
                        "HIGH" => f.severity.yellow().bold().to_string(),
                        _ => f.severity.cyan().to_string(),
                    };
                    println!(
                        "  [{}] {} in `{}:{}` (Entropy: {:.2}, Preview: {})",
                        badge,
                        f.rule_name.bold(),
                        f.file,
                        f.line_number,
                        f.entropy,
                        f.preview.dimmed()
                    );
                }
            }
            println!();
        }

        // ── tdd ──────────────────────────────────────────────────────────────
        Commands::Tdd { goal } => {
            ui::panel_header("Autonomous TDD Coder Agent", Some(&goal));
            let sp = spinner("Synthesizing test cases and verified implementation…");
            let result = agents_swarm::TddAgent::run_tdd_loop(&goal)?;
            sp.finish_and_clear();
            println!("{}\n", result);
            ui::ok("TDD cycle generated successfully.");
            println!();
        }

        // ── redteam ──────────────────────────────────────────────────────────
        Commands::RedTeam { path } => {
            let target = path.unwrap_or_else(|| ".".to_string());
            ui::panel_header("AppSec Red-Team Vulnerability Audit", Some(&target));
            let sp = spinner("Executing deep security & vulnerability analysis…");
            let result = agents_swarm::RedTeamAgent::audit_codebase(&target)?;
            sp.finish_and_clear();
            println!("{}\n", result);
            ui::ok("Red-Team security audit complete.");
            println!();
        }

        // ── triager ──────────────────────────────────────────────────────────
        Commands::Triager { error_snippet } => {
            ui::panel_header("Production Incident & SRE Triager", None);
            let sp = spinner("Analyzing stack trace and formulating patch diff…");
            let result = agents_swarm::SreTriager::triage_log(&error_snippet)?;
            sp.finish_and_clear();
            println!("{}\n", result);
            println!();
        }

        // ── hyde ─────────────────────────────────────────────────────────────
        Commands::Hyde { query } => {
            ui::panel_header("HyDE Contextual RAG Search", Some(&query));
            let sp = spinner("Generating hypothetical answer & executing hybrid vector retrieval…");
            let (answer, meta) = esprit_rag::ask_with_hyde(&query)?;
            sp.finish_and_clear();
            println!("{}\n", answer);
            ui::ok(&format!(
                "Answer generated in {:.2}s ({} tokens)",
                meta.duration_ms as f32 / 1000.0,
                meta.tokens
            ));
            println!();
        }

        // ── brain-export ─────────────────────────────────────────────────────
        Commands::BrainExport { output } => {
            let dest = output.unwrap_or_else(|| ".esprit_brain.json".to_string());
            let sp = spinner(&format!("Exporting vector knowledge brain to `{}`…", dest));
            let count = esprit_rag::export_brain_to_file(&dest)?;
            sp.finish_and_clear();
            ui::ok(&format!(
                "Exported knowledge brain package with {} vectors to `{}`.",
                count, dest
            ));
            println!();
        }

        // ── ci-gen ───────────────────────────────────────────────────────────
        Commands::CiGen { lang } => {
            let language = lang.unwrap_or_else(|| "rust".to_string());
            let sp = spinner(&format!("Scaffolding CI workflow for `{}`…", language));
            let path = devops::CiGenerator::write_ci_file(&language, ".")?;
            sp.finish_and_clear();
            ui::ok(&format!(
                "Created GitHub Actions matrix workflow at `{}`.",
                path
            ));
            println!();
        }

        // ── docker-gen ───────────────────────────────────────────────────────
        Commands::DockerGen { app_name } => {
            let name = app_name.unwrap_or_else(|| "esprit".to_string());
            let dockerfile = devops::DockerGenerator::generate_distroless_dockerfile(&name);
            let file_path = "Dockerfile.distroless";
            std::fs::write(file_path, &dockerfile)?;
            ui::ok(&format!(
                "Generated multi-stage distroless Dockerfile at `{}`.",
                file_path
            ));
            println!();
        }

        // ── cost-estimate ────────────────────────────────────────────────────
        Commands::CostEstimate {
            service,
            memory_gb,
            cpu_cores,
        } => {
            let svc = service.unwrap_or_else(|| "Production API Container".to_string());
            let estimate =
                devops::CostEstimator::estimate_workload_costs(&svc, memory_gb, cpu_cores);
            ui::panel_header("Cloud Infrastructure Cost Estimator", Some(&svc));
            println!("{}\n", estimate);
        }

        // ── onboarding ───────────────────────────────────────────────────────
        Commands::Onboarding => {
            let sp = spinner("Analyzing codebase structure and generating ONBOARDING.md…");
            let path = collab::OnboardingGenerator::generate_onboarding_guide(".")?;
            sp.finish_and_clear();
            ui::ok(&format!(
                "Comprehensive developer onboarding guide generated at `{}`.",
                path
            ));
            println!();
        }

        // ── style-enforce ────────────────────────────────────────────────────
        Commands::StyleEnforce { path } => {
            let target = path.unwrap_or_else(|| ".".to_string());
            let sp = spinner(&format!(
                "Auditing code conventions & safety in `{}`…",
                target
            ));
            let violations = collab::StyleEnforcer::enforce_conventions(&target)?;
            sp.finish_and_clear();

            ui::panel_header("Code Style & Convention Audit", Some(&target));
            if violations.is_empty() {
                ui::ok("100% compliant with Esprit production code standards.");
            } else {
                ui::warn(&format!(
                    "Found {} convention advisory warnings:",
                    violations.len()
                ));
                for v in violations.iter().take(15) {
                    println!("  {} {}", "•".yellow(), v);
                }
                if violations.len() > 15 {
                    println!("  … and {} more advisories.", violations.len() - 15);
                }
            }
            println!();
        }

        // ── clean ────────────────────────────────────────────────────────────
        Commands::Clean {
            scan,
            all,
            cache,
            leftovers,
            threats,
            dry_run,
            yes,
            json,
        } => {
            cleaner_cmd::run_cleaner(cleaner_cmd::CleanOptions {
                scan,
                all,
                cache,
                leftovers,
                threats,
                dry_run,
                yes,
                json,
            })?;
        }
    }

    Ok(())
}

fn format_size(bytes: u64) -> String {
    if bytes >= 1_048_576 {
        format!("{:.1}M", bytes as f64 / 1_048_576.0)
    } else if bytes >= 1_024 {
        format!("{:.0}K", bytes as f64 / 1_024.0)
    } else {
        format!("{bytes}B")
    }
}

// ── Developer Diary ────────────────────────────────────────────────────────
pub mod diary {
    use anyhow::Result;
    use rusqlite::{params, Connection};
    use std::fs;

    fn db() -> Result<Connection> {
        let dir = esprit_config::Config::load()?.workspace.join(".esprit");
        fs::create_dir_all(&dir)?;
        let conn = Connection::open(dir.join("diary.db"))?;
        conn.execute(
            "CREATE TABLE IF NOT EXISTS diary (
                id INTEGER PRIMARY KEY,
                branch TEXT NOT NULL,
                note TEXT NOT NULL,
                created_at INTEGER NOT NULL DEFAULT (unixepoch())
            )",
            [],
        )?;
        Ok(conn)
    }

    pub fn add_note(branch: &str, note: &str) -> Result<()> {
        let conn = db()?;
        conn.execute(
            "INSERT INTO diary (branch, note) VALUES (?1, ?2)",
            params![branch, note],
        )?;
        Ok(())
    }

    pub fn list_notes(branch: &str) -> Result<Vec<String>> {
        let conn = db()?;
        let mut stmt =
            conn.prepare("SELECT note FROM diary WHERE branch = ?1 ORDER BY created_at DESC")?;
        let rows = stmt.query_map([branch], |r| r.get(0))?;
        Ok(rows.filter_map(Result::ok).collect())
    }
}

// ── Dashboard ────────────────────────────────────────────────────────
pub mod dashboard {
    use anyhow::Result;
    use crossterm::{
        event::{self, Event, KeyCode},
        execute,
        terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    };
    use ratatui::{
        backend::CrosstermBackend,
        layout::{Constraint, Direction, Layout},
        style::{Color, Style},
        widgets::{Block, Borders, Paragraph},
        Terminal,
    };
    use std::io;

    pub fn run() -> Result<()> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        loop {
            terminal.draw(|f| {
                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .margin(1)
                    .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
                    .split(f.area());

                let stats = format!(
                    "Models Installed: {}\nVectors Indexed: {}\nConversations in Memory: {}\n",
                    esprit_models::list_status()
                        .unwrap_or_default()
                        .iter()
                        .filter(|(_, x)| *x)
                        .count(),
                    esprit_vectors::count().unwrap_or(0),
                    esprit_memory::count().unwrap_or(0)
                );

                let top = Paragraph::new(stats)
                    .block(
                        Block::default()
                            .title("Esprit System Health")
                            .borders(Borders::ALL),
                    )
                    .style(Style::default().fg(Color::Cyan));

                let bottom = Paragraph::new("Press 'q' to quit.")
                    .block(Block::default().title("Controls").borders(Borders::ALL));

                f.render_widget(top, chunks[0]);
                f.render_widget(bottom, chunks[1]);
            })?;

            if event::poll(std::time::Duration::from_millis(100))? {
                if let Event::Key(key) = event::read()? {
                    if let KeyCode::Char('q') = key.code {
                        break;
                    }
                }
            }
        }

        disable_raw_mode()?;
        execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
        terminal.show_cursor()?;
        Ok(())
    }
}
