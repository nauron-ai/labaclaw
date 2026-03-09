use super::WhatsAppIdentity;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum WhatsAppAllowlistEntry {
    Any,
    Identity(WhatsAppIdentity),
}

impl WhatsAppAllowlistEntry {
    pub(crate) fn matches(&self, identity: &WhatsAppIdentity) -> bool {
        match self {
            Self::Any => true,
            Self::Identity(allowed_identity) => allowed_identity.matches_identity(identity),
        }
    }
}

impl From<&str> for WhatsAppAllowlistEntry {
    fn from(value: &str) -> Self {
        if value.trim() == "*" {
            Self::Any
        } else {
            Self::Identity(WhatsAppIdentity::from(value))
        }
    }
}
