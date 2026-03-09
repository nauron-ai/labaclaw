use super::super::WhatsAppChannel;

pub(super) fn make_channel() -> WhatsAppChannel {
    WhatsAppChannel::new(
        "test-token".into(),
        "123456789".into(),
        "verify-me".into(),
        vec!["+1234567890".into()],
    )
}

pub(super) fn wildcard_channel() -> WhatsAppChannel {
    WhatsAppChannel::new("tok".into(), "123".into(), "ver".into(), vec!["*".into()])
}
