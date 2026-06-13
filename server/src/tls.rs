//! TLS: self-signed certificate bootstrap + rustls config for axum-server.
//!
//! V1 uses a self-signed cert generated on first run and cached to disk
//! (SECURITY.md: "no plaintext" -> `wss://` is mandatory). Phase 2 can replace
//! this with a real CA / `mkcert` workflow.

use std::path::Path;

use anyhow::Context;

/// Ensure a self-signed cert+key exist on disk, generating them on first run.
pub fn ensure_self_signed(cert_path: &Path, key_path: &Path) -> anyhow::Result<()> {
    if cert_path.exists() && key_path.exists() {
        return Ok(());
    }
    tracing::info!(cert = ?cert_path, "generating self-signed certificate");
    let cert = rcgen::generate_simple_self_signed(vec!["localhost".to_string()])
        .context("generate self-signed cert")?;
    std::fs::write(cert_path, cert.serialize_pem()?).context("write cert.pem")?;
    std::fs::write(key_path, cert.serialize_private_key_pem()).context("write key.pem")?;
    Ok(())
}

/// Build a rustls config from the on-disk cert+key.
pub async fn rustls_config(
    cert_path: &Path,
    key_path: &Path,
) -> anyhow::Result<axum_server::tls_rustls::RustlsConfig> {
    let cert = std::fs::read(cert_path).context("read cert")?;
    let key = std::fs::read(key_path).context("read key")?;
    Ok(axum_server::tls_rustls::RustlsConfig::from_pem(cert, key).await?)
}
