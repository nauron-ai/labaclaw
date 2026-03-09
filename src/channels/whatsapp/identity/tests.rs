use super::model::WhatsAppIdentityStatus;
use super::{WhatsAppAllowlistEntry, WhatsAppIdentity};

#[test]
fn display_name_prefers_verified_name() {
    let identity = WhatsAppIdentity {
        verified_name: Some("ZeroClawAgent".into()),
        full_name: Some("zeroclaw_user".into()),
        push_name: Some("zeroclaw_node".into()),
        ..WhatsAppIdentity::default()
    };

    assert_eq!(identity.display_name(), Some("ZeroClawAgent"));
    assert_eq!(identity.status(), WhatsAppIdentityStatus::Recognized);
}

#[test]
fn identity_matches_allowlist_by_phone() {
    let identity = WhatsAppIdentity {
        lid: Some("83962416386094".into()),
        phone_number: Some("+48792717677".into()),
        ..WhatsAppIdentity::default()
    };

    assert!(identity.matches_allowlist(&[WhatsAppAllowlistEntry::from("+48792717677",)]));
}

#[test]
fn identity_falls_back_to_last_seen_jid_for_allowlist() {
    let identity = WhatsAppIdentity {
        lid: Some("83962416386094".into()),
        last_seen_jid: Some("83962416386094@lid".into()),
        ..WhatsAppIdentity::default()
    };

    assert!(identity.matches_allowlist(&[WhatsAppAllowlistEntry::from("+83962416386094",)]));
}
