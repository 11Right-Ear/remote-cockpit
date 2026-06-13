//! phone-sim: a minimal client that simulates the Android app for testing.
//!
//! Two subcommands:
//!   phone-sim gen-token --device-id X --role phone [--secret S]
//!   phone-sim run --url wss://127.0.0.1:8443/ws --token T --device-id X
//!
//! `run` authenticates, opens a PTY session, forwards stdin lines as terminal
//! input, and prints received text/binary frames (PTY output goes to stdout,
//! control messages and danger warnings to stderr).

use std::io::Write;
use std::sync::Arc;

use anyhow::{Context, Result};
use base64::Engine;
use clap::{Parser, Subcommand};
use futures_util::{SinkExt, StreamExt};
use jsonwebtoken::{EncodingKey, Header};
use protocol::auth::Claims;
use protocol::frame::{decode, stream_tag};
use protocol::{ClientMessage, ClientRole, DeviceId, ServerMessage, SessionId};
use rustls::client::danger::{HandshakeSignatureValid, ServerCertVerified, ServerCertVerifier};
use rustls::pki_types::{CertificateDer, ServerName, UnixTime};
use rustls::{ClientConfig, DigitallySignedStruct, SignatureScheme};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio_tungstenite::{
    connect_async_tls_with_config,
    tungstenite::client::IntoClientRequest,
    tungstenite::Message,
    Connector,
};

#[derive(Parser)]
#[command(name = "phone-sim", about = "Simulated phone client for remote-cockpit testing")]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Generate a JWT signed with the shared secret.
    GenToken {
        #[arg(long)]
        device_id: String,
        #[arg(long, value_parser = parse_role)]
        role: ClientRole,
        #[arg(long, env = "RC_GATEWAY_SECRET", default_value = "dev-secret")]
        secret: String,
    },
    /// Connect to the gateway and run an interactive session.
    Run {
        #[arg(long)]
        url: String,
        #[arg(long)]
        token: String,
        #[arg(long)]
        device_id: String,
    },
}

fn parse_role(s: &str) -> std::result::Result<ClientRole, String> {
    match s {
        "desktop" => Ok(ClientRole::Desktop),
        "phone" => Ok(ClientRole::Phone),
        _ => Err(format!("role must be 'desktop' or 'phone', got {s}")),
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    match Cli::parse().cmd {
        Cmd::GenToken { device_id, role, secret } => {
            println!("{}", issue_token(&secret, &device_id, role)?);
            Ok(())
        }
        Cmd::Run { url, token, device_id } => run(&url, &token, &device_id).await,
    }
}

fn issue_token(secret: &str, device_id: &str, role: ClientRole) -> Result<String> {
    let exp = (chrono::Utc::now() + chrono::Duration::hours(24)).timestamp() as usize;
    let claims = Claims {
        device_id: device_id.to_string(),
        role: match role {
            ClientRole::Desktop => "desktop",
            ClientRole::Phone => "phone",
        }
        .to_string(),
        exp,
    };
    Ok(jsonwebtoken::encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )?)
}

// --- dev-only insecure TLS (accepts the gateway's self-signed cert) ---

#[derive(Debug)]
struct AcceptAllVerifier;

impl ServerCertVerifier for AcceptAllVerifier {
    fn verify_server_cert(
        &self,
        _: &CertificateDer<'_>,
        _: &[CertificateDer<'_>],
        _: &ServerName<'_>,
        _: &[u8],
        _: UnixTime,
    ) -> std::result::Result<ServerCertVerified, rustls::Error> {
        Ok(ServerCertVerified::assertion())
    }
    fn verify_tls12_signature(
        &self,
        _: &[u8],
        _: &CertificateDer<'_>,
        _: &DigitallySignedStruct,
    ) -> std::result::Result<HandshakeSignatureValid, rustls::Error> {
        Ok(HandshakeSignatureValid::assertion())
    }
    fn verify_tls13_signature(
        &self,
        _: &[u8],
        _: &CertificateDer<'_>,
        _: &DigitallySignedStruct,
    ) -> std::result::Result<HandshakeSignatureValid, rustls::Error> {
        Ok(HandshakeSignatureValid::assertion())
    }
    fn supported_verify_schemes(&self) -> Vec<SignatureScheme> {
        rustls::crypto::aws_lc_rs::default_provider()
            .signature_verification_algorithms
            .supported_schemes()
    }
}

fn insecure_connector() -> Connector {
    let config = ClientConfig::builder()
        .dangerous()
        .with_custom_certificate_verifier(Arc::new(AcceptAllVerifier))
        .with_no_client_auth();
    Connector::Rustls(Arc::new(config))
}

type WsStream =
    tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>;

async fn connect(url: &str) -> Result<WsStream> {
    let request = url.into_client_request()?;
    let (ws, _resp) =
        connect_async_tls_with_config(request, None, false, Some(insecure_connector())).await?;
    Ok(ws)
}

// --- interactive run ---

async fn run(url: &str, token: &str, device_id: &str) -> Result<()> {
    let ws = connect(url).await.context("connect")?;
    let (mut ws_tx, mut ws_rx) = ws.split();

    // Authenticate.
    let auth = ClientMessage::Auth {
        token: token.to_string(),
        role: ClientRole::Phone,
        device_id: DeviceId::new(device_id.to_string()),
    };
    ws_tx
        .send(Message::Text(serde_json::to_string(&auth)?))
        .await?;

    let first = ws_rx
        .next()
        .await
        .ok_or_else(|| anyhow::anyhow!("closed before auth response"))??;
    let t = match first {
        Message::Text(t) => t,
        _ => anyhow::bail!("non-text auth response"),
    };
    match serde_json::from_str::<ServerMessage>(&t)? {
        ServerMessage::AuthOk { .. } => {}
        ServerMessage::AuthFail { reason } => anyhow::bail!("auth failed: {reason}"),
        other => anyhow::bail!("expected AuthOk, got {other:?}"),
    }

    // Open a PTY session; wait for SessionOpened (DesktopOnline may arrive first).
    ws_tx
        .send(Message::Text(serde_json::to_string(&ClientMessage::OpenSession {
            shell: None,
        })?))
        .await?;

    let mut session_id: Option<SessionId> = None;
    while session_id.is_none() {
        let msg = ws_rx
            .next()
            .await
            .ok_or_else(|| anyhow::anyhow!("closed before session opened"))??;
        if let Message::Text(t) = msg {
            match serde_json::from_str::<ServerMessage>(&t) {
                Ok(ServerMessage::SessionOpened { session_id: sid }) => session_id = Some(sid),
                Ok(ServerMessage::DesktopOnline { .. }) => {}
                Ok(other) => eprintln!("[msg] {other:?}"),
                Err(e) => eprintln!("[parse] {e}"),
            }
        }
    }
    let sid = session_id.unwrap();
    eprintln!("[session opened: {}]", sid.0);
    eprintln!("Type commands. Ctrl-D (or empty line x2) to exit.");

    let stdin = tokio::io::stdin();
    let mut lines = BufReader::new(stdin).lines();
    let mut stdin_done = false;

    loop {
        if stdin_done {
            // stdin closed: drain remaining PTY output / SessionClosed so the
            // echo output and danger warnings are not lost on piped input.
            match tokio::time::timeout(std::time::Duration::from_secs(5), ws_rx.next()).await {
                Ok(Some(Ok(msg))) => {
                    if handle_ws_msg(msg).await? {
                        break;
                    }
                }
                _ => break, // closed, error, or drain timeout
            }
        } else {
            tokio::select! {
                line = lines.next_line() => {
                    match line.context("stdin")? {
                        Some(l) => {
                            let mut bytes = l.into_bytes();
                            // CRLF: cmd.exe needs CR to execute a line; bash
                            // tolerates the extra CR. Most portable choice.
                            bytes.extend_from_slice(b"\r\n");
                            let data = base64::engine::general_purpose::STANDARD.encode(&bytes);
                            let msg = ClientMessage::TerminalInput { session_id: sid, data };
                            ws_tx.send(Message::Text(serde_json::to_string(&msg)?)).await?;
                        }
                        None => { stdin_done = true; }
                    }
                }
                msg = ws_rx.next() => {
                    let msg = match msg {
                        Some(Ok(m)) => m,
                        Some(Err(e)) => return Err(anyhow::anyhow!("ws read: {e}")),
                        None => break,
                    };
                    if handle_ws_msg(msg).await? {
                        break;
                    }
                }
            }
        }
    }
    Ok(())
}

/// Handle one inbound ws frame. Returns `Ok(true)` if the connection should end.
async fn handle_ws_msg(msg: Message) -> Result<bool> {
    match msg {
        Message::Text(t) => match serde_json::from_str::<ServerMessage>(&t) {
            Ok(ServerMessage::DangerWarn { command, pattern, .. }) => {
                eprintln!("\n[DANGER] pattern={pattern}  cmd={command}");
            }
            Ok(ServerMessage::SessionClosed { .. }) => {
                eprintln!("\n[session closed]");
                return Ok(true);
            }
            Ok(ServerMessage::DesktopOffline { .. }) => {
                eprintln!("\n[desktop offline]");
            }
            Ok(other) => eprintln!("\n[msg] {other:?}"),
            Err(e) => eprintln!("\n[parse] {e}: {t}"),
        },
        Message::Binary(b) => {
            if let Some((tag, payload)) = decode(&b) {
                if tag == stream_tag::TERMINAL {
                    let _ = std::io::stdout().write_all(&payload);
                    let _ = std::io::stdout().flush();
                }
            }
        }
        Message::Close(_) => return Ok(true),
        _ => {}
    }
    Ok(false)
}
