use super::super::WhatsAppIdentity;
use super::*;
use tempfile::NamedTempFile;
use wa_rs_binary::jid::Jid;
use wa_rs_core::store::traits::{LidPnMappingEntry, ProtocolStore};
use wa_rs_core::types::message::{AddressingMode, MessageInfo, MessageSource};

fn make_store() -> IdentityStore {
    let tmp = NamedTempFile::new().unwrap();
    let backend = Arc::new(RusqliteStore::new(tmp.path()).unwrap());
    IdentityStore::from(backend)
}

fn make_message_info(sender: Jid, chat: Jid, push_name: &str) -> MessageInfo {
    MessageInfo {
        source: MessageSource {
            chat,
            sender,
            is_from_me: false,
            is_group: false,
            addressing_mode: Some(AddressingMode::Lid),
            sender_alt: None,
            recipient_alt: None,
            broadcast_list_owner: None,
            recipient: None,
        },
        id: "msg-1".into(),
        server_id: 1,
        r#type: "text".into(),
        push_name: push_name.into(),
        timestamp: chrono::Utc::now(),
        category: "chat".into(),
        multicast: false,
        media_type: String::new(),
        edit: Default::default(),
        bot_info: None,
        meta_info: Default::default(),
        verified_name: None,
        device_sent_meta: None,
    }
}

#[tokio::test]
async fn resolve_message_sender_uses_lid_phone_mapping() {
    let store = make_store();
    let entry = LidPnMappingEntry {
        lid: "83962416386094".into(),
        phone_number: "48792717677".into(),
        created_at: 1_700_000_000,
        updated_at: 1_700_000_100,
        learning_source: "peer_lid_message".into(),
    };
    ProtocolStore::put_lid_mapping(store.backend.as_ref(), &entry)
        .await
        .unwrap();

    let identity = store
        .resolve_message_sender(&make_message_info(
            Jid::lid("83962416386094"),
            Jid::lid("83962416386094"),
            "Przemek 2",
        ))
        .unwrap();

    assert_eq!(identity.phone_number(), Some("+48792717677"));
    assert_eq!(identity.lid(), Some("83962416386094"));
}

#[tokio::test]
async fn persist_identity_merges_rows_created_from_lid_and_phone() {
    let store = make_store();
    store
        .persist_identity(WhatsAppIdentity {
            lid: Some("83962416386094".into()),
            push_name: Some("Przemek 2".into()),
            updated_at: 1_700_000_000,
            ..WhatsAppIdentity::default()
        })
        .unwrap();
    store
        .persist_identity(WhatsAppIdentity {
            phone_number: Some("+48792717677".into()),
            full_name: Some("Przemyslaw Olszewski".into()),
            updated_at: 1_700_000_100,
            ..WhatsAppIdentity::default()
        })
        .unwrap();

    ProtocolStore::put_lid_mapping(
        store.backend.as_ref(),
        &LidPnMappingEntry {
            lid: "83962416386094".into(),
            phone_number: "48792717677".into(),
            created_at: 1_700_000_000,
            updated_at: 1_700_000_200,
            learning_source: "peer_lid_message".into(),
        },
    )
    .await
    .unwrap();

    let merged = store
        .persist_identity(WhatsAppIdentity {
            lid: Some("83962416386094".into()),
            phone_number: Some("+48792717677".into()),
            updated_at: 1_700_000_300,
            ..WhatsAppIdentity::default()
        })
        .unwrap();

    assert_eq!(merged.full_name.as_deref(), Some("Przemyslaw Olszewski"));
    assert_eq!(merged.push_name.as_deref(), Some("Przemek 2"));
}
