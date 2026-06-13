//! rc-desktop: the Desktop Agent.
//!
//! Connects to the gateway, authenticates, and bridges the phone's terminal
//! I/O to a local PTY. Reconnects with exponential backoff on transient
//! disconnects; exits on a fatal auth failure.

mod agent_loop;
mod config;
mod conn;
mod danger;
mod pty;

use std::time::Duration;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info,desktop=debug")),
        )
        .init();

    let config = config::DesktopConfig::from_env()?;
    tracing::info!(
        gateway = %config.gateway_url,
        device = %config.device_id,
        shell = %config.shell,
        "starting rc-desktop"
    );

    let mut backoff = Duration::from_millis(250);
    loop {
        let ws = match conn::connect(&config.gateway_url).await {
            Ok(ws) => ws,
            Err(e) => {
                tracing::warn!(error = %e, backoff_ms = backoff.as_millis(), "connect failed; retrying");
                tokio::time::sleep(backoff).await;
                backoff = (backoff * 2).min(Duration::from_secs(30));
                continue;
            }
        };

        // Reset backoff once we establish a connection.
        backoff = Duration::from_millis(250);

        match agent_loop::run(ws, &config).await {
            Ok(()) => tracing::info!("session closed cleanly; reconnecting"),
            Err(e) => {
                let msg = e.to_string();
                if let Some(reason) = msg.strip_prefix("FATAL:") {
                    tracing::error!(reason = reason.trim(), "fatal error; not reconnecting");
                    return Err(e);
                }
                tracing::warn!(error = %e, "session ended; reconnecting");
            }
        }

        tokio::time::sleep(backoff).await;
        backoff = (backoff * 2).min(Duration::from_secs(30));
    }
}
