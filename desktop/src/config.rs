//! Desktop Agent configuration, loaded from environment variables.

#[derive(Debug, Clone)]
pub struct DesktopConfig {
    /// Gateway WebSocket URL, e.g. `wss://127.0.0.1:8443/ws`.
    pub gateway_url: String,
    /// JWT authenticating this desktop agent.
    pub token: String,
    /// Device identifier registered with the gateway.
    pub device_id: String,
    /// Shell to spawn in the PTY by default.
    pub shell: String,
}

impl DesktopConfig {
    /// Load from environment.
    ///
    /// Required: `RC_TOKEN`.
    /// Optional: `RC_GATEWAY_URL` (default `wss://127.0.0.1:8443/ws`),
    ///           `RC_DEVICE_ID` (default = hostname), `RC_SHELL` (default OS shell).
    pub fn from_env() -> anyhow::Result<Self> {
        let gateway_url = std::env::var("RC_GATEWAY_URL")
            .unwrap_or_else(|_| "wss://127.0.0.1:8443/ws".to_string());
        let token = std::env::var("RC_TOKEN")
            .map_err(|_| anyhow::anyhow!("RC_TOKEN env var is required (desktop JWT)"))?;
        let device_id = std::env::var("RC_DEVICE_ID").unwrap_or_else(|_| default_device_id());
        let shell = std::env::var("RC_SHELL").unwrap_or_else(|_| default_shell());
        Ok(Self {
            gateway_url,
            token,
            device_id,
            shell,
        })
    }
}

fn default_device_id() -> String {
    std::env::var("COMPUTERNAME")
        .or_else(|_| std::env::var("HOSTNAME"))
        .unwrap_or_else(|_| "desktop".to_string())
}

fn default_shell() -> String {
    if cfg!(windows) {
        // PowerShell is the sane default on modern Windows; fall back to COMSPEC.
        std::env::var("COMSPEC").unwrap_or_else(|_| "powershell.exe".to_string())
    } else {
        std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".to_string())
    }
}
