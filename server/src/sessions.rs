//! Per-connection outbound channel types and the desktop/phone link.
//!
//! The gateway is a router. Each long-lived socket owns a writer task that
//! drains an unbounded mpsc channel; other tasks push onto it. This keeps
//! sends non-blocking and avoids borrowing the socket across await points.

use std::sync::Mutex;

use tokio::sync::mpsc;

/// Message pushed to a desktop connection's writer.
#[derive(Debug)]
pub enum DesktopOut {
    /// A JSON text frame to forward to the desktop (e.g. a phone's
    /// `TerminalInput` / `OpenSession`).
    Text(String),
    /// Tell the writer to close the socket.
    Close,
}

/// Message pushed to a phone connection's writer.
#[derive(Debug)]
pub enum PhoneOut {
    /// A JSON text frame (`ServerMessage`).
    Text(String),
    /// A binary frame (raw tagged stream bytes, e.g. PTY output).
    Binary(Vec<u8>),
    /// Tell the writer to close the socket. (Reserved; not emitted in V1.)
    #[allow(dead_code)]
    Close,
}

/// A registered desktop agent plus its currently linked phone (V1: at most one
/// phone per desktop).
pub struct DesktopLink {
    /// Push messages to the desktop socket writer.
    pub desktop_tx: mpsc::UnboundedSender<DesktopOut>,
    /// Push messages to the linked phone socket writer (`None` while no phone
    /// is connected). Guarded by a Mutex so desktop + phone tasks can update
    /// it concurrently.
    pub phone_tx: Mutex<Option<mpsc::UnboundedSender<PhoneOut>>>,
}
