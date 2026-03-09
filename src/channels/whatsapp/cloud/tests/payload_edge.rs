use super::support::{make_channel, wildcard_channel};
use serde_json::json;

#[test]
fn whatsapp_parse_missing_entry_array() {
    assert!(make_channel()
        .parse_webhook_payload(&json!({ "object": "whatsapp_business_account" }))
        .is_empty());
}

#[test]
fn whatsapp_parse_entry_not_array() {
    assert!(make_channel()
        .parse_webhook_payload(&json!({ "entry": "not_an_array" }))
        .is_empty());
}

#[test]
fn whatsapp_parse_missing_changes_array() {
    assert!(make_channel()
        .parse_webhook_payload(&json!({ "entry": [{ "id": "123" }] }))
        .is_empty());
}

#[test]
fn whatsapp_parse_changes_not_array() {
    assert!(make_channel()
        .parse_webhook_payload(&json!({ "entry": [{ "changes": "not_an_array" }] }))
        .is_empty());
}

#[test]
fn whatsapp_parse_missing_value() {
    assert!(make_channel()
        .parse_webhook_payload(&json!({ "entry": [{ "changes": [{ "field": "messages" }] }] }))
        .is_empty());
}

#[test]
fn whatsapp_parse_missing_messages_array() {
    assert!(make_channel()
        .parse_webhook_payload(
            &json!({ "entry": [{ "changes": [{ "value": { "metadata": {} } }] }] })
        )
        .is_empty());
}

#[test]
fn whatsapp_parse_messages_not_array() {
    assert!(make_channel()
        .parse_webhook_payload(
            &json!({ "entry": [{ "changes": [{ "value": { "messages": "not_an_array" } }] }] })
        )
        .is_empty());
}

#[test]
fn whatsapp_parse_missing_from_field() {
    let payload = json!({
        "entry": [{
            "changes": [{
                "value": {
                    "messages": [{
                        "timestamp": "1",
                        "type": "text",
                        "text": { "body": "No sender" }
                    }]
                }
            }]
        }]
    });
    assert!(wildcard_channel()
        .parse_webhook_payload(&payload)
        .is_empty());
}

#[test]
fn whatsapp_parse_missing_text_body() {
    let payload = json!({
        "entry": [{
            "changes": [{
                "value": {
                    "messages": [{
                        "from": "111",
                        "timestamp": "1",
                        "type": "text",
                        "text": {}
                    }]
                }
            }]
        }]
    });
    assert!(wildcard_channel()
        .parse_webhook_payload(&payload)
        .is_empty());
}

#[test]
fn whatsapp_parse_null_text_body() {
    let payload = json!({
        "entry": [{
            "changes": [{
                "value": {
                    "messages": [{
                        "from": "111",
                        "timestamp": "1",
                        "type": "text",
                        "text": { "body": null }
                    }]
                }
            }]
        }]
    });
    assert!(wildcard_channel()
        .parse_webhook_payload(&payload)
        .is_empty());
}

#[test]
fn whatsapp_parse_invalid_timestamp_uses_current() {
    let payload = json!({
        "entry": [{
            "changes": [{
                "value": {
                    "messages": [{
                        "from": "111",
                        "timestamp": "not_a_number",
                        "type": "text",
                        "text": { "body": "Hello" }
                    }]
                }
            }]
        }]
    });
    let msgs = wildcard_channel().parse_webhook_payload(&payload);
    assert_eq!(msgs.len(), 1);
    assert!(msgs[0].timestamp > 0);
}

#[test]
fn whatsapp_parse_missing_timestamp_uses_current() {
    let payload = json!({
        "entry": [{
            "changes": [{
                "value": {
                    "messages": [{
                        "from": "111",
                        "type": "text",
                        "text": { "body": "Hello" }
                    }]
                }
            }]
        }]
    });
    let msgs = wildcard_channel().parse_webhook_payload(&payload);
    assert_eq!(msgs.len(), 1);
    assert!(msgs[0].timestamp > 0);
}

#[test]
fn whatsapp_parse_status_update_ignored() {
    let payload = json!({
        "entry": [{
            "changes": [{
                "value": {
                    "statuses": [{
                        "id": "wamid.xxx",
                        "status": "delivered",
                        "timestamp": "1699999999"
                    }]
                }
            }]
        }]
    });
    assert!(make_channel().parse_webhook_payload(&payload).is_empty());
}

#[test]
fn whatsapp_parse_empty_messages_array() {
    assert!(make_channel()
        .parse_webhook_payload(
            &json!({ "entry": [{ "changes": [{ "value": { "messages": [] } }] }] })
        )
        .is_empty());
}

#[test]
fn whatsapp_parse_empty_entry_array() {
    assert!(make_channel()
        .parse_webhook_payload(&json!({ "entry": [] }))
        .is_empty());
}

#[test]
fn whatsapp_parse_empty_changes_array() {
    assert!(make_channel()
        .parse_webhook_payload(&json!({ "entry": [{ "changes": [] }] }))
        .is_empty());
}
