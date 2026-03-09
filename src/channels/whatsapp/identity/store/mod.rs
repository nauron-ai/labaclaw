mod db;

use super::{WhatsAppAllowlistEntry, WhatsAppIdentity};
use crate::channels::whatsapp::storage::RusqliteStore;
use anyhow::Result;
use chrono::Utc;
use std::collections::BTreeSet;
use std::sync::Arc;
use wa_rs_core::types::events::{
    BusinessStatusUpdate, ContactUpdate, PushNameUpdate, UserAboutUpdate,
};
use wa_rs_core::types::message::MessageInfo;

pub(crate) use db::init_schema;

#[derive(Clone)]
pub(crate) struct IdentityStore {
    backend: Arc<RusqliteStore>,
}

impl From<Arc<RusqliteStore>> for IdentityStore {
    fn from(backend: Arc<RusqliteStore>) -> Self {
        Self { backend }
    }
}

impl IdentityStore {
    pub(crate) fn resolve_message_sender(&self, info: &MessageInfo) -> Result<WhatsAppIdentity> {
        self.persist_identity(WhatsAppIdentity::from(info))
    }

    pub(crate) fn resolve_recipient(&self, recipient: &str) -> Result<WhatsAppIdentity> {
        self.resolve_identity(WhatsAppIdentity::from(recipient))
    }

    pub(crate) fn record_contact_update(&self, update: &ContactUpdate) -> Result<WhatsAppIdentity> {
        self.persist_identity(WhatsAppIdentity::from(update))
    }

    pub(crate) fn record_push_name_update(
        &self,
        update: &PushNameUpdate,
    ) -> Result<WhatsAppIdentity> {
        self.persist_identity(WhatsAppIdentity::from(update))
    }

    pub(crate) fn record_user_about_update(
        &self,
        update: &UserAboutUpdate,
    ) -> Result<WhatsAppIdentity> {
        self.persist_identity(WhatsAppIdentity::from(update))
    }

    pub(crate) fn record_business_status_update(
        &self,
        update: &BusinessStatusUpdate,
    ) -> Result<WhatsAppIdentity> {
        self.persist_identity(WhatsAppIdentity::from(update))
    }

    pub(crate) fn log_allowed_identity_summary(
        &self,
        allowed_identities: &[WhatsAppAllowlistEntry],
    ) -> Result<()> {
        let identities = self.allowlisted_identities(allowed_identities)?;
        if identities.is_empty() {
            return Ok(());
        }

        tracing::info!("WhatsApp identity summary ready count={}", identities.len());
        for identity in identities {
            tracing::info!("WhatsApp identity allowlist contact {}", identity.summary());
        }
        Ok(())
    }

    fn persist_identity(&self, identity: WhatsAppIdentity) -> Result<WhatsAppIdentity> {
        let identity = self.resolve_identity(identity)?;
        if identity.lid().is_none() && identity.phone_number().is_none() {
            return Ok(identity);
        }

        self.backend.with_connection(|conn| {
            let device_id = self.backend.device_id();
            let lid_match = db::fetch_by_lid(conn, device_id, identity.lid())?;
            let phone_match = db::fetch_by_phone_number(conn, device_id, identity.phone_number())?;

            let merged = [lid_match.as_ref(), phone_match.as_ref()]
                .into_iter()
                .flatten()
                .fold(identity.clone(), |merged, existing| {
                    existing.identity.merge(&merged)
                });

            let primary = db::primary_row_id(lid_match.as_ref(), phone_match.as_ref());
            if let (Some(lid_match), Some(phone_match), Some(primary)) =
                (lid_match.as_ref(), phone_match.as_ref(), primary)
            {
                let duplicate = if lid_match.row_id == primary {
                    phone_match.row_id
                } else {
                    lid_match.row_id
                };
                if duplicate != primary {
                    db::delete_identity(conn, device_id, duplicate)?;
                }
            }

            if let Some(primary) = primary {
                db::update_identity(conn, device_id, primary, &merged)?;
            } else {
                db::insert_identity(conn, device_id, &merged)?;
            }

            Ok(merged)
        })
    }

    fn resolve_identity(&self, identity: WhatsAppIdentity) -> Result<WhatsAppIdentity> {
        let mut identity = identity;
        if identity.updated_at == 0 {
            identity = identity.with_updated_at(Utc::now().timestamp());
        }

        self.backend.with_connection(|conn| {
            let device_id = self.backend.device_id();
            if identity.phone_number().is_none() {
                identity.phone_number = db::lookup_phone_number(conn, device_id, identity.lid())?;
            }
            if identity.lid().is_none() {
                identity.lid = db::lookup_lid(conn, device_id, identity.phone_number())?;
            }

            if let Some(existing) = db::fetch_by_lid(conn, device_id, identity.lid())? {
                identity = existing.identity.merge(&identity);
            }
            if let Some(existing) =
                db::fetch_by_phone_number(conn, device_id, identity.phone_number())?
            {
                identity = existing.identity.merge(&identity);
            }

            Ok(identity)
        })
    }

    fn allowlisted_identities(
        &self,
        allowed_identities: &[WhatsAppAllowlistEntry],
    ) -> Result<Vec<WhatsAppIdentity>> {
        let mut seen = BTreeSet::new();
        let mut identities = Vec::new();

        for allowed_identity in allowed_identities {
            let WhatsAppAllowlistEntry::Identity(allowed_identity) = allowed_identity else {
                continue;
            };

            let identity = self.resolve_identity(
                allowed_identity
                    .clone()
                    .with_updated_at(Utc::now().timestamp()),
            )?;
            let key = identity
                .phone_number()
                .map(ToOwned::to_owned)
                .or_else(|| identity.lid().map(ToOwned::to_owned))
                .unwrap_or_else(|| allowed_identity.summary());

            if seen.insert(key) {
                identities.push(identity);
            }
        }

        Ok(identities)
    }
}

#[cfg(test)]
mod tests;
