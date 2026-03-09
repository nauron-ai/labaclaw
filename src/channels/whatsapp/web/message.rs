use super::super::identity::WhatsAppIdentity;
use super::{WebEventContext, WhatsAppWebChannel};
use crate::channels::traits::ChannelMessage;
use wa_rs_core::proto_helpers::MessageExt;
use wa_rs_core::types::message::MessageInfo;
use wa_rs_proto::whatsapp::Message;

impl WebEventContext {
    pub(super) async fn handle_message(
        &self,
        msg: Box<Message>,
        info: MessageInfo,
        client: &wa_rs::Client,
    ) {
        let text = msg.text_content().unwrap_or("");
        let identity = self
            .identity_store
            .resolve_message_sender(&info)
            .unwrap_or_else(|err| {
                tracing::warn!("WhatsApp Web: failed to resolve sender identity: {}", err);
                WhatsAppIdentity::from(&info)
            });

        tracing::info!(
            "WhatsApp Web message from {} sender_jid={} chat_jid={}: {}",
            identity.summary(),
            info.source.sender,
            info.source.chat,
            text
        );

        if !identity.matches_allowlist(&self.allowed_identities) {
            tracing::warn!(
                "WhatsApp Web: message not in allowed list {} sender_jid={} chat_jid={}",
                identity.summary(),
                info.source.sender,
                info.source.chat
            );
            return;
        }

        let Some(content) = self
            .message_content(msg.as_ref(), &info, &identity, client)
            .await
        else {
            return;
        };

        if let Err(err) = self
            .tx
            .send(ChannelMessage {
                id: uuid::Uuid::new_v4().to_string(),
                channel: "whatsapp".to_string(),
                sender: identity
                    .canonical_sender()
                    .unwrap_or_else(|| info.source.sender.to_string()),
                reply_target: info.source.chat.to_string(),
                content,
                timestamp: chrono::Utc::now().timestamp() as u64,
                thread_ts: None,
            })
            .await
        {
            tracing::error!("Failed to send message to channel: {}", err);
        }
    }

    async fn message_content(
        &self,
        msg: &Message,
        info: &MessageInfo,
        identity: &WhatsAppIdentity,
        client: &wa_rs::Client,
    ) -> Option<String> {
        let trimmed = msg.text_content().unwrap_or("").trim();
        if !trimmed.is_empty() {
            return Some(trimmed.to_string());
        }

        let Some(transcription) = self.transcription.as_ref() else {
            tracing::debug!(
                "WhatsApp Web: ignoring empty or non-text message from {} sender_jid={} chat_jid={}",
                identity.summary(),
                info.source.sender,
                info.source.chat
            );
            return None;
        };

        let Some(audio_msg) = msg.audio_message.as_ref() else {
            tracing::debug!(
                "WhatsApp Web: ignoring non-text/non-audio message from {} sender_jid={} chat_jid={}",
                identity.summary(),
                info.source.sender,
                info.source.chat
            );
            return None;
        };

        let duration_secs = audio_msg.seconds.unwrap_or(0) as u64;
        if duration_secs > transcription.max_duration_secs {
            tracing::info!(
                "WhatsApp Web: voice message too long ({}s > {}s), skipping",
                duration_secs,
                transcription.max_duration_secs
            );
            return None;
        }

        let mime = audio_msg.mimetype.as_deref().unwrap_or("audio/ogg");
        let file_name = WhatsAppWebChannel::audio_mime_to_filename(mime);
        match client.download(audio_msg.as_ref()).await {
            Ok(audio_bytes) => match crate::channels::transcription::transcribe_audio(
                audio_bytes,
                file_name,
                transcription,
            )
            .await
            {
                Ok(text) if !text.trim().is_empty() => Some(format!("[Voice] {}", text.trim())),
                Ok(_) => {
                    tracing::info!(
                        "WhatsApp Web: voice transcription returned empty text, skipping"
                    );
                    None
                }
                Err(err) => {
                    tracing::warn!("WhatsApp Web: voice transcription failed: {err}");
                    None
                }
            },
            Err(err) => {
                tracing::warn!("WhatsApp Web: failed to download voice audio: {err}");
                None
            }
        }
    }
}
