use super::support::wildcard_channel;
use serde_json::json;

#[test]
fn whatsapp_parse_audio_message_skipped() {
    let payload = json!({
        "entry": [{ "changes": [{ "value": { "messages": [{
            "from": "111", "timestamp": "1", "type": "audio",
            "audio": { "id": "audio123", "mime_type": "audio/ogg" }
        }] } }] }]
    });
    assert!(wildcard_channel()
        .parse_webhook_payload(&payload)
        .is_empty());
}

#[test]
fn whatsapp_parse_video_message_skipped() {
    let payload = json!({
        "entry": [{ "changes": [{ "value": { "messages": [{
            "from": "111", "timestamp": "1", "type": "video",
            "video": { "id": "video123" }
        }] } }] }]
    });
    assert!(wildcard_channel()
        .parse_webhook_payload(&payload)
        .is_empty());
}

#[test]
fn whatsapp_parse_document_message_skipped() {
    let payload = json!({
        "entry": [{ "changes": [{ "value": { "messages": [{
            "from": "111", "timestamp": "1", "type": "document",
            "document": { "id": "doc123", "filename": "file.pdf" }
        }] } }] }]
    });
    assert!(wildcard_channel()
        .parse_webhook_payload(&payload)
        .is_empty());
}

#[test]
fn whatsapp_parse_sticker_message_skipped() {
    let payload = json!({
        "entry": [{ "changes": [{ "value": { "messages": [{
            "from": "111", "timestamp": "1", "type": "sticker",
            "sticker": { "id": "sticker123" }
        }] } }] }]
    });
    assert!(wildcard_channel()
        .parse_webhook_payload(&payload)
        .is_empty());
}

#[test]
fn whatsapp_parse_location_message_skipped() {
    let payload = json!({
        "entry": [{ "changes": [{ "value": { "messages": [{
            "from": "111", "timestamp": "1", "type": "location",
            "location": { "latitude": 40.7128, "longitude": -74.0060 }
        }] } }] }]
    });
    assert!(wildcard_channel()
        .parse_webhook_payload(&payload)
        .is_empty());
}

#[test]
fn whatsapp_parse_contacts_message_skipped() {
    let payload = json!({
        "entry": [{ "changes": [{ "value": { "messages": [{
            "from": "111", "timestamp": "1", "type": "contacts",
            "contacts": [{ "name": { "formatted_name": "John" } }]
        }] } }] }]
    });
    assert!(wildcard_channel()
        .parse_webhook_payload(&payload)
        .is_empty());
}

#[test]
fn whatsapp_parse_reaction_message_skipped() {
    let payload = json!({
        "entry": [{ "changes": [{ "value": { "messages": [{
            "from": "111", "timestamp": "1", "type": "reaction",
            "reaction": { "message_id": "wamid.xxx", "emoji": "👍" }
        }] } }] }]
    });
    assert!(wildcard_channel()
        .parse_webhook_payload(&payload)
        .is_empty());
}
