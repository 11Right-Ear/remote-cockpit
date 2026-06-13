//! Shared wire protocol for remote-cockpit.
//!
//! See `docs/decisions/0001-websocket-protocol-and-rust.md` for the rationale:
//! - text frames  = JSON control-plane envelopes ([`ClientMessage`] /
//!   [`ServerMessage`])
//! - binary frames = tagged raw streams ([`frame`])
//!
//! This crate is intentionally pure data with no I/O, so it can be unit-tested
//! without an async runtime.

pub mod frame;
pub mod message;
pub mod session;

pub use frame::{decode, encode, stream_tag};
pub use message::{ClientMessage, ServerMessage};
pub use session::{ClientRole, DeviceId, SessionId};
