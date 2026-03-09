use super::support::{make_channel, wildcard_channel};
use serde_json::json;

#[test]
fn whatsapp_parse_empty_payload() {
    let msgs = make_channel().parse_webhook_payload(&json!({}));
    assert!(msgs.is_empty());
}

#[test]
fn whatsapp_parse_valid_text_message() {
    let payload = json!({
        "object": "whatsapp_business_account",
        "entry": [{
            "id": "123",
            "changes": [{
                "value": {
                    "messaging_product": "whatsapp",
                    "metadata": {
                        "display_phone_number": "15551234567",
                        "phone_number_id": "123456789"
                    },
                    "messages": [{
                        "from": "1234567890",
                        "id": "wamid.xxx",
                        "timestamp": "1699999999",
                        "type": "text",
                        "text": { "body": "Hello ZeroClaw!" }
                    }]
                },
                "field": "messages"
            }]
        }]
    });

    let msgs = make_channel().parse_webhook_payload(&payload);
    assert_eq!(msgs.len(), 1);
    assert_eq!(msgs[0].sender, "+1234567890");
    assert_eq!(msgs[0].reply_target, "+1234567890");
    assert_eq!(msgs[0].content, "Hello ZeroClaw!");
    assert_eq!(msgs[0].channel, "whatsapp");
    assert_eq!(msgs[0].timestamp, 1_699_999_999);
}

#[test]
fn whatsapp_parse_unauthorized_number() {
    let payload = json!({
        "entry": [{
            "changes": [{
                "value": {
                    "messages": [{
                        "from": "9999999999",
                        "timestamp": "1699999999",
                        "type": "text",
                        "text": { "body": "Spam" }
                    }]
                }
            }]
        }]
    });
    assert!(make_channel().parse_webhook_payload(&payload).is_empty());
}

#[test]
fn whatsapp_parse_non_text_message_skipped() {
    let payload = json!({
        "entry": [{
            "changes": [{
                "value": {
                    "messages": [{
                        "from": "1234567890",
                        "timestamp": "1699999999",
                        "type": "image",
                        "image": { "id": "img123" }
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
fn whatsapp_parse_multiple_messages() {
    let payload = json!({
        "entry": [{
            "changes": [{
                "value": {
                    "messages": [
                        { "from": "111", "timestamp": "1", "type": "text", "text": { "body": "First" } },
                        { "from": "222", "timestamp": "2", "type": "text", "text": { "body": "Second" } }
                    ]
                }
            }]
        }]
    });
    let msgs = wildcard_channel().parse_webhook_payload(&payload);
    assert_eq!(msgs.len(), 2);
    assert_eq!(msgs[0].content, "First");
    assert_eq!(msgs[1].content, "Second");
}

#[test]
fn whatsapp_parse_normalizes_phone_with_plus() {
    let channel = super::super::WhatsAppChannel::new(
        "tok".into(),
        "123".into(),
        "ver".into(),
        vec!["+1234567890".into()],
    );
    let payload = json!({
        "entry": [{
            "changes": [{
                "value": {
                    "messages": [{
                        "from": "1234567890",
                        "timestamp": "1",
                        "type": "text",
                        "text": { "body": "Hi" }
                    }]
                }
            }]
        }]
    });
    let msgs = channel.parse_webhook_payload(&payload);
    assert_eq!(msgs.len(), 1);
    assert_eq!(msgs[0].sender, "+1234567890");
}

#[test]
fn whatsapp_empty_text_skipped() {
    let payload = json!({
        "entry": [{
            "changes": [{
                "value": {
                    "messages": [{
                        "from": "111",
                        "timestamp": "1",
                        "type": "text",
                        "text": { "body": "" }
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
fn whatsapp_parse_multiple_entries() {
    let payload = json!({
        "entry": [
            { "changes": [{ "value": { "messages": [{ "from": "111", "timestamp": "1", "type": "text", "text": { "body": "Entry 1" } }] } }] },
            { "changes": [{ "value": { "messages": [{ "from": "222", "timestamp": "2", "type": "text", "text": { "body": "Entry 2" } }] } }] }
        ]
    });
    let msgs = wildcard_channel().parse_webhook_payload(&payload);
    assert_eq!(msgs.len(), 2);
    assert_eq!(msgs[0].content, "Entry 1");
    assert_eq!(msgs[1].content, "Entry 2");
}

#[test]
fn whatsapp_parse_multiple_changes() {
    let payload = json!({
        "entry": [{
            "changes": [
                { "value": { "messages": [{ "from": "111", "timestamp": "1", "type": "text", "text": { "body": "Change 1" } }] } },
                { "value": { "messages": [{ "from": "222", "timestamp": "2", "type": "text", "text": { "body": "Change 2" } }] } }
            ]
        }]
    });
    let msgs = wildcard_channel().parse_webhook_payload(&payload);
    assert_eq!(msgs.len(), 2);
    assert_eq!(msgs[0].content, "Change 1");
    assert_eq!(msgs[1].content, "Change 2");
}

#[test]
fn whatsapp_parse_mixed_authorized_unauthorized() {
    let channel = super::super::WhatsAppChannel::new(
        "tok".into(),
        "123".into(),
        "ver".into(),
        vec!["+1111111111".into()],
    );
    let payload = json!({
        "entry": [{
            "changes": [{
                "value": {
                    "messages": [
                        { "from": "1111111111", "timestamp": "1", "type": "text", "text": { "body": "Allowed" } },
                        { "from": "9999999999", "timestamp": "2", "type": "text", "text": { "body": "Blocked" } },
                        { "from": "1111111111", "timestamp": "3", "type": "text", "text": { "body": "Also allowed" } }
                    ]
                }
            }]
        }]
    });
    let msgs = channel.parse_webhook_payload(&payload);
    assert_eq!(msgs.len(), 2);
    assert_eq!(msgs[0].content, "Allowed");
    assert_eq!(msgs[1].content, "Also allowed");
}
