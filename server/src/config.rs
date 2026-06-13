//! Gateway configuration, loaded from environment variables.

use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct GatewayConfig {
    /// Socket address to bind the TLS WebSocket server.
    pub bind_addr: String,
    /// Shared HS256 secret used to sign/verify JWTs.
    pub jwt_secret: String,
    /// Path to the SQLite audit database.
    pub db_path: PathBuf,
    /// Path to the TLS certificate (PEM).
    pub cert_path: PathBuf,
    /// Path to the TLS private key (PEM).
    pub key_path: PathBuf,
}

impl GatewayConfig {
    /// Load configuration from environment variables.
    ///
    /// Required: `RC_GATEWAY_SECRET`.
    /// Optional: `RC_BIND_ADDR` (default `127.0.0.1:8443`),
    ///           `RC_DATA_DIR` (default `./.remote-cockpit`).
    pub fn from_env() -> anyhow::Result<Self> {
        let bind_addr =
            std::env::var("RC_BIND_ADDR").unwrap_or_else(|_| "127.0.0.1:8443".to_string());

        let jwt_secret = std::env::var("RC_GATEWAY_SECRET").map_err(|_| {
            anyhow::anyhow!("RC_GATEWAY_SECRET env var is required (JWT signing secret)")
        })?;

        let data_dir = std::env::var("RC_DATA_DIR").map_or_else(
            |_| {
                std::env::current_dir()
                    .unwrap_or_else(|_| PathBuf::from("."))
                    .join(".remote-cockpit")
            },
            PathBuf::from,
        );
        std::fs::create_dir_all(&data_dir).ok();

        Ok(Self {
            bind_addr,
            jwt_secret,
            db_path: data_dir.join("audit.db"),
            cert_path: data_dir.join("cert.pem"),
            key_path: data_dir.join("key.pem"),
        })
    }
}
