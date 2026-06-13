//! JWT claims for authentication (ADR 0001: shared-secret HS256).
//!
//! This module holds only the claim shape — pure data, no crypto dependency.
//! Signing lives in `phone-sim` (and dev tooling); verification lives in the
//! gateway (`server`). Both share this type so the two sides agree on the
//! token contract.

use serde::{Deserialize, Serialize};

/// JWT claims. V1 uses a single shared secret; per-device key agreement is
/// deferred to Phase 2 (see ADR 0001).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    /// Device this token is bound to.
    pub device_id: String,
    /// `"desktop"` or `"phone"`.
    pub role: String,
    /// Expiry, Unix seconds.
    pub exp: usize,
}
