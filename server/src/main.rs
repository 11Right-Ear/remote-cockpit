//! rc-gateway: the Backend Gateway.
//!
//! Public TLS WebSocket entry point. Authenticates connections, classifies them
//! as desktop or phone, and routes terminal traffic between them. Heavy work
//! (PTY, file, ROS2) lives in the Desktop Agent — the gateway only relays.

mod audit;
mod auth;
mod config;
mod sessions;
mod state;
mod tls;
mod ws;

use std::net::SocketAddr;

use axum::{routing::get, Router};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info,server=debug")),
        )
        .init();

    let config = config::GatewayConfig::from_env()?;
    tracing::info!(bind = %config.bind_addr, "starting rc-gateway");

    let audit = audit::spawn(&config.db_path)?;
    tls::ensure_self_signed(&config.cert_path, &config.key_path)?;
    let tls_config = tls::rustls_config(&config.cert_path, &config.key_path).await?;

    let state = state::AppState::new(config.clone(), audit);

    let app = Router::new()
        .route("/ws", get(ws::handler::ws_handler))
        .route("/health", get(health))
        .with_state(state);

    let addr: SocketAddr = config.bind_addr.parse()?;
    tracing::info!("listening on wss://{addr}");
    axum_server::bind_rustls(addr, tls_config)
        .serve(app.into_make_service())
        .await?;
    Ok(())
}

async fn health() -> &'static str {
    "ok"
}
