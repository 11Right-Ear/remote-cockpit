//! Binary-frame stream tagging.
//!
//! ## Transport rule (ADR 0001)
//! - **Text frames** carry a JSON control-plane envelope (`ClientMessage` /
//!   `ServerMessage`).
//! - **Binary frames** carry raw stream bytes, prefixed with a 1-byte stream
//!   tag, so the wire format can grow new streams (files, images) later
//!   without a protocol redesign.
//!
//! Bulk terminal output rides binary frames (zero-copy, no base64 tax). Small
//! or out-of-band payloads use the `TerminalOutput` text message instead.

/// Stream-tag constants: the first byte of every binary frame.
pub mod stream_tag {
    /// Terminal stdout/stderr (multiplexed by the PTY into one stream).
    pub const TERMINAL: u8 = 0x01;
}

/// Wrap raw stream bytes with their 1-byte tag: `[tag][payload...]`.
pub fn encode(tag: u8, payload: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(1 + payload.len());
    out.push(tag);
    out.extend_from_slice(payload);
    out
}

/// Split a tagged binary frame into `(stream_tag, payload)`.
///
/// Returns `None` if the frame is empty (no tag byte).
pub fn decode(frame: &[u8]) -> Option<(u8, &[u8])> {
    let (&tag, rest) = frame.split_first()?;
    Some((tag, rest))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip() {
        let payload = b"hello world";
        let encoded = encode(stream_tag::TERMINAL, payload);
        let (tag, decoded) = decode(&encoded).unwrap();
        assert_eq!(tag, stream_tag::TERMINAL);
        assert_eq!(decoded, payload);
    }

    #[test]
    fn empty_frame_is_none() {
        assert!(decode(&[]).is_none());
    }

    #[test]
    fn tag_only_frame_has_empty_payload() {
        let encoded = encode(stream_tag::TERMINAL, b"");
        let (tag, decoded) = decode(&encoded).unwrap();
        assert_eq!(tag, stream_tag::TERMINAL);
        assert!(decoded.is_empty());
    }
}
