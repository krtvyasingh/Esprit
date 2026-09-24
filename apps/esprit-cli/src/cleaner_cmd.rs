#![forbid(unsafe_code)]

use anyhow::Result;
use esprit_cleaner::{format_bytes, CleanCategory, SystemCleaner, ThreatSeverity};
use owo_colors::OwoColorize;
use std::io::{self, Write};
use std::path::PathBuf;
use std::time::Instant;

use crate::ui;

pub struct CleanOptions {
    pub scan: bool,
    pub all: bool,
    pub cache: bool,
    pub leftovers: bool,
    pub threats: bool,
    pub dry_run: bool,
    pub yes: bool,
    pub json: bool,
}

pub fn run_cleaner(opts: CleanOptions) -> Result<()> {
    let cleaner = SystemCleaner::new();

    if !opts.json {
        ui::banner();
        ui::panel_header(
            "Deep System Diagnostic & Storage Cleaner",
            Some("macOS / Local-First"),
        );
    }

    let start_scan = Instant::now();
    let sp = if !opts.json {
        Some(ui::spinner(
            "Deep scanning system caches, ghost app remnants & LaunchAgents…",
        ))
    } else {
        None
    };

    let report = cleaner.scan_system()?;

    if let Some(s) = sp {
        s.finish_and_clear();
    }

    if opts.json {
        let json_str = serde_json::to_string_pretty(&report)?;
        println!("{}", json_str);
        return Ok(());
    }

    // ── 1. Live Memory & RAM Pressure Card ──────────────────────────────────
    let ram_used = report.memory_stats.used_ram_bytes;
    let ram_total = report.memory_stats.total_ram_bytes;
    let ram_avail = report.memory_stats.available_ram_bytes;
    let ram_percent = (ram_used as f64 / ram_total as f64 * 100.0).round();
    let ram_bar = ui::meter_bar(ram_used, ram_total, 24);

    let swap_used = report.memory_stats.swap_used_bytes;
    let swap_total = report.memory_stats.swap_total_bytes;
    let swap_str = if swap_total > 0 {
        format!(
            "{} / {} (Swap)",
            format_bytes(swap_used),
            format_bytes(swap_total)
        )
    } else {
        "0 B (No Swap Pressure)".to_string()
    };

    ui::card(
        "LIVE UNIFIED MEMORY & SYSTEM PRESSURE",
        &[
            format!("RAM Utilization:     [{}] {:.0}%", ram_bar, ram_percent),
            format!(
                "Physical Memory:     {} Used / {} Total",
                format_bytes(ram_used).bold(),
                format_bytes(ram_total).dimmed()
            ),
            format!(
                "Available Headroom:  {}",
                format_bytes(ram_avail).green().bold()
            ),
            format!("Virtual Memory Swap: {}", swap_str.dimmed()),
        ],
    );
    println!();

    // ── 2. Storage Reclaimability Overview Card ─────────────────────────────
    let total_reclaimable_str = format_bytes(report.total_reclaimable_bytes);
    ui::card(
        "RECLAIMABLE STORAGE SUMMARY",
        &[
            format!(
                "Total Space Reclaimable: {}",
                total_reclaimable_str.cyan().bold()
            ),
            format!(
                "Total Files Analyzed:    {} files",
                report.total_files_scanned.to_string().bold()
            ),
            format!(
                "Cleanable Targets Found: {} categories",
                report.category_summaries.len().to_string().bold()
            ),
            format!(
                "Scan Duration:           {}",
                ui::elapsed(start_scan).dimmed()
            ),
        ],
    );
    println!();

    // ── 3. Category Breakdown ───────────────────────────────────────────────
    ui::section("Storage & Cache Categorization");
    for cat in &report.category_summaries {
        let name_with_icon = format!("{} {}", cat.category.icon(), cat.category.display_name());
        let val_str = format!(
            "{} ({} files)",
            format_bytes(cat.total_bytes).bold().cyan(),
            cat.item_count
        );
        ui::kv_dot(&name_with_icon, &val_str);
    }
    println!();

    // ── 4. Deleted Application Leftovers Section ────────────────────────────
    if !report.app_leftovers.is_empty() {
        ui::section("Uninstalled / Ghost Application Leftovers");
        println!(
            "  {}",
            "Found leftover configuration & container data for deleted apps:".dimmed()
        );
        for item in report.app_leftovers.iter().take(8) {
            println!(
                "  {} {:<28} {:<12} {}",
                "👻".dimmed(),
                item.app_name.bold(),
                format_bytes(item.size_bytes).yellow(),
                item.path.display().to_string().dimmed()
            );
        }
        if report.app_leftovers.len() > 8 {
            println!(
                "  … and {} more ghost app folders",
                report.app_leftovers.len() - 8
            );
        }
        println!();
    }

    // ── 5. Malware, Trojan & Persistence Security Shield ────────────────────
    ui::section("Security, Malware & Persistence Shield");
    if report.threats.is_empty() {
        ui::ok("Zero threats, cryptominers, or suspicious persistence daemons detected.");
        println!(
            "  {} All LaunchAgents and startup scripts passed heuristic integrity verification.\n",
            "🛡️".green()
        );
    } else {
        let mut alert_lines = Vec::new();
        alert_lines.push(format!(
            "Detected {} suspicious persistence or security items:",
            report.threats.len()
        ));
        for t in &report.threats {
            let badge = match t.severity {
                ThreatSeverity::Critical => "[CRITICAL]".red().bold().to_string(),
                ThreatSeverity::High => "[HIGH]".yellow().bold().to_string(),
                ThreatSeverity::Medium => "[MEDIUM]".cyan().to_string(),
                ThreatSeverity::Info => "[INFO]".dimmed().to_string(),
            };
            alert_lines.push(format!(
                "{} {}: {}",
                badge,
                t.name.bold(),
                t.file_path.display()
            ));
            alert_lines.push(format!("   Remediation: {}", t.remediation.dimmed()));
        }
        ui::alert_box("POTENTIAL SECURITY THREATS DETECTED", &alert_lines);
    }

    // ── 6. Determine Targets for Cleanup Execution ───────────────────────────
    let wants_clean = opts.all || opts.cache || opts.leftovers || opts.threats;

    if !wants_clean && !opts.scan {
        println!("  ────────────────────────────────────────────────────────");
        println!(
            "  {} Run with action flags to reclaim disk space:",
            "💡".yellow().bold()
        );
        println!(
            "    {} {}  Clean all safe caches, app remnants, and temp files",
            "•".cyan(),
            "esprit clean --all".bold().green()
        );
        println!(
            "    {} {}  Clean developer build caches (Xcode, Cargo, npm, pip)",
            "•".cyan(),
            "esprit clean --cache".bold().green()
        );
        println!(
            "    {} {}  Remove only uninstalled application remnants",
            "•".cyan(),
            "esprit clean --leftovers".bold().green()
        );
        println!(
            "    {} {}  Preview deletion without touching disk",
            "•".cyan(),
            "esprit clean --dry-run".bold().cyan()
        );
        println!();
        return Ok(());
    }

    if opts.scan && !wants_clean {
        ui::ok("Scan diagnostic complete. No changes were made to disk.");
        println!();
        return Ok(());
    }

    // Filter targets based on user selection
    let mut selected_paths: Vec<PathBuf> = Vec::new();
    for item in &report.clean_items {
        let include = if opts.all {
            item.is_safe
        } else if opts.cache
            && matches!(
                item.category,
                CleanCategory::DeveloperCache
                    | CleanCategory::SystemCache
                    | CleanCategory::BrowserCache
            )
        {
            item.is_safe
        } else if opts.leftovers && matches!(item.category, CleanCategory::AppLeftovers) {
            true
        } else {
            false
        };

        if include {
            selected_paths.push(item.path.clone());
        }
    }

    if opts.threats {
        for t in &report.threats {
            selected_paths.push(t.file_path.clone());
        }
    }

    if selected_paths.is_empty() {
        ui::info("No items matched the requested cleaning categories.");
        println!();
        return Ok(());
    }

    let target_bytes: u64 = report
        .clean_items
        .iter()
        .filter(|i| selected_paths.contains(&i.path))
        .map(|i| i.size_bytes)
        .sum();

    // ── Dry Run Simulation Mode ─────────────────────────────────────────────
    if opts.dry_run {
        ui::panel_header("Dry-Run Simulation Mode", Some("Zero Disk Writes"));
        println!(
            "  Would remove {} target locations, freeing up ~{}:\n",
            selected_paths.len().to_string().bold(),
            format_bytes(target_bytes).cyan().bold()
        );
        for p in selected_paths.iter().take(10) {
            println!("  {} {}", "○".dimmed(), p.display());
        }
        if selected_paths.len() > 10 {
            println!("  … and {} more paths", selected_paths.len() - 10);
        }
        println!();
        ui::ok("Dry run simulation finished safely.");
        println!();
        return Ok(());
    }

    // ── Interactive Human-In-The-Loop Confirmation ──────────────────────────
    if !opts.yes {
        print!(
            "  {} Are you sure you want to clean {} targets and reclaim {}? [y/N]: ",
            "⚠️".yellow().bold(),
            selected_paths.len().to_string().bold(),
            format_bytes(target_bytes).cyan().bold()
        );
        io::stdout().flush()?;
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let trimmed = input.trim().to_lowercase();
        if trimmed != "y" && trimmed != "yes" {
            println!("  Cleanup aborted by user. No files were removed.\n");
            return Ok(());
        }
    }

    // ── Execute Cleanup ─────────────────────────────────────────────────────
    let clean_start = Instant::now();
    let clean_sp = ui::spinner("Purging cache targets and reclaiming disk space…");

    let result = cleaner.clean_items(&selected_paths, false);
    clean_sp.finish_and_clear();

    ui::panel_header("System Cleanup Complete", Some("Success"));
    ui::card(
        "RECLAIMED DISK RECOVERY",
        &[
            format!(
                "Total Disk Space Reclaimed: {}",
                format_bytes(result.total_bytes_freed).green().bold()
            ),
            format!(
                "Target Locations Purged:   {} items",
                result.items_cleaned.to_string().bold()
            ),
            format!(
                "Operation Completed in:    {}",
                ui::elapsed(clean_start).dimmed()
            ),
        ],
    );

    if !result.failed_items.is_empty() {
        println!();
        ui::warn(&format!(
            "{} items could not be deleted (system locked or permission restricted):",
            result.failed_items.len()
        ));
        for (p, err) in result.failed_items.iter().take(5) {
            println!("  {} {}: {}", "•".dimmed(), p.display(), err.dimmed());
        }
    }

    println!();
    ui::ok("Your system has been tuned and optimized!");
    println!();

    Ok(())
}
