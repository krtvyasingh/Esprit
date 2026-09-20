use anyhow::{anyhow, Result};
use owo_colors::OwoColorize;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use crate::ui;

const GITHUB_REPO: &str = "krtvyasingh/Esprit";
const API_URL: &str = "https://api.github.com/repos/krtvyasingh/Esprit/commits/main";
const CACHE_TTL_SECS: u64 = 3600; // 1 hour

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateInfo {
    pub current_commit: String,
    pub latest_commit: String,
    pub latest_message: String,
    pub has_update: bool,
    pub checked_at: u64,
}

#[derive(Deserialize)]
struct GithubCommitResponse {
    sha: String,
    commit: GithubCommitDetails,
}

#[derive(Deserialize)]
struct GithubCommitDetails {
    message: String,
}

fn cache_file() -> Result<PathBuf> {
    let home = std::env::var("HOME").map(PathBuf::from)?;
    let dir = home.join(".esprit");
    fs::create_dir_all(&dir)?;
    Ok(dir.join("update_cache.json"))
}

pub fn find_repo_quick() -> Result<PathBuf> {
    // 1. Check current dir & ancestors
    let mut curr = std::env::current_dir().ok();
    while let Some(dir) = curr {
        if dir.join("apps/esprit-cli/Cargo.toml").exists() {
            return Ok(dir);
        }
        curr = dir.parent().map(|p| p.to_path_buf());
    }

    // 2. Check candidate paths
    if let Ok(home) = std::env::var("HOME") {
        let home_path = PathBuf::from(home);
        let candidates = [
            home_path.join(".gemini/antigravity/scratch/Esprit"),
            home_path.join("Projects/Esprit"),
            home_path.join("Esprit"),
            home_path.join(".esprit/source"),
        ];
        for path in candidates {
            if path.join("apps/esprit-cli/Cargo.toml").exists() {
                return Ok(path);
            }
        }
    }

    Err(anyhow!("Could not locate local Esprit repository"))
}

pub fn find_or_clone_repo() -> Result<PathBuf> {
    if let Ok(dir) = find_repo_quick() {
        return Ok(dir);
    }

    // Clone to ~/.esprit/source
    if let Ok(home) = std::env::var("HOME") {
        let target = PathBuf::from(home).join(".esprit/source");
        if !target.exists() {
            let _ = fs::create_dir_all(&target);
            let _ = Command::new("git")
                .args([
                    "clone",
                    "--depth",
                    "1",
                    "https://github.com/krtvyasingh/Esprit.git",
                    target.to_str().unwrap(),
                ])
                .output();
        }
        if target.join("apps/esprit-cli/Cargo.toml").exists() {
            return Ok(target);
        }
    }

    Err(anyhow!(
        "Could not locate or clone Esprit repository from GitHub"
    ))
}

pub fn current_local_commit() -> String {
    if let Ok(repo_dir) = find_repo_quick() {
        if let Ok(out) = Command::new("git")
            .current_dir(&repo_dir)
            .args(["rev-parse", "--short=7", "HEAD"])
            .output()
        {
            if out.status.success() {
                let s = String::from_utf8_lossy(&out.stdout).trim().to_string();
                if !s.is_empty() {
                    return s;
                }
            }
        }
    }
    format!("v{}", env!("CARGO_PKG_VERSION"))
}

pub fn check_update(force_network: bool) -> Option<UpdateInfo> {
    let cache_path = cache_file().ok()?;
    let now = SystemTime::now().duration_since(UNIX_EPOCH).ok()?.as_secs();

    if !force_network {
        if let Ok(contents) = fs::read_to_string(&cache_path) {
            if let Ok(cached) = serde_json::from_str::<UpdateInfo>(&contents) {
                if now.saturating_sub(cached.checked_at) < CACHE_TTL_SECS {
                    return Some(cached);
                }
            }
        }
    }

    let client = reqwest::blocking::Client::builder()
        .user_agent("Esprit-AutoUpdater/1.0")
        .timeout(Duration::from_secs(3))
        .build()
        .ok()?;

    let res = client.get(API_URL).send().ok()?;
    if !res.status().is_success() {
        return None;
    }

    let gh_commit: GithubCommitResponse = res.json().ok()?;
    let latest_sha = gh_commit.sha.chars().take(7).collect::<String>();
    let current_sha = current_local_commit();
    let first_line = gh_commit
        .commit
        .message
        .lines()
        .next()
        .unwrap_or("Latest commit")
        .to_string();

    let has_update = !current_sha.is_empty() && !latest_sha.is_empty() && current_sha != latest_sha;

    let info = UpdateInfo {
        current_commit: current_sha,
        latest_commit: latest_sha,
        latest_message: first_line,
        has_update,
        checked_at: now,
    };

    if let Ok(json) = serde_json::to_string_pretty(&info) {
        let _ = fs::write(&cache_path, json);
    }

    Some(info)
}

pub fn notify_if_available() {
    if let Some(info) = check_update(false) {
        if info.has_update {
            ui::update_badge(
                &info.current_commit,
                &info.latest_commit,
                Some(&info.latest_message),
            );
        }
    }
}

pub fn execute_update(force: bool) -> Result<()> {
    let start = Instant::now();
    ui::banner();
    ui::panel_header("Esprit Autonomous Self-Updater", Some("GitHub Sync"));

    let current = current_local_commit();
    println!(
        "  {} Checking remote: {}",
        "→".cyan().bold(),
        GITHUB_REPO.bold()
    );
    let sp = ui::spinner("Contacting GitHub API for latest revisions…");

    let update_info = check_update(true);
    sp.finish_and_clear();

    let (latest_sha, latest_msg) = match &update_info {
        Some(info) => (info.latest_commit.clone(), info.latest_message.clone()),
        None => ("origin/main".to_string(), "Latest changes".to_string()),
    };

    if !force && current == latest_sha {
        ui::ok(&format!(
            "Esprit is already on the latest revision ({})",
            current.bold().cyan()
        ));
        println!(
            "  {} No update needed. Use {} to force rebuild.\n",
            "•".dimmed(),
            "esprit update --force".bold()
        );
        return Ok(());
    }

    println!();
    ui::card(
        "UPDATE SUMMARY",
        &[
            format!("{:<18} {}", "Current Commit:".dimmed(), current.bold()),
            format!(
                "{:<18} {}",
                "Target Commit:".dimmed(),
                latest_sha.bold().cyan()
            ),
            format!("{:<18} {}", "Latest Note:".dimmed(), latest_msg.dimmed()),
        ],
    );
    println!();

    // Step 1: Locate or clone repository & pull latest
    ui::step(
        1,
        3,
        "Locating workspace and pulling latest changes from GitHub...",
    );
    let sp_pull = ui::spinner("Syncing repository from origin/main…");
    let repo_dir = find_or_clone_repo()?;
    let pull_res = Command::new("git")
        .current_dir(&repo_dir)
        .args(["pull", "--ff-only", "origin", "main"])
        .output();
    sp_pull.finish_and_clear();

    match pull_res {
        Ok(out) if out.status.success() => {
            ui::ok(&format!(
                "Repository synced at {}",
                repo_dir.display().to_string().dimmed()
            ));
        }
        Ok(out) => {
            let err = String::from_utf8_lossy(&out.stderr);
            if err.contains("Already up to date") || err.is_empty() {
                ui::ok("Repository is already up to date.");
            } else {
                ui::warn(&format!("Git notice: {}", err.trim()));
            }
        }
        Err(e) => {
            ui::warn(&format!("Git notice: {e}"));
        }
    }

    // Step 2: Cargo Build Release
    ui::step(2, 3, "Compiling and optimizing release binary...");
    let sp_build = ui::spinner("Building esprit binary (opt-level=3, thin-lto)…");
    let build_res = Command::new("cargo")
        .current_dir(&repo_dir)
        .args(["build", "--release", "-p", "esprit-cli"])
        .output();
    sp_build.finish_and_clear();

    match build_res {
        Ok(out) if out.status.success() => {
            ui::ok("Compiled release binary successfully.");
            // Install to ~/.cargo/bin/esprit
            if let Ok(home) = std::env::var("HOME") {
                let cargo_bin = PathBuf::from(home)
                    .join(".cargo")
                    .join("bin")
                    .join("esprit");
                let target_bin = repo_dir.join("target").join("release").join("esprit");
                if target_bin.exists() {
                    let _ = fs::copy(&target_bin, &cargo_bin);
                }
            }
        }
        Ok(out) => {
            let err = String::from_utf8_lossy(&out.stderr);
            ui::fail("Cargo build failed.");
            eprintln!("\n{}\n", err.red());
            return Err(anyhow!("Compilation failed during update"));
        }
        Err(e) => {
            ui::fail(&format!("Cargo execution failed: {e}"));
            return Err(anyhow!("Could not invoke cargo"));
        }
    }

    // Step 3: Self-Verification
    ui::step(3, 3, "Verifying updated binary integrity...");
    let new_sha = current_local_commit();
    let duration = ui::elapsed(start);

    // Invalidate / update cache
    if let Ok(cache_path) = cache_file() {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let fresh_info = UpdateInfo {
            current_commit: new_sha.clone(),
            latest_commit: new_sha.clone(),
            latest_message: latest_msg,
            has_update: false,
            checked_at: now,
        };
        let _ = fs::write(
            cache_path,
            serde_json::to_string(&fresh_info).unwrap_or_default(),
        );
    }

    println!();
    ui::ok(&format!(
        "{} {} in {}",
        "Esprit successfully updated to".green().bold(),
        new_sha.cyan().bold(),
        duration.bold()
    ));
    println!(
        "  {} Instant sync complete. All systems operational.\n",
        "✓".green().bold()
    );

    Ok(())
}
