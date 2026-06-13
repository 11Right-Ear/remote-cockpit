//! Shared application state, cheaply cloneable via an `Arc` interior.

use std::sync::Arc;

use dashmap::DashMap;
use protocol::DeviceId;

use crate::audit::AuditHandle;
use crate::config::GatewayConfig;
use crate::sessions::DesktopLink;

/// Cloneable handle to shared gateway state.
#[derive(Clone)]
pub struct AppState {
    inner: Arc<AppStateInner>,
}

struct AppStateInner {
    config: GatewayConfig,
    audit: AuditHandle,
    desktops: DashMap<DeviceId, DesktopLink>,
}

impl AppState {
    pub fn new(config: GatewayConfig, audit: AuditHandle) -> Self {
        Self {
            inner: Arc::new(AppStateInner {
                config,
                audit,
                desktops: DashMap::new(),
            }),
        }
    }

    pub fn config(&self) -> &GatewayConfig {
        &self.inner.config
    }

    pub fn audit(&self) -> &AuditHandle {
        &self.inner.audit
    }

    /// Registered desktop agents by device id.
    pub fn desktops(&self) -> &DashMap<DeviceId, DesktopLink> {
        &self.inner.desktops
    }
}
