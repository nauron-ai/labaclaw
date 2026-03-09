use super::attachments::{parse_wa_attachment_markers, WaAttachmentKind};
use super::WhatsAppWebChannel;
use crate::channels::traits::Channel;

fn make_channel() -> WhatsAppWebChannel {
    WhatsAppWebChannel::new(
        "/tmp/test-whatsapp.db".into(),
        None,
        None,
        vec!["+1234567890".into()],
    )
}

#[test]
fn whatsapp_web_channel_name() {
    assert_eq!(make_channel().name(), "whatsapp");
}

#[test]
fn whatsapp_web_render_pairing_qr_rejects_empty_payload() {
    let err = WhatsAppWebChannel::render_pairing_qr("   ").expect_err("empty payload");
    assert!(err.to_string().contains("empty"));
}

#[test]
fn whatsapp_web_render_pairing_qr_outputs_multiline_text() {
    let rendered = WhatsAppWebChannel::render_pairing_qr("https://example.com/whatsapp-pairing")
        .expect("rendered QR");
    assert!(rendered.lines().count() > 10);
    assert!(rendered.trim().len() > 64);
}

#[tokio::test]
async fn whatsapp_web_health_check_disconnected() {
    assert!(!make_channel().health_check().await);
}

#[test]
fn parse_wa_markers_image() {
    let (text, attachments) =
        parse_wa_attachment_markers("Here is the timeline [IMAGE:/tmp/chart.png]");
    assert_eq!(text, "Here is the timeline");
    assert_eq!(attachments.len(), 1);
    assert_eq!(attachments[0].target, "/tmp/chart.png");
    assert!(matches!(attachments[0].kind, WaAttachmentKind::Image));
}

#[test]
fn parse_wa_markers_multiple() {
    let (text, attachments) =
        parse_wa_attachment_markers("Text [IMAGE:/a.png] more [DOCUMENT:/b.pdf]");
    assert_eq!(text, "Text  more");
    assert_eq!(attachments.len(), 2);
    assert!(matches!(attachments[0].kind, WaAttachmentKind::Image));
    assert!(matches!(attachments[1].kind, WaAttachmentKind::Document));
}

#[test]
fn parse_wa_markers_no_markers() {
    let (text, attachments) = parse_wa_attachment_markers("Just regular text");
    assert_eq!(text, "Just regular text");
    assert!(attachments.is_empty());
}

#[test]
fn parse_wa_markers_unknown_kind_preserved() {
    let (text, attachments) = parse_wa_attachment_markers("Check [UNKNOWN:/foo] out");
    assert_eq!(text, "Check [UNKNOWN:/foo] out");
    assert!(attachments.is_empty());
}

#[test]
fn with_transcription_sets_config_when_enabled() {
    let mut config = crate::config::TranscriptionConfig::default();
    config.enabled = true;
    assert!(make_channel()
        .with_transcription(config)
        .transcription
        .is_some());
}

#[test]
fn with_transcription_skips_when_disabled() {
    assert!(make_channel()
        .with_transcription(crate::config::TranscriptionConfig::default())
        .transcription
        .is_none());
}

#[test]
fn audio_mime_to_filename_maps_whatsapp_voice_note() {
    assert_eq!(
        WhatsAppWebChannel::audio_mime_to_filename("audio/ogg; codecs=opus"),
        "voice.ogg"
    );
    assert_eq!(
        WhatsAppWebChannel::audio_mime_to_filename("audio/ogg"),
        "voice.ogg"
    );
    assert_eq!(
        WhatsAppWebChannel::audio_mime_to_filename("audio/opus"),
        "voice.opus"
    );
    assert_eq!(
        WhatsAppWebChannel::audio_mime_to_filename("audio/mp4"),
        "voice.m4a"
    );
    assert_eq!(
        WhatsAppWebChannel::audio_mime_to_filename("audio/mpeg"),
        "voice.mp3"
    );
    assert_eq!(
        WhatsAppWebChannel::audio_mime_to_filename("audio/wav"),
        "voice.wav"
    );
    assert_eq!(
        WhatsAppWebChannel::audio_mime_to_filename("audio/webm"),
        "voice.webm"
    );
    assert_eq!(
        WhatsAppWebChannel::audio_mime_to_filename("audio/webm; codecs=opus"),
        "voice.webm"
    );
    assert_eq!(
        WhatsAppWebChannel::audio_mime_to_filename("audio/x-wav"),
        "voice.wav"
    );
    assert_eq!(
        WhatsAppWebChannel::audio_mime_to_filename("application/octet-stream"),
        "voice.ogg"
    );
}
