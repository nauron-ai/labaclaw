use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum CodexTransport {
    Auto,
    WebSocket,
    Sse,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct ParseCodexTransportError;
impl std::fmt::Display for ParseCodexTransportError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "expected one of: auto, websocket|ws, sse|http")
    }
}
impl std::error::Error for ParseCodexTransportError {}
impl std::str::FromStr for CodexTransport {
    type Err = ParseCodexTransportError;
    fn from_str(raw: &str) -> Result<Self, Self::Err> {
        let normalized = raw.trim().to_ascii_lowercase().replace(['-', '_'], "");
        match normalized.as_str() {
            "auto" => Ok(Self::Auto),
            "websocket" | "ws" => Ok(Self::WebSocket),
            "sse" | "http" => Ok(Self::Sse),
            _ => Err(ParseCodexTransportError),
        }
    }
}

#[derive(Debug, Serialize)]
pub(super) struct ResponsesRequest {
    model: String,
    input: Vec<ResponsesInput>,
    instructions: String,
    store: bool,
    stream: bool,
    text: ResponsesTextOptions,
    reasoning: ResponsesReasoningOptions,
    include: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tools: Option<Vec<Value>>,
    tool_choice: ResponsesToolChoice,
    parallel_tool_calls: bool,
}

impl ResponsesRequest {
    pub(super) fn new(
        model: impl Into<String>,
        input: Vec<ResponsesInput>,
        instructions: impl Into<String>,
        reasoning_effort: ReasoningEffort,
        tools: Option<Vec<Value>>,
    ) -> Self {
        let model = model.into();
        let reasoning_effort = effective_reasoning_effort(&model, reasoning_effort);
        Self {
            model,
            input,
            instructions: instructions.into(),
            store: false,
            stream: true,
            text: ResponsesTextOptions::medium(),
            reasoning: ResponsesReasoningOptions::new(reasoning_effort),
            include: vec!["reasoning.encrypted_content".to_string()],
            tools: tools.filter(|tool_list| !tool_list.is_empty()),
            tool_choice: ResponsesToolChoice::Auto,
            parallel_tool_calls: true,
        }
    }
}

#[derive(Debug, Serialize, Clone, Copy, PartialEq, Eq)]
pub(super) enum ResponsesClientEventKind {
    #[serde(rename = "response.create")]
    ResponseCreate,
}

#[derive(Debug, Serialize)]
pub(super) struct ResponsesCreateEvent<'a> {
    #[serde(rename = "type")]
    kind: ResponsesClientEventKind,
    #[serde(flatten)]
    request: &'a ResponsesRequest,
}

impl<'a> ResponsesCreateEvent<'a> {
    pub(super) fn new(request: &'a ResponsesRequest) -> Self {
        Self {
            kind: ResponsesClientEventKind::ResponseCreate,
            request,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub(super) struct ResponsesInput {
    role: ResponsesRole,
    content: Vec<ResponsesInputContent>,
}

impl ResponsesInput {
    pub(super) fn new(role: ResponsesRole, content: Vec<ResponsesInputContent>) -> Self {
        Self { role, content }
    }

    pub(super) fn role(&self) -> ResponsesRole {
        self.role
    }
    pub(super) fn content(&self) -> &[ResponsesInputContent] {
        &self.content
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub(super) struct ResponsesInputContent {
    #[serde(rename = "type")]
    kind: ResponsesInputContentKind,
    #[serde(skip_serializing_if = "Option::is_none")]
    text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    image_url: Option<String>,
}

impl ResponsesInputContent {
    pub(super) fn input_text(text: impl Into<String>) -> Self {
        Self::new(
            ResponsesInputContentKind::InputText,
            Some(text.into()),
            None,
        )
    }

    pub(super) fn output_text(text: impl Into<String>) -> Self {
        Self::new(
            ResponsesInputContentKind::OutputText,
            Some(text.into()),
            None,
        )
    }

    pub(super) fn input_image(image_url: impl Into<String>) -> Self {
        Self::new(
            ResponsesInputContentKind::InputImage,
            None,
            Some(image_url.into()),
        )
    }

    pub(super) fn kind(&self) -> ResponsesInputContentKind {
        self.kind
    }

    fn new(
        kind: ResponsesInputContentKind,
        text: Option<String>,
        image_url: Option<String>,
    ) -> Self {
        Self {
            kind,
            text,
            image_url,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(super) enum ResponsesInputContentKind {
    InputText,
    InputImage,
    OutputText,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub(super) enum ResponsesRole {
    User,
    Assistant,
}

#[derive(Debug, Serialize)]
struct ResponsesTextOptions {
    verbosity: ResponsesVerbosity,
}
impl ResponsesTextOptions {
    fn medium() -> Self {
        Self {
            verbosity: ResponsesVerbosity::Medium,
        }
    }
}

#[derive(Debug, Serialize)]
struct ResponsesReasoningOptions {
    effort: ReasoningEffort,
    summary: ResponsesReasoningSummary,
}
impl ResponsesReasoningOptions {
    fn new(effort: ReasoningEffort) -> Self {
        Self {
            effort,
            summary: ResponsesReasoningSummary::Auto,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub(super) enum ReasoningEffort {
    Minimal,
    Low,
    Medium,
    High,
    Xhigh,
}

#[derive(Debug, Clone, Copy)]
struct ReasoningPolicy {
    supports_minimal: bool,
    supports_xhigh: bool,
}

impl ReasoningPolicy {
    const fn standard() -> Self {
        Self {
            supports_minimal: true,
            supports_xhigh: true,
        }
    }

    const fn high_capped() -> Self {
        Self {
            supports_minimal: false,
            supports_xhigh: false,
        }
    }

    fn normalize(self, requested: ReasoningEffort) -> ReasoningEffort {
        match requested {
            ReasoningEffort::Minimal if !self.supports_minimal => ReasoningEffort::High,
            ReasoningEffort::Xhigh if !self.supports_xhigh => ReasoningEffort::High,
            _ => requested,
        }
    }
}

pub(super) fn effective_reasoning_effort(
    model: &str,
    requested: ReasoningEffort,
) -> ReasoningEffort {
    reasoning_policy(model).normalize(requested)
}

fn reasoning_policy(model: &str) -> ReasoningPolicy {
    let model_id = model.rsplit('/').next().unwrap_or(model);
    if matches!(model_id, "gpt-5-codex" | "gpt-5.1" | "gpt-5.1-codex-mini")
        || model_id.starts_with("codex-1p-")
    {
        return ReasoningPolicy::high_capped();
    }

    ReasoningPolicy::standard()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct ParseReasoningEffortError;
impl std::fmt::Display for ParseReasoningEffortError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "expected one of: minimal, low, medium, high, xhigh")
    }
}
impl std::error::Error for ParseReasoningEffortError {}
impl std::str::FromStr for ReasoningEffort {
    type Err = ParseReasoningEffortError;
    fn from_str(raw: &str) -> Result<Self, Self::Err> {
        match raw.trim().to_ascii_lowercase().as_str() {
            "minimal" => Ok(Self::Minimal),
            "low" => Ok(Self::Low),
            "medium" => Ok(Self::Medium),
            "high" => Ok(Self::High),
            "xhigh" => Ok(Self::Xhigh),
            _ => Err(ParseReasoningEffortError),
        }
    }
}

#[derive(Debug, Serialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
enum ResponsesVerbosity {
    Medium,
}
#[derive(Debug, Serialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
enum ResponsesReasoningSummary {
    Auto,
}
#[derive(Debug, Serialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
enum ResponsesToolChoice {
    Auto,
}
