//! The agent's main connection loop: WebSocket <-> PTY <-> heartbeat.
//!
//! After authenticating, it waits for the phone (via the gateway) to open a
//! session. PTY output is pumped back as binary frames; phone input is written
//! to the PTY (with danger checks). Heartbeats keep the connection alive.

use std::io::Read;
use std::time::Duration;

use anyhow::Context;
use base64::Engine;
use futures_util::{SinkExt, StreamExt};
use tokio::sync::mpsc;
use tokio_tungstenite::tungstenite::Message;

use protocol::frame::{encode, stream_tag};
use protocol::{ClientMessage, ServerMessage, SessionId};

use crate::config::DesktopConfig;
use crate::conn::WsStream;
use crate::danger::DangerChecker;
use crate::pty::PtySession;

const HEARTBEAT_SECS: u64 = 30;
const TERM_COLS: u16 = 80;
const TERM_ROWS: u16 = 24;

/// Run one connection session to completion (auth + routing loop).
///
/// Returns `Ok(())` on a clean close, `Err` on a fatal error (caller decides
/// whether to reconnect).
pub async fn run(ws: WsStream, config: &DesktopConfig) -> anyhow::Result<()> {
    let (mut ws_tx, mut ws_rx) = ws.split();

    // --- Authenticate ---
    let auth = ClientMessage::Auth {
        token: config.token.clone(),
        role: protocol::ClientRole::Desktop,
        device_id: protocol::DeviceId::new(config.device_id.clone()),
    };
    ws_tx
        .send(Message::Text(serde_json::to_string(&auth)?))
        .await
        .context("send auth")?;

    let first = ws_rx
        .next()
        .await
        .ok_or_else(|| anyhow::anyhow!("gateway closed before auth response"))?
        .context("read auth response")?;
    let first_text = match first {
        Message::Text(t) => t,
        _ => anyhow::bail!("unexpected non-text auth response"),
    };
    match serde_json::from_str::<ServerMessage>(&first_text)? {
        ServerMessage::AuthOk { .. } => {}
        ServerMessage::AuthFail { reason } => anyhow::bail!("FATAL: authentication rejected: {reason}"),
        other => anyhow::bail!("expected AuthOk, got {other:?}"),
    }
    tracing::info!("authenticated; waiting for sessions");

    // --- State ---
    let danger = DangerChecker::new();
    let (pty_out_tx, mut pty_out_rx) = mpsc::channel::<Vec<u8>>(64);
    let mut pty: Option<PtySession> = None;
    let mut current_session: Option<SessionId> = None;
    // Accumulate input and check danger per line: xterm sends one char per
    // frame, so a per-frame check never matches multi-char patterns (rm -rf).
    let mut input_buffer: Vec<u8> = Vec::with_capacity(512);
    let mut heartbeat = tokio::time::interval(Duration::from_secs(HEARTBEAT_SECS));
    heartbeat.tick().await; // discard immediate first tick

    loop {
        tokio::select! {
            msg = ws_rx.next() => {
                let msg = match msg {
                    Some(Ok(m)) => m,
                    Some(Err(e)) => return Err(anyhow::anyhow!("ws read: {e}")),
                    None => return Ok(()),
                };
                match msg {
                    Message::Text(t) => {
                        // A text frame is either a phone-forwarded ClientMessage
                        // or a gateway ServerMessage (e.g. Pong). Try both.
                        match serde_json::from_str::<ClientMessage>(&t) {
                            Ok(m) => {
                                handle_text(
                                    &m, config, &mut pty, &mut current_session, &danger,
                                    &mut input_buffer, &mut ws_tx, &pty_out_tx,
                                ).await?;
                            }
                            Err(_) => match serde_json::from_str::<ServerMessage>(&t) {
                                Ok(ServerMessage::Pong { .. }) => { /* heartbeat acknowledged */ }
                                Ok(other) => tracing::warn!(?other, "unexpected server message"),
                                Err(e) => tracing::warn!(error=%e, "unparseable text frame"),
                            },
                        }
                    }
                    Message::Binary(_) => { /* desktop does not receive binary */ }
                    Message::Close(_) => return Ok(()),
                    _ => {}
                }
            }
            out = pty_out_rx.recv() => match out {
                Some(bytes) => {
                    let frame = encode(stream_tag::TERMINAL, &bytes);
                    if ws_tx.send(Message::Binary(frame)).await.is_err() {
                        return Ok(());
                    }
                }
                None => {
                    // All PTY-reader senders dropped => session ended.
                    if let Some(mut session) = pty.take() {
                        session.kill();
                    }
                    if let Some(sid) = current_session.take() {
                        let r = ClientMessage::ReportSessionClosed { session_id: sid };
                        let _ = ws_tx.send(Message::Text(serde_json::to_string(&r)?)).await;
                    }
                }
            },
            _ = heartbeat.tick() => {
                let ping = ClientMessage::Ping { ts_ms: chrono::Utc::now().timestamp_millis() };
                let _ = ws_tx.send(Message::Text(serde_json::to_string(&ping)?)).await;
            }
        }
    }
}

/// Dispatch a text frame received from the gateway.
async fn handle_text(
    m: &ClientMessage,
    config: &DesktopConfig,
    pty: &mut Option<PtySession>,
    current_session: &mut Option<SessionId>,
    danger: &DangerChecker,
    input_buffer: &mut Vec<u8>,
    ws_tx: &mut futures_util::stream::SplitSink<WsStream, Message>,
    pty_out_tx: &mpsc::Sender<Vec<u8>>,
) -> anyhow::Result<()> {
    match m {
        ClientMessage::OpenSession { shell } => {
            let shell = shell.clone().unwrap_or_else(|| config.shell.clone());
            match PtySession::spawn(&shell, TERM_COLS, TERM_ROWS) {
                Ok((session, reader)) => {
                    if let Some(mut old) = pty.take() {
                        old.kill();
                    }
                    let sid = SessionId::new();
                    *current_session = Some(sid);
                    *pty = Some(session);

                    // Pump PTY output into the channel from a blocking task.
                    let tx = pty_out_tx.clone();
                    tokio::task::spawn_blocking(move || {
                        let mut reader = reader;
                        let mut buf = [0u8; 8192];
                        loop {
                            match reader.read(&mut buf) {
                                Ok(0) => break,
                                Ok(n) => {
                                    if tx.blocking_send(buf[..n].to_vec()).is_err() {
                                        break;
                                    }
                                }
                                Err(_) => break,
                            }
                        }
                    });

                    let r = ClientMessage::ReportSessionOpened { session_id: sid };
                    send(ws_tx, &r).await?;
                    tracing::info!(session_id = %sid.0, "session opened");
                }
                Err(e) => {
                    tracing::error!(error=%e, "pty spawn failed");
                    let sid = SessionId::new();
                    let r = ClientMessage::ReportSessionError {
                        session_id: sid,
                        message: format!("spawn failed: {e}"),
                    };
                    send(ws_tx, &r).await?;
                }
            }
        }
        ClientMessage::TerminalInput { data, .. } => {
            let bytes = match base64::engine::general_purpose::STANDARD.decode(data) {
                Ok(b) => b,
                Err(e) => {
                    tracing::warn!(error=%e, "bad base64 terminal input");
                    return Ok(());
                }
            };
            // Write to the PTY first so echo/execution stay real-time.
            if let Some(session) = pty.as_mut() {
                if let Err(e) = session.write(&bytes) {
                    tracing::warn!(error=%e, "pty write failed");
                }
            }
            // Accumulate and check danger per line (xterm sends one char per
            // frame, so a per-frame check never matches multi-char patterns).
            input_buffer.extend_from_slice(&bytes);
            if bytes.contains(&b'\r') || bytes.contains(&b'\n') {
                if let Some(hit) = danger.check(input_buffer) {
                    let sid = current_session.unwrap_or_else(SessionId::new);
                    let r = ClientMessage::ReportDanger {
                        session_id: sid,
                        command: hit.command,
                        pattern: hit.pattern,
                    };
                    send(ws_tx, &r).await?;
                }
                input_buffer.clear();
            }
        }
        ClientMessage::TerminalResize { cols, rows, .. } => {
            if let Some(session) = pty.as_ref() {
                let _ = session.resize(*cols, *rows);
            }
        }
        ClientMessage::CloseSession { session_id } => {
            if let Some(mut session) = pty.take() {
                session.kill();
                *current_session = None;
                let r = ClientMessage::ReportSessionClosed {
                    session_id: *session_id,
                };
                send(ws_tx, &r).await?;
            }
        }
        ClientMessage::ListDir { request_id, path } => {
            let request_id = request_id.clone();
            let req_path = path.clone();
            // Filesystem I/O is blocking — run it off the async runtime.
            let result =
                tokio::task::spawn_blocking(move || crate::fsbrowse::list_dir(&req_path)).await;
            match result {
                Ok(Ok((canon, entries))) => {
                    // Strip the Windows `\\?\` verbatim prefix for display.
                    let path = canon
                        .to_string_lossy()
                        .strip_prefix(r"\\?\")
                        .map(str::to_owned)
                        .unwrap_or_else(|| canon.to_string_lossy().into_owned());
                    let r = ClientMessage::ReportDirListing {
                        request_id,
                        path,
                        entries,
                    };
                    send(ws_tx, &r).await?;
                }
                Ok(Err(e)) => {
                    // Unreadable / outside jail: report an empty listing for the
                    // requested path so the phone doesn't hang. The gateway
                    // still audits the attempt by path.
                    tracing::warn!(error = %e, path = %path, "list_dir failed");
                    let r = ClientMessage::ReportDirListing {
                        request_id,
                        path: path.clone(),
                        entries: Vec::new(),
                    };
                    send(ws_tx, &r).await?;
                }
                Err(e) => tracing::error!(error = %e, "list_dir task panicked"),
            }
        }
        other => tracing::warn!(?other, "unexpected message from gateway"),
    }
    Ok(())
}

/// Serialize and send a `ClientMessage` text frame.
async fn send(
    ws_tx: &mut futures_util::stream::SplitSink<WsStream, Message>,
    msg: &ClientMessage,
) -> anyhow::Result<()> {
    ws_tx
        .send(Message::Text(serde_json::to_string(msg)?))
        .await?;
    Ok(())
}
