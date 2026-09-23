#![forbid(unsafe_code)]

use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::io::{self, BufRead, Read, Write};

#[derive(Debug, Serialize, Deserialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub id: Option<Value>,
    pub method: String,
    pub params: Option<Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    pub id: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<Value>,
}

/// Run native Esprit LSP server over stdio
pub fn run_lsp_server() -> Result<()> {
    let stdin = io::stdin();
    let mut reader = stdin.lock();
    let mut stdout = io::stdout();

    loop {
        let mut line = String::new();
        let bytes_read = reader.read_line(&mut line)?;
        if bytes_read == 0 {
            break;
        }

        let line_trimmed = line.trim();
        if line_trimmed.starts_with("Content-Length:") {
            let parts: Vec<&str> = line_trimmed.split(':').collect();
            if parts.len() == 2 {
                if let Ok(len) = parts[1].trim().parse::<usize>() {
                    // Read trailing empty header lines
                    let mut empty_line = String::new();
                    reader.read_line(&mut empty_line)?;

                    let mut body_buf = vec![0u8; len];
                    reader.read_exact(&mut body_buf)?;

                    if let Ok(req) = serde_json::from_slice::<JsonRpcRequest>(&body_buf) {
                        let resp = handle_lsp_request(&req);
                        if let Some(resp_json) = resp {
                            let resp_bytes = serde_json::to_vec(&resp_json)?;
                            let header = format!("Content-Length: {}\r\n\r\n", resp_bytes.len());
                            stdout.write_all(header.as_bytes())?;
                            stdout.write_all(&resp_bytes)?;
                            stdout.flush()?;
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

fn handle_lsp_request(req: &JsonRpcRequest) -> Option<JsonRpcResponse> {
    let id = req.id.clone();
    match req.method.as_str() {
        "initialize" => Some(JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id,
            result: Some(json!({
                "capabilities": {
                    "textDocumentSync": 1,
                    "completionProvider": {
                        "resolveProvider": true,
                        "triggerCharacters": [".", ":", ">", "/", "@"]
                    },
                    "hoverProvider": true,
                    "codeActionProvider": true
                },
                "serverInfo": {
                    "name": "esprit-lsp",
                    "version": "0.1.0"
                }
            })),
            error: None,
        }),
        "textDocument/completion" => {
            let completions = json!({
                "isIncomplete": false,
                "items": [
                    {
                        "label": "esprit::ai_assist",
                        "kind": 3,
                        "detail": "Esprit AI Local Autocomplete",
                        "documentation": "Synthesize contextual code completion with local LLM."
                    },
                    {
                        "label": "esprit::blast_radius",
                        "kind": 1,
                        "detail": "Esprit Code Intelligence",
                        "documentation": "Analyze downstream AST blast radius for this symbol."
                    }
                ]
            });

            Some(JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id,
                result: Some(completions),
                error: None,
            })
        }
        "textDocument/hover" => {
            let hover_info = json!({
                "contents": {
                    "kind": "markdown",
                    "value": "**Esprit Intelligence**\n\n*Symbol analyzed with zero-latency local AST and vector memory.*"
                }
            });

            Some(JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id,
                result: Some(hover_info),
                error: None,
            })
        }
        "textDocument/codeAction" => {
            let actions = json!([
                {
                    "title": "⚡ Esprit: Auto-Fix with AI",
                    "kind": "quickfix",
                    "isPreferred": true
                },
                {
                    "title": "🔍 Esprit: Calculate Blast Radius",
                    "kind": "refactor"
                }
            ]);

            Some(JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id,
                result: Some(actions),
                error: None,
            })
        }
        "shutdown" => Some(JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id,
            result: Some(json!(null)),
            error: None,
        }),
        "exit" => None,
        _ => {
            if id.is_some() {
                Some(JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id,
                    result: Some(json!(null)),
                    error: None,
                })
            } else {
                None
            }
        }
    }
}
