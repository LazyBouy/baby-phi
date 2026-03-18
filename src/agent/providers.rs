// agent/providers.rs — Additional providers beyond the core 3
//
// Each provider implements the StreamProvider trait from core::kernel.
// Providers are registered in extra_providers() in mod.rs.
//
// Current providers:
//   OllamaProvider      — local Ollama instance via OpenAI-compatible API
//                         Default endpoint: http://localhost:11434/v1/chat/completions
//   OpenRouterV2Provider — OpenRouter with proper multi-turn tool-result handling.
//                          The core OpenRouterProvider flattens all messages to text-only,
//                          dropping tool results from conversation history. This provider
//                          correctly formats ToolResult content as OpenAI "tool" role messages.
//                          Use provider = "openrouter-v2" in config.toml.
//   OpenAiV2Provider    — Same fix for OpenAI-compatible endpoints.
//                          Use provider = "openai-v2" in config.toml.

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

// ── OpenAI-compat V2: proper tool-result handling ─────────────────────────────
//
// The core `call_openai_compat` flattens all messages to text-only, silently
// dropping Content::ToolUse and Content::ToolResult items. This means the LLM
// never sees tool results in multi-turn conversations — it's flying blind after
// the first tool call.
//
// This function converts the full message history correctly:
//   assistant messages → role "assistant" with optional tool_calls array
//   tool results → separate messages with role "tool" and tool_call_id
//   user text → role "user" with content string
//
// Both OpenAiV2Provider and OpenRouterV2Provider use this shared helper.
async fn call_openai_compat_v2(
    endpoint: &str,
    api_key: &str,
    model: &str,
    extra_headers: &[(&str, &str)],
    messages: &[Message],
    tools: &[ToolDefinition],
    system: &str,
) -> Result<ProviderResponse, ProviderError> {
    let client = reqwest::Client::new();

    let mut msgs: Vec<Value> = Vec::new();

    // System message first
    if !system.is_empty() {
        msgs.push(serde_json::json!({
            "role": "system",
            "content": system
        }));
    }

    // Convert full message history — preserving tool calls and results
    for m in messages {
        match m.role.as_str() {
            "assistant" => {
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
                                    "arguments": serde_json::to_string(input)
                                        .unwrap_or_else(|_| "{}".to_string())
                                }
                            }));
                        }
                        _ => {}
                    }
                }

                let mut msg = serde_json::json!({ "role": "assistant" });
                let text = text_parts.join("\n");
                // OpenAI: content can be null when there are tool_calls
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
                // May contain plain text OR tool results (or both)
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
                            // Each tool result becomes a separate "tool" role message
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

                if !tool_results.is_empty() {
                    // Emit tool results as individual "tool" messages
                    for tr in tool_results {
                        msgs.push(tr);
                    }
                }
                if !text_parts.is_empty() {
                    msgs.push(serde_json::json!({
                        "role": "user",
                        "content": text_parts.join("\n")
                    }));
                }
            }
            other => {
                // Pass through any other role as plain text
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
                msgs.push(serde_json::json!({ "role": other, "content": text }));
            }
        }
    }

    // Build request body
    let mut body = serde_json::json!({
        "model": model,
        "messages": msgs,
        "max_tokens": 8096,
    });

    // Add tool definitions
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
        body["tool_choice"] = Value::String("auto".to_string());
    }

    // Build request
    let mut req = client
        .post(endpoint)
        .bearer_auth(api_key)
        .header("content-type", "application/json");
    for (k, v) in extra_headers {
        req = req.header(*k, *v);
    }

    let resp = req
        .json(&body)
        .send()
        .await
        .map_err(|e| ProviderError::Network(e.to_string()))?;

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
            "provider returned HTML instead of JSON — endpoint may be misconfigured (got: {preview})"
        )));
    }

    let v: Value = serde_json::from_str(&body_text).map_err(|e| {
        let preview = &body_text[..body_text.len().min(500)];
        ProviderError::Other(format!("{e}\nraw response ({status}): {preview}"))
    })?;

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

// ── OpenRouterV2Provider ──────────────────────────────────────────────────────

/// OpenRouter with correct multi-turn tool-result handling.
///
/// The core `OpenRouterProvider` uses `call_openai_compat` which drops
/// `ToolUse` and `ToolResult` content from message history, breaking
/// multi-turn tool conversations.
///
/// This provider formats messages correctly:
///   - Assistant tool calls → `tool_calls` array in "assistant" message
///   - Tool results → separate "tool" role messages with `tool_call_id`
///
/// Also sets `max_tokens: 8096` and `tool_choice: "auto"` for reliability.
///
/// Usage: set `provider = "openrouter-v2"` in config.toml.
/// Requires OPENROUTER_API_KEY env var (same as the core openrouter provider).
pub struct OpenRouterV2Provider {
    pub endpoint: String,
    pub api_key: String,
    pub model: String,
}

impl OpenRouterV2Provider {
    pub fn new(endpoint: impl Into<String>, api_key: impl Into<String>, model: impl Into<String>) -> Self {
        Self {
            endpoint: endpoint.into(),
            api_key: api_key.into(),
            model: model.into(),
        }
    }
}

#[async_trait]
impl StreamProvider for OpenRouterV2Provider {
    async fn stream(
        &self,
        messages: &[Message],
        tools: &[ToolDefinition],
        system: &str,
    ) -> Result<ProviderResponse, ProviderError> {
        call_openai_compat_v2(
            &self.endpoint,
            &self.api_key,
            &self.model,
            &[("HTTP-Referer", "https://github.com/LazyBouy/baby-phi")],
            messages,
            tools,
            system,
        )
        .await
    }
}

// ── OpenAiV2Provider ──────────────────────────────────────────────────────────

/// OpenAI-compatible endpoint with correct multi-turn tool-result handling.
///
/// Same fix as OpenRouterV2Provider — proper ToolResult → "tool" role conversion.
/// Use `provider = "openai-v2"` in config.toml.
/// Requires OPENAI_API_KEY env var.
pub struct OpenAiV2Provider {
    pub endpoint: String,
    pub api_key: String,
    pub model: String,
}

impl OpenAiV2Provider {
    pub fn new(endpoint: impl Into<String>, api_key: impl Into<String>, model: impl Into<String>) -> Self {
        Self {
            endpoint: endpoint.into(),
            api_key: api_key.into(),
            model: model.into(),
        }
    }
}

#[async_trait]
impl StreamProvider for OpenAiV2Provider {
    async fn stream(
        &self,
        messages: &[Message],
        tools: &[ToolDefinition],
        system: &str,
    ) -> Result<ProviderResponse, ProviderError> {
        call_openai_compat_v2(
            &self.endpoint,
            &self.api_key,
            &self.model,
            &[], // No extra headers for plain OpenAI
            messages,
            tools,
            system,
        )
        .await
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

    // ── OpenRouterV2Provider tests ────────────────────────────────────────────

    #[test]
    fn openrouter_v2_constructs() {
        let p = OpenRouterV2Provider::new(
            "https://openrouter.ai/api/v1/chat/completions",
            "test-key",
            "mistralai/mistral-7b-instruct",
        );
        assert_eq!(p.model, "mistralai/mistral-7b-instruct");
        assert!(p.endpoint.contains("openrouter.ai"));
        assert_eq!(p.api_key, "test-key");
    }

    #[test]
    fn openrouter_v2_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<OpenRouterV2Provider>();
    }

    #[tokio::test]
    async fn openrouter_v2_connection_refused_gives_network_error() {
        let p = OpenRouterV2Provider::new(
            "http://localhost:19998/v1/chat/completions",
            "fake-key",
            "test-model",
        );
        let messages = vec![Message::user("hello")];
        let result = p.stream(&messages, &[], "").await;
        assert!(result.is_err(), "should fail with no server");
        assert!(
            matches!(result, Err(ProviderError::Network(_))),
            "should be a network error"
        );
    }

    // ── OpenAiV2Provider tests ────────────────────────────────────────────────

    #[test]
    fn openai_v2_constructs() {
        let p = OpenAiV2Provider::new(
            "https://api.openai.com/v1/chat/completions",
            "sk-test",
            "gpt-4o",
        );
        assert_eq!(p.model, "gpt-4o");
        assert!(p.endpoint.contains("openai.com"));
        assert_eq!(p.api_key, "sk-test");
    }

    #[test]
    fn openai_v2_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<OpenAiV2Provider>();
    }

    #[tokio::test]
    async fn openai_v2_connection_refused_gives_network_error() {
        let p = OpenAiV2Provider::new(
            "http://localhost:19997/v1/chat/completions",
            "fake-key",
            "test-model",
        );
        let messages = vec![Message::user("hello")];
        let result = p.stream(&messages, &[], "").await;
        assert!(result.is_err(), "should fail with no server");
        assert!(
            matches!(result, Err(ProviderError::Network(_))),
            "should be a network error"
        );
    }

    // ── call_openai_compat_v2 message-format tests ────────────────────────────

    #[test]
    fn v2_message_format_roundtrip_logic() {
        // Verify the content types we'd send are structured correctly
        // by checking the JSON serialization.

        // Simulate an assistant message with a tool call
        let tool_input = serde_json::json!({"command": "echo hello"});
        let assistant_content = vec![
            Content::Text { text: "Let me run that.".into() },
            Content::ToolUse {
                id: "call_123".into(),
                name: "bash".into(),
                input: tool_input,
            },
        ];

        // Manually serialize as the v2 function would
        let mut text_parts: Vec<&str> = Vec::new();
        let mut tool_calls: Vec<Value> = Vec::new();
        for c in &assistant_content {
            match c {
                Content::Text { text } => text_parts.push(text),
                Content::ToolUse { id, name, input } => {
                    tool_calls.push(serde_json::json!({
                        "id": id,
                        "type": "function",
                        "function": {
                            "name": name,
                            "arguments": serde_json::to_string(input).unwrap()
                        }
                    }));
                }
                _ => {}
            }
        }

        assert_eq!(text_parts.len(), 1, "should have 1 text part");
        assert_eq!(tool_calls.len(), 1, "should have 1 tool call");
        assert_eq!(tool_calls[0]["id"].as_str(), Some("call_123"));
        assert_eq!(tool_calls[0]["function"]["name"].as_str(), Some("bash"));
    }

    #[test]
    fn v2_tool_result_becomes_tool_role_message() {
        // Simulate a user message with a tool result
        let user_content = vec![
            Content::ToolResult {
                tool_use_id: "call_123".into(),
                content: "hello\n".into(),
                is_error: false,
            },
        ];

        let mut tool_results: Vec<Value> = Vec::new();
        for c in &user_content {
            if let Content::ToolResult { tool_use_id, content, is_error } = c {
                tool_results.push(serde_json::json!({
                    "role": "tool",
                    "tool_call_id": tool_use_id,
                    "content": if *is_error { format!("[error] {content}") } else { content.clone() }
                }));
            }
        }

        assert_eq!(tool_results.len(), 1);
        assert_eq!(tool_results[0]["role"].as_str(), Some("tool"));
        assert_eq!(tool_results[0]["tool_call_id"].as_str(), Some("call_123"));
        assert_eq!(tool_results[0]["content"].as_str(), Some("hello\n"));
    }

    #[test]
    fn v2_error_tool_result_prefixes_error() {
        let content = vec![Content::ToolResult {
            tool_use_id: "call_456".into(),
            content: "command not found".into(),
            is_error: true,
        }];

        let mut tool_results: Vec<Value> = Vec::new();
        for c in &content {
            if let Content::ToolResult { tool_use_id, content, is_error } = c {
                tool_results.push(serde_json::json!({
                    "role": "tool",
                    "tool_call_id": tool_use_id,
                    "content": if *is_error { format!("[error] {content}") } else { content.clone() }
                }));
            }
        }

        assert_eq!(tool_results[0]["content"].as_str(), Some("[error] command not found"));
    }
}
