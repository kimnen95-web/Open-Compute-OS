# Security

0.1 authenticates a **device key**. It does not yet encrypt the TCP session.

## What is protected

- Each node has an Ed25519 key. `identity.json` is written with mode `0600` on Unix.
- A hello is signed. The node id is derived from the public key, so a peer cannot present somebody else's id.
- An unknown node must present the six-digit pairing code currently shown by the listener.
- A wrong code does not enter the trust store.
- The fabric will not place work on a node marked untrusted.

## What is not protected yet

The bytes on the TCP connection are signed at hello time and then sent as JSON. Anyone who can sit in the middle of that connection after pairing can read a session snapshot. **Do not pair across a hostile network, and do not put secrets in `session.json`, until the channel is encrypted.**

TLS for the agent connection is an open task. See [docs/ROADMAP.md](docs/ROADMAP.md).

USB attachment alone is not trust. A new device still needs the pairing step. Later images should require the same check before a gadget interface comes up for an unknown host.

## Reporting

Open a GitHub issue for defects that are not yet exploitable beyond the limitation above. If you believe you have found a way to forge a device id or to skip pairing, describe it in the issue and avoid including a turnkey attack.
