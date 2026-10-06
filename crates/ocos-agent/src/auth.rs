//! Signed Hello messages.

use anyhow::{bail, Result};
use ocos_capability::Capability;
use ocos_identity::{node_id_for_public_key, parse_public_key, verify, DeviceIdentity};
use ocos_protocol::{Hello, OCOS_VERSION, PROTOCOL_VERSION};
use rand::RngCore;

pub fn make_hello(identity: &DeviceIdentity, capability: &Capability) -> Hello {
    let mut nonce = [0u8; 16];
    rand::rngs::OsRng.fill_bytes(&mut nonce);
    let mut hello = Hello {
        protocol_version: PROTOCOL_VERSION,
        ocos_version: OCOS_VERSION.to_string(),
        node_id: identity.node_id.clone(),
        public_key: identity.public_key.clone(),
        nonce: hex(&nonce),
        capability: capability.clone(),
        signature: String::new(),
    };
    let payload = hello.signed_payload().expect("hello payload is valid json");
    hello.signature = identity.sign(&payload);
    hello
}

pub fn verify_hello(hello: &Hello) -> Result<()> {
    if hello.protocol_version != PROTOCOL_VERSION {
        bail!(
            "protocol {} is not supported (this agent speaks {PROTOCOL_VERSION})",
            hello.protocol_version
        );
    }
    if hello.ocos_version != OCOS_VERSION {
        bail!(
            "peer runs Open Compute OS {}, this node runs {OCOS_VERSION}",
            hello.ocos_version
        );
    }
    if hello.capability.node_id != hello.node_id {
        bail!("capability node id does not match the hello");
    }
    let payload = hello.signed_payload()?;
    verify(&hello.public_key, &payload, &hello.signature)
        .map_err(|_| anyhow::anyhow!("hello signature is invalid"))?;
    let public_key = parse_public_key(&hello.public_key)
        .map_err(|_| anyhow::anyhow!("public key is malformed"))?;
    let expected = node_id_for_public_key(&public_key);
    if expected != hello.node_id {
        bail!("node id is not the id of the signing key");
    }
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
