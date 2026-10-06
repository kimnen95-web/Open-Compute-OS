# Node protocol

Version: 1  
Default TCP port: `9740`  
Encoding: one frame, then another. Not a JSON stream without lengths.

## Frame

```text
 0                   1                   2                   3
 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|                        body length (u32 BE)                   |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|                         JSON body (UTF-8)                     |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
```

The length is the number of body bytes. It must be from 1 to 1 MiB inclusive. The body is one JSON object with a `kind` tag.

Implementations in this repository live in `ocos-protocol`.

## Messages

`kind` is one of:

| kind | Direction in the attach flow | Meaning |
| --- | --- | --- |
| `hello` | both | Signed introduction and capability |
| `pair` | connector → listener | Six-digit pairing code |
| `pair_result` | listener → connector | Accepted or rejected |
| `session_offer` | identity anchor → desktop target | Live windows, still owned by the anchor |
| `session_accept` | desktop → anchor | `preserved` must be true |
| `desktop_mode` | desktop → anchor | Desktop shell is showing that session |
| `heartbeat` | either | Liveness. Not required in 0.1 handshake |
| `error` | either | Human-readable failure |

### `hello`

```json
{
  "kind": "hello",
  "protocol_version": 1,
  "ocos_version": "0.1.0",
  "node_id": "node_<64 lowercase hex chars of the raw public key>",
  "public_key": "<standard base64 of the 32-byte Ed25519 public key>",
  "nonce": "<32 lowercase hex chars>",
  "capability": {},
  "signature": "<standard base64 of the 64-byte Ed25519 signature>"
}
```

`capability.node_id` must equal `node_id`.

The signature covers the UTF-8 JSON of the same fields **except** `kind` and `signature`, serialized from this struct field order:

```text
protocol_version, ocos_version, node_id, public_key, nonce, capability
```

`node_id` must be `node_` plus the hex encoding of the 32-byte public key. A hello that fails verification closes the connection. No session is offered.

### `pair` and `pair_result`

```json
{ "kind": "pair", "code": "482193" }
{ "kind": "pair_result", "accepted": true, "reason": "trusted" }
```

The listener compares `code` with the code it is currently displaying. The comparison does not stop at the first differing digit. A mismatch returns `accepted: false` and stores nothing.

If the listener already trusts `node_id`, the connector must not send `pair`. The listener moves straight to the session offer when the roles require it.

### `session_offer`, `session_accept`, `desktop_mode`

```json
{
  "kind": "session_offer",
  "snapshot": {
    "session_id": "ses_demo",
    "owner_node_id": "node_…",
    "mode": "mobile",
    "windows": []
  }
}
```

```json
{ "kind": "session_accept", "session_id": "ses_demo", "preserved": true }
```

```json
{
  "kind": "desktop_mode",
  "active": true,
  "session_id": "ses_demo",
  "anchor_node_id": "node_…",
  "display_node_id": "node_…"
}
```

These three are sent only when the listener's capability has the `identity` role and the connector's capability has both `compute` and `display`. The snapshot is a description of windows already running on the anchor. Accepting it must not be implemented as "start a new process from scratch and hope the draft matches."

## Roles and who dials

0.1's `serve` command is the listener and `connect` is the connector. The reference demo makes the phone listen and the desktop dial, which matches USB (phone is the gadget, desktop is the host). The message rules follow **roles**, not which side called `connect`. A later USB transport can keep this handshake and replace the TCP socket.

## Transport limits in version 1

TCP carries the frames. There is no TLS in version 1. See [SECURITY.md](../../SECURITY.md). A future version should negotiate encryption before `session_offer`. Until then, do not put credentials into a snapshot.
