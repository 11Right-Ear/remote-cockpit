//! Desktop Agent connection: registers the agent, routes its frames.

use std::sync::Mutex;

use axum::extract::ws::{Message, WebSocket};
use futures_util::{SinkExt, StreamExt};
use protocol::{auth::Claims, ClientMessage, DeviceId, ServerMessage};
use tokio::sync::mpsc;

use crate::audit::event;
use crate::sessions::{DesktopLink, DesktopOut, PhoneOut};
use crate::state::AppState;

pub async fn run(socket: WebSocket, state: AppState, claims: Claims) -> anyhow::Result<()> {
    let device_id = DeviceId::new(claims.device_id.clone());
    let actor = format!("desktop:{}", claims.device_id);

    let (mut sender, mut receiver) = socket.split();
    let (desktop_tx, mut desktop_rx) = mpsc::unbounded_channel::<DesktopOut>();

    // Register this desktop, evicting any prior connection for the same device.
    {
        let link = DesktopLink {
            desktop_tx: desktop_tx.clone(),
            phone_tx: Mutex::new(None),
        };
        if let Some(old) = state.desktops().insert(device_id.clone(), link) {
            let _ = old.desktop_tx.send(DesktopOut::Close);
        }
    }
    state
        .audit()
        .record(event::DESKTOP_ONLINE, &actor, None, None);

    // Writer task: drain the desktop outbound channel onto the socket.
    let writer = tokio::spawn(async move {
        while let Some(out) = desktop_rx.recv().await {
            match out {
                DesktopOut::Text(t) => {
                    if sender.send(Message::Text(t)).await.is_err() {
                        break;
                    }
                }
                DesktopOut::Close => {
                    let _ = sender.close().await;
                    break;
                }
            }
        }
    });

    // Reader loop: route desktop frames.
    while let Some(msg) = receiver.next().await {
        match msg {
            Ok(Message::Text(t)) => {
                if let Err(e) = handle_text(&state, &device_id, &actor, t).await {
                    tracing::warn!(error = %e, "desktop text handling failed");
                }
            }
            Ok(Message::Binary(b)) => {
                // PTY output: forward to the linked phone as a binary frame.
                forward_to_phone(&state, &device_id, PhoneOut::Binary(b));
            }
            Ok(Message::Close(_)) | Err(_) => break,
            _ => {}
        }
    }

    // Cleanup.
    drop(desktop_tx);
    let _ = writer.await;
    state.desktops().remove(&device_id);
    state
        .audit()
        .record(event::DESKTOP_OFFLINE, &actor, None, None);
    Ok(())
}

/// Handle a text frame from the desktop: a report or a ping.
async fn handle_text(
    state: &AppState,
    device_id: &DeviceId,
    actor: &str,
    raw: String,
) -> anyhow::Result<()> {
    let msg: ClientMessage = serde_json::from_str(&raw)?;
    match msg {
        ClientMessage::Ping { ts_ms } => {
            let pong = ServerMessage::Pong {
                ts_ms,
                server_time_ms: chrono::Utc::now().timestamp_millis(),
            };
            forward_to_desktop(state, device_id, DesktopOut::Text(serde_json::to_string(&pong)?));
        }
        ClientMessage::ReportSessionOpened { session_id } => {
            let sid = session_id.0.to_string();
            state.audit().record(event::SESSION_OPEN, actor, Some(sid), None);
            let fwd = ServerMessage::SessionOpened { session_id };
            forward_to_phone(state, device_id, PhoneOut::Text(serde_json::to_string(&fwd)?));
        }
        ClientMessage::ReportSessionClosed { session_id } => {
            let sid = session_id.0.to_string();
            state.audit().record(event::SESSION_CLOSE, actor, Some(sid), None);
            let fwd = ServerMessage::SessionClosed { session_id };
            forward_to_phone(state, device_id, PhoneOut::Text(serde_json::to_string(&fwd)?));
        }
        ClientMessage::ReportSessionError { session_id, message } => {
            let fwd = ServerMessage::SessionError { session_id, message };
            forward_to_phone(state, device_id, PhoneOut::Text(serde_json::to_string(&fwd)?));
        }
        ClientMessage::ReportDanger { session_id, command, pattern } => {
            let sid = session_id.0.to_string();
            state.audit().record(
                event::DANGER_WARN,
                actor,
                Some(sid),
                Some(format!("cmd={command}; pattern={pattern}")),
            );
            let fwd = ServerMessage::DangerWarn { session_id, command, pattern };
            forward_to_phone(state, device_id, PhoneOut::Text(serde_json::to_string(&fwd)?));
        }
        ClientMessage::ReportDirListing { request_id, path, entries } => {
            // Phase 2 file browser: read-only dir listing. Not tied to a PTY
            // session, so audit by path only.
            state
                .audit()
                .record(event::DIR_LIST, actor, None, Some(format!("path={path}")));
            let fwd = ServerMessage::DirListing { request_id, path, entries };
            forward_to_phone(state, device_id, PhoneOut::Text(serde_json::to_string(&fwd)?));
        }
        ClientMessage::ReportFileContent {
            request_id,
            path,
            content,
            truncated,
            error,
        } => {
            // Phase 2 file browser: read-only file content. Audit the path
            // (and note an error if the read failed).
            state.audit().record(
                event::FILE_READ,
                actor,
                None,
                Some(match &error {
                    Some(e) => format!("path={path}; error={e}"),
                    None => format!("path={path}"),
                }),
            );
            let fwd = ServerMessage::FileContent {
                request_id,
                path,
                content,
                truncated,
                error,
            };
            forward_to_phone(state, device_id, PhoneOut::Text(serde_json::to_string(&fwd)?));
        }
        ClientMessage::ReportImageContent {
            request_id,
            path,
            mime_type,
            data_base64,
            truncated,
            error,
        } => {
            state.audit().record(
                event::IMAGE_READ,
                actor,
                None,
                Some(match &error {
                    Some(e) => format!("path={path}; error={e}"),
                    None => format!("path={path}"),
                }),
            );
            let fwd = ServerMessage::ImageContent {
                request_id,
                path,
                mime_type,
                data_base64,
                truncated,
                error,
            };
            forward_to_phone(state, device_id, PhoneOut::Text(serde_json::to_string(&fwd)?));
        }
        // Desktop should not send phone-originated messages.
        other => tracing::warn!(?other, "unexpected message from desktop"),
    }
    Ok(())
}

fn forward_to_phone(state: &AppState, device_id: &DeviceId, out: PhoneOut) {
    if let Some(link) = state.desktops().get(device_id) {
        if let Ok(phone_tx) = link.phone_tx.lock() {
            if let Some(tx) = phone_tx.as_ref() {
                let _ = tx.send(out);
            }
        }
    }
}

fn forward_to_desktop(state: &AppState, device_id: &DeviceId, out: DesktopOut) {
    if let Some(link) = state.desktops().get(device_id) {
        let _ = link.desktop_tx.send(out);
    }
}
