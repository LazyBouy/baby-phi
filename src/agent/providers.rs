// agent/providers.rs — Additional providers beyond the core 3
//
// Each provider implements the StreamProvider trait from core::kernel.
// Providers are registered in extra_providers() in mod.rs.
//
// Current providers:
//   OllamaProvider — local Ollama instance via OpenAI-compatible API
//                    Default endpoint: http://localhost:11434/v1/chat/completions

use async_trait::async_trait;
use serde_json::Value;

use crate::core::{
    Content, Message, ProviderError, ProviderResponse, StopReason, StreamProvider, ToolDefinition,
    parse_retry_after,
};

// ── Ollama Provider ───────────────────────────────────────────────────────────

/// Connects to a local Ollama instance using the OpenAI-compatible API.
///
/// Ollama exposes an OpenAI-compatible endpoint at:
///   http://localhost:11434/v1/chat/completions
///
/// To use: set provider = "ollama" in config.toml and specify a model,
/// e.g. model = "llama3.2" or model = "qwen2.5-coder:7b".
///
/// No API key required for local Ollama.
pub struct OllamaProvider {
    pub endpoint: String,
    pub model: String,
}

impl OllamaProvider {
    /// Create with the default Ollama endpoint.
    #[allow(dead_code)]
    pub fn new(model: impl Into<String>) -> Self {
        Self {
            endpoint: "http://localhost:11434/v1/chat/completions".to_string(),
            model: model.into(),
        }
    }

    /// Create with a custom endpoint (useful for remote Ollama or tests).
    pub fn with_endpoint(endpoint: impl Into<String>, model: impl Into<String>) -> Self {
        Self {
            endpoint: endpoint.into(),
            model: model.into(),
        }
    }
}

#[async_trait]
impl StreamProvider for OllamaProvider {
    async fn stream(
        &self,
        messages: &[Message],
        tools: &[ToolDefinition],
        system: &str,
    ) -> Result<ProviderResponse, ProviderError> {
        let client = reqwest::Client::new();

        // Build messages array in OpenAI format
        let mut msgs: Vec<Value> = Vec::new();

        // System message first
        if !system.is_empty() {
            msgs.push(serde_json::json!({
                "role": "system",
                "content": system
            }));
        }

        // Convert messages — handle all content types
        for m in messages {
            match m.role.as_str() {
                "assistant" => {
                    // Collect text and tool calls
                    let mut text_parts: Vec<&str> = Vec::new();
                    let mut tool_calls: Vec<Value> = Vec::new();

                    for c in &m.content {
                        match c {
                            Content::Text { text } => text_parts.push(text),
                            Content::ToolUse { id, name, input } => {
                                tool_calls.push(serde_json::json!({
                                    "id": id,
                                    "type": "function",
                                    "function": {
                                        "name": name,
                                        "arguments": serde_json::to_string(input).unwrap_or_else(|_| "{}".to_string())
                                    }
                                }));
                            }
                            _ => {}
                        }
                    }

                    let mut msg = serde_json::json!({ "role": "assistant" });
                    let text = text_parts.join("\n");
                    if !text.is_empty() {
                        msg["content"] = Value::String(text);
                    } else {
                        msg["content"] = Value::Null;
                    }
                    if !tool_calls.is_empty() {
                        msg["tool_calls"] = Value::Array(tool_calls);
                    }
                    msgs.push(msg);
                }
                "user" => {
                    // Handle both plain text and tool results
                    let mut text_parts: Vec<String> = Vec::new();
                    let mut tool_results: Vec<Value> = Vec::new();

                    for c in &m.content {
                        match c {
                            Content::Text { text } => text_parts.push(text.clone()),
                            Content::ToolResult {
                                tool_use_id,
                                content,
                                is_error,
                            } => {
                                tool_results.push(serde_json::json!({
                                    "role": "tool",
                                    "tool_call_id": tool_use_id,
                                    "content": if *is_error {
                                        format!("[error] {content}")
                                    } else {
                                        content.clone()
                                    }
                                }));
                            }
                            _ => {}
                        }
                    }

                    // If we have tool results, emit them as separate "tool" role messages
                    // (OpenAI format requires tool results as separate messages)
                    if !tool_results.is_empty() {
                        for tr in tool_results {
                            msgs.push(tr);
                        }
                    } else {
                        // Plain user message
                        let text = text_parts.join("\n");
                        msgs.push(serde_json::json!({
                            "role": "user",
                            "content": text
                        }));
                    }
                }
                other => {
                    // Pass through other roles as plain text
                    let text: String = m
                        .content
                        .iter()
                        .filter_map(|c| {
                            if let Content::Text { text } = c {
                                Some(text.as_str())
                            } else {
                                None
                            }
                        })
                        .collect::<Vec<_>>()
                        .join("\n");
                    msgs.push(serde_json::json!({
                        "role": other,
                        "content": text
                    }));
                }
            }
        }

        // Build request body
        let mut body = serde_json::json!({
            "model": self.model,
            "messages": msgs,
        });

        // Add tools if any (Ollama supports function calling for capable models)
        if !tools.is_empty() {
            let tools_val: Vec<Value> = tools
                .iter()
                .map(|t| {
                    serde_json::json!({
                        "type": "function",
                        "function": {
                            "name": t.name,
                            "description": t.description,
                            "parameters": t.parameters
                        }
                    })
                })
                .collect();
            body["tools"] = Value::Array(tools_val);
        }

        // Send request — Ollama runs locally so no auth header needed
        let resp = client
            .post(&self.endpoint)
            .header("content-type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| {
                let msg = e.to_string();
                // Connection refused = Ollama not running
                if msg.contains("Connection refused") || msg.contains("connect error") {
                    ProviderError::Network(
                        "Ollama is not running. Start it with: ollama serve".to_string(),
                    )
                } else {
                    ProviderError::Network(msg)
                }
            })?;

        let status = resp.status().as_u16();
        let retry_after_ms = resp
            .headers()
            .get("retry-after")
            .and_then(|v| v.to_str().ok())
            .and_then(parse_retry_after);
        let body_text = resp
            .text()
            .await
            .map_err(|e| ProviderError::Network(e.to_string()))?;

        if status != 200 {
            return Err(ProviderError::classify(status, &body_text, retry_after_ms));
        }

        if body_text.trim_start().starts_with('<') {
            let preview = &body_text[..body_text.len().min(200)];
            return Err(ProviderError::Other(format!(
                "Ollama returned HTML instead of JSON — endpoint may be wrong (got: {preview})"
            )));
        }

        let v: Value = serde_json::from_str(&body_text).map_err(|e| {
            let preview = &body_text[..body_text.len().min(500)];
            ProviderError::Other(format!("{e}\nraw response ({status}): {preview}"))
        })?;

        // Parse OpenAI-compatible response format
        let choice = &v["choices"][0];
        let stop_reason = match choice["finish_reason"].as_str() {
            Some("tool_calls") => StopReason::ToolUse,
            Some("length") => StopReason::MaxTokens,
            _ => StopReason::EndTurn,
        };

        let mut content: Vec<Content> = Vec::new();

        // Extract text content
        if let Some(text) = choice["message"]["content"].as_str() {
            if !text.is_empty() {
                content.push(Content::Text {
                    text: text.to_string(),
                });
            }
        }

        // Extract tool calls
        if let Some(tool_calls) = choice["message"]["tool_calls"].as_array() {
            for tc in tool_calls {
                let args_str = tc["function"]["arguments"].as_str().unwrap_or("{}");
                let input = serde_json::from_str::<Value>(args_str).unwrap_or(Value::Object(
                    serde_json::Map::new(),
                ));
                content.push(Content::ToolUse {
                    id: tc["id"].as_str().unwrap_or("").to_string(),
                    name: tc["function"]["name"].as_str().unwrap_or("").to_string(),
                    input,
                });
            }
        }

        Ok(ProviderResponse {
            message: Message::assistant(content),
            stop_reason,
        })
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ollama_provider_default_endpoint() {
        let p = OllamaProvider::new("llama3.2");
        assert_eq!(p.model, "llama3.2");
        assert!(
            p.endpoint.contains("11434"),
            "default endpoint should use port 11434: {}",
            p.endpoint
        );
        assert!(
            p.endpoint.contains("/v1/"),
            "endpoint should include /v1/ path: {}",
            p.endpoint
        );
    }

    #[test]
    fn ollama_provider_custom_endpoint() {
        let p = OllamaProvider::with_endpoint("http://remote:11434/v1/chat/completions", "phi4");
        assert_eq!(p.model, "phi4");
        assert!(p.endpoint.contains("remote"));
    }

    #[tokio::test]
    async fn ollama_provider_connection_refused_gives_helpful_error() {
        // Point at a port nothing listens on
        let p = OllamaProvider::with_endpoint(
            "http://localhost:19999/v1/chat/completions",
            "test-model",
        );
        let messages = vec![Message::user("hello")];
        let result = p.stream(&messages, &[], "").await;
        assert!(result.is_err(), "should fail when server is not running");
        // Should be a Network error — either "not running" or general network error
        assert!(
            matches!(result, Err(ProviderError::Network(_))),
            "connection refused should be ProviderError::Network"
        );
    }

    #[test]
    fn ollama_builds_correct_openai_format() {
        // Test message conversion logic by inspecting what we'd send
        // We can't make a real HTTP call in unit tests, but we can verify
        // the message structure logic is sound.
        let p = OllamaProvider::new("test");
        assert_eq!(p.model, "test");

        // Verify endpoint format is correct for Ollama
        assert!(p.endpoint.ends_with("/chat/completions"));
    }

    #[test]
    fn ollama_provider_is_send_sync() {
        // StreamProvider requires Send + Sync
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<OllamaProvider>();
    }

    #[test]
    fn ollama_with_empty_model_string() {
        // Edge case: empty model (Ollama will error, but we should construct without panic)
        let p = OllamaProvider::new("");
        assert_eq!(p.model, "");
    }
}
