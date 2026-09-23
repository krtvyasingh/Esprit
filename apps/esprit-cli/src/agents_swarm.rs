use anyhow::Result;
use esprit_ai::Ai;

pub struct TddAgent;

impl TddAgent {
    pub fn run_tdd_loop(goal: &str) -> Result<String> {
        let ai = Ai::default_model()?;

        // Step 1: Generate failing unit test
        let test_prompt = format!(
            "You are an expert TDD engineer. Write a concise Rust unit test that captures this requirement:\n\nRequirement: {}\n\nReturn ONLY the `#[test]` function block in valid Rust.",
            goal
        );
        let (test_code, _) = ai.ask_with_meta(&test_prompt)?;

        // Step 2: Generate implementation code
        let impl_prompt = format!(
            "You are an expert software developer. Given the following test requirement:\n{}\n\nAnd test code:\n{}\n\nGenerate the minimal, robust implementation code to make this test pass.",
            goal, test_code
        );
        let (impl_code, _) = ai.ask_with_meta(&impl_prompt)?;

        Ok(format!(
            "### 🧪 TDD Specification\n\n```rust\n{}\n```\n\n### ⚡ Implementation Code\n\n```rust\n{}\n```",
            test_code.trim(),
            impl_code.trim()
        ))
    }
}

pub struct RedTeamAgent;

impl RedTeamAgent {
    pub fn audit_codebase(path_target: &str) -> Result<String> {
        let ai = Ai::default_model()?;
        let sec_scanner = esprit_security::SecretScanner::new();
        let findings = sec_scanner.scan_directory(path_target).unwrap_or_default();

        let mut findings_summary = String::new();
        if findings.is_empty() {
            findings_summary
                .push_str("0 high-entropy secrets or hardcoded credentials detected on disk.\n");
        } else {
            for f in findings.iter().take(5) {
                findings_summary.push_str(&format!(
                    "- [{}] {} in `{}` (Line {})\n",
                    f.severity, f.rule_name, f.file, f.line_number
                ));
            }
        }

        let prompt = format!(
            "You are a ruthless Application Security (AppSec) Red-Team Auditor.\nAnalyze the target repository `{}`.\n\nSecret Scanner Findings:\n{}\n\nProvide an AppSec audit covering: 1. Injection attack vectors 2. Unsafe memory / FFI surface 3. Authentication & input validation recommendations.",
            path_target, findings_summary
        );

        let (audit_report, _) = ai.ask_with_meta(&prompt)?;
        Ok(audit_report)
    }
}

pub struct SreTriager;

impl SreTriager {
    pub fn triage_log(log_snippet: &str) -> Result<String> {
        let ai = Ai::default_model()?;
        let prompt = format!(
            "You are an SRE & Production Incident Triager. Analyze the following runtime error/stack trace/compiler failure:\n\n```\n{}\n```\n\nIdentify: 1. Root cause 2. Exact failure location 3. Unified patch/diff to resolve the issue.",
            log_snippet
        );

        let (triage_result, _) = ai.ask_with_meta(&prompt)?;
        Ok(triage_result)
    }
}
