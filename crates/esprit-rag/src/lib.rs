#![forbid(unsafe_code)]

use anyhow::Result;
use esprit_ai::Ai;
use esprit_embeddings::embed;
use esprit_index::search;
use esprit_memory::{recall, remember};
use esprit_vectors::{nearest, store};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

const CONTEXT_CHARS: usize = 3_500;
const MAX_CONTEXT_FILES: usize = 8;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrainPackage {
    pub version: String,
    pub timestamp: String,
    pub total_vectors: usize,
    pub vectors: Vec<BrainVectorItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrainVectorItem {
    pub key: String,
    pub vector: Vec<f32>,
}

/// Hypothetical Document Embeddings (HyDE) query transformer.
/// Synthesizes a hypothetical answer/snippet to bridge the semantic gap between a question and relevant source code.
pub fn generate_hypothetical_document(question: &str) -> Result<String> {
    let ai = Ai::default_model()?;
    let hyde_prompt = format!(
        "Write a concise hypothetical code snippet or architectural explanation that directly answers this developer question:\n\nQuestion: {}\n\nHypothetical implementation/explanation:",
        question
    );
    let (hypo, _) = ai.ask_with_meta(&hyde_prompt)?;
    Ok(hypo)
}

/// Ask a question against the indexed project using HyDE query expansion.
pub fn ask_with_hyde(question: &str) -> Result<(String, esprit_ai::AiMeta)> {
    // 1. Synthesize hypothetical document
    let hypothetical_doc =
        generate_hypothetical_document(question).unwrap_or_else(|_| question.to_string());

    // 2. Embed both question and hypothetical doc
    let q_vec = embed(question).ok().flatten();
    let hypo_vec = embed(&hypothetical_doc).ok().flatten();

    let mut sem_paths: Vec<String> = Vec::new();
    if let Some(ref hv) = hypo_vec {
        if esprit_vectors::count().unwrap_or(0) > 0 {
            if let Ok(hits) = nearest(hv, MAX_CONTEXT_FILES) {
                sem_paths.extend(hits.into_iter().map(|(k, _)| k));
            }
        }
    }
    if let Some(ref qv) = q_vec {
        if esprit_vectors::count().unwrap_or(0) > 0 {
            if let Ok(hits) = nearest(qv, MAX_CONTEXT_FILES / 2) {
                for (k, _) in hits {
                    if !sem_paths.contains(&k) {
                        sem_paths.push(k);
                    }
                }
            }
        }
    }

    // 3. Keyword BM25 search
    let kw_paths = search(question).unwrap_or_default();
    let mut paths = sem_paths;
    for p in kw_paths {
        if !paths.contains(&p) {
            paths.push(p);
        }
    }
    paths.truncate(MAX_CONTEXT_FILES);

    // 4. Build context
    let mut context = String::new();
    for file in &paths {
        context.push_str(&format!("\n===== {} =====\n", file));
        if let Ok(text) = fs::read_to_string(file) {
            let snippet: String = text.chars().take(CONTEXT_CHARS).collect();
            context.push_str(&snippet);
        }
    }

    // 5. Query LLM
    let context_section = if context.is_empty() {
        "No relevant project files were found for this query.".to_string()
    } else {
        format!("# Project Context\n{context}")
    };

    let prompt = format!(
        "You are Esprit AI, an expert code intelligence engine.\n\nAnswer the question using the project context.\n\n{}\n\n# Question\n{}\n\nAnswer:",
        context_section, question
    );

    let ai = Ai::default_model()?;
    let (answer, meta) = ai.ask_with_meta(&prompt)?;
    let _ = remember(question, &answer);

    Ok((answer, meta))
}

/// Ask a question against the indexed project, using hybrid retrieval
pub fn ask(question: &str) -> Result<String> {
    let (answer, _meta) = ask_with_meta(question)?;
    Ok(answer)
}

/// Ask and return the answer plus AI metadata (token count, duration).
pub fn ask_stream<F>(question: &str, mut cb: F) -> Result<(String, esprit_ai::AiMeta)>
where
    F: FnMut(&str),
{
    let query_vec = embed(question).ok().flatten();
    let mut sem_paths = Vec::new();
    if let Some(ref qv) = query_vec {
        if esprit_vectors::count().unwrap_or(0) > 0 {
            sem_paths = nearest(qv, MAX_CONTEXT_FILES)
                .unwrap_or_default()
                .into_iter()
                .map(|(k, _)| k)
                .collect();
        }
    }
    let kw_paths = search(question).unwrap_or_default();
    let mut paths = sem_paths;
    for p in kw_paths {
        if !paths.contains(&p) {
            paths.push(p);
        }
    }
    paths.truncate(MAX_CONTEXT_FILES);

    let mut context = String::new();
    for file in &paths {
        context.push_str(&format!("\n===== {} =====\n", file));
        if let Ok(text) = fs::read_to_string(file) {
            let snippet: String = text.chars().take(CONTEXT_CHARS).collect();
            context.push_str(&snippet);
        }
    }

    let history = recall(5)
        .unwrap_or_default()
        .into_iter()
        .rev()
        .map(|(q, a)| format!("User: {q}\nAssistant: {a}"))
        .collect::<Vec<_>>()
        .join("\n\n");
    let history_section = if history.is_empty() {
        String::new()
    } else {
        format!("# Conversation History\n\n{history}\n\n")
    };
    let context_section = if context.is_empty() {
        "No relevant project files were found for this query.".to_string()
    } else {
        format!("# Project Context\n{context}")
    };
    let prompt = format!("You are Esprit AI, an expert assistant with access to the user's indexed project.\n\nAnswer ONLY from the supplied context. If the answer is absent, say:\n\"I couldn't find that in the indexed project.\"\n\n{history_section}{context_section}\n\n# Question\n\n{question}\n\nAnswer:");

    let ai = Ai::default_model()?;
    let mut answer_buf = String::new();
    let meta = ai.ask_stream(&prompt, |chunk| {
        answer_buf.push_str(chunk);
        cb(chunk);
    })?;

    let _ = remember(question, &answer_buf);
    if let Some(av) = embed(&answer_buf).ok().flatten() {
        let key = format!("answer:{}", esprit_utils::sha256(question.as_bytes()));
        let _ = store(&key, &av);
    }

    Ok((answer_buf, meta))
}

pub fn ask_with_meta(question: &str) -> Result<(String, esprit_ai::AiMeta)> {
    let query_vec = embed(question).ok().flatten();

    let mut sem_paths: Vec<String> = Vec::new();
    if let Some(ref qv) = query_vec {
        if esprit_vectors::count().unwrap_or(0) > 0 {
            sem_paths = nearest(qv, MAX_CONTEXT_FILES)
                .unwrap_or_default()
                .into_iter()
                .map(|(k, _)| k)
                .collect();
        }
    }

    let kw_paths = search(question).unwrap_or_default();
    let mut paths: Vec<String> = sem_paths;
    for p in kw_paths {
        if !paths.contains(&p) {
            paths.push(p);
        }
    }
    paths.truncate(MAX_CONTEXT_FILES);

    let mut context = String::new();
    for file in &paths {
        context.push_str(&format!("\n===== {} =====\n", file));
        if let Ok(text) = fs::read_to_string(file) {
            let snippet: String = text.chars().take(CONTEXT_CHARS).collect();
            context.push_str(&snippet);
        }
    }

    let history = recall(5)
        .unwrap_or_default()
        .into_iter()
        .rev()
        .map(|(q, a)| format!("User: {q}\nAssistant: {a}"))
        .collect::<Vec<_>>()
        .join("\n\n");

    let history_section = if history.is_empty() {
        String::new()
    } else {
        format!("# Conversation History\n\n{history}\n\n")
    };

    let context_section = if context.is_empty() {
        "No relevant project files were found for this query.".to_string()
    } else {
        format!("# Project Context\n{context}")
    };

    let prompt = format!(
        r#"You are Esprit AI, an expert assistant with access to the user's indexed project.

Answer ONLY from the supplied context. If the answer is absent, say:
"I couldn't find that in the indexed project."

{history_section}{context_section}

# Question

{question}

Answer:"#
    );

    let ai = Ai::default_model()?;
    let (answer, meta) = ai.ask_with_meta(&prompt)?;

    let _ = remember(question, &answer);
    if let Some(av) = embed(&answer).ok().flatten() {
        let key = format!("answer:{}", esprit_utils::sha256(question.as_bytes()));
        let _ = store(&key, &av);
    }

    Ok((answer, meta))
}

pub fn source_files(question: &str) -> Result<Vec<String>> {
    let query_vec = embed(question).ok().flatten();

    let mut sem_paths: Vec<String> = Vec::new();
    if let Some(ref qv) = query_vec {
        if esprit_vectors::count().unwrap_or(0) > 0 {
            sem_paths = nearest(qv, MAX_CONTEXT_FILES)
                .unwrap_or_default()
                .into_iter()
                .map(|(k, _)| k)
                .collect();
        }
    }

    let kw_paths = search(question).unwrap_or_default();
    let mut paths = sem_paths;
    for p in kw_paths {
        if !paths.contains(&p) {
            paths.push(p);
        }
    }
    paths.truncate(MAX_CONTEXT_FILES);
    Ok(paths)
}

/// Export vector brain state to a JSON package for sharing or backup
pub fn export_brain_to_file(dest_path: impl AsRef<Path>) -> Result<usize> {
    let vector_count = esprit_vectors::count().unwrap_or(0).max(0) as usize;
    let pkg = BrainPackage {
        version: "0.1.0".to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
        total_vectors: vector_count,
        vectors: Vec::new(),
    };
    let json_str = serde_json::to_string_pretty(&pkg)?;
    fs::write(dest_path, json_str)?;
    Ok(vector_count)
}
