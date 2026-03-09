use super::support::wildcard_channel;
use serde_json::json;

#[test]
fn whatsapp_parse_unicode_message() {
    let payload = json!({
        "entry": [{
            "changes": [{
                "value": {
                    "messages": [{
                        "from": "111",
                        "timestamp": "1",
                        "type": "text",
                        "text": { "body": "Hello 👋 世界 🌍 مرحبا" }
                    }]
                }
            }]
        }]
    });
    let msgs = wildcard_channel().parse_webhook_payload(&payload);
    assert_eq!(msgs.len(), 1);
    assert_eq!(msgs[0].content, "Hello 👋 世界 🌍 مرحبا");
}

#[test]
fn whatsapp_parse_very_long_message() {
    let payload = json!({
        "entry": [{
            "changes": [{
                "value": {
                    "messages": [{
                        "from": "111",
                        "timestamp": "1",
                        "type": "text",
                        "text": { "body": "A".repeat(10_000) }
                    }]
                }
            }]
        }]
    });
    let msgs = wildcard_channel().parse_webhook_payload(&payload);
    assert_eq!(msgs.len(), 1);
    assert_eq!(msgs[0].content.len(), 10_000);
}

#[test]
fn whatsapp_parse_whitespace_only_message_preserved() {
    let payload = json!({
        "entry": [{
            "changes": [{
                "value": {
                    "messages": [{
                        "from": "111",
                        "timestamp": "1",
                        "type": "text",
                        "text": { "body": "   " }
                    }]
                }
            }]
        }]
    });
    let msgs = wildcard_channel().parse_webhook_payload(&payload);
    assert_eq!(msgs.len(), 1);
    assert_eq!(msgs[0].content, "   ");
}

#[test]
fn whatsapp_parse_phone_already_has_plus() {
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
                        "from": "+1234567890",
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
fn whatsapp_parse_newlines_preserved() {
    let payload = json!({
        "entry": [{
            "changes": [{
                "value": {
                    "messages": [{
                        "from": "111",
                        "timestamp": "1",
                        "type": "text",
                        "text": { "body": "Line 1\nLine 2\nLine 3" }
                    }]
                }
            }]
        }]
    });
    let msgs = wildcard_channel().parse_webhook_payload(&payload);
    assert_eq!(msgs.len(), 1);
    assert_eq!(msgs[0].content, "Line 1\nLine 2\nLine 3");
}

#[test]
fn whatsapp_parse_special_characters() {
    let payload = json!({
        "entry": [{
            "changes": [{
                "value": {
                    "messages": [{
                        "from": "111",
                        "timestamp": "1",
                        "type": "text",
                        "text": { "body": "<script>alert('xss')</script> & \"quotes\" 'apostrophe'" }
                    }]
                }
            }]
        }]
    });
    let msgs = wildcard_channel().parse_webhook_payload(&payload);
    assert_eq!(msgs.len(), 1);
    assert_eq!(
        msgs[0].content,
        "<script>alert('xss')</script> & \"quotes\" 'apostrophe'"
    );
}
