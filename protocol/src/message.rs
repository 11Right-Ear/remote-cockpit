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

/// One entry in a directory listing (file or subdirectory). Phase 2 file
/// browser carries only read-only metadata; no content in V1.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DirEntry {
    pub name: String,
    pub is_dir: bool,
    pub size: u64,
    pub modified_ms: i64,
}

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

    /// Phone asks the desktop to list a directory (Phase 2 file browser, V1
    /// read-only). `request_id` correlates the async `DirListing` response.
    /// Phone -> desktop (routed).
    ListDir {
        request_id: String,
        path: String,
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

    /// Desktop's response to a `ListDir` request. Desktop -> gateway; the
    /// gateway forwards the equivalent `ServerMessage::DirListing` to the phone.
    ReportDirListing {
        request_id: String,
        path: String,
        entries: Vec<DirEntry>,
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

    /// Read-only directory listing (Phase 2 file browser). Response to a
    /// phone's `list_dir` request, correlated by `request_id`.
    DirListing {
        request_id: String,
        path: String,
        entries: Vec<DirEntry>,
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

    #[test]
    fn list_dir_roundtrip() {
        let msg = ClientMessage::ListDir {
            request_id: "req-1".into(),
            path: "/home/dev/project".into(),
        };
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains(r#""type":"list_dir""#));
        assert!(json.contains(r#""request_id":"req-1""#));
        let back: ClientMessage = serde_json::from_str(&json).unwrap();
        match back {
            ClientMessage::ListDir { request_id, path } => {
                assert_eq!(request_id, "req-1");
                assert_eq!(path, "/home/dev/project");
            }
            _ => panic!("decoded wrong variant"),
        }
    }

    #[test]
    fn dir_listing_roundtrip() {
        let entries = vec![
            DirEntry {
                name: "src".into(),
                is_dir: true,
                size: 0,
                modified_ms: 1_700_000_000_000,
            },
            DirEntry {
                name: "README.md".into(),
                is_dir: false,
                size: 42,
                modified_ms: 1_700_000_001_000,
            },
        ];
        // desktop -> gateway report
        let report = ClientMessage::ReportDirListing {
            request_id: "req-1".into(),
            path: "/home/dev/project".into(),
            entries: entries.clone(),
        };
        let j = serde_json::to_string(&report).unwrap();
        assert!(j.contains(r#""type":"report_dir_listing""#));
        let back: ClientMessage = serde_json::from_str(&j).unwrap();
        match back {
            ClientMessage::ReportDirListing { request_id, entries, .. } => {
                assert_eq!(request_id, "req-1");
                assert_eq!(entries.len(), 2);
                assert!(entries[0].is_dir);
            }
            _ => panic!("decoded wrong variant"),
        }

        // gateway -> phone response
        let resp = ServerMessage::DirListing {
            request_id: "req-1".into(),
            path: "/home/dev/project".into(),
            entries,
        };
        let j = serde_json::to_string(&resp).unwrap();
        assert!(j.contains(r#""type":"dir_listing""#));
        let back: ServerMessage = serde_json::from_str(&j).unwrap();
        match back {
            ServerMessage::DirListing { request_id, entries, .. } => {
                assert_eq!(request_id, "req-1");
                assert_eq!(entries.len(), 2);
                assert_eq!(entries[1].name, "README.md");
            }
            _ => panic!("decoded wrong variant"),
        }
    }
}
