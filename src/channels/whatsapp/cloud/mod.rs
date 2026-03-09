use super::super::traits::{Channel, ChannelMessage, SendMessage};
use super::identity::{WhatsAppAllowlistEntry, WhatsAppIdentity};
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use uuid::Uuid;

fn ensure_https(url: &str) -> anyhow::Result<()> {
    if !url.starts_with("https://") {
        anyhow::bail!(
            "Refusing to transmit sensitive data over non-HTTPS URL: URL scheme must be https"
        );
    }
    Ok(())
}

pub struct WhatsAppChannel {
    access_token: String,
    endpoint_id: String,
    verify_token: String,
    allowed_identities: Vec<WhatsAppAllowlistEntry>,
}

impl WhatsAppChannel {
    pub fn new(
        access_token: String,
        endpoint_id: String,
        verify_token: String,
        allowed_numbers: Vec<String>,
    ) -> Self {
        Self {
            access_token,
            endpoint_id,
            verify_token,
            allowed_identities: allowed_numbers
                .iter()
                .map(|allowed_number| WhatsAppAllowlistEntry::from(allowed_number.as_str()))
                .collect(),
        }
    }

    fn http_client(&self) -> reqwest::Client {
        crate::config::build_runtime_proxy_client("channel.whatsapp")
    }

    pub fn verify_token(&self) -> &str {
        &self.verify_token
    }

    pub fn parse_webhook_payload(&self, payload: &serde_json::Value) -> Vec<ChannelMessage> {
        let mut messages = Vec::new();
        let Some(entries) = payload.get("entry").and_then(|entry| entry.as_array()) else {
            return messages;
        };

        for entry in entries {
            let Some(changes) = entry.get("changes").and_then(|changes| changes.as_array()) else {
                continue;
            };

            for change in changes {
                let Some(value) = change.get("value") else {
                    continue;
                };
                let Some(raw_messages) = value
                    .get("messages")
                    .and_then(|messages| messages.as_array())
                else {
                    continue;
                };

                for raw_message in raw_messages {
                    let Some(from) = raw_message.get("from").and_then(|from| from.as_str()) else {
                        continue;
                    };
                    let Some(sender_identity) = self.allowlisted_identity(from) else {
                        tracing::warn!(
                            "WhatsApp Cloud: ignoring message from unauthorized sender {}",
                            WhatsAppIdentity::from(from).summary()
                        );
                        continue;
                    };

                    let content = raw_message
                        .get("text")
                        .and_then(|text| text.get("body"))
                        .and_then(|body| body.as_str())
                        .unwrap_or("")
                        .to_owned();
                    if content.is_empty() {
                        if raw_message.get("text").is_none() {
                            tracing::debug!(
                                "WhatsApp Cloud: skipping non-text message from {}",
                                sender_identity.summary()
                            );
                        }
                        continue;
                    }

                    let timestamp = raw_message
                        .get("timestamp")
                        .and_then(|timestamp| timestamp.as_str())
                        .and_then(|timestamp| timestamp.parse::<u64>().ok())
                        .unwrap_or_else(current_unix_timestamp);
                    let sender = sender_identity
                        .canonical_sender()
                        .unwrap_or_else(|| from.to_owned());

                    messages.push(ChannelMessage {
                        id: Uuid::new_v4().to_string(),
                        reply_target: sender.clone(),
                        sender,
                        content,
                        channel: "whatsapp".to_string(),
                        timestamp,
                        thread_ts: None,
                    });
                }
            }
        }

        messages
    }

    fn allowlisted_identity(&self, value: &str) -> Option<WhatsAppIdentity> {
        let identity = WhatsAppIdentity::from(value);
        identity
            .matches_allowlist(&self.allowed_identities)
            .then_some(identity)
    }
}

#[async_trait]
impl Channel for WhatsAppChannel {
    fn name(&self) -> &str {
        "whatsapp"
    }

    async fn send(&self, message: &SendMessage) -> Result<()> {
        let Some(identity) = self.allowlisted_identity(&message.recipient) else {
            tracing::warn!(
                "WhatsApp Cloud: recipient not in allowlist {}",
                WhatsAppIdentity::from(message.recipient.as_str()).summary()
            );
            return Ok(());
        };
        let to = identity
            .phone_number()
            .map(|phone_number| phone_number.trim_start_matches('+').to_owned())
            .ok_or_else(|| anyhow!("WhatsApp Cloud API requires a phone-number recipient"))?;

        let url = format!(
            "https://graph.facebook.com/v18.0/{}/messages",
            self.endpoint_id
        );
        let body = serde_json::json!({
            "messaging_product": "whatsapp",
            "recipient_type": "individual",
            "to": to,
            "type": "text",
            "text": { "preview_url": false, "body": message.content }
        });

        ensure_https(&url)?;
        let response = self
            .http_client()
            .post(&url)
            .bearer_auth(&self.access_token)
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_body = response.text().await.unwrap_or_default();
            let sanitized = crate::providers::sanitize_api_error(&error_body);
            tracing::error!("WhatsApp send failed: {status} — {sanitized}");
            anyhow::bail!("WhatsApp API error: {status}");
        }

        Ok(())
    }

    async fn listen(&self, _tx: tokio::sync::mpsc::Sender<ChannelMessage>) -> Result<()> {
        tracing::info!(
            "WhatsApp channel active (webhook mode). Configure Meta webhook to POST to /whatsapp."
        );
        loop {
            tokio::time::sleep(std::time::Duration::from_secs(3600)).await;
        }
    }

    async fn health_check(&self) -> bool {
        let url = format!("https://graph.facebook.com/v18.0/{}", self.endpoint_id);
        if ensure_https(&url).is_err() {
            return false;
        }

        self.http_client()
            .get(&url)
            .bearer_auth(&self.access_token)
            .send()
            .await
            .map(|response| response.status().is_success())
            .unwrap_or(false)
    }
}

fn current_unix_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

#[cfg(test)]
mod tests;
