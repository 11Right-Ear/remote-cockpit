//! Session and device identifiers.
//!
//! These newtypes serialize transparently: a `SessionId` is a bare UUID string
//! on the wire, a `DeviceId` a bare string. Keeping them as newtypes (rather
//! than raw `Uuid`/`String`) prevents mixing them up at call sites.

use serde::{Deserialize, Serialize};

/// Server-assigned identifier for a terminal (PTY) session.
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct SessionId(pub uuid::Uuid);

impl SessionId {
    /// Generate a fresh random session id.
    pub fn new() -> Self {
        Self(uuid::Uuid::new_v4())
    }
}

impl Default for SessionId {
    fn default() -> Self {
        Self::new()
    }
}

/// Device identifier. V1: recorded in the audit log only; the "unknown device
/// requires approval" flow is deferred (see SECURITY.md).
#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct DeviceId(pub String);

impl DeviceId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }
}

impl AsRef<str> for DeviceId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

/// Which side of the connection a client speaks for. Determined from the
/// first `Auth` frame; the gateway routes accordingly.
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ClientRole {
    Desktop,
    Phone,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn session_id_serializes_as_bare_uuid() {
        let id = SessionId::new();
        let json = serde_json::to_string(&id).unwrap();
        // A bare UUID string, not an object.
        assert!(json.starts_with('"') && json.ends_with('"'));
        let back: SessionId = serde_json::from_str(&json).unwrap();
        assert_eq!(back, id);
    }

    #[test]
    fn device_id_serializes_as_bare_string() {
        let did = DeviceId::new("dev-ws");
        assert_eq!(serde_json::to_string(&did).unwrap(), r#""dev-ws""#);
    }

    #[test]
    fn role_snake_case() {
        assert_eq!(
            serde_json::to_string(&ClientRole::Desktop).unwrap(),
            r#""desktop""#
        );
        assert_eq!(
            serde_json::to_string(&ClientRole::Phone).unwrap(),
            r#""phone""#
        );
    }
}
