#![forbid(unsafe_code)]

use anyhow::Result;
use esprit_ai::Ai;
use std::fs;
use std::path::Path;

pub struct OnboardingGenerator;

impl OnboardingGenerator {
    pub fn generate_onboarding_guide(project_root: impl AsRef<Path>) -> Result<String> {
        let ai = Ai::default_model()?;

        let mut sample_files = Vec::new();
        for entry in walkdir::WalkDir::new(&project_root)
            .max_depth(2)
            .into_iter()
            .filter_entry(|e| {
                let name = e.file_name().to_string_lossy();
                !name.starts_with('.') && name != "target" && name != "node_modules"
            })
        {
            if let Ok(e) = entry {
                sample_files.push(e.path().display().to_string());
            }
        }

        let structure_preview = sample_files
            .iter()
            .take(25)
            .cloned()
            .collect::<Vec<_>>()
            .join("\n");

        let prompt = format!(
            "You are a Principal Engineering Lead. Generate a comprehensive `ONBOARDING.md` developer guide for new team members joining this repository.\n\nKey Files & Directory Structure:\n{}\n\nInclude: 1. High-Level Architecture Overview 2. Prerequisites & Environment Setup 3. Core Development Workflows (Testing, Linting, Building) 4. Key Subsystems & Crates Breakdown 5. Contribution & PR Guidelines.",
            structure_preview
        );

        let (guide, _) = ai.ask_with_meta(&prompt)?;
        let output_path = project_root.as_ref().join("ONBOARDING.md");
        fs::write(&output_path, &guide)?;

        Ok(output_path.display().to_string())
    }
}

pub struct StyleEnforcer;

impl StyleEnforcer {
    pub fn enforce_conventions(target_dir: impl AsRef<Path>) -> Result<Vec<String>> {
        let mut violations = Vec::new();

        for entry in walkdir::WalkDir::new(target_dir)
            .into_iter()
            .filter_entry(|e| {
                let name = e.file_name().to_string_lossy();
                !name.starts_with('.') && name != "target" && name != "node_modules"
            })
        {
            if let Ok(e) = entry {
                if e.file_type().is_file() {
                    let path = e.path();
                    let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("");
                    if ext == "rs" {
                        if let Ok(content) = fs::read_to_string(path) {
                            for (idx, line) in content.lines().enumerate() {
                                let trimmed = line.trim();
                                // Flag raw println! in core library crates
                                if trimmed.starts_with("println!(")
                                    && !path.to_string_lossy().contains("apps/esprit-cli")
                                    && !path.to_string_lossy().contains("examples")
                                    && !path.to_string_lossy().contains("tests")
                                {
                                    violations.push(format!("{}:{}: Use `tracing::info!` or `tracing::debug!` instead of raw `println!` in library crates.", path.display(), idx + 1));
                                }
                                // Flag unwrap() without expect() in production libraries
                                if trimmed.contains(".unwrap()")
                                    && !path.to_string_lossy().contains("tests")
                                    && !path.to_string_lossy().contains("test")
                                {
                                    violations.push(format!("{}:{}: Unchecked `.unwrap()` call in production path. Prefer `?` or `.expect(\"...\")` with clear context.", path.display(), idx + 1));
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(violations)
    }
}

pub struct SecurityReportGenerator;

impl SecurityReportGenerator {
    pub fn generate_bug_bounty_report(
        findings: &[esprit_security::SecretFinding],
        output_path: impl AsRef<Path>,
    ) -> Result<()> {
        let mut md = String::new();
        md.push_str("# 🛡️ Esprit Security Audit & Bug Bounty Remediation Report\n\n");
        md.push_str(&format!(
            "**Generated:** {}\n",
            chrono::Utc::now().to_rfc3339()
        ));
        md.push_str("**Status:** Complete\n\n");

        md.push_str("## Executive Summary\n\n");
        md.push_str(&format!("Total Findings: **{}**\n\n", findings.len()));

        md.push_str("## Detailed Findings\n\n");
        if findings.is_empty() {
            md.push_str("✅ Zero security vulnerabilities or credential leaks detected across audited paths.\n");
        } else {
            for (idx, f) in findings.iter().enumerate() {
                md.push_str(&format!("### Finding #{}: {}\n\n", idx + 1, f.rule_name));
                md.push_str(&format!("- **File:** `{}:{}`\n", f.file, f.line_number));
                md.push_str(&format!("- **Severity:** `{}`\n", f.severity));
                md.push_str(&format!("- **Entropy:** `{:.2}`\n", f.entropy));
                md.push_str(&format!("- **Masked Preview:** `{}`\n\n", f.preview));
                md.push_str("#### Remediation Plan\nRotate the credential immediately, purge git history with `git filter-repo`, and enforce `.gitignore` rules.\n\n");
            }
        }

        fs::write(output_path, md)?;
        Ok(())
    }
}
