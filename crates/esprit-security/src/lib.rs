#![forbid(unsafe_code)]

use anyhow::Result;
use chrono::{DateTime, Utc};
use regex::Regex;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

/// Compute standard SHA-256 checksum
pub fn checksum(bytes: impl AsRef<[u8]>) -> String {
    let mut h = Sha256::new();
    h.update(bytes.as_ref());
    hex::encode(h.finalize())
}

/// Calculate Shannon entropy for a given string.
/// High entropy (> 4.5) on strings longer than 20 chars typically indicates a raw secret/token.
pub fn calculate_shannon_entropy(s: &str) -> f64 {
    if s.is_empty() {
        return 0.0;
    }
    let mut freq: HashMap<char, usize> = HashMap::new();
    let len = s.chars().count() as f64;

    for c in s.chars() {
        *freq.entry(c).or_insert(0) += 1;
    }

    let mut entropy = 0.0;
    for &count in freq.values() {
        let p = (count as f64) / len;
        entropy -= p * p.log2();
    }
    entropy
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecretFinding {
    pub file: String,
    pub line_number: usize,
    pub rule_name: String,
    pub severity: String,
    pub preview: String,
    pub entropy: f64,
}

#[derive(Debug, Clone)]
pub struct SecretScanner {
    patterns: Vec<(&'static str, &'static str, Regex)>,
}

impl Default for SecretScanner {
    fn default() -> Self {
        Self::new()
    }
}

impl SecretScanner {
    pub fn new() -> Self {
        let rules: Vec<(&'static str, &'static str, &'static str)> = vec![
            (
                "AWS Access Key ID",
                "CRITICAL",
                r"(?i)\b(AKIA[0-9A-Z]{16})\b",
            ),
            (
                "AWS Secret Access Key",
                "CRITICAL",
                r#"(?i)aws_secret_access_key\s*=\s*['"][A-Za-z0-9/+=]{40}['"]"#,
            ),
            (
                "GitHub Personal Access Token",
                "HIGH",
                r"\b(gh[pousr]_[A-Za-z0-9_]{20,255})\b",
            ),
            (
                "Slack Bot/User Token",
                "HIGH",
                r"\bxox[baprs]-[0-9]{10,13}-[0-9]{10,13}[a-zA-Z0-9-]*\b",
            ),
            (
                "OpenAI / Anthropic API Key",
                "CRITICAL",
                r"\b(sk-[a-zA-Z0-9]{20,64}|sk-ant-[a-zA-Z0-9-_]{20,80})\b",
            ),
            (
                "Google Cloud / Gemini API Key",
                "CRITICAL",
                r"\b(AIza[0-9A-Za-z-_]{35})\b",
            ),
            (
                "Stripe Secret API Key",
                "HIGH",
                r"\b(sk_live_[0-9a-zA-Z]{24})\b",
            ),
            (
                "Private RSA/SSH Key",
                "CRITICAL",
                r"-----BEGIN (RSA|EC|OPENSSH|DSA|PRIVATE) KEY-----",
            ),
            (
                "Database Connection String with Password",
                "HIGH",
                r#"(postgres|mysql|mongodb|redis)://[^:]+:([^@]+)@"#,
            ),
            (
                "Generic High Entropy API Key Assignment",
                "MEDIUM",
                r#"(?i)(api[_-]?key|secret|password|bearer|auth[_-]?token)\s*[:=]\s*['"]([A-Za-z0-9_/\-+=]{24,})['"]"#,
            ),
        ];

        let mut compiled = Vec::new();
        for (name, sev, pat) in rules {
            if let Ok(re) = Regex::new(pat) {
                compiled.push((name, sev, re));
            }
        }

        Self { patterns: compiled }
    }

    /// Scan a single string for secrets
    pub fn scan_text(&self, text: &str, file_label: &str) -> Vec<SecretFinding> {
        let mut findings = Vec::new();
        for (idx, line) in text.lines().enumerate() {
            let line_num = idx + 1;
            let trimmed = line.trim();

            // Skip comments that look like example code or tests
            if trimmed.starts_with("//")
                && (trimmed.contains("example")
                    || trimmed.contains("dummy")
                    || trimmed.contains("test"))
            {
                continue;
            }

            for (rule_name, severity, re) in &self.patterns {
                if let Some(mat) = re.find(line) {
                    let secret_match = mat.as_str();
                    let entropy = calculate_shannon_entropy(secret_match);

                    // Redact middle of secret for preview safety
                    let preview = if secret_match.len() > 8 {
                        format!(
                            "{}...{}",
                            &secret_match[..4],
                            &secret_match[secret_match.len() - 3..]
                        )
                    } else {
                        "***".to_string()
                    };

                    findings.push(SecretFinding {
                        file: file_label.to_string(),
                        line_number: line_num,
                        rule_name: rule_name.to_string(),
                        severity: severity.to_string(),
                        preview,
                        entropy,
                    });
                }
            }
        }
        findings
    }

    /// Recursively scan a directory, ignoring hidden git/target directories
    pub fn scan_directory(&self, dir: impl AsRef<Path>) -> Result<Vec<SecretFinding>> {
        let mut findings = Vec::new();
        let walker = WalkDir::new(dir)
            .follow_links(false)
            .into_iter()
            .filter_entry(|e| {
                let name = e.file_name().to_string_lossy();
                !name.starts_with(".git") && name != "target" && name != "node_modules"
            });

        for entry in walker {
            let entry = match entry {
                Ok(e) => e,
                Err(_) => continue,
            };

            if entry.file_type().is_file() {
                let path = entry.path();
                // Avoid scanning large binaries (>2MB)
                if let Ok(meta) = entry.metadata() {
                    if meta.len() > 2_000_000 {
                        continue;
                    }
                }

                if let Ok(content) = fs::read_to_string(path) {
                    let mut file_findings = self.scan_text(&content, &path.display().to_string());
                    findings.append(&mut file_findings);
                }
            }
        }

        Ok(findings)
    }
}

/// Tamper-evident, cryptographically chained Merkle Audit Entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    pub index: u64,
    pub timestamp: DateTime<Utc>,
    pub actor: String,
    pub action: String,
    pub payload_hash: String,
    pub previous_hash: String,
    pub block_hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MerkleAuditLog {
    pub entries: Vec<AuditEntry>,
}

impl MerkleAuditLog {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    pub fn append(&mut self, actor: &str, action: &str, payload: &[u8]) -> AuditEntry {
        let index = self.entries.len() as u64;
        let timestamp = Utc::now();
        let payload_hash = checksum(payload);
        let previous_hash = if let Some(last) = self.entries.last() {
            last.block_hash.clone()
        } else {
            "0000000000000000000000000000000000000000000000000000000000000000".to_string()
        };

        let block_content = format!(
            "{}:{}:{}:{}:{}:{}",
            index,
            timestamp.to_rfc3339(),
            actor,
            action,
            payload_hash,
            previous_hash
        );
        let block_hash = checksum(block_content.as_bytes());

        let entry = AuditEntry {
            index,
            timestamp,
            actor: actor.to_string(),
            action: action.to_string(),
            payload_hash,
            previous_hash,
            block_hash,
        };

        self.entries.push(entry.clone());
        entry
    }

    /// Verify full cryptographic chain integrity
    pub fn verify_integrity(&self) -> bool {
        for (i, entry) in self.entries.iter().enumerate() {
            let expected_prev = if i == 0 {
                "0000000000000000000000000000000000000000000000000000000000000000"
            } else {
                &self.entries[i - 1].block_hash
            };

            if entry.previous_hash != expected_prev {
                return false;
            }

            let block_content = format!(
                "{}:{}:{}:{}:{}:{}",
                entry.index,
                entry.timestamp.to_rfc3339(),
                entry.actor,
                entry.action,
                entry.payload_hash,
                entry.previous_hash
            );
            let expected_hash = checksum(block_content.as_bytes());
            if entry.block_hash != expected_hash {
                return false;
            }
        }
        true
    }
}

/// Air-Gap enforcement validator
pub struct AirGapGuard;

impl AirGapGuard {
    /// Return whether offline mode is currently enforced or demanded
    pub fn is_air_gapped_environment() -> bool {
        std::env::var("ESPRIT_OFFLINE")
            .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
            .unwrap_or(false)
    }

    /// Validate an operation against air-gap policies
    pub fn check_network_allowed() -> Result<()> {
        if Self::is_air_gapped_environment() {
            anyhow::bail!("Security Violation: Network access is strictly forbidden in Air-Gapped Mode (--offline).");
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shannon_entropy() {
        let low_entropy = "aaaaaaaabbbbbbbb";
        let high_entropy = "4kP$9z!Lq#2mX8@v1R";
        assert!(calculate_shannon_entropy(high_entropy) > calculate_shannon_entropy(low_entropy));
    }

    #[test]
    fn test_secret_scanner() {
        let scanner = SecretScanner::new();
        let sample = "let github_key = \"ghp_1234567890abcdefghijklmnopqrstuvwx\";";
        let findings = scanner.scan_text(sample, "test.rs");
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].rule_name, "GitHub Personal Access Token");
    }

    #[test]
    fn test_merkle_audit_log() {
        let mut log = MerkleAuditLog::new();
        log.append("developer", "run_migration", b"schema_v2");
        log.append("agent", "refactor_auth", b"auth_module");
        assert!(log.verify_integrity());
        assert_eq!(log.entries.len(), 2);
    }
}
