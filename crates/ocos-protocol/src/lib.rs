//! Control-plane messages between compute nodes.
//!
//! A frame is a big-endian `u32` length followed by a JSON body. The length
//! counts the body only. Bodies larger than 1 MiB are rejected.
//!
//! The bytes covered by a Hello signature are [`Hello::signed_payload`], which
//! omits the signature field. See `docs/spec/PROTOCOL.md`.

use ocos_capability::Capability;
use ocos_session::SessionSnapshot;
use serde::{Deserialize, Serialize};

/// Protocol version spoken by this crate.
pub const PROTOCOL_VERSION: u32 = 1;
/// Release both images must share. Bump this when the on-wire session format changes.
pub const OCOS_VERSION: &str = "0.1.0";
/// Default TCP port for the agent.
pub const DEFAULT_PORT: u16 = 9740;
/// Largest accepted JSON body.
pub const MAX_FRAME_BYTES: usize = 1024 * 1024;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Message {
    Hello(Box<Hello>),
    Pair(Pair),
    PairResult(PairResult),
    SessionOffer(Box<SessionOffer>),
    SessionAccept(SessionAccept),
    DesktopMode(DesktopMode),
    Heartbeat(Heartbeat),
    Error(ErrorMessage),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Hello {
    pub protocol_version: u32,
    pub ocos_version: String,
    pub node_id: String,
    pub public_key: String,
    pub nonce: String,
    pub capability: Capability,
    pub signature: String,
}

impl Hello {
    /// Canonical bytes that must be signed. The signature field is not included.
    pub fn signed_payload(&self) -> Result<Vec<u8>, ProtocolError> {
        let payload = HelloSigned {
            protocol_version: self.protocol_version,
            ocos_version: &self.ocos_version,
            node_id: &self.node_id,
            public_key: &self.public_key,
            nonce: &self.nonce,
            capability: &self.capability,
        };
        Ok(serde_json::to_vec(&payload)?)
    }
}

#[derive(Serialize)]
struct HelloSigned<'a> {
    protocol_version: u32,
    ocos_version: &'a str,
    node_id: &'a str,
    public_key: &'a str,
    nonce: &'a str,
    capability: &'a Capability,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Pair {
    pub code: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PairResult {
    pub accepted: bool,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SessionOffer {
    pub snapshot: SessionSnapshot,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionAccept {
    pub session_id: String,
    pub preserved: bool,
}

/// The desktop node opened its own copy of this session.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DesktopMode {
    pub active: bool,
    pub session_id: String,
    pub anchor_node_id: String,
    pub display_node_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Heartbeat {
    pub node_id: String,
    pub monotonic_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ErrorMessage {
    pub message: String,
}

/// Encode one message as a length-prefixed frame.
pub fn encode_frame(message: &Message) -> Result<Vec<u8>, ProtocolError> {
    let body = serde_json::to_vec(message)?;
    if body.len() > MAX_FRAME_BYTES {
        return Err(ProtocolError::FrameTooLarge(body.len()));
    }
    if body.is_empty() {
        return Err(ProtocolError::EmptyFrame);
    }
    let mut frame = Vec::with_capacity(4 + body.len());
    frame.extend_from_slice(&(body.len() as u32).to_be_bytes());
    frame.extend_from_slice(&body);
    Ok(frame)
}

/// Read the body length from the first four bytes of a frame.
pub fn frame_body_len(header: [u8; 4]) -> Result<usize, ProtocolError> {
    let len = u32::from_be_bytes(header) as usize;
    if len == 0 {
        return Err(ProtocolError::EmptyFrame);
    }
    if len > MAX_FRAME_BYTES {
        return Err(ProtocolError::FrameTooLarge(len));
    }
    Ok(len)
}

/// Decode a JSON body that has already been split from its length header.
pub fn decode_body(body: &[u8]) -> Result<Message, ProtocolError> {
    Ok(serde_json::from_slice(body)?)
}

/// Decode the first frame in `bytes` and return how many bytes it consumed.
pub fn decode_one(bytes: &[u8]) -> Result<(Message, usize), ProtocolError> {
    if bytes.len() < 4 {
        return Err(ProtocolError::Truncated);
    }
    let mut header = [0u8; 4];
    header.copy_from_slice(&bytes[..4]);
    let len = frame_body_len(header)?;
    let total = 4 + len;
    if bytes.len() < total {
        return Err(ProtocolError::Truncated);
    }
    let message = decode_body(&bytes[4..total])?;
    Ok((message, total))
}

#[derive(Debug, thiserror::Error)]
pub enum ProtocolError {
    #[error("frame is truncated")]
    Truncated,
    #[error("frame is empty")]
    EmptyFrame,
    #[error("frame is {0} bytes, over the 1 MiB limit")]
    FrameTooLarge(usize),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
}

#[cfg(test)]
mod tests {
    use super::*;
    use ocos_capability::Capability;

    #[test]
    fn hello_frame_roundtrip_keeps_capability() {
        let hello = Hello {
            protocol_version: PROTOCOL_VERSION,
            ocos_version: OCOS_VERSION.into(),
            node_id: "node_abc".into(),
            public_key: "pubkey".into(),
            nonce: "n".into(),
            capability: Capability::reference_pixel8("node_abc"),
            signature: "sig".into(),
        };
        let frame = encode_frame(&Message::Hello(Box::new(hello.clone()))).unwrap();
        let (decoded, consumed) = decode_one(&frame).unwrap();
        assert_eq!(consumed, frame.len());
        match decoded {
            Message::Hello(parsed) => {
                assert_eq!(*parsed, hello);
                assert!(parsed.capability.is_identity_anchor());
            }
            other => panic!("unexpected message: {other:?}"),
        }
    }

    #[test]
    fn signed_payload_omits_the_signature() {
        let hello = Hello {
            protocol_version: 1,
            ocos_version: OCOS_VERSION.into(),
            node_id: "node_abc".into(),
            public_key: "pubkey".into(),
            nonce: "n".into(),
            capability: Capability::reference_ryzen_desktop("node_abc"),
            signature: "this-must-not-appear".into(),
        };
        let payload = String::from_utf8(hello.signed_payload().unwrap()).unwrap();
        assert!(!payload.contains("this-must-not-appear"));
        assert!(payload.contains("node_abc"));
        assert!(payload.contains("x86_64"));
    }

    #[test]
    fn oversized_header_is_rejected() {
        let header = (MAX_FRAME_BYTES as u32 + 1).to_be_bytes();
        assert!(matches!(
            frame_body_len(header),
            Err(ProtocolError::FrameTooLarge(_))
        ));
    }
}
