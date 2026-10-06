# Contributing

Open Compute OS is an open platform. The useful way to help is to extend a boundary that is already named, and to keep the rules in [docs/VISION.md](docs/VISION.md) intact.

## Setup

Rust stable is enough. A full AOSP tree is not required to work on the agent, the protocol, or the specs.

```sh
cargo test
cargo run -p ocos-agent -- demo
```

`cargo test` is the check that must pass before a change is merged.

## Where to work

| Area | Start here |
| --- | --- |
| Node descriptor | `crates/ocos-capability` and [docs/spec/CAPABILITY.md](docs/spec/CAPABILITY.md) |
| Messages | `crates/ocos-protocol` and [docs/spec/PROTOCOL.md](docs/spec/PROTOCOL.md) |
| Pairing and device keys | `crates/ocos-identity` |
| Sessions and Desktop Mode | `crates/ocos-session` and [docs/spec/SESSION.md](docs/spec/SESSION.md) |
| Placement | `crates/ocos-fabric` |
| Agent behavior | `crates/ocos-agent` |
| Pixel and desktop images | `os/` — packaging notes, not a full ROM tree |
| USB-C attach | [docs/spec/USB-EXPANSION.md](docs/spec/USB-EXPANSION.md) |

If you change a message or a descriptor field, change the spec in the same pull request. The spec is the contract other implementations will speak.

## What not to do in this phase

- Do not add phone RAM to desktop RAM and call the result one machine.
- Do not live-migrate arbitrary ARM processes onto x86.
- Do not require a cloud account for the basic plug-in flow.
- Do not treat a stock Android app, a browser extension, or scrcpy as the product. The agent belongs to the OS image.

## Pull requests

- Explain why the change exists.
- Add or update a test when behavior changes. The demo test in `ocos-agent` is the end-to-end check for pairing, session preservation, and placement.
- Keep new code free of `unsafe`. The workspace forbids it.
- Prefer a small crate change over a new framework.

## Security-sensitive changes

Pairing, signatures, and the trust store are security boundaries. Describe the threat you considered. Unencrypted TCP is a known 0.1 limit; see [SECURITY.md](SECURITY.md). Do not silently start sending real user files across it.
