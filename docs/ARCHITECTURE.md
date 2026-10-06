# Architecture

This document describes the 0.1 system and the two images around it. The crates in this repository are the control plane. They are what a contributor can build on a normal machine.

## Two images, one platform

```text
                Personal computing environment
                (identity, data, apps, sessions)
                              │
                    open-compute-agent
                              │
              ┌───────────────┴───────────────┐
              │                               │
     ocos-mobile-arm64                ocos-desktop-x86_64
     Pixel 8 / shiba                  Ryzen + GTX 1070
     AOSP + system service            Linux + systemd service
     USB gadget (device)              USB host
```

The agent binary is built twice from the same crate: `aarch64` for the phone image and `x86_64` for the desktop image. The protocol does not change between them.

Building the phone *image* means compiling AOSP for `shiba` and installing the agent into it. That build is large. Building the *agent* is not. See [os/mobile-shiba/README.md](../os/mobile-shiba/README.md).

## Crates

```text
ocos-agent
  ├── ocos-protocol ── ocos-capability
  │                 └── ocos-session
  ├── ocos-identity
  └── ocos-fabric ──── ocos-capability
```

| Crate | Responsibility |
| --- | --- |
| `ocos-capability` | The JSON description of a node, plus a host probe |
| `ocos-identity` | Ed25519 device key, pairing code, trust store |
| `ocos-session` | Window snapshot handed between the aarch64 and x86_64 app builds |
| `ocos-protocol` | Frames and message types |
| `ocos-fabric` | Choose one node for a workload |
| `ocos-agent` | CLI, TCP session, reference demo |

Shells and image packaging sit outside the crates. They consume the same messages.

## Attach sequence

The phone listens. The desktop connects. On a finished image the phone is the USB device (CDC-NCM) and the desktop is the USB host. In 0.1 any TCP address works, which is how the demo runs on one laptop.

```text
Desktop                         Phone
   │  Hello (signed)               │
   │ ────────────────────────────► │
   │            Hello (signed)     │
   │ ◄──────────────────────────── │
   │  Pair { code }                │  only if not yet trusted
   │ ────────────────────────────► │
   │         PairResult            │
   │ ◄──────────────────────────── │
   │      SessionOffer             │  phone still owns the processes
   │ ◄──────────────────────────── │
   │  SessionAccept { preserved }  │
   │ ────────────────────────────► │
   │  DesktopMode { active }       │
   │ ────────────────────────────► │
```

After this exchange the desktop has opened its own x86 build at the same documents and cursors. `runtime_node_id` is the desktop. Unplugging writes the latest snapshot back and the phone opens its aarch64 build again.

A second connection between the same keys skips `Pair`.

## Placement

`ocos-fabric` sees a set of trusted, connected capabilities. For a workload it keeps a node only when that node's own memory is enough and, if requested, that node has a discrete GPU (`gpu.memory_bytes > 0`). A shared phone GPU with zero dedicated memory is not an offload target.

If several nodes fit, an AC-powered node is preferred when the workload asks for it. The scheduler then picks the larger memory. It never sums memory across the set. A 20 GiB job is refused when the largest node has 16 GiB, even if the phone's 8 GiB would cover the gap on paper.

## What stays on the phone

| Stays with the user | Opens on the desktop runtime when connected |
| --- | --- |
| Identity and the data the phone owns | The x86 build of each OCOS app in the session |
| Draft, cursor, and document id | GPU and large-memory jobs |
| Sensors, radios, battery | Extra storage, when a data layer exists |

Pixel streaming of those windows, and a worker that actually runs `ffmpeg` or a model on the GTX 1070, are the next layers. The control plane already decides *that* the windows move and *which* node gets the job.

## Profiles

`ocos-agent init --profile phone` creates an identity anchor: identity, input, display, sensors, battery.

`ocos-agent init --profile desktop` creates a compute node: compute, display, storage, AC power. The GTX 1070 is declared in `node.json` on the real desktop because a probe cannot invent a GPU the development laptop does not have. `memory_bytes` must be non-zero or the fabric will ignore it.

The reference constructors `Capability::reference_pixel8` and `Capability::reference_ryzen_desktop` are the hardware story used by tests. They are not a substitute for the probe on a real device.
