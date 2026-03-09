use super::super::traits::ChannelMessage;
use super::identity::{IdentityStore, WhatsAppAllowlistEntry, WhatsAppIdentity};
use anyhow::{anyhow, Result};
use parking_lot::Mutex;
use std::sync::Arc;
use tokio::sync::mpsc::Sender;

mod attachments;
mod events;
mod message;
mod outbound;
mod runtime;

pub struct WhatsAppWebChannel {
    session_path: String,
    pair_phone: Option<String>,
    pair_code: Option<String>,
    allowed_identities: Vec<WhatsAppAllowlistEntry>,
    bot_handle: Arc<Mutex<Option<tokio::task::JoinHandle<()>>>>,
    client: Arc<Mutex<Option<Arc<wa_rs::Client>>>>,
    tx: Arc<Mutex<Option<Sender<ChannelMessage>>>>,
    identity_store: Arc<Mutex<Option<IdentityStore>>>,
    transcription: Option<crate::config::TranscriptionConfig>,
}

#[derive(Clone)]
struct WebEventContext {
    tx: Sender<ChannelMessage>,
    allowed_identities: Vec<WhatsAppAllowlistEntry>,
    transcription: Option<crate::config::TranscriptionConfig>,
    identity_store: IdentityStore,
}

struct DeliveryTarget {
    identity: WhatsAppIdentity,
    jid: wa_rs_binary::jid::Jid,
}

impl WhatsAppWebChannel {
    pub fn new(
        session_path: String,
        pair_phone: Option<String>,
        pair_code: Option<String>,
        allowed_numbers: Vec<String>,
    ) -> Self {
        Self {
            session_path,
            pair_phone,
            pair_code,
            allowed_identities: allowed_numbers
                .iter()
                .map(|allowed_number| WhatsAppAllowlistEntry::from(allowed_number.as_str()))
                .collect(),
            bot_handle: Arc::new(Mutex::new(None)),
            client: Arc::new(Mutex::new(None)),
            tx: Arc::new(Mutex::new(None)),
            identity_store: Arc::new(Mutex::new(None)),
            transcription: None,
        }
    }

    pub fn with_transcription(mut self, config: crate::config::TranscriptionConfig) -> Self {
        if config.enabled {
            self.transcription = Some(config);
        }
        self
    }

    fn active_client(&self) -> Result<Arc<wa_rs::Client>> {
        self.client
            .lock()
            .clone()
            .ok_or_else(|| anyhow!("WhatsApp Web client not connected. Initialize the bot first."))
    }

    fn audio_mime_to_filename(mime: &str) -> &'static str {
        let base = mime
            .split(';')
            .next()
            .unwrap_or("")
            .trim()
            .to_ascii_lowercase();
        match base.as_str() {
            "audio/ogg" | "audio/oga" => "voice.ogg",
            "audio/webm" => "voice.webm",
            "audio/opus" => "voice.opus",
            "audio/mp4" | "audio/m4a" | "audio/aac" => "voice.m4a",
            "audio/mpeg" | "audio/mp3" => "voice.mp3",
            "audio/wav" | "audio/x-wav" => "voice.wav",
            _ => "voice.ogg",
        }
    }

    fn resolve_direct_identity(&self, value: &str) -> WhatsAppIdentity {
        let fallback = WhatsAppIdentity::from(value);
        self.identity_store
            .lock()
            .clone()
            .map(|store| store.resolve_recipient(value))
            .transpose()
            .map_err(|err| {
                tracing::warn!(
                    "WhatsApp Web: failed to resolve identity for {}: {}",
                    value,
                    err
                );
                err
            })
            .ok()
            .flatten()
            .unwrap_or(fallback)
    }

    fn render_pairing_qr(code: &str) -> Result<String> {
        let payload = code.trim();
        if payload.is_empty() {
            anyhow::bail!("QR payload is empty");
        }

        let qr = qrcode::QrCode::new(payload.as_bytes())
            .map_err(|err| anyhow!("Failed to encode WhatsApp Web QR payload: {err}"))?;

        Ok(qr
            .render::<qrcode::render::unicode::Dense1x2>()
            .quiet_zone(true)
            .build())
    }
}

#[cfg(test)]
mod tests;
