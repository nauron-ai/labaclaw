use crate::providers::ToolCall as ProviderToolCall;
use serde_json::Value;

#[derive(Debug, Clone)]
pub(crate) struct AssistantToolCallsPayload {
    pub content: Option<String>,
    pub reasoning_content: Option<String>,
    pub tool_calls: Vec<ProviderToolCall>,
}

#[derive(Debug, Clone)]
pub(crate) struct ToolResultPayload {
    pub tool_call_id: Option<String>,
    pub content: Option<String>,
}

pub(crate) fn parse_assistant_tool_calls_payload(raw: &str) -> Option<AssistantToolCallsPayload> {
    let parsed = serde_json::from_str::<Value>(raw).ok()?;
    let tool_calls_value = parsed.get("tool_calls")?;
    let tool_calls = serde_json::from_value::<Vec<ProviderToolCall>>(tool_calls_value.clone()).ok()?;

    Some(AssistantToolCallsPayload {
        content: parsed
            .get("content")
            .and_then(Value::as_str)
            .map(ToString::to_string),
        reasoning_content: parsed
            .get("reasoning_content")
            .and_then(Value::as_str)
            .map(ToString::to_string),
        tool_calls,
    })
}

pub(crate) fn parse_tool_result_payload(raw: &str) -> Option<ToolResultPayload> {
    let parsed = serde_json::from_str::<Value>(raw).ok()?;
    let tool_call_id = parsed
        .get("tool_call_id")
        .and_then(Value::as_str)
        .or_else(|| parsed.get("toolUseId").and_then(Value::as_str))
        .or_else(|| parsed.get("tool_use_id").and_then(Value::as_str))
        .map(ToString::to_string);

    let content = match parsed.get("content") {
        Some(Value::String(text)) => Some(text.clone()),
        Some(other) => Some(other.to_string()),
        None => None,
    };

    Some(ToolResultPayload {
        tool_call_id,
        content,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_assistant_payload_with_tool_calls() {
        let payload = parse_assistant_tool_calls_payload(
            r#"{"content":"checking","reasoning_content":"r","tool_calls":[{"id":"t1","name":"shell","arguments":"{}"}]}"#,
        )
        .expect("payload should parse");

        assert_eq!(payload.content.as_deref(), Some("checking"));
        assert_eq!(payload.reasoning_content.as_deref(), Some("r"));
        assert_eq!(payload.tool_calls.len(), 1);
        assert_eq!(payload.tool_calls[0].id, "t1");
    }

    #[test]
    fn parses_tool_result_with_alias_id_field() {
        let payload = parse_tool_result_payload(r#"{"toolUseId":"abc","content":"done"}"#)
            .expect("payload should parse");

        assert_eq!(payload.tool_call_id.as_deref(), Some("abc"));
        assert_eq!(payload.content.as_deref(), Some("done"));
    }

    #[test]
    fn module_stays_under_250_loc_budget() {
        let loc = include_str!("shared.rs").lines().count();
        assert!(
            loc <= 250,
            "openai/shared.rs exceeded 250 LOC budget: {loc}"
        );
    }
}
