# Roadmap

0.1 is the control plane in this repository. The list below is the work that turns it into the system described in [VISION.md](VISION.md). None of these items require pooling RAM or writing a new kernel.

## Now

- [x] Capability descriptor, including the Pixel 8 and Ryzen reference nodes
- [x] Ed25519 device identity, pairing code, trust store
- [x] Session snapshot that opens on the desktop runtime and can return to the phone runtime
- [x] Fabric that places work on one node and refuses pooled memory
- [x] `ocos-agent` handshake over TCP
- [x] Reference demo and tests

## Next

- [ ] TLS on the agent connection, before any real user file is sent
- [ ] USB CDC-NCM gadget on the Pixel image, brought up by the system, not by a tethering toggle
- [ ] Package `ocos-agent` as a system service on `ocos-mobile-arm64` and as a systemd unit on `ocos-desktop-x86_64`
- [ ] Desktop shell that opens `session_offer` windows and sends keyboard and pointer back
- [ ] One offload worker (video encode or a small inference job) that runs only after the fabric selects the desktop GPU
- [ ] Declare the GTX 1070 from the desktop image without hand-editing JSON on every boot

## After the plug-in flow works

- [ ] Local-first data layer: encrypted per-user store, phone as the canonical copy, desktop replica only while trusted
- [ ] Reconnect without showing the pairing code
- [ ] More than one application id, still without migrating processes
- [ ] A third node type (laptop or a remote machine on a trusted network) using the same descriptor

## Explicitly later

Unified RAM, heterogeneous SMP, live migration of arbitrary processes, a custom kernel, and a custom connector. Do not start those inside a pull request that is trying to finish Desktop Mode.
