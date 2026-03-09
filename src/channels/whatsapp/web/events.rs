use super::super::identity::WhatsAppIdentity;
use super::{WebEventContext, WhatsAppWebChannel};
use wa_rs_core::types::events::Event;

impl WebEventContext {
    pub(super) fn new(
        tx: tokio::sync::mpsc::Sender<crate::channels::traits::ChannelMessage>,
        allowed_identities: Vec<super::super::identity::WhatsAppAllowlistEntry>,
        transcription: Option<crate::config::TranscriptionConfig>,
        identity_store: super::super::identity::IdentityStore,
    ) -> Self {
        Self {
            tx,
            allowed_identities,
            transcription,
            identity_store,
        }
    }

    pub(super) async fn handle_event(&self, event: Event, client: &wa_rs::Client) {
        tracing::debug!("WhatsApp Web event received: {}", event_kind(&event));

        match event {
            Event::Message(msg, info) => self.handle_message(msg, info, client).await,
            Event::Connected(_) => {
                tracing::info!("WhatsApp Web connected successfully");
                self.log_allowed_identity_summary("failed to log identity summary");
            }
            Event::Disconnected(_) => tracing::warn!("WhatsApp Web disconnected from upstream"),
            Event::PairSuccess(success) => tracing::info!(
                "WhatsApp Web pair success id={} lid={} platform={} business_name={}",
                success.id,
                success.lid,
                success.platform,
                success.business_name
            ),
            Event::PairError(err) => tracing::error!(
                "WhatsApp Web pair error id={} lid={} platform={} error={}",
                err.id,
                err.lid,
                err.platform,
                err.error
            ),
            Event::LoggedOut(_) => tracing::warn!("WhatsApp Web was logged out"),
            Event::ConnectFailure(failure) => tracing::error!(
                "WhatsApp Web connect failure reason={:?} message={}",
                failure.reason,
                failure.message
            ),
            Event::StreamReplaced(_) => {
                tracing::warn!("WhatsApp Web stream replaced by another session")
            }
            Event::TemporaryBan(ban) => tracing::error!(
                "WhatsApp Web temporary ban code={} expire={:?}",
                ban.code,
                ban.expire
            ),
            Event::StreamError(stream_error) => {
                tracing::error!("WhatsApp Web stream error: {:?}", stream_error)
            }
            Event::UndecryptableMessage(msg) => tracing::warn!(
                "WhatsApp Web undecryptable message from {} in {} unavailable={} unavailable_type={:?} decrypt_fail_mode={:?}",
                msg.info.source.sender,
                msg.info.source.chat,
                msg.is_unavailable,
                msg.unavailable_type,
                msg.decrypt_fail_mode
            ),
            Event::Receipt(receipt) => tracing::info!(
                "WhatsApp Web receipt from {} in {} type={:?} ids={}",
                receipt.message_sender,
                receipt.source.chat,
                receipt.r#type,
                receipt.message_ids.len()
            ),
            Event::Presence(presence) => tracing::info!(
                "WhatsApp Web presence from {} unavailable={} last_seen={:?}",
                presence.from,
                presence.unavailable,
                presence.last_seen
            ),
            Event::ChatPresence(presence) => tracing::info!(
                "WhatsApp Web chat presence sender={} chat={} state={:?} media={:?}",
                presence.source.sender,
                presence.source.chat,
                presence.state,
                presence.media
            ),
            Event::Notification(node) => tracing::info!("WhatsApp Web notification node: {:?}", node),
            Event::OfflineSyncPreview(preview) => tracing::info!(
                "WhatsApp Web offline sync preview total={} app_data_changes={} messages={} notifications={} receipts={}",
                preview.total,
                preview.app_data_changes,
                preview.messages,
                preview.notifications,
                preview.receipts
            ),
            Event::OfflineSyncCompleted(done) => {
                tracing::info!("WhatsApp Web offline sync completed count={}", done.count);
                self.log_allowed_identity_summary("failed to log post-sync identity summary");
            }
            Event::HistorySync(sync) => tracing::info!(
                "WhatsApp Web history sync progress={:?} sync_type={:?}",
                sync.progress,
                sync.sync_type
            ),
            Event::ContactUpdate(update) => {
                self.log_identity_update("contact sync", self.identity_store.record_contact_update(&update))
            }
            Event::PushNameUpdate(update) => {
                self.log_identity_update("push name", self.identity_store.record_push_name_update(&update))
            }
            Event::UserAboutUpdate(update) => {
                self.log_identity_update("about", self.identity_store.record_user_about_update(&update))
            }
            Event::BusinessStatusUpdate(update) => self.log_identity_update(
                "business status",
                self.identity_store.record_business_status_update(&update),
            ),
            Event::ClientOutdated(_) => tracing::error!("WhatsApp Web client is outdated"),
            Event::PairingCode { code, .. } => {
                tracing::info!("WhatsApp Web pair code received: {}", code);
                tracing::info!(
                    "Link your phone by entering this code in WhatsApp > Linked Devices"
                );
            }
            Event::PairingQrCode { code, .. } => match WhatsAppWebChannel::render_pairing_qr(&code) {
                Ok(rendered) => {
                    tracing::info!(
                        "WhatsApp Web QR code received (scan with WhatsApp > Linked Devices)"
                    );
                    eprintln!();
                    eprintln!("WhatsApp Web QR code (scan in WhatsApp > Linked Devices):");
                    eprintln!("{rendered}");
                    eprintln!();
                }
                Err(err) => {
                    tracing::warn!(
                        "WhatsApp Web: failed to render pairing QR in terminal: {}",
                        err
                    );
                    tracing::info!("WhatsApp Web QR payload: {}", code);
                }
            },
            _ => tracing::debug!(
                "WhatsApp Web event {} received without dedicated log",
                event_kind(&event)
            ),
        }
    }

    fn log_allowed_identity_summary(&self, error_context: &str) {
        if let Err(err) = self
            .identity_store
            .log_allowed_identity_summary(&self.allowed_identities)
        {
            tracing::warn!("WhatsApp Web: {}: {}", error_context, err);
        }
    }

    fn log_identity_update(&self, source: &str, result: anyhow::Result<WhatsAppIdentity>) {
        match result {
            Ok(identity) => tracing::info!(
                "WhatsApp identity updated from {} {}",
                source,
                identity.summary()
            ),
            Err(err) => tracing::warn!("WhatsApp Web: failed to persist {}: {}", source, err),
        }
    }
}

fn event_kind(event: &Event) -> &'static str {
    match event {
        Event::Connected(_) => "Connected",
        Event::Disconnected(_) => "Disconnected",
        Event::PairSuccess(_) => "PairSuccess",
        Event::PairError(_) => "PairError",
        Event::LoggedOut(_) => "LoggedOut",
        Event::PairingQrCode { .. } => "PairingQrCode",
        Event::PairingCode { .. } => "PairingCode",
        Event::QrScannedWithoutMultidevice(_) => "QrScannedWithoutMultidevice",
        Event::ClientOutdated(_) => "ClientOutdated",
        Event::Message(_, _) => "Message",
        Event::Receipt(_) => "Receipt",
        Event::UndecryptableMessage(_) => "UndecryptableMessage",
        Event::Notification(_) => "Notification",
        Event::ChatPresence(_) => "ChatPresence",
        Event::Presence(_) => "Presence",
        Event::PictureUpdate(_) => "PictureUpdate",
        Event::UserAboutUpdate(_) => "UserAboutUpdate",
        Event::JoinedGroup(_) => "JoinedGroup",
        Event::GroupInfoUpdate { .. } => "GroupInfoUpdate",
        Event::ContactUpdate(_) => "ContactUpdate",
        Event::PushNameUpdate(_) => "PushNameUpdate",
        Event::SelfPushNameUpdated(_) => "SelfPushNameUpdated",
        Event::PinUpdate(_) => "PinUpdate",
        Event::MuteUpdate(_) => "MuteUpdate",
        Event::ArchiveUpdate(_) => "ArchiveUpdate",
        Event::MarkChatAsReadUpdate(_) => "MarkChatAsReadUpdate",
        Event::HistorySync(_) => "HistorySync",
        Event::OfflineSyncPreview(_) => "OfflineSyncPreview",
        Event::OfflineSyncCompleted(_) => "OfflineSyncCompleted",
        Event::DeviceListUpdate(_) => "DeviceListUpdate",
        Event::BusinessStatusUpdate(_) => "BusinessStatusUpdate",
        Event::StreamReplaced(_) => "StreamReplaced",
        Event::TemporaryBan(_) => "TemporaryBan",
        Event::ConnectFailure(_) => "ConnectFailure",
        Event::StreamError(_) => "StreamError",
    }
}
