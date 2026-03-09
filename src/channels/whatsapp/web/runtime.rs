use super::super::storage::RusqliteStore;
use super::{WebEventContext, WhatsAppWebChannel};
use anyhow::Result;
use async_trait::async_trait;
use std::sync::Arc;

impl WhatsAppWebChannel {
    pub(super) async fn listen_loop(
        &self,
        tx: tokio::sync::mpsc::Sender<crate::channels::traits::ChannelMessage>,
    ) -> Result<()> {
        *self.tx.lock() = Some(tx.clone());

        use wa_rs::bot::Bot;
        use wa_rs::pair_code::PairCodeOptions;
        use wa_rs::store::{Device, DeviceStore};
        use wa_rs_tokio_transport::TokioWebSocketTransportFactory;
        use wa_rs_ureq_http::UreqHttpClient;

        tracing::info!(
            "WhatsApp Web channel starting (session: {})",
            self.session_path
        );

        let storage = RusqliteStore::new(&self.session_path)?;
        let backend = Arc::new(storage);
        let mut device = Device::new(backend.clone());

        if backend.exists().await? {
            tracing::info!("WhatsApp Web: found existing session, loading device");
            if let Some(core_device) = backend.load().await? {
                device.load_from_serializable(core_device);
            } else {
                anyhow::bail!("Device exists but failed to load");
            }
        } else {
            tracing::info!(
                "WhatsApp Web: no existing session, new device will be created during pairing"
            );
        }

        let mut transport_factory = TokioWebSocketTransportFactory::new();
        if let Ok(ws_url) = std::env::var("WHATSAPP_WS_URL") {
            transport_factory = transport_factory.with_url(ws_url);
        }

        let identity_store = super::super::identity::IdentityStore::from(backend.clone());
        *self.identity_store.lock() = Some(identity_store.clone());

        let event_context = WebEventContext::new(
            tx,
            self.allowed_identities.clone(),
            self.transcription.clone(),
            identity_store,
        );
        let mut builder = Bot::builder()
            .with_backend(backend)
            .with_transport_factory(transport_factory)
            .with_http_client(UreqHttpClient::new())
            .on_event(move |event, client| {
                let event_context = event_context.clone();
                async move { event_context.handle_event(event, &client).await }
            });

        if let Some(ref phone) = self.pair_phone {
            tracing::info!("WhatsApp Web: pair-code flow enabled for configured phone number");
            builder = builder.with_pair_code(PairCodeOptions {
                phone_number: phone.clone(),
                custom_code: self.pair_code.clone(),
                ..Default::default()
            });
        } else if self.pair_code.is_some() {
            tracing::warn!(
                "WhatsApp Web: pair_code is set but pair_phone is missing; pair code config is ignored"
            );
        }

        let mut bot = builder.build().await?;
        *self.client.lock() = Some(bot.client());
        *self.bot_handle.lock() = Some(bot.run().await?);

        tokio::signal::ctrl_c().await.ok();
        tracing::info!("WhatsApp Web channel received Ctrl+C, shutting down");

        *self.client.lock() = None;
        *self.identity_store.lock() = None;
        if let Some(handle) = self.bot_handle.lock().take() {
            handle.abort();
        }

        Ok(())
    }
}

#[async_trait]
impl crate::channels::traits::Channel for WhatsAppWebChannel {
    fn name(&self) -> &str {
        "whatsapp"
    }

    async fn send(&self, message: &crate::channels::traits::SendMessage) -> Result<()> {
        self.send_outbound(message).await
    }

    async fn listen(
        &self,
        tx: tokio::sync::mpsc::Sender<crate::channels::traits::ChannelMessage>,
    ) -> Result<()> {
        self.listen_loop(tx).await
    }

    async fn health_check(&self) -> bool {
        self.bot_handle.lock().is_some()
    }

    async fn start_typing(&self, recipient: &str) -> Result<()> {
        self.start_typing_for(recipient).await
    }

    async fn stop_typing(&self, recipient: &str) -> Result<()> {
        self.stop_typing_for(recipient).await
    }
}
