#![forbid(unsafe_code)]

use anyhow::Result;
use chrono::{DateTime, Utc};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use sysinfo::System;
use walkdir::WalkDir;

// ── Data Models ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CleanCategory {
    SystemCache,
    DeveloperCache,
    BrowserCache,
    AppLeftovers,
    SystemTrash,
    TemporaryFiles,
    LogReports,
}

impl CleanCategory {
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::SystemCache => "System & OS Caches",
            Self::DeveloperCache => "Developer Build & Tool Caches",
            Self::BrowserCache => "Web Browser Caches",
            Self::AppLeftovers => "Deleted App Leftovers & Remnants",
            Self::SystemTrash => "Trash Bin Leftovers",
            Self::TemporaryFiles => "Temporary Files & Staging",
            Self::LogReports => "Diagnostic Logs & Crash Dumps",
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            Self::SystemCache => "⚙️",
            Self::DeveloperCache => "🛠️",
            Self::BrowserCache => "🌐",
            Self::AppLeftovers => "👻",
            Self::SystemTrash => "🗑️",
            Self::TemporaryFiles => "⏱️",
            Self::LogReports => "📜",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleanItem {
    pub category: CleanCategory,
    pub name: String,
    pub path: PathBuf,
    pub size_bytes: u64,
    pub file_count: usize,
    pub is_safe: bool,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppLeftoverItem {
    pub app_name: String,
    pub path: PathBuf,
    pub size_bytes: u64,
    pub file_count: usize,
    pub confidence: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ThreatSeverity {
    Info,
    Medium,
    High,
    Critical,
}

impl ThreatSeverity {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Info => "INFO",
            Self::Medium => "MEDIUM",
            Self::High => "HIGH",
            Self::Critical => "CRITICAL",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreatFinding {
    pub name: String,
    pub severity: ThreatSeverity,
    pub category: String,
    pub file_path: PathBuf,
    pub description: String,
    pub signature_matched: String,
    pub remediation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategorySummary {
    pub category: CleanCategory,
    pub total_bytes: u64,
    pub item_count: usize,
    pub target_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryStats {
    pub total_ram_bytes: u64,
    pub used_ram_bytes: u64,
    pub available_ram_bytes: u64,
    pub swap_total_bytes: u64,
    pub swap_used_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemScanReport {
    pub timestamp: DateTime<Utc>,
    pub total_reclaimable_bytes: u64,
    pub total_files_scanned: usize,
    pub memory_stats: MemoryStats,
    pub category_summaries: Vec<CategorySummary>,
    pub clean_items: Vec<CleanItem>,
    pub app_leftovers: Vec<AppLeftoverItem>,
    pub threats: Vec<ThreatFinding>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleanExecutionResult {
    pub total_bytes_freed: u64,
    pub items_cleaned: usize,
    pub failed_items: Vec<(PathBuf, String)>,
    pub dry_run: bool,
}

// ── System Cleaner Engine ───────────────────────────────────────────────────

pub struct SystemCleaner {
    home_dir: Option<PathBuf>,
}

impl Default for SystemCleaner {
    fn default() -> Self {
        Self::new()
    }
}

impl SystemCleaner {
    pub fn new() -> Self {
        let home = directories::BaseDirs::new().map(|d| d.home_dir().to_path_buf());
        Self { home_dir: home }
    }

    /// Run full multi-stage system diagnostic and scan
    pub fn scan_system(&self) -> Result<SystemScanReport> {
        let mut clean_items = Vec::new();
        let mut total_files_scanned = 0;

        // 1. Scan System, Developer, and Browser Caches
        let (cache_items, files_count) = self.scan_cache_targets();
        clean_items.extend(cache_items);
        total_files_scanned += files_count;

        // 2. Deep scan for Deleted App Remnants / Leftovers
        let (leftovers, leftover_files) = self.scan_app_leftovers();
        total_files_scanned += leftover_files;
        for leftover in &leftovers {
            clean_items.push(CleanItem {
                category: CleanCategory::AppLeftovers,
                name: format!("Ghost App: {}", leftover.app_name),
                path: leftover.path.clone(),
                size_bytes: leftover.size_bytes,
                file_count: leftover.file_count,
                is_safe: true,
                description: format!(
                    "Orphaned configuration & support files for uninstalled app `{}`",
                    leftover.app_name
                ),
            });
        }

        // 3. Scan for Malicious Persistence, Mining scripts & Trojan droppers
        let (threats, threat_files) = self.scan_malware_and_threats();
        total_files_scanned += threat_files;

        // 4. Calculate Memory Metrics
        let mut sys = System::new_all();
        sys.refresh_all();
        let memory_stats = MemoryStats {
            total_ram_bytes: sys.total_memory(),
            used_ram_bytes: sys.used_memory(),
            available_ram_bytes: sys.available_memory(),
            swap_total_bytes: sys.total_swap(),
            swap_used_bytes: sys.used_swap(),
        };

        // 5. Aggregate category summaries
        let mut cat_map: HashMap<CleanCategory, (u64, usize, usize)> = HashMap::new();
        for item in &clean_items {
            let entry = cat_map.entry(item.category).or_insert((0, 0, 0));
            entry.0 += item.size_bytes;
            entry.1 += item.file_count;
            entry.2 += 1;
        }

        let mut category_summaries = Vec::new();
        for (cat, (bytes, files, targets)) in cat_map {
            category_summaries.push(CategorySummary {
                category: cat,
                total_bytes: bytes,
                item_count: files,
                target_count: targets,
            });
        }
        category_summaries.sort_by_key(|c| std::cmp::Reverse(c.total_bytes));

        let total_reclaimable_bytes = clean_items.iter().map(|i| i.size_bytes).sum();

        Ok(SystemScanReport {
            timestamp: Utc::now(),
            total_reclaimable_bytes,
            total_files_scanned,
            memory_stats,
            category_summaries,
            clean_items,
            app_leftovers: leftovers,
            threats,
        })
    }

    /// Perform safe cleanup of targeted directories/files
    pub fn clean_items(&self, targets: &[PathBuf], dry_run: bool) -> CleanExecutionResult {
        let mut total_bytes_freed = 0;
        let mut items_cleaned = 0;
        let mut failed_items = Vec::new();

        for target in targets {
            // Safety guard: never touch critical system root directories
            if !self.is_path_safe_to_clean(target) {
                failed_items.push((
                    target.clone(),
                    "Safety Guard: Directory is protected or critical system root".to_string(),
                ));
                continue;
            }

            if !target.exists() {
                continue;
            }

            let (size, _) = self.calculate_path_size(target);

            if dry_run {
                total_bytes_freed += size;
                items_cleaned += 1;
            } else {
                let res = if target.is_dir() {
                    fs::remove_dir_all(target)
                } else {
                    fs::remove_file(target)
                };

                match res {
                    Ok(_) => {
                        total_bytes_freed += size;
                        items_cleaned += 1;
                    }
                    Err(e) => {
                        failed_items.push((target.clone(), e.to_string()));
                    }
                }
            }
        }

        CleanExecutionResult {
            total_bytes_freed,
            items_cleaned,
            failed_items,
            dry_run,
        }
    }

    // ── Internal Scanners ───────────────────────────────────────────────────

    fn scan_cache_targets(&self) -> (Vec<CleanItem>, usize) {
        let mut items = Vec::new();
        let mut total_files = 0;

        let home = match &self.home_dir {
            Some(h) => h.clone(),
            None => PathBuf::from("/"),
        };

        // Cache candidate targets with metadata
        let targets = vec![
            // Developer Caches
            (
                CleanCategory::DeveloperCache,
                "Xcode DerivedData",
                home.join("Library/Developer/Xcode/DerivedData"),
                "Intermediate build objects, indexing databases, and module caches",
            ),
            (
                CleanCategory::DeveloperCache,
                "Xcode Archives & Simulator Caches",
                home.join("Library/Developer/CoreSimulator/Caches"),
                "iOS Simulator runtime data & build cache",
            ),
            (
                CleanCategory::DeveloperCache,
                "Cargo & Rust Target Caches",
                home.join(".cargo/registry/cache"),
                "Cargo crate download cache and crate indexes",
            ),
            (
                CleanCategory::DeveloperCache,
                "Node / npm Cache",
                home.join(".npm/_cacache"),
                "Global npm package tarball and metadata cache",
            ),
            (
                CleanCategory::DeveloperCache,
                "pnpm Global Store Cache",
                home.join(".local/share/pnpm/store"),
                "Global pnpm content-addressable storage cache",
            ),
            (
                CleanCategory::DeveloperCache,
                "Yarn Berry Cache",
                home.join(".yarn/berry/cache"),
                "Yarn global package cache",
            ),
            (
                CleanCategory::DeveloperCache,
                "Python Pip Cache",
                home.join(".cache/pip"),
                "Cached wheel builds and downloaded Python packages",
            ),
            (
                CleanCategory::DeveloperCache,
                "Gradle Build Cache",
                home.join(".gradle/caches"),
                "Gradle downloaded dependencies and daemon build caches",
            ),
            (
                CleanCategory::DeveloperCache,
                "Maven Repository Cache",
                home.join(".m2/repository"),
                "Maven downloaded plugin and dependency cache",
            ),
            (
                CleanCategory::DeveloperCache,
                "Homebrew Bottle Cache",
                home.join("Library/Caches/Homebrew"),
                "Downloaded Homebrew bottles and formula tarballs",
            ),
            (
                CleanCategory::DeveloperCache,
                "Go Module Cache",
                home.join("go/pkg/mod/cache"),
                "Go build cache and downloaded module zip files",
            ),
            // Web Browser Caches
            (
                CleanCategory::BrowserCache,
                "Google Chrome Cache",
                home.join("Library/Caches/Google/Chrome"),
                "Cached web pages, scripts, images, and media",
            ),
            (
                CleanCategory::BrowserCache,
                "Brave Browser Cache",
                home.join("Library/Caches/BraveSoftware/Brave-Browser"),
                "Cached browsing session data and media files",
            ),
            (
                CleanCategory::BrowserCache,
                "Arc Browser Cache",
                home.join("Library/Caches/company.thebrowser.Browser"),
                "Arc web browser cached scripts and textures",
            ),
            (
                CleanCategory::BrowserCache,
                "Mozilla Firefox Cache",
                home.join("Library/Caches/Firefox"),
                "Firefox web disk cache and offline storage",
            ),
            (
                CleanCategory::BrowserCache,
                "Microsoft Edge Cache",
                home.join("Library/Caches/Microsoft Edge"),
                "Edge web resources and download cache",
            ),
            // System Caches & Trash
            (
                CleanCategory::SystemCache,
                "User Application Caches",
                home.join("Library/Caches"),
                "macOS User application caches and runtime temp files",
            ),
            (
                CleanCategory::SystemTrash,
                "macOS User Trash",
                home.join(".Trash"),
                "Deleted files waiting in Trash bin",
            ),
            (
                CleanCategory::LogReports,
                "Diagnostic & Crash Reports",
                home.join("Library/Logs/DiagnosticReports"),
                "Crash reports, spin dumps, and system diagnostic logs",
            ),
            (
                CleanCategory::LogReports,
                "User Application Logs",
                home.join("Library/Logs"),
                "Application execution trace logs",
            ),
            (
                CleanCategory::TemporaryFiles,
                "macOS User Temp Staging",
                PathBuf::from("/private/tmp"),
                "Temporary socket files and ephemeral staging files",
            ),
            (
                CleanCategory::TemporaryFiles,
                "System Temp Directory",
                PathBuf::from("/tmp"),
                "System temporary process buffers",
            ),
        ];

        for (cat, name, path, desc) in targets {
            if path.exists() {
                let (size, count) = self.calculate_path_size(&path);
                total_files += count;
                if size > 0 {
                    items.push(CleanItem {
                        category: cat,
                        name: name.to_string(),
                        path,
                        size_bytes: size,
                        file_count: count,
                        is_safe: cat != CleanCategory::TemporaryFiles,
                        description: desc.to_string(),
                    });
                }
            }
        }

        (items, total_files)
    }

    /// Deep inspection of orphaned app leftovers (apps deleted from /Applications)
    fn scan_app_leftovers(&self) -> (Vec<AppLeftoverItem>, usize) {
        let mut leftovers = Vec::new();
        let mut scanned_count = 0;

        let home = match &self.home_dir {
            Some(h) => h.clone(),
            None => return (leftovers, 0),
        };

        // 1. Gather all currently installed application names
        let mut installed_apps: HashSet<String> = HashSet::new();
        let app_dirs = vec![
            PathBuf::from("/Applications"),
            PathBuf::from("/System/Applications"),
            PathBuf::from("/System/Applications/Utilities"),
            home.join("Applications"),
        ];

        for app_dir in app_dirs {
            if let Ok(entries) = fs::read_dir(app_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_dir() && path.extension().and_then(|s| s.to_str()) == Some("app") {
                        if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                            let normalized = stem.to_lowercase().replace(' ', "");
                            installed_apps.insert(normalized);
                        }
                    }
                }
            }
        }

        // Whitelist common Apple & Unix developer directories
        let system_whitelist: HashSet<&str> = [
            "apple",
            "com.apple",
            "google",
            "microsoft",
            "adobe",
            "icloud",
            "mobilesync",
            "addressbook",
            "callhistorydb",
            "dock",
            "finder",
            "safari",
            "siri",
            "spotlight",
            "terminal",
            "systemprefs",
            "itunes",
            "music",
            "photos",
            "mail",
            "messages",
            "notes",
            "keychain",
            "security",
            "code",
            "esprit",
            "cargo",
            "rust",
            "git",
            "homebrew",
            "zsh",
            "bash",
            "local",
            "ssh",
        ]
        .into_iter()
        .collect();

        // 2. Scan ~/Library/Application Support for remnants
        let app_support = home.join("Library/Application Support");
        if let Ok(entries) = fs::read_dir(app_support) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    let folder_name = entry.file_name().to_string_lossy().to_string();
                    let normalized = folder_name.to_lowercase().replace(' ', "");

                    // Skip whitelisted system namespaces
                    let is_whitelisted = system_whitelist.iter().any(|w| normalized.contains(w));
                    if is_whitelisted {
                        continue;
                    }

                    // If not found in installed apps
                    let is_installed = installed_apps
                        .iter()
                        .any(|app| normalized.contains(app) || app.contains(&normalized));

                    if !is_installed {
                        let (size, count) = self.calculate_path_size(&path);
                        scanned_count += count;

                        if size > 1_000_000 {
                            // Only flag leftovers > 1 MB to avoid noise
                            leftovers.push(AppLeftoverItem {
                                app_name: folder_name,
                                path,
                                size_bytes: size,
                                file_count: count,
                                confidence: "HIGH (No matching .app in /Applications)".to_string(),
                            });
                        }
                    }
                }
            }
        }

        // 3. Scan ~/Library/Saved Application State
        let saved_state = home.join("Library/Saved Application State");
        if let Ok(entries) = fs::read_dir(saved_state) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    let folder_name = entry.file_name().to_string_lossy().to_string();
                    let normalized = folder_name.to_lowercase();

                    let is_whitelisted = system_whitelist.iter().any(|w| normalized.contains(w));
                    if !is_whitelisted {
                        let is_installed =
                            installed_apps.iter().any(|app| normalized.contains(app));
                        if !is_installed {
                            let (size, count) = self.calculate_path_size(&path);
                            scanned_count += count;
                            if size > 100_000 {
                                leftovers.push(AppLeftoverItem {
                                    app_name: folder_name.replace(".savedState", ""),
                                    path,
                                    size_bytes: size,
                                    file_count: count,
                                    confidence: "MEDIUM (Orphaned saved state)".to_string(),
                                });
                            }
                        }
                    }
                }
            }
        }

        (leftovers, scanned_count)
    }

    /// Threat, Malware, Malicious Persistence & Crypto-Miner Scanner
    fn scan_malware_and_threats(&self) -> (Vec<ThreatFinding>, usize) {
        let mut findings = Vec::new();
        let mut files_scanned = 0;

        let home = match &self.home_dir {
            Some(h) => h.clone(),
            None => PathBuf::from("/"),
        };

        // Threat rules & regex signatures
        let threat_signatures: Vec<(&str, ThreatSeverity, &str, &str, &str)> = vec![
            (
                "Reverse Shell / Remote Command Execution",
                ThreatSeverity::Critical,
                r#"(?i)(nc\s+-e\s+/bin/(bash|sh)|/dev/tcp/[0-9]+\.[0-9]+\.[0-9]+\.[0-9]+|bash\s+-i\s+>&|python\s+-c\s+['"]import\s+socket,subprocess)"#,
                "Detected active reverse shell payload in startup script or config.",
                "Quarantine file immediately and inspect network connections.",
            ),
            (
                "Hidden Cryptocurrency Miner",
                ThreatSeverity::Critical,
                r#"(?i)(stratum\+tcp://|xmrig|minerd|cryptonight|mine\.moneropool|xmr-stak|pool\.hashvault\.pro)"#,
                "Cryptocurrency mining pool connection or mining binary reference found.",
                "Terminate process, remove associated LaunchAgent/Daemon, and inspect crontab.",
            ),
            (
                "Obfuscated Curl/Wget Execution Dropper",
                ThreatSeverity::High,
                r#"(?i)(curl\s+-[fsSLk]+\s+https?://[^\s|]+\s*\|\s*(sh|bash|zsh)|wget\s+-[qO-]+\s+https?://[^\s|]+\s*\|\s*(sh|bash|zsh)|base64\s+-d\s*\|\s*(sh|bash))"#,
                "Suspicious unverified pipe-to-shell download dropper.",
                "Inspect URL host and remove unverified startup execution hook.",
            ),
            (
                "Suspicious /tmp Binary Execution in LaunchAgent",
                ThreatSeverity::High,
                r#"(?i)(/tmp/|/var/tmp/|/private/tmp/)[a-zA-Z0-9_\-\.]+\.(sh|py|bin|run|exe)"#,
                "LaunchAgent executes ephemeral binary in /tmp directory.",
                "Review plist configuration and delete anomalous persistence daemon.",
            ),
            (
                "Browser Search Hijacker / Policy Injector",
                ThreatSeverity::Medium,
                r#"(?i)(ExtensionInstallForcelist|ExtensionSettings|DefaultSearchProviderSearchURL)"#,
                "Forced browser policy configuration or search provider override.",
                "Review enterprise browser management policies.",
            ),
        ];

        let mut compiled_rules = Vec::new();
        for (name, sev, pat, desc, rem) in threat_signatures {
            if let Ok(re) = Regex::new(pat) {
                compiled_rules.push((name, sev, re, desc, rem));
            }
        }

        // Directories to inspect for malicious persistence
        let inspect_dirs = vec![
            home.join("Library/LaunchAgents"),
            PathBuf::from("/Library/LaunchAgents"),
            PathBuf::from("/Library/LaunchDaemons"),
        ];

        for dir in inspect_dirs {
            if !dir.exists() {
                continue;
            }

            for entry in WalkDir::new(dir).max_depth(2).into_iter().flatten() {
                if entry.file_type().is_file() {
                    files_scanned += 1;
                    let path = entry.path();
                    if let Ok(content) = fs::read_to_string(path) {
                        for (name, sev, re, desc, rem) in &compiled_rules {
                            if let Some(mat) = re.find(&content) {
                                findings.push(ThreatFinding {
                                    name: name.to_string(),
                                    severity: *sev,
                                    category: "Persistence / LaunchAgent".to_string(),
                                    file_path: path.to_path_buf(),
                                    description: desc.to_string(),
                                    signature_matched: mat.as_str().to_string(),
                                    remediation: rem.to_string(),
                                });
                            }
                        }
                    }
                }
            }
        }

        // Check Shell RC Files
        let rc_files = vec![
            home.join(".zshrc"),
            home.join(".bashrc"),
            home.join(".bash_profile"),
            home.join(".profile"),
        ];

        for rc in rc_files {
            if rc.exists() {
                files_scanned += 1;
                if let Ok(content) = fs::read_to_string(&rc) {
                    for (name, sev, re, desc, rem) in &compiled_rules {
                        if let Some(mat) = re.find(&content) {
                            findings.push(ThreatFinding {
                                name: name.to_string(),
                                severity: *sev,
                                category: "Shell Startup Profile".to_string(),
                                file_path: rc.clone(),
                                description: desc.to_string(),
                                signature_matched: mat.as_str().to_string(),
                                remediation: rem.to_string(),
                            });
                        }
                    }
                }
            }
        }

        // Check for standalone rogue executable Mach-O/ELF binaries in /tmp
        let tmp_dirs = vec![PathBuf::from("/tmp"), PathBuf::from("/var/tmp")];
        for tmp in tmp_dirs {
            if let Ok(entries) = fs::read_dir(tmp) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_file() {
                        files_scanned += 1;
                        if let Ok(meta) = entry.metadata() {
                            // Check executable bit
                            #[cfg(unix)]
                            {
                                use std::os::unix::fs::PermissionsExt;
                                let is_exec = meta.permissions().mode() & 0o111 != 0;
                                let name = entry.file_name().to_string_lossy().to_string();
                                if is_exec
                                    && !name.starts_with("al_")
                                    && !name.starts_with("com.apple")
                                {
                                    findings.push(ThreatFinding {
                                        name: "Unregistered Executable in Temporary Staging".to_string(),
                                        severity: ThreatSeverity::Medium,
                                        category: "Temporary Binary Execution".to_string(),
                                        file_path: path.clone(),
                                        description: format!("Binary `{}` has executable permissions in world-writable temp folder.", name),
                                        signature_matched: format!("chmod +x on {}", path.display()),
                                        remediation: "Verify application origin or remove with `esprit clean --threats`.".to_string(),
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }

        (findings, files_scanned)
    }

    // ── Helper Utilities ────────────────────────────────────────────────────

    fn calculate_path_size(&self, path: &Path) -> (u64, usize) {
        if !path.exists() {
            return (0, 0);
        }

        if path.is_file() {
            let len = fs::metadata(path).map(|m| m.len()).unwrap_or(0);
            return (len, 1);
        }

        let mut total_bytes = 0;
        let mut total_files = 0;

        for entry in WalkDir::new(path).into_iter().flatten() {
            if entry.file_type().is_file() {
                total_files += 1;
                if let Ok(meta) = entry.metadata() {
                    total_bytes += meta.len();
                }
            }
        }

        (total_bytes, total_files)
    }

    fn is_path_safe_to_clean(&self, path: &Path) -> bool {
        let p_str = path.to_string_lossy();

        // Strict blacklist of essential system roots and user home roots
        if p_str == "/"
            || p_str == "/System"
            || p_str == "/usr"
            || p_str == "/bin"
            || p_str == "/sbin"
            || p_str == "/etc"
            || p_str == "/var"
            || p_str == "/Library"
        {
            return false;
        }

        if let Some(home) = &self.home_dir {
            if path == home
                || path == &home.join("Documents")
                || path == &home.join("Desktop")
                || path == &home.join("Downloads")
                || path == &home.join("Pictures")
            {
                return false;
            }
        }

        true
    }
}

pub fn format_bytes(bytes: u64) -> String {
    const KB: f64 = 1024.0;
    const MB: f64 = KB * 1024.0;
    const GB: f64 = MB * 1024.0;
    const TB: f64 = GB * 1024.0;

    let b = bytes as f64;
    if b >= TB {
        format!("{:.2} TB", b / TB)
    } else if b >= GB {
        format!("{:.2} GB", b / GB)
    } else if b >= MB {
        format!("{:.1} MB", b / MB)
    } else if b >= KB {
        format!("{:.0} KB", b / KB)
    } else {
        format!("{} B", bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_bytes() {
        assert_eq!(format_bytes(500), "500 B");
        assert_eq!(format_bytes(2048), "2 KB");
        assert_eq!(format_bytes(1048576 * 150), "150.0 MB");
        assert_eq!(format_bytes(1073741824 * 3), "3.00 GB");
    }

    #[test]
    fn test_cleaner_scan() {
        let cleaner = SystemCleaner::new();
        let report = cleaner.scan_system();
        assert!(report.is_ok());
        let rep = report.unwrap();
        assert!(rep.memory_stats.total_ram_bytes > 0);
    }
}
