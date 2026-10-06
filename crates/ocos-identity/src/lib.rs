//! Device identity that is not tied to one chassis.
//!
//! Each compute node has an Ed25519 key. The user pairs two nodes once. After
//! that, a known public key is enough to recognize the node. User data is not
//! stored in this crate; this crate only answers "which device is this?"

use base64::{engine::general_purpose::STANDARD as B64, Engine};
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Write;
use std::path::Path;

/// Stable id derived from the device public key.
pub fn node_id_for_public_key(public_key: &[u8; 32]) -> String {
    format!("node_{}", hex(public_key))
}

/// A node key kept on that node.
#[derive(Clone)]
pub struct DeviceIdentity {
    pub node_id: String,
    pub public_key: String,
    signing_key: SigningKey,
}

impl std::fmt::Debug for DeviceIdentity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DeviceIdentity")
            .field("node_id", &self.node_id)
            .field("public_key", &self.public_key)
            .finish_non_exhaustive()
    }
}

impl DeviceIdentity {
    pub fn generate() -> Self {
        let signing_key = SigningKey::generate(&mut rand::rngs::OsRng);
        Self::from_signing_key(signing_key)
    }

    pub fn sign(&self, message: &[u8]) -> String {
        let signature = self.signing_key.sign(message);
        B64.encode(signature.to_bytes())
    }

    pub fn save(&self, path: &Path) -> Result<(), IdentityError> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let file = IdentityFile {
            node_id: self.node_id.clone(),
            public_key: self.public_key.clone(),
            secret_key: B64.encode(self.signing_key.to_bytes()),
        };
        let json = serde_json::to_vec_pretty(&file)?;
        write_private(path, &json)?;
        Ok(())
    }

    pub fn load(path: &Path) -> Result<Self, IdentityError> {
        let bytes = fs::read(path)?;
        let file: IdentityFile = serde_json::from_slice(&bytes)?;
        let secret = decode_array::<32>(&file.secret_key)?;
        let signing_key = SigningKey::from_bytes(&secret);
        let identity = Self::from_signing_key(signing_key);
        if identity.node_id != file.node_id || identity.public_key != file.public_key {
            return Err(IdentityError::CorruptIdentity);
        }
        Ok(identity)
    }

    fn from_signing_key(signing_key: SigningKey) -> Self {
        let public = signing_key.verifying_key().to_bytes();
        Self {
            node_id: node_id_for_public_key(&public),
            public_key: B64.encode(public),
            signing_key,
        }
    }
}

/// Decode a base64 device public key.
pub fn parse_public_key(value: &str) -> Result<[u8; 32], IdentityError> {
    decode_array(value)
}

/// Check a signature produced by [`DeviceIdentity::sign`].
pub fn verify(
    public_key_b64: &str,
    message: &[u8],
    signature_b64: &str,
) -> Result<(), IdentityError> {
    let public_key = parse_public_key(public_key_b64)?;
    let signature = decode_array::<64>(signature_b64)?;
    let verifying = VerifyingKey::from_bytes(&public_key).map_err(|_| IdentityError::BadKey)?;
    let signature = Signature::from_bytes(&signature);
    verifying
        .verify(message, &signature)
        .map_err(|_| IdentityError::BadSignature)?;
    Ok(())
}

/// Six-digit code the user reads on one node and confirms on the other.
pub fn new_pairing_code() -> String {
    let value: u32 = rand::rngs::OsRng.gen_range(0..1_000_000);
    format!("{value:06}")
}

/// Compare pairing codes without leaking the matching prefix through timing.
pub fn pairing_codes_match(expected: &str, provided: &str) -> bool {
    if expected.len() != 6 || provided.len() != 6 {
        return false;
    }
    let mut diff = 0u8;
    for (a, b) in expected.bytes().zip(provided.bytes()) {
        diff |= a ^ b;
    }
    diff == 0
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TrustedPeer {
    pub node_id: String,
    pub public_key: String,
    pub name: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct TrustStore {
    pub peers: Vec<TrustedPeer>,
}

impl TrustStore {
    pub fn load(path: &Path) -> Result<Self, IdentityError> {
        if !path.exists() {
            return Ok(Self::default());
        }
        let bytes = fs::read(path)?;
        Ok(serde_json::from_slice(&bytes)?)
    }

    pub fn save(&self, path: &Path) -> Result<(), IdentityError> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_vec_pretty(self)?;
        write_private(path, &json)?;
        Ok(())
    }

    pub fn contains(&self, node_id: &str) -> bool {
        self.peers.iter().any(|peer| peer.node_id == node_id)
    }

    pub fn insert(&mut self, peer: TrustedPeer) {
        if let Some(existing) = self
            .peers
            .iter_mut()
            .find(|item| item.node_id == peer.node_id)
        {
            *existing = peer;
        } else {
            self.peers.push(peer);
        }
    }
}

#[derive(Serialize, Deserialize)]
struct IdentityFile {
    node_id: String,
    public_key: String,
    secret_key: String,
}

fn decode_array<const N: usize>(value: &str) -> Result<[u8; N], IdentityError> {
    let bytes = B64.decode(value).map_err(|_| IdentityError::BadKey)?;
    let array: [u8; N] = bytes.try_into().map_err(|_| IdentityError::BadKey)?;
    Ok(array)
}

fn write_private(path: &Path, bytes: &[u8]) -> Result<(), IdentityError> {
    let mut options = fs::OpenOptions::new();
    options.write(true).create(true).truncate(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let mut file = options.open(path)?;
    file.write_all(bytes)?;
    Ok(())
}

fn hex(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        out.push(HEX[(byte >> 4) as usize] as char);
        out.push(HEX[(byte & 0x0f) as usize] as char);
    }
    out
}

#[derive(Debug, thiserror::Error)]
pub enum IdentityError {
    #[error("identity file does not match its key")]
    CorruptIdentity,
    #[error("public key or signature is malformed")]
    BadKey,
    #[error("signature verification failed")]
    BadSignature,
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn signature_roundtrip_and_tamper_detection() {
        let identity = DeviceIdentity::generate();
        let signature = identity.sign(b"ocos-hello");
        assert!(verify(&identity.public_key, b"ocos-hello", &signature).is_ok());
        assert!(verify(&identity.public_key, b"ocos-hello!", &signature).is_err());
    }

    #[test]
    fn node_id_is_derived_from_the_public_key() {
        let identity = DeviceIdentity::generate();
        let loaded_public = B64.decode(&identity.public_key).unwrap();
        let public: [u8; 32] = loaded_public.try_into().unwrap();
        assert_eq!(identity.node_id, node_id_for_public_key(&public));
    }

    #[test]
    fn identity_survives_a_restart() {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("ocos-identity-{nanos}"));
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("identity.json");
        let identity = DeviceIdentity::generate();
        identity.save(&path).unwrap();
        let loaded = DeviceIdentity::load(&path).unwrap();
        assert_eq!(loaded.node_id, identity.node_id);
        let signature = loaded.sign(b"again");
        assert!(verify(&identity.public_key, b"again", &signature).is_ok());
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn pairing_code_is_six_digits_and_compared_exactly() {
        let code = new_pairing_code();
        assert_eq!(code.len(), 6);
        assert!(code.chars().all(|ch| ch.is_ascii_digit()));
        assert!(pairing_codes_match(&code, &code));
        let other = if code == "000000" { "000001" } else { "000000" };
        assert!(!pairing_codes_match(&code, other));
        assert!(!pairing_codes_match("12345", "123456"));
    }
}
