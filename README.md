# Open Compute OS

Open Compute OS is a Linux-based personal computing platform. A phone, a desktop, a laptop, or a future device is not a separate computer. Each one is a **compute node** of the same personal computing environment.

The user does not switch computers. They attach a node with the CPU, GPU, memory, storage, and display they need right now. Identity, data, applications, and sessions stay with the user.

This repository is the **0.1 control plane**: the agent, the capability model, device identity, session handoff, and the rule that memory is never pooled across nodes. It is MIT licensed. The protocol and the module boundaries are meant to be extended by other people.

Open Compute OS is **not** the [Open Compute Project](https://www.opencompute.org/). The names are similar. The projects are unrelated.

## The idea

Today a phone and a desktop are two stacks:

```text
Phone                         Desktop
├── OS                        ├── OS
├── Apps                      ├── Apps
├── Data                      ├── Data
├── Identity                  ├── Identity
└── Hardware                  └── Hardware
```

Open Compute OS splits that differently. The personal environment is stable. Hardware is replaceable.

```text
User + Identity + Data + Apps + Sessions
                 │
          Compute Fabric
                 │
     ┌───────────┼───────────┐
     │           │           │
  Phone       Desktop      (later: laptop, cloud, …)
  ARM64       x86-64
```

The first hardware pair is a **Pixel 8** and a **Ryzen desktop** (Ryzen 5 5600X, 16 GiB RAM, GTX 1070). They do not boot the same kernel. Tensor G3 and a Ryzen CPU cannot share one kernel image. They boot two images of one platform:

Both machines run **Open Compute OS 0.1.0**. They do not share one kernel image. The bases below are the ones that actually boot this hardware. See [docs/BASES.md](docs/BASES.md).

| Image | Machine | Base | Shared OCOS |
| --- | --- | --- | --- |
| `ocos-mobile-arm64` | Pixel 8 (`shiba`) | AOSP `android-latest-release` | 0.1.0, protocol 1 |
| `ocos-desktop-x86_64` | Ryzen PC | Ubuntu 24.04 LTS | 0.1.0, protocol 1 |

Plug the phone into the PC over USB-C and the open work is handed to the desktop runtime. Unplug it and the latest state returns to the phone runtime.

## What plugging in does

An Open Compute OS app is built twice from one project: `aarch64` on the Pixel image and `x86_64` on the Ubuntu image. Connecting synchronizes identity and the open session. The desktop opens **its** build at the same document and cursor. Disconnecting writes that state back and the phone opens **its** build again.

```text
Notes on Pixel (aarch64 runtime)
        │
        │  USB-C, same OCOS 0.1.0
        ▼
Notes on Ryzen (x86_64 runtime), same draft and cursor
        │
        │  unplug, state synced back
        ▼
Notes on Pixel again
```

An app that exists only as an Android package, with no x86 build, has nothing to open on the desktop. Heavy work is a separate decision: video encoding, a large compile, or inference can run on the desktop CPU and GPU. The phone's 8 GiB and the desktop's 16 GiB are never added together.

## What 0.1 runs today

`ocos-agent` can already:

1. Create an Ed25519 device identity for a phone profile or a desktop profile.
2. Exchange a signed hello and a one-time pairing code.
3. Publish a capability descriptor (architecture, CPU, memory, GPU, power, roles).
4. Hand Notes to the desktop runtime without dropping the draft or cursor, and record that the x86 node is now hosting it.
5. Place a GPU workload on the desktop node alone, and reject a job that would fit only if RAM were pooled.
6. Refuse a peer that speaks a different Open Compute OS version.

The flashable Pixel image and the Ubuntu desktop image are specified in `os/` and are not built by `cargo test`. See [the roadmap](docs/ROADMAP.md).

## Try the reference scenario

Install Rust, then:

```sh
cargo test
cargo run -p ocos-agent -- demo
```

The demo starts a phone node and a desktop node on localhost. It uses the Pixel 8 and Ryzen capability profiles even when your laptop is neither of those machines. A successful run prints mutual trust, Desktop Mode, the original Notes draft, and a video-encode placement on the desktop node only.

Two real processes:

```sh
cargo build -p ocos-agent
./target/debug/ocos-agent init --data-dir ./nodes/phone --profile phone
./target/debug/ocos-agent init --data-dir ./nodes/desktop --profile desktop

# Terminal A — the pairing code is printed here.
./target/debug/ocos-agent serve --data-dir ./nodes/phone --listen 127.0.0.1:9740

# Terminal B — use the code from terminal A.
./target/debug/ocos-agent connect --data-dir ./nodes/desktop --peer 127.0.0.1:9740 --code 123456
```

Put a session file at `nodes/phone/session.json` before `serve` if you want the phone to offer real window state. See [examples/phone-session.json](examples/phone-session.json). On the Ryzen machine, declare the GTX 1070 in `node.json` (`gpu.memory_bytes` must be greater than zero for offload). The phone image will probe itself; the checked-in reference profile is for the demo and tests.

## Repository map

```text
crates/ocos-capability   What a node is
crates/ocos-identity     Device keys, pairing codes, trust store
crates/ocos-session      Window snapshots for Mobile and Desktop Mode
crates/ocos-protocol     Length-prefixed messages
crates/ocos-fabric       Place a workload on one node
crates/ocos-agent        The `ocos-agent` binary
docs/                    Vision, architecture, and protocol specs
os/                      How the two images will carry the agent
examples/                A sample phone session
```

## Design rules

1. The user is not tied to one device.
2. Data belongs to the user. The first version is local-first. Cloud sync is not required.
3. Hardware can be replaced. The personal environment remains.
4. CPU, GPU, memory, and storage are resources offered by nodes.
5. A node is authenticated before it can see sessions or receive work.
6. The protocols and APIs in this repository are open.
7. Linux is the first kernel. This project does not write a new kernel.
8. Do not pool RAM or build a heterogeneous SMP system in this phase.

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md). Issues and pull requests are welcome, including protocol review, the desktop shell, the Pixel image overlay, USB-C discovery, and a real offload worker.

## License

[MIT](LICENSE).
