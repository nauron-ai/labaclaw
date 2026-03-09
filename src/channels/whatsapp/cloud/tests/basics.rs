use super::super::WhatsAppChannel;
use super::support::make_channel;
use crate::channels::traits::Channel;

#[test]
fn whatsapp_channel_name() {
    let channel = make_channel();
    assert_eq!(channel.name(), "whatsapp");
}

#[test]
fn whatsapp_verify_token() {
    let channel = make_channel();
    assert_eq!(channel.verify_token(), "verify-me");
}

#[test]
fn whatsapp_number_allowed_exact() {
    let channel = make_channel();
    assert!(channel.allowlisted_identity("+1234567890").is_some());
    assert!(channel.allowlisted_identity("+9876543210").is_none());
}

#[test]
fn whatsapp_number_allowed_wildcard() {
    let channel = WhatsAppChannel::new("tok".into(), "123".into(), "ver".into(), vec!["*".into()]);
    assert!(channel.allowlisted_identity("+1234567890").is_some());
    assert!(channel.allowlisted_identity("+9999999999").is_some());
}

#[test]
fn whatsapp_number_denied_empty() {
    let channel = WhatsAppChannel::new("tok".into(), "123".into(), "ver".into(), vec![]);
    assert!(channel.allowlisted_identity("+1234567890").is_none());
}

#[test]
fn whatsapp_number_allowed_multiple_numbers() {
    let channel = WhatsAppChannel::new(
        "tok".into(),
        "123".into(),
        "ver".into(),
        vec![
            "+1111111111".into(),
            "+2222222222".into(),
            "+3333333333".into(),
        ],
    );
    assert!(channel.allowlisted_identity("+1111111111").is_some());
    assert!(channel.allowlisted_identity("+2222222222").is_some());
    assert!(channel.allowlisted_identity("+3333333333").is_some());
    assert!(channel.allowlisted_identity("+4444444444").is_none());
}

#[test]
fn whatsapp_number_allowed_case_sensitive() {
    let channel = WhatsAppChannel::new(
        "tok".into(),
        "123".into(),
        "ver".into(),
        vec!["+1234567890".into()],
    );
    assert!(channel.allowlisted_identity("+1234567890").is_some());
    assert!(channel.allowlisted_identity("+1234567891").is_none());
}

#[test]
fn whatsapp_channel_fields_stored_correctly() {
    let channel = WhatsAppChannel::new(
        "my-access-token".into(),
        "phone-id-123".into(),
        "my-verify-token".into(),
        vec!["+111".into(), "+222".into()],
    );
    assert_eq!(channel.verify_token(), "my-verify-token");
    assert!(channel.allowlisted_identity("+111").is_some());
    assert!(channel.allowlisted_identity("+222").is_some());
    assert!(channel.allowlisted_identity("+333").is_none());
}
