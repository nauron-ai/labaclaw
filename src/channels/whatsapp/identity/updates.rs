use super::normalize::{normalize_lid, normalize_phone_number, normalize_text};
use super::WhatsAppIdentity;
use std::fmt;
use wa_rs_binary::jid::{Jid, JidExt};
use wa_rs_core::types::events::{
    BusinessStatusUpdate, ContactUpdate, PushNameUpdate, UserAboutUpdate,
};
use wa_rs_core::types::message::MessageInfo;

#[derive(Debug)]
pub(crate) enum WhatsAppIdentityJidError {
    MissingTarget,
    InvalidJid(wa_rs_binary::jid::JidError),
}

impl fmt::Display for WhatsAppIdentityJidError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingTarget => f.write_str("WhatsApp identity does not contain a target JID"),
            Self::InvalidJid(err) => err.fmt(f),
        }
    }
}

impl std::error::Error for WhatsAppIdentityJidError {}

impl From<wa_rs_binary::jid::JidError> for WhatsAppIdentityJidError {
    fn from(err: wa_rs_binary::jid::JidError) -> Self {
        Self::InvalidJid(err)
    }
}

impl From<&Jid> for WhatsAppIdentity {
    fn from(jid: &Jid) -> Self {
        let mut identity = Self::default();
        if jid.is_lid() {
            identity.lid = normalize_lid(jid.user());
        } else if jid.is_pn() {
            identity.phone_number = normalize_phone_number(jid.user());
        }
        identity.last_seen_jid = normalize_text(&jid.to_string());
        identity
    }
}

impl From<&MessageInfo> for WhatsAppIdentity {
    fn from(info: &MessageInfo) -> Self {
        let mut identity = Self::from(&info.source.sender);
        identity.push_name = normalize_text(info.push_name.as_str());
        identity.last_seen_jid = normalize_text(&info.source.sender.to_string());
        identity
    }
}

impl From<&ContactUpdate> for WhatsAppIdentity {
    fn from(update: &ContactUpdate) -> Self {
        let mut identity = Self::from(&update.jid);
        identity.lid = update
            .action
            .lid_jid
            .as_deref()
            .and_then(normalize_lid)
            .or(identity.lid);
        identity.phone_number = update
            .action
            .pn_jid
            .as_deref()
            .and_then(normalize_phone_number)
            .or(identity.phone_number);
        identity.full_name = update.action.full_name.as_deref().and_then(normalize_text);
        identity.first_name = update.action.first_name.as_deref().and_then(normalize_text);
        identity.username = update.action.username.as_deref().and_then(normalize_text);
        identity.last_seen_jid = normalize_text(&update.jid.to_string());
        identity
    }
}

impl From<&PushNameUpdate> for WhatsAppIdentity {
    fn from(update: &PushNameUpdate) -> Self {
        let mut identity = Self::from(&update.jid);
        identity.push_name = normalize_text(update.new_push_name.as_str());
        identity.last_seen_jid = normalize_text(&update.jid.to_string());
        identity
    }
}

impl From<&UserAboutUpdate> for WhatsAppIdentity {
    fn from(update: &UserAboutUpdate) -> Self {
        let mut identity = Self::from(&update.jid);
        identity.about = normalize_text(update.status.as_str());
        identity.last_seen_jid = normalize_text(&update.jid.to_string());
        identity
    }
}

impl From<&BusinessStatusUpdate> for WhatsAppIdentity {
    fn from(update: &BusinessStatusUpdate) -> Self {
        let target = update.target_jid.as_ref().unwrap_or(&update.jid);
        let mut identity = Self::from(target);
        identity.verified_name = update.verified_name.as_deref().and_then(normalize_text);
        identity.last_seen_jid = normalize_text(&target.to_string());
        identity
    }
}

impl TryFrom<&WhatsAppIdentity> for Jid {
    type Error = WhatsAppIdentityJidError;

    fn try_from(identity: &WhatsAppIdentity) -> Result<Self, Self::Error> {
        if let Some(last_seen_jid) = identity.last_seen_jid.as_deref() {
            if let Ok(jid) = last_seen_jid.parse::<Jid>() {
                return Ok(jid);
            }
        }

        if let Some(phone_number) = identity.phone_number() {
            let digits: String = phone_number
                .chars()
                .filter(|c| c.is_ascii_digit())
                .collect();
            if !digits.is_empty() {
                return Ok(Jid::pn(digits));
            }
        }

        identity
            .lid()
            .map(Jid::lid)
            .ok_or(WhatsAppIdentityJidError::MissingTarget)
    }
}
