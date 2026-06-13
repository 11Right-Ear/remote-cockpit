//! WebSocket upgrade + first-frame authentication + role dispatch.

use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::Response,
};
use futures_util::StreamExt;

use protocol::ClientMessage;

use crate::auth::{role_from_claims, verify_token};
use crate::audit::event;
use crate::state::AppState;

/// axum route handler: upgrade to WebSocket, then run the connection.
pub async fn ws_handler(ws: WebSocketUpgrade, State(state): State<AppState>) -> Response {
    ws.on_upgrade(move |socket| async move {
        if let Err(e) = handle(socket, state).await {
            tracing::warn!(error = %e, "connection ended with error");
        }
    })
}

async fn handle(mut socket: WebSocket, state: AppState) -> anyhow::Result<()> {
    // The first frame must be Auth.
    let first = socket
        .next()
        .await
        .ok_or_else(|| anyhow::anyhow!("connection closed before auth"))??;

    let text = match first {
        Message::Text(t) => t,
        _ => {
            finish_authfail(&mut socket, "first frame must be Auth (text)").await?;
            return Ok(());
        }
    };

    let msg: ClientMessage = serde_json::from_str(&text)?;
    let (token, claimed_device, role) = match msg {
        ClientMessage::Auth {
            token,
            device_id,
            role,
        } => (token, device_id, role),
        _ => {
            finish_authfail(&mut socket, "first frame must be Auth").await?;
            return Ok(());
        }
    };

    // Verify the JWT.
    let secret = state.config().jwt_secret.clone();
    let claims = match verify_token(&secret, &token) {
        Ok(c) => c,
        Err(e) => {
            let reason = format!("invalid token: {e}");
            state.audit().record(
                event::AUTH_FAIL,
                format!("?:{}", claimed_device.as_ref()),
                None,
                Some(reason.clone()),
            );
            finish_authfail(&mut socket, &reason).await?;
            return Ok(());
        }
    };

    // Token role must match the declared role.
    let token_role = role_from_claims(&claims)?;
    if token_role != role {
        finish_authfail(&mut socket, "role mismatch between token and declared role").await?;
        return Ok(());
    }

    state.audit().record(
        event::AUTH_OK,
        format!("{}:{}", claims.role, claims.device_id),
        None,
        None,
    );

    // Acknowledge authentication.
    let auth_ok = protocol::ServerMessage::AuthOk {
        session_id: protocol::SessionId::new(),
        server_time_ms: chrono::Utc::now().timestamp_millis(),
    };
    socket
        .send(Message::Text(serde_json::to_string(&auth_ok)?))
        .await?;

    match role {
        protocol::ClientRole::Desktop => {
            crate::ws::desktop_conn::run(socket, state, claims).await
        }
        protocol::ClientRole::Phone => crate::ws::phone_conn::run(socket, state, claims).await,
    }
}

/// Send an `AuthFail`; the socket drops (closing the connection) when the
/// caller returns. `WebSocket::close` consumes `self`, so we cannot call it
/// through a `&mut` borrow — dropping is sufficient.
async fn finish_authfail(socket: &mut WebSocket, reason: &str) -> anyhow::Result<()> {
    let msg = protocol::ServerMessage::AuthFail {
        reason: reason.to_string(),
    };
    socket
        .send(Message::Text(serde_json::to_string(&msg)?))
        .await?;
    Ok(())
}
