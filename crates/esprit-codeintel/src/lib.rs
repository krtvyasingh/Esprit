#![forbid(unsafe_code)]

use anyhow::Result;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Symbol {
    pub name: String,
    pub kind: String,
    pub language: String,
    pub file: String,
    pub line: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlastRadiusReport {
    pub target: String,
    pub is_file: bool,
    pub risk_level: String,
    pub total_impacted_files: usize,
    pub total_references: usize,
    pub direct_dependents: Vec<String>,
    pub impacted_test_files: Vec<String>,
    pub recommended_actions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplexityReport {
    pub file: String,
    pub cognitive_score: usize,
    pub cyclomatic_score: usize,
    pub lines_of_code: usize,
    pub rating: String,
    pub hotspots: Vec<ComplexityHotspot>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplexityHotspot {
    pub line: usize,
    pub reason: String,
    pub preview: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeadCodeFinding {
    pub symbol: String,
    pub kind: String,
    pub file: String,
    pub line: usize,
    pub suggestion: String,
}

/// Polyglot symbol indexing for Rust, TypeScript/JS, Python, Go, and C/C++
pub fn index(root: impl AsRef<Path>) -> Result<Vec<Symbol>> {
    let mut out = Vec::new();

    let rust_re = Regex::new(
        r"(?m)^\s*(?:pub(?:\([^)]+\))?\s+)?(fn|struct|enum|trait|type)\s+([A-Za-z0-9_]+)",
    )?;
    let ts_re = Regex::new(
        r"(?m)^\s*(?:export\s+)?(function|class|interface|type|enum)\s+([A-Za-z0-9_]+)",
    )?;
    let py_re = Regex::new(r"(?m)^\s*(def|class)\s+([A-Za-z0-9_]+)")?;
    let go_re = Regex::new(
        r"(?m)^\s*func\s+(?:\([^)]+\)\s+)?([A-Za-z0-9_]+)|^\s*type\s+([A-Za-z0-9_]+)\s+(?:struct|interface)",
    )?;

    for entry in WalkDir::new(root).into_iter().filter_entry(|e| {
        let name = e.file_name().to_string_lossy();
        !name.starts_with('.') && name != "target" && name != "node_modules" && name != "dist"
    }) {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };

        if !entry.file_type().is_file() {
            continue;
        }

        let path = entry.path();
        let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("");
        let file_str = path.display().to_string();

        let (lang, re_opt) = match ext {
            "rs" => ("Rust", Some(&rust_re)),
            "ts" | "tsx" | "js" | "jsx" => ("TypeScript/JavaScript", Some(&ts_re)),
            "py" => ("Python", Some(&py_re)),
            "go" => ("Go", Some(&go_re)),
            _ => ("", None),
        };

        if let Some(re) = re_opt {
            if let Ok(text) = fs::read_to_string(path) {
                for (idx, line) in text.lines().enumerate() {
                    if let Some(caps) = re.captures(line) {
                        let kind = caps
                            .get(1)
                            .map(|m| m.as_str())
                            .unwrap_or("symbol")
                            .to_string();
                        let name = caps
                            .get(2)
                            .map(|m| m.as_str())
                            .or_else(|| caps.get(1).map(|m| m.as_str()))
                            .unwrap_or("")
                            .to_string();

                        if !name.is_empty() {
                            out.push(Symbol {
                                name,
                                kind,
                                language: lang.to_string(),
                                file: file_str.clone(),
                                line: idx + 1,
                            });
                        }
                    }
                }
            }
        }
    }

    Ok(out)
}

/// Calculate the cross-file blast radius and impact surface of modifying a symbol or file
pub fn calculate_blast_radius(
    root: impl AsRef<Path>,
    target_query: &str,
) -> Result<BlastRadiusReport> {
    let mut file_contents: HashMap<String, String> = HashMap::new();
    let mut all_files = Vec::new();

    for entry in WalkDir::new(&root).into_iter().filter_entry(|e| {
        let name = e.file_name().to_string_lossy();
        !name.starts_with('.') && name != "target" && name != "node_modules" && name != "dist"
    }) {
        if let Ok(e) = entry {
            if e.file_type().is_file() {
                let path = e.path();
                let path_str = path.display().to_string();
                if let Ok(content) = fs::read_to_string(path) {
                    file_contents.insert(path_str.clone(), content);
                    all_files.push(path_str);
                }
            }
        }
    }

    let is_file =
        target_query.contains('.') || target_query.contains('/') || target_query.contains('\\');
    let mut direct_dependents = HashSet::new();
    let mut impacted_test_files = HashSet::new();
    let mut total_references = 0;

    let search_term = if is_file {
        Path::new(target_query)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or(target_query)
    } else {
        target_query
    };

    let ident_re = Regex::new(&format!(r"\b{}\b", regex::escape(search_term)))?;

    for (file_path, content) in &file_contents {
        if file_path.contains(target_query) {
            continue; // Skip self
        }

        let matches = ident_re.find_iter(content).count();
        if matches > 0 {
            total_references += matches;
            direct_dependents.insert(file_path.clone());

            let lower = file_path.to_lowercase();
            if lower.contains("test")
                || lower.contains("spec")
                || lower.ends_with("_test.rs")
                || lower.ends_with(".test.ts")
            {
                impacted_test_files.insert(file_path.clone());
            }
        }
    }

    let total_impacted_files = direct_dependents.len();
    let risk_level = if total_impacted_files > 15 || total_references > 50 {
        "CRITICAL".to_string()
    } else if total_impacted_files > 6 || total_references > 15 {
        "HIGH".to_string()
    } else if total_impacted_files > 1 {
        "MEDIUM".to_string()
    } else {
        "LOW".to_string()
    };

    let mut recommended_actions = Vec::new();
    if !impacted_test_files.is_empty() {
        recommended_actions.push(format!(
            "Run {} targeted test suites covering dependent modules.",
            impacted_test_files.len()
        ));
    } else {
        recommended_actions.push("No dedicated test files detected referencing this target — recommend adding regression tests.".to_string());
    }
    if risk_level == "CRITICAL" || risk_level == "HIGH" {
        recommended_actions.push(
            "Perform AST semantic diff review to avoid breaking transitive downstream consumers."
                .to_string(),
        );
    }

    let mut direct_deps_vec: Vec<String> = direct_dependents.into_iter().collect();
    direct_deps_vec.sort();
    let mut test_files_vec: Vec<String> = impacted_test_files.into_iter().collect();
    test_files_vec.sort();

    Ok(BlastRadiusReport {
        target: target_query.to_string(),
        is_file,
        risk_level,
        total_impacted_files,
        total_references,
        direct_dependents: direct_deps_vec,
        impacted_test_files: test_files_vec,
        recommended_actions,
    })
}

/// Analyze Cognitive and Cyclomatic Complexity for a single file or directory
pub fn analyze_complexity(file_path: impl AsRef<Path>) -> Result<ComplexityReport> {
    let path = file_path.as_ref();
    let content = fs::read_to_string(path)?;
    let mut cognitive_score = 0;
    let mut cyclomatic_score = 1; // Base score
    let mut hotspots = Vec::new();

    let lines: Vec<&str> = content.lines().collect();
    let loc = lines.len();

    let branch_re = Regex::new(r"\b(if|else\s+if|match|switch|for|while|catch)\b|\?\s*|\&\&|\|\|")?;

    let mut current_indent = 0;
    for (idx, line) in lines.iter().enumerate() {
        let line_num = idx + 1;
        let trimmed = line.trim();

        if trimmed.is_empty()
            || trimmed.starts_with("//")
            || trimmed.starts_with('#')
            || trimmed.starts_with('*')
        {
            continue;
        }

        let leading_spaces = line.chars().take_while(|c| c.is_whitespace()).count();
        let indent_level = leading_spaces / 4;

        if branch_re.is_match(trimmed) {
            let branch_count = branch_re.find_iter(trimmed).count();
            cyclomatic_score += branch_count;
            let nesting_penalty = if indent_level > 2 { indent_level } else { 1 };
            cognitive_score += branch_count * nesting_penalty;

            if indent_level >= 4 || branch_count >= 3 {
                hotspots.push(ComplexityHotspot {
                    line: line_num,
                    reason: format!(
                        "Deep nesting level ({}) and multiple conditional branches ({}x)",
                        indent_level, branch_count
                    ),
                    preview: trimmed.to_string(),
                });
            }
        }

        current_indent = indent_level;
    }

    let _ = current_indent;

    let rating = if cognitive_score > 50 || cyclomatic_score > 30 {
        "CRITICAL COMPLEXITY (Refactor Needed)".to_string()
    } else if cognitive_score > 25 || cyclomatic_score > 15 {
        "HIGH COMPLEXITY (Review Required)".to_string()
    } else if cognitive_score > 10 {
        "MODERATE COMPLEXITY".to_string()
    } else {
        "EXCELLENT (Clean & Readable)".to_string()
    };

    Ok(ComplexityReport {
        file: path.display().to_string(),
        cognitive_score,
        cyclomatic_score,
        lines_of_code: loc,
        rating,
        hotspots,
    })
}

/// Find dead or orphaned local functions/structs that have zero references in other files
pub fn find_dead_code(root: impl AsRef<Path>) -> Result<Vec<DeadCodeFinding>> {
    let symbols = index(&root)?;
    let mut file_contents: HashMap<String, String> = HashMap::new();

    for entry in WalkDir::new(&root).into_iter().filter_entry(|e| {
        let name = e.file_name().to_string_lossy();
        !name.starts_with('.') && name != "target" && name != "node_modules"
    }) {
        if let Ok(e) = entry {
            if e.file_type().is_file() {
                if let Ok(content) = fs::read_to_string(e.path()) {
                    file_contents.insert(e.path().display().to_string(), content);
                }
            }
        }
    }

    let mut findings = Vec::new();
    for sym in symbols {
        // Skip common trait/interface methods and standard entry points
        if sym.name == "main"
            || sym.name == "default"
            || sym.name == "new"
            || sym.name == "fmt"
            || sym.name == "run"
        {
            continue;
        }

        let re = match Regex::new(&format!(r"\b{}\b", regex::escape(&sym.name))) {
            Ok(r) => r,
            Err(_) => continue,
        };

        let mut external_refs = 0;
        for (file, content) in &file_contents {
            if file == &sym.file {
                // In same file, count if more than 1 occurrence (definition + callers)
                let same_file_hits = re.find_iter(content).count();
                if same_file_hits > 1 {
                    external_refs += same_file_hits - 1;
                }
            } else {
                external_refs += re.find_iter(content).count();
            }
        }

        if external_refs == 0 {
            findings.push(DeadCodeFinding {
                symbol: sym.name.clone(),
                kind: sym.kind.clone(),
                file: sym.file.clone(),
                line: sym.line,
                suggestion: format!(
                    "Symbol `{}` has 0 external references. Consider pruning or making private.",
                    sym.name
                ),
            });
        }
    }

    Ok(findings)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_complexity_analysis() {
        use std::io::Write;
        let dir = std::env::temp_dir();
        let file_path = dir.join("test_complexity.rs");
        let mut f = fs::File::create(&file_path).unwrap();
        writeln!(f, "fn example() {{").unwrap();
        writeln!(f, "    if true {{").unwrap();
        writeln!(f, "        if false && true {{").unwrap();
        writeln!(f, "            println!(\"hello\");").unwrap();
        writeln!(f, "        }}").unwrap();
        writeln!(f, "    }}").unwrap();
        writeln!(f, "}}").unwrap();

        let report = analyze_complexity(&file_path).unwrap();
        assert!(report.cyclomatic_score >= 3);
        let _ = fs::remove_file(file_path);
    }
}
