use super::attachments::{
    mime_from_path, parse_wa_attachment_markers, WaAttachment, WaAttachmentKind,
};
use super::{DeliveryTarget, WhatsAppWebChannel};
use anyhow::{anyhow, Result};
use std::{path::Path, sync::Arc};
use wa_rs_binary::jid::Jid;

impl WhatsAppWebChannel {
    fn delivery_target(&self, recipient: &str, target: &str) -> Result<Option<DeliveryTarget>> {
        let identity = self.resolve_direct_identity(recipient);
        let jid = Jid::try_from(&identity).map_err(|err| {
            anyhow!("Recipient `{recipient}` does not contain a valid target: {err}")
        })?;
        let direct_identity_target = jid.is_pn() || jid.is_lid();

        if direct_identity_target && !identity.matches_allowlist(&self.allowed_identities) {
            tracing::warn!(
                "WhatsApp Web: {} not in allowed list {} recipient={}",
                target,
                identity.summary(),
                recipient
            );
            return Ok(None);
        }

        Ok(Some(DeliveryTarget { identity, jid }))
    }

    async fn send_media_attachment(
        &self,
        client: &Arc<wa_rs::Client>,
        to: &Jid,
        attachment: &WaAttachment,
    ) -> Result<()> {
        let path = Path::new(&attachment.target);
        if !path.exists() {
            anyhow::bail!("Media file not found: {}", attachment.target);
        }

        let data = tokio::fs::read(path).await?;
        let mimetype = mime_from_path(path).to_string();

        tracing::info!(
            "WhatsApp Web: uploading {:?} ({} bytes, {})",
            attachment.kind,
            data.len(),
            mimetype
        );

        let upload = client.upload(data, attachment.kind.media_type()).await?;
        let outgoing = match attachment.kind {
            WaAttachmentKind::Image => wa_rs_proto::whatsapp::Message {
                image_message: Some(Box::new(wa_rs_proto::whatsapp::message::ImageMessage {
                    url: Some(upload.url),
                    direct_path: Some(upload.direct_path),
                    media_key: Some(upload.media_key),
                    file_enc_sha256: Some(upload.file_enc_sha256),
                    file_sha256: Some(upload.file_sha256),
                    file_length: Some(upload.file_length),
                    mimetype: Some(mimetype),
                    ..Default::default()
                })),
                ..Default::default()
            },
            WaAttachmentKind::Document => wa_rs_proto::whatsapp::Message {
                document_message: Some(Box::new(wa_rs_proto::whatsapp::message::DocumentMessage {
                    url: Some(upload.url),
                    direct_path: Some(upload.direct_path),
                    media_key: Some(upload.media_key),
                    file_enc_sha256: Some(upload.file_enc_sha256),
                    file_sha256: Some(upload.file_sha256),
                    file_length: Some(upload.file_length),
                    mimetype: Some(mimetype),
                    file_name: Some(
                        path.file_name()
                            .and_then(|name| name.to_str())
                            .unwrap_or("file")
                            .to_string(),
                    ),
                    ..Default::default()
                })),
                ..Default::default()
            },
            WaAttachmentKind::Video => wa_rs_proto::whatsapp::Message {
                video_message: Some(Box::new(wa_rs_proto::whatsapp::message::VideoMessage {
                    url: Some(upload.url),
                    direct_path: Some(upload.direct_path),
                    media_key: Some(upload.media_key),
                    file_enc_sha256: Some(upload.file_enc_sha256),
                    file_sha256: Some(upload.file_sha256),
                    file_length: Some(upload.file_length),
                    mimetype: Some(mimetype),
                    ..Default::default()
                })),
                ..Default::default()
            },
            WaAttachmentKind::Audio => wa_rs_proto::whatsapp::Message {
                audio_message: Some(Box::new(wa_rs_proto::whatsapp::message::AudioMessage {
                    url: Some(upload.url),
                    direct_path: Some(upload.direct_path),
                    media_key: Some(upload.media_key),
                    file_enc_sha256: Some(upload.file_enc_sha256),
                    file_sha256: Some(upload.file_sha256),
                    file_length: Some(upload.file_length),
                    mimetype: Some(mimetype),
                    ..Default::default()
                })),
                ..Default::default()
            },
        };

        let message_id = client.send_message(to.clone(), outgoing).await?;
        tracing::info!(
            "WhatsApp Web: sent {:?} media (id: {})",
            attachment.kind,
            message_id
        );
        Ok(())
    }

    pub(super) async fn send_outbound(
        &self,
        message: &crate::channels::traits::SendMessage,
    ) -> Result<()> {
        let client = self.active_client()?;
        let Some(target) = self.delivery_target(&message.recipient, "recipient")? else {
            return Ok(());
        };

        let (text_without_markers, attachments) = parse_wa_attachment_markers(&message.content);
        if !text_without_markers.is_empty() {
            let text_message = wa_rs_proto::whatsapp::Message {
                conversation: Some(text_without_markers.clone()),
                ..Default::default()
            };
            let message_id = client
                .send_message(target.jid.clone(), text_message)
                .await?;
            tracing::debug!(
                "WhatsApp Web: sent text to {} recipient={} (id: {})",
                target.identity.summary(),
                message.recipient,
                message_id
            );
        }

        for attachment in &attachments {
            if let Err(err) = self
                .send_media_attachment(&client, &target.jid, attachment)
                .await
            {
                tracing::error!(
                    "WhatsApp Web: failed to send {:?} attachment {}: {}",
                    attachment.kind,
                    attachment.target,
                    err
                );
                let fallback = wa_rs_proto::whatsapp::Message {
                    conversation: Some(format!(
                        "[Failed to send media: {}]",
                        Path::new(&attachment.target)
                            .file_name()
                            .and_then(|name| name.to_str())
                            .unwrap_or("attachment")
                    )),
                    ..Default::default()
                };
                let _ = client.send_message(target.jid.clone(), fallback).await;
            }
        }

        if attachments.is_empty()
            && text_without_markers.is_empty()
            && !message.content.trim().is_empty()
        {
            let outgoing = wa_rs_proto::whatsapp::Message {
                conversation: Some(message.content.clone()),
                ..Default::default()
            };
            let message_id = client.send_message(target.jid, outgoing).await?;
            tracing::debug!(
                "WhatsApp Web: sent message to {} recipient={} (id: {})",
                target.identity.summary(),
                message.recipient,
                message_id
            );
        }

        Ok(())
    }

    pub(super) async fn start_typing_for(&self, recipient: &str) -> Result<()> {
        let client = self.active_client()?;
        let Some(target) = self.delivery_target(recipient, "typing target")? else {
            return Ok(());
        };

        client
            .chatstate()
            .send_composing(&target.jid)
            .await
            .map_err(|err| anyhow!("Failed to send typing state (composing): {err}"))?;

        tracing::debug!(
            "WhatsApp Web: start typing for {} recipient={}",
            target.identity.summary(),
            recipient
        );
        Ok(())
    }

    pub(super) async fn stop_typing_for(&self, recipient: &str) -> Result<()> {
        let client = self.active_client()?;
        let Some(target) = self.delivery_target(recipient, "typing target")? else {
            return Ok(());
        };

        client
            .chatstate()
            .send_paused(&target.jid)
            .await
            .map_err(|err| anyhow!("Failed to send typing state (paused): {err}"))?;

        tracing::debug!(
            "WhatsApp Web: stop typing for {} recipient={}",
            target.identity.summary(),
            recipient
        );
        Ok(())
    }
}
