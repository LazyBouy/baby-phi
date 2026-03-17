// agent.rs — baby-phi agent core
// Types, traits, 3 provider adapters, 6 tools, retry logic, agent loop.
// Initial version: non-streaming provider calls (streaming is baby-phi's to add).

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::process::Stdio;
use std::time::Duration;

// ── Types ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: String,
    pub content: Vec<Content>,
}

impl Message {
    pub fn user(text: impl Into<String>) -> Self {
        Self { role: "user".into(), content: vec![Content::Text { text: text.into() }] }
    }
    pub fn assistant(content: Vec<Content>) -> Self {
        Self { role: "assistant".into(), content }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Content {
    Text { text: String },
    ToolUse { id: String, name: String, input: Value },
    ToolResult { tool_use_id: String, content: String, is_error: bool },
}

#[derive(Debug, Clone, PartialEq)]
pub enum StopReason { EndTurn, ToolUse, MaxTokens, Error }

pub struct ProviderResponse {
    pub message: Message,
    pub stop_reason: StopReason,
}

#[derive(Debug)]
pub enum ProviderError {
    RateLimited(String),
    Network(String),
    ContextTooLong,
    InvalidRequest(String),
    Other(String),
}

impl std::fmt::Display for ProviderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::RateLimited(s) => write!(f, "rate limited: {s}"),
            Self::Network(s)     => write!(f, "network: {s}"),
            Self::ContextTooLong => write!(f, "context too long"),
            Self::InvalidRequest(s) => write!(f, "invalid request: {s}"),
            Self::Other(s)       => write!(f, "error: {s}"),
        }
    }
}

impl ProviderError {
    pub fn classify(status: u16, body: &str) -> Self {
        match status {
            429 => Self::RateLimited(body.to_string()),
            400 if body.to_lowercase().contains("context") => Self::ContextTooLong,
            400 => Self::InvalidRequest(body.to_string()),
            500..=599 => Self::Network(body.to_string()),
            _ => Self::Other(format!("HTTP {status}: {body}")),
        }
    }
    pub fn is_retryable(&self) -> bool {
        matches!(self, Self::RateLimited(_) | Self::Network(_))
    }
}

pub struct RetryConfig {
    pub max_attempts: u32,
    pub base_delay_ms: u64,
    pub max_delay_ms: u64,
    pub jitter_factor: f64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self { max_attempts: 3, base_delay_ms: 1_000, max_delay_ms: 30_000, jitter_factor: 0.2 }
    }
}

impl RetryConfig {
    pub fn delay_for(&self, attempt: u32) -> Duration {
        let base = (self.base_delay_ms as f64) * (2u64.pow(attempt) as f64);
        let capped = base.min(self.max_delay_ms as f64);
        // deterministic ±jitter based on attempt parity (rand dep avoided)
        let sign = if attempt % 2 == 0 { 1.0 } else { -1.0 };
        Duration::from_millis((capped * (1.0 + sign * self.jitter_factor * 0.5)) as u64)
    }
}

#[derive(Debug, Clone)]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub parameters: Value,
}

pub enum AgentEvent {
    TurnStart { turn: u32 },
    TextDelta(String),
    ToolStart { name: String, input: Value },
    ToolEnd   { name: String, output: String, is_error: bool },
    TurnEnd,
    AgentEnd,
}

pub struct ToolResult {
    pub content: String,
    pub is_error: bool,
}

// ── Traits ────────────────────────────────────────────────────────────────────

#[async_trait]
pub trait StreamProvider: Send + Sync {
    async fn stream(
        &self,
        messages: &[Message],
        tools: &[ToolDefinition],
        system: &str,
    ) -> Result<ProviderResponse, ProviderError>;
}

#[async_trait]
pub trait AgentTool: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn parameters_schema(&self) -> Value;
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: self.name().to_string(),
            description: self.description().to_string(),
            parameters: self.parameters_schema(),
        }
    }
    async fn execute(&self, input: Value) -> ToolResult;
}

// ── Provider Adapters ─────────────────────────────────────────────────────────

pub struct AnthropicProvider { pub endpoint: String, pub api_key: String, pub model: String }
pub struct OpenAiProvider    { pub endpoint: String, pub api_key: String, pub model: String }
pub struct OpenRouterProvider{ pub endpoint: String, pub api_key: String, pub model: String }

#[async_trait]
impl StreamProvider for AnthropicProvider {
    async fn stream(&self, messages: &[Message], tools: &[ToolDefinition], system: &str) -> Result<ProviderResponse, ProviderError> {
        let client = reqwest::Client::new();
        let msgs: Vec<Value> = messages.iter().map(|m| {
            let content: Vec<Value> = m.content.iter().map(|c| match c {
                Content::Text { text } =>
                    serde_json::json!({ "type": "text", "text": text }),
                Content::ToolUse { id, name, input } =>
                    serde_json::json!({ "type": "tool_use", "id": id, "name": name, "input": input }),
                Content::ToolResult { tool_use_id, content, is_error } =>
                    serde_json::json!({ "type": "tool_result", "tool_use_id": tool_use_id, "content": content, "is_error": is_error }),
            }).collect();
            serde_json::json!({ "role": m.role, "content": content })
        }).collect();

        let tools_val: Vec<Value> = tools.iter().map(|t| serde_json::json!({
            "name": t.name, "description": t.description, "input_schema": t.parameters
        })).collect();

        let mut body = serde_json::json!({
            "model": self.model, "max_tokens": 8096, "messages": msgs
        });
        if !system.is_empty() { body["system"] = Value::String(system.to_string()); }
        if !tools.is_empty()  { body["tools"]  = Value::Array(tools_val); }

        let resp = client.post(&self.endpoint)
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&body).send().await
            .map_err(|e| ProviderError::Network(e.to_string()))?;

        let status = resp.status().as_u16();
        let body_text = resp.text().await.unwrap_or_default();
        if status != 200 { return Err(ProviderError::classify(status, &body_text)); }

        let v: Value = serde_json::from_str(&body_text)
            .map_err(|e| ProviderError::Other(e.to_string()))?;

        let stop_reason = match v["stop_reason"].as_str() {
            Some("tool_use")   => StopReason::ToolUse,
            Some("max_tokens") => StopReason::MaxTokens,
            Some("error")      => StopReason::Error,
            _                  => StopReason::EndTurn,
        };

        let mut content: Vec<Content> = vec![];
        for block in v["content"].as_array().unwrap_or(&vec![]) {
            match block["type"].as_str() {
                Some("text") => content.push(Content::Text {
                    text: block["text"].as_str().unwrap_or("").to_string()
                }),
                Some("tool_use") => content.push(Content::ToolUse {
                    id:    block["id"].as_str().unwrap_or("").to_string(),
                    name:  block["name"].as_str().unwrap_or("").to_string(),
                    input: block["input"].clone(),
                }),
                _ => {}
            }
        }
        Ok(ProviderResponse { message: Message::assistant(content), stop_reason })
    }
}

// Shared non-streaming call for OpenAI-compatible providers
async fn call_openai_compat(
    endpoint: &str,
    api_key: &str,
    model: &str,
    extra_headers: &[(&str, &str)],
    messages: &[Message],
    tools: &[ToolDefinition],
    system: &str,
) -> Result<ProviderResponse, ProviderError> {
    let client = reqwest::Client::new();
    let mut msgs: Vec<Value> = vec![];
    if !system.is_empty() {
        msgs.push(serde_json::json!({ "role": "system", "content": system }));
    }
    for m in messages {
        let text: String = m.content.iter().filter_map(|c| match c {
            Content::Text { text } => Some(text.clone()),
            _ => None,
        }).collect::<Vec<_>>().join("\n");
        msgs.push(serde_json::json!({ "role": m.role, "content": text }));
    }

    let tools_val: Vec<Value> = tools.iter().map(|t| serde_json::json!({
        "type": "function",
        "function": { "name": t.name, "description": t.description, "parameters": t.parameters }
    })).collect();

    let mut body = serde_json::json!({ "model": model, "messages": msgs });
    if !tools.is_empty() { body["tools"] = Value::Array(tools_val); }

    let mut req = client.post(endpoint)
        .bearer_auth(api_key)
        .header("content-type", "application/json");
    for (k, v) in extra_headers { req = req.header(*k, *v); }

    let resp = req.json(&body).send().await
        .map_err(|e| ProviderError::Network(e.to_string()))?;

    let status = resp.status().as_u16();
    let body_text = resp.text().await.unwrap_or_default();
    if status != 200 { return Err(ProviderError::classify(status, &body_text)); }

    let v: Value = serde_json::from_str(&body_text)
        .map_err(|e| ProviderError::Other(e.to_string()))?;

    let choice = &v["choices"][0];
    let stop_reason = match choice["finish_reason"].as_str() {
        Some("tool_calls") => StopReason::ToolUse,
        Some("length")     => StopReason::MaxTokens,
        _                  => StopReason::EndTurn,
    };

    let mut content: Vec<Content> = vec![];
    if let Some(text) = choice["message"]["content"].as_str() {
        if !text.is_empty() { content.push(Content::Text { text: text.to_string() }); }
    }
    if let Some(tcs) = choice["message"]["tool_calls"].as_array() {
        for tc in tcs {
            let args = tc["function"]["arguments"].as_str().unwrap_or("{}");
            let input = serde_json::from_str::<Value>(args).unwrap_or(Value::Null);
            content.push(Content::ToolUse {
                id:    tc["id"].as_str().unwrap_or("").to_string(),
                name:  tc["function"]["name"].as_str().unwrap_or("").to_string(),
                input,
            });
        }
    }
    Ok(ProviderResponse { message: Message::assistant(content), stop_reason })
}

#[async_trait]
impl StreamProvider for OpenAiProvider {
    async fn stream(&self, messages: &[Message], tools: &[ToolDefinition], system: &str) -> Result<ProviderResponse, ProviderError> {
        call_openai_compat(&self.endpoint, &self.api_key, &self.model, &[], messages, tools, system).await
    }
}

#[async_trait]
impl StreamProvider for OpenRouterProvider {
    async fn stream(&self, messages: &[Message], tools: &[ToolDefinition], system: &str) -> Result<ProviderResponse, ProviderError> {
        call_openai_compat(&self.endpoint, &self.api_key, &self.model,
            &[("HTTP-Referer", "https://github.com/baby-phi")],
            messages, tools, system).await
    }
}

// ── Tools ─────────────────────────────────────────────────────────────────────

pub struct BashTool;
impl BashTool { pub fn new() -> Self { Self } }

const BASH_DENY: &[&str] = &["rm -rf /", "rm -rf ~", ":(){ :|:& };:"];

#[async_trait]
impl AgentTool for BashTool {
    fn name(&self) -> &str { "bash" }
    fn description(&self) -> &str { "Run a shell command. Timeout 30s. stdout+stderr returned." }
    fn parameters_schema(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": { "command": { "type": "string" } },
            "required": ["command"]
        })
    }
    async fn execute(&self, input: Value) -> ToolResult {
        let cmd = match input["command"].as_str() {
            Some(c) => c.to_string(),
            None => return ToolResult { content: "missing 'command'".into(), is_error: true },
        };
        for pat in BASH_DENY {
            if cmd.contains(pat) {
                return ToolResult { content: format!("blocked: '{pat}'"), is_error: true };
            }
        }
        match tokio::time::timeout(Duration::from_secs(30),
            tokio::process::Command::new("sh").arg("-c").arg(&cmd)
                .stdout(Stdio::piped()).stderr(Stdio::piped()).output()
        ).await {
            Err(_) => ToolResult { content: "timeout after 30s".into(), is_error: true },
            Ok(Err(e)) => ToolResult { content: format!("exec error: {e}"), is_error: true },
            Ok(Ok(out)) => {
                let mut combined = String::from_utf8_lossy(&out.stdout).to_string();
                combined.push_str(&String::from_utf8_lossy(&out.stderr));
                if combined.len() > 50_000 { combined.truncate(50_000); combined.push_str("\n[truncated]"); }
                ToolResult { content: combined, is_error: !out.status.success() }
            }
        }
    }
}

pub struct ReadFileTool;
impl ReadFileTool { pub fn new() -> Self { Self } }

#[async_trait]
impl AgentTool for ReadFileTool {
    fn name(&self) -> &str { "read_file" }
    fn description(&self) -> &str { "Read a text file. Max 1MB." }
    fn parameters_schema(&self) -> Value {
        serde_json::json!({ "type": "object", "properties": { "path": { "type": "string" } }, "required": ["path"] })
    }
    async fn execute(&self, input: Value) -> ToolResult {
        let path = match input["path"].as_str() {
            Some(p) => p,
            None => return ToolResult { content: "missing 'path'".into(), is_error: true },
        };
        match tokio::fs::metadata(path).await {
            Err(e) => return ToolResult { content: format!("not found: {e}"), is_error: true },
            Ok(m) if m.len() > 1_048_576 =>
                return ToolResult { content: format!("too large ({} bytes)", m.len()), is_error: true },
            _ => {}
        }
        match tokio::fs::read(path).await {
            Err(e) => ToolResult { content: format!("read error: {e}"), is_error: true },
            Ok(bytes) => {
                if bytes.contains(&0u8) {
                    return ToolResult { content: "binary file".into(), is_error: true };
                }
                ToolResult { content: String::from_utf8_lossy(&bytes).into_owned(), is_error: false }
            }
        }
    }
}

pub struct WriteFileTool;
impl WriteFileTool { pub fn new() -> Self { Self } }

#[async_trait]
impl AgentTool for WriteFileTool {
    fn name(&self) -> &str { "write_file" }
    fn description(&self) -> &str { "Write content to a file, creating parent dirs as needed." }
    fn parameters_schema(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": { "path": { "type": "string" }, "content": { "type": "string" } },
            "required": ["path", "content"]
        })
    }
    async fn execute(&self, input: Value) -> ToolResult {
        let path = match input["path"].as_str() { Some(p) => p, None => return ToolResult { content: "missing 'path'".into(), is_error: true } };
        let content = match input["content"].as_str() { Some(c) => c, None => return ToolResult { content: "missing 'content'".into(), is_error: true } };
        if let Some(parent) = std::path::Path::new(path).parent() {
            if !parent.as_os_str().is_empty() {
                let _ = tokio::fs::create_dir_all(parent).await;
            }
        }
        match tokio::fs::write(path, content).await {
            Ok(_)  => ToolResult { content: format!("wrote {path}"), is_error: false },
            Err(e) => ToolResult { content: format!("write error: {e}"), is_error: true },
        }
    }
}

pub struct EditFileTool;
impl EditFileTool { pub fn new() -> Self { Self } }

#[async_trait]
impl AgentTool for EditFileTool {
    fn name(&self) -> &str { "edit_file" }
    fn description(&self) -> &str { "Replace exactly one occurrence of old_string with new_string in a file." }
    fn parameters_schema(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "path":       { "type": "string" },
                "old_string": { "type": "string" },
                "new_string": { "type": "string" }
            },
            "required": ["path", "old_string", "new_string"]
        })
    }
    async fn execute(&self, input: Value) -> ToolResult {
        let path   = match input["path"].as_str()       { Some(v) => v, None => return ToolResult { content: "missing 'path'".into(), is_error: true } };
        let old    = match input["old_string"].as_str() { Some(v) => v, None => return ToolResult { content: "missing 'old_string'".into(), is_error: true } };
        let new    = match input["new_string"].as_str() { Some(v) => v, None => return ToolResult { content: "missing 'new_string'".into(), is_error: true } };
        let text   = match tokio::fs::read_to_string(path).await { Ok(t) => t, Err(e) => return ToolResult { content: format!("read error: {e}"), is_error: true } };
        match text.matches(old).count() {
            0 => ToolResult { content: format!("old_string not found in {path}"), is_error: true },
            1 => match tokio::fs::write(path, text.replacen(old, new, 1)).await {
                Ok(_)  => ToolResult { content: format!("edited {path}"), is_error: false },
                Err(e) => ToolResult { content: format!("write error: {e}"), is_error: true },
            },
            n => ToolResult { content: format!("{n} matches for old_string in {path}; must be unique"), is_error: true },
        }
    }
}

pub struct ListFilesTool;
impl ListFilesTool { pub fn new() -> Self { Self } }

#[async_trait]
impl AgentTool for ListFilesTool {
    fn name(&self) -> &str { "list_files" }
    fn description(&self) -> &str { "List files recursively in a directory. Up to 200 entries." }
    fn parameters_schema(&self) -> Value {
        serde_json::json!({ "type": "object", "properties": { "path": { "type": "string" } }, "required": ["path"] })
    }
    async fn execute(&self, input: Value) -> ToolResult {
        let path = input["path"].as_str().unwrap_or(".");
        let mut entries: Vec<String> = vec![];
        collect_entries(std::path::Path::new(path), &mut entries, 0);
        let truncated = entries.len() > 200;
        entries.truncate(200);
        let mut out = entries.join("\n");
        if truncated { out.push_str("\n[truncated at 200 entries]"); }
        ToolResult { content: out, is_error: false }
    }
}

fn collect_entries(dir: &std::path::Path, out: &mut Vec<String>, depth: usize) {
    if depth > 8 || out.len() >= 210 { return; }
    let Ok(rd) = std::fs::read_dir(dir) else { return };
    let mut entries: Vec<_> = rd.flatten().collect();
    entries.sort_by_key(|e| e.path());
    for entry in entries {
        out.push(entry.path().display().to_string());
        if entry.path().is_dir() { collect_entries(&entry.path(), out, depth + 1); }
    }
}

pub struct SearchTool;
impl SearchTool { pub fn new() -> Self { Self } }

#[async_trait]
impl AgentTool for SearchTool {
    fn name(&self) -> &str { "search" }
    fn description(&self) -> &str { "Search for a regex pattern in files. Uses rg or falls back to grep." }
    fn parameters_schema(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "pattern": { "type": "string" },
                "path":    { "type": "string" }
            },
            "required": ["pattern"]
        })
    }
    async fn execute(&self, input: Value) -> ToolResult {
        let pattern = match input["pattern"].as_str() {
            Some(p) => p,
            None => return ToolResult { content: "missing 'pattern'".into(), is_error: true },
        };
        let path = input["path"].as_str().unwrap_or(".");
        let use_rg = std::process::Command::new("which").arg("rg").output()
            .map(|o| o.status.success()).unwrap_or(false);
        let (cmd, args): (&str, Vec<&str>) = if use_rg {
            ("rg", vec!["-n", "--no-heading", pattern, path])
        } else {
            ("grep", vec!["-rn", pattern, path])
        };
        match tokio::time::timeout(Duration::from_secs(15),
            tokio::process::Command::new(cmd).args(&args).output()
        ).await {
            Err(_)     => ToolResult { content: "search timed out".into(), is_error: true },
            Ok(Err(e)) => ToolResult { content: format!("search error: {e}"), is_error: true },
            Ok(Ok(out)) => {
                let raw = String::from_utf8_lossy(&out.stdout).to_string();
                let lines: Vec<&str> = raw.lines().take(100).collect();
                let mut content = lines.join("\n");
                if raw.lines().count() > 100 { content.push_str("\n[truncated at 100 matches]"); }
                ToolResult { content, is_error: false }
            }
        }
    }
}

// ── Agent Loop ────────────────────────────────────────────────────────────────

pub async fn agent_loop(
    messages: &mut Vec<Message>,
    provider: &dyn StreamProvider,
    tools: &[Box<dyn AgentTool>],
    system: &str,
    retry: &RetryConfig,
    on_event: &mut dyn FnMut(AgentEvent),
) -> Result<(), ProviderError> {
    let tool_defs: Vec<ToolDefinition> = tools.iter().map(|t| t.definition()).collect();
    let mut turn: u32 = 0;

    loop {
        on_event(AgentEvent::TurnStart { turn });
        let response = call_with_retry(messages, provider, &tool_defs, system, retry, on_event).await?;
        messages.push(response.message.clone());

        if response.stop_reason != StopReason::ToolUse {
            on_event(AgentEvent::TurnEnd);
            on_event(AgentEvent::AgentEnd);
            return Ok(());
        }

        let tool_uses: Vec<(String, String, Value)> = response.message.content.iter().filter_map(|c| {
            if let Content::ToolUse { id, name, input } = c { Some((id.clone(), name.clone(), input.clone())) }
            else { None }
        }).collect();

        let mut result_contents: Vec<Content> = vec![];
        for (id, name, input) in &tool_uses {
            on_event(AgentEvent::ToolStart { name: name.clone(), input: input.clone() });
            let result = match tools.iter().find(|t| t.name() == name) {
                Some(tool) => tool.execute(input.clone()).await,
                None => ToolResult { content: format!("unknown tool: {name}"), is_error: true },
            };
            on_event(AgentEvent::ToolEnd { name: name.clone(), output: result.content.clone(), is_error: result.is_error });
            result_contents.push(Content::ToolResult {
                tool_use_id: id.clone(), content: result.content, is_error: result.is_error,
            });
        }
        messages.push(Message { role: "user".into(), content: result_contents });
        turn += 1;
    }
}

async fn call_with_retry(
    messages: &[Message],
    provider: &dyn StreamProvider,
    tool_defs: &[ToolDefinition],
    system: &str,
    retry: &RetryConfig,
    on_event: &mut dyn FnMut(AgentEvent),
) -> Result<ProviderResponse, ProviderError> {
    for attempt in 0..retry.max_attempts {
        match provider.stream(messages, tool_defs, system).await {
            Ok(resp) => return Ok(resp),
            Err(e) if e.is_retryable() && attempt + 1 < retry.max_attempts => {
                let delay = retry.delay_for(attempt);
                on_event(AgentEvent::TextDelta(format!("[retry in {}ms]\n", delay.as_millis())));
                tokio::time::sleep(delay).await;
            }
            Err(e) => return Err(e),
        }
    }
    unreachable!()
}
