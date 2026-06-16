//! Phone connection: links to a desktop agent, routes its frames.

use axum::extract::ws::{Message, WebSocket};
use futures_util::{SinkExt, StreamExt};
use protocol::{auth::Claims, ClientMessage, DeviceId, ServerMessage};
use tokio::sync::mpsc;

use crate::sessions::{DesktopOut, PhoneOut};
use crate::state::AppState;

pub async fn run(socket: WebSocket, state: AppState, claims: Claims) -> anyhow::Result<()> {
    let device_id = DeviceId::new(claims.device_id.clone());
    let (mut sender, mut receiver) = socket.split();
    let (phone_tx, mut phone_rx) = mpsc::unbounded_channel::<PhoneOut>();

    // Link this phone to the desktop agent for the target device (if online).
    let desktop_online = {
        if let Some(link) = state.desktops().get(&device_id) {
            let mut guard = link.phone_tx.lock().unwrap();
            *guard = Some(phone_tx.clone());
            true
        } else {
            false
        }
    };

    if desktop_online {
        let online = ServerMessage::DesktopOnline {
            device_id: device_id.clone(),
        };
        sender
            .send(Message::Text(serde_json::to_string(&online)?))
            .await?;
    } else {
        // No desktop registered: tell the phone and end. V1 does not wait.
        let offline = ServerMessage::DesktopOffline {
            device_id: device_id.clone(),
        };
        sender
            .send(Message::Text(serde_json::to_string(&offline)?))
            .await?;
        let _ = sender.close().await;
        return Ok(());
    }

    // Writer task: drain the phone outbound channel onto the socket.
    let writer = tokio::spawn(async move {
        while let Some(out) = phone_rx.recv().await {
            match out {
                PhoneOut::Text(t) => {
                    if sender.send(Message::Text(t)).await.is_err() {
                        break;
                    }
                }
                PhoneOut::Binary(b) => {
                    if sender.send(Message::Binary(b)).await.is_err() {
                        break;
                    }
                }
                PhoneOut::Close => {
                    let _ = sender.close().await;
                    break;
                }
            }
        }
    });

    // Reader loop: route phone frames.
    while let Some(msg) = receiver.next().await {
        match msg {
            Ok(Message::Text(t)) => {
                if let Err(e) = handle_text(&state, &device_id, t).await {
                    tracing::warn!(error = %e, "phone text handling failed");
                }
            }
            Ok(Message::Close(_)) | Err(_) => break,
            _ => {}
        }
    }

    // Cleanup: unlink the phone (only if it is still us), then stop the writer
    // by dropping our sender.
    if let Some(link) = state.desktops().get(&device_id) {
        let mut guard = link.phone_tx.lock().unwrap();
        if guard
            .as_ref()
            .map(|tx| tx.same_channel(&phone_tx))
            .unwrap_or(false)
        {
            *guard = None;
        }
    }
    drop(phone_tx);
    let _ = writer.await;
    Ok(())
}

/// Handle a text frame from the phone: a ping or a desktop-bound command.
async fn handle_text(
    state: &AppState,
    device_id: &DeviceId,
    raw: String,
) -> anyhow::Result<()> {
    let msg: ClientMessage = serde_json::from_str(&raw)?;
    match msg {
        ClientMessage::Ping { ts_ms } => {
            let pong = ServerMessage::Pong {
                ts_ms,
                server_time_ms: chrono::Utc::now().timestamp_millis(),
            };
            if let Some(link) = state.desktops().get(device_id) {
                if let Some(tx) = link.phone_tx.lock().unwrap().as_ref() {
                    let _ = tx.send(PhoneOut::Text(serde_json::to_string(&pong)?));
                }
            }
        }
        // Forward phone-originated terminal commands transparently to desktop.
        ClientMessage::TerminalInput { .. }
        | ClientMessage::TerminalResize { .. }
        | ClientMessage::OpenSession { .. }
        | ClientMessage::CloseSession { .. }
        | ClientMessage::ListDir { .. }
        | ClientMessage::ReadFile { .. } => {
            if let Some(link) = state.desktops().get(device_id) {
                let _ = link.desktop_tx.send(DesktopOut::Text(raw));
            }
        }
        other => tracing::warn!(?other, "unexpected message from phone"),
    }
    Ok(())
}
