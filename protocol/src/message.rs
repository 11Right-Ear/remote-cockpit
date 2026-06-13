//! Wire messages between client (phone/desktop) and server (gateway).
//!
//! ## Direction
//! Message direction is implied by the authenticated role, so we do **not**
//! embed routing labels here — the gateway attaches them based on which side
//! sent the frame.
//!
//! ## Tagging
//! `#[serde(tag = "type", rename_all = "snake_case")]` makes JSON
//! self-describing (`{"type":"auth",...}`) and lets new variants be added
//! without breaking older clients (an unknown variant is a recoverable error,
//! not a connection crash).

use serde::{Deserialize, Serialize};

use crate::session::{ClientRole, DeviceId, SessionId};

// ---------------------------------------------------------------------------
// Client -> Server (uplink): phone or desktop -> gateway
// ---------------------------------------------------------------------------

/// Messages a client (phone or desktop) sends to the gateway.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ClientMessage {
    /// First frame on a connection. Authenticates and classifies the role.
    Auth {
        token: String,
        role: ClientRole,
        device_id: DeviceId,
    },

    /// Keystrokes/input destined for the PTY. `data` is base64-encoded bytes.
    /// Phone -> desktop (routed).
    TerminalInput {
        session_id: SessionId,
        data: String,
    },

    /// Resize the PTY. Phone -> desktop (routed). V1 spawns at 80x24.
    TerminalResize {
        session_id: SessionId,
        cols: u16,
        rows: u16,
    },

    /// Phone asks the desktop to spawn a new PTY session.
    OpenSession {
        shell: Option<String>,
    },

    /// Phone asks to close a PTY session.
    CloseSession {
        session_id: SessionId,
    },

    // --- desktop -> gateway: session lifecycle reports ---
    // The desktop agent emits these; the gateway audits them and forwards the
    // equivalent `ServerMessage` to the linked phone. Phones never send these.

    /// Desktop spawned the requested PTY.
    ReportSessionOpened {
        session_id: SessionId,
    },

    /// PTY session ended.
    ReportSessionClosed {
        session_id: SessionId,
    },

    /// Session-level error on the desktop side.
    ReportSessionError {
        session_id: SessionId,
        message: String,
    },

    /// A dangerous-command pattern matched on the desktop (V1: warn only,
    /// execution still proceeds per SECURITY.md).
    ReportDanger {
        session_id: SessionId,
        command: String,
        pattern: String,
    },

    /// Heartbeat.
    Ping {
        ts_ms: i64,
    },
}

// ---------------------------------------------------------------------------
// Server -> Client (downlink): gateway -> phone or desktop
// ---------------------------------------------------------------------------

/// Messages the gateway sends to a client (phone or desktop).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ServerMessage {
    /// Authentication succeeded. `session_id` is a connection-scoped id
    /// (distinct from PTY session ids, which arrive via `SessionOpened`).
    AuthOk {
        session_id: SessionId,
        server_time_ms: i64,
    },

    /// Authentication failed. The connection will be closed.
    AuthFail {
        reason: String,
    },

    /// Small / out-of-band terminal payload. Bulk output rides binary frames.
    TerminalOutput {
        session_id: SessionId,
        data: String,
    },

    /// A dangerous-command pattern matched. V1: warn only.
    DangerWarn {
        session_id: SessionId,
        command: String,
        pattern: String,
    },

    /// Desktop spawned the requested PTY session.
    SessionOpened {
        session_id: SessionId,
    },

    /// PTY session ended.
    SessionClosed {
        session_id: SessionId,
    },

    /// A session-level error.
    SessionError {
        session_id: SessionId,
        message: String,
    },

    /// Desktop agent connection state, routed to phones.
    DesktopOnline {
        device_id: DeviceId,
    },

    /// Desktop agent disconnected.
    DesktopOffline {
        device_id: DeviceId,
    },

    /// Heartbeat reply.
    Pong {
        ts_ms: i64,
        server_time_ms: i64,
    },

    /// Generic protocol-level error.
    Error {
        code: String,
        message: String,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn auth_roundtrip_and_shape() {
        let msg = ClientMessage::Auth {
            token: "tok".into(),
            role: ClientRole::Phone,
            device_id: DeviceId::new("dev-ws"),
        };
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains(r#""type":"auth""#), "json was: {json}");
        assert!(json.contains(r#""role":"phone""#));
        assert!(json.contains(r#""device_id":"dev-ws""#));

        let back: ClientMessage = serde_json::from_str(&json).unwrap();
        match back {
            ClientMessage::Auth { role, device_id, .. } => {
                assert_eq!(role, ClientRole::Phone);
                assert_eq!(device_id.as_ref(), "dev-ws");
            }
            _ => panic!("decoded wrong variant"),
        }
    }

    #[test]
    fn open_session_roundtrip() {
        let msg = ClientMessage::OpenSession { shell: None };
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains(r#""type":"open_session""#));
        let back: ClientMessage = serde_json::from_str(&json).unwrap();
        assert!(matches!(back, ClientMessage::OpenSession { shell: None }));
    }

    #[test]
    fn terminal_input_roundtrip_preserves_session_id() {
        let sid = SessionId::new();
        let msg = ClientMessage::TerminalInput {
            session_id: sid,
            data: "ZWNobw==".into(), // base64 "echo"
        };
        let json = serde_json::to_string(&msg).unwrap();
        let back: ClientMessage = serde_json::from_str(&json).unwrap();
        match back {
            ClientMessage::TerminalInput { session_id, data } => {
                assert_eq!(session_id, sid);
                assert_eq!(data, "ZWNobw==");
            }
            _ => panic!("decoded wrong variant"),
        }
    }

    #[test]
    fn close_session_json_shape() {
        let sid = SessionId::new();
        let msg = ClientMessage::CloseSession { session_id: sid };
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains(r#""type":"close_session""#));
    }

    #[test]
    fn report_variants_shape() {
        let sid = SessionId::new();
        let opened = ClientMessage::ReportSessionOpened { session_id: sid };
        assert!(serde_json::to_string(&opened)
            .unwrap()
            .contains(r#""type":"report_session_opened""#));

        let danger = ClientMessage::ReportDanger {
            session_id: sid,
            command: "rm -rf /tmp".into(),
            pattern: r"rm\s+-rf".into(),
        };
        let j = serde_json::to_string(&danger).unwrap();
        assert!(j.contains(r#""type":"report_danger""#));
        let back: ClientMessage = serde_json::from_str(&j).unwrap();
        match back {
            ClientMessage::ReportDanger { command, .. } => assert_eq!(command, "rm -rf /tmp"),
            _ => panic!("decoded wrong variant"),
        }
    }

    #[test]
    fn danger_warn_roundtrip() {
        let sid = SessionId::new();
        let msg = ServerMessage::DangerWarn {
            session_id: sid,
            command: "rm -rf /tmp".into(),
            pattern: r"rm\s+-rf".into(),
        };
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains(r#""type":"danger_warn""#));
        let back: ServerMessage = serde_json::from_str(&json).unwrap();
        match back {
            ServerMessage::DangerWarn { session_id, command, .. } => {
                assert_eq!(session_id, sid);
                assert_eq!(command, "rm -rf /tmp");
            }
            _ => panic!("decoded wrong variant"),
        }
    }

    #[test]
    fn auth_fail_shape() {
        let msg = ServerMessage::AuthFail {
            reason: "bad token".into(),
        };
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains(r#""type":"auth_fail""#));
        assert!(json.contains(r#""reason":"bad token""#));
    }

    #[test]
    fn desktop_online_offline_shape() {
        let online = ServerMessage::DesktopOnline {
            device_id: DeviceId::new("dev-ws"),
        };
        assert!(serde_json::to_string(&online)
            .unwrap()
            .contains(r#""type":"desktop_online""#));

        let offline = ServerMessage::DesktopOffline {
            device_id: DeviceId::new("dev-ws"),
        };
        assert!(serde_json::to_string(&offline)
            .unwrap()
            .contains(r#""type":"desktop_offline""#));
    }

    #[test]
    fn ping_pong_roundtrip() {
        let ping = ClientMessage::Ping { ts_ms: 1000 };
        let j = serde_json::to_string(&ping).unwrap();
        assert!(j.contains(r#""type":"ping""#));
        assert!(matches!(
            serde_json::from_str::<ClientMessage>(&j).unwrap(),
            ClientMessage::Ping { ts_ms: 1000 }
        ));

        let pong = ServerMessage::Pong {
            ts_ms: 1000,
            server_time_ms: 1005,
        };
        let j = serde_json::to_string(&pong).unwrap();
        assert!(j.contains(r#""type":"pong""#));
    }
}
