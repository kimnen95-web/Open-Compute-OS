# Desktop image (`ocos-desktop-x86_64`)

The desktop build is a Linux system for the Ryzen machine. Ubuntu 24.04 is the first base. The agent and the desktop shell are part of that system. The user installs the image once. Plugging in the phone does not install anything else.

## First target

- AMD Ryzen 5 5600X
- 16 GiB RAM
- 2 TB SSD
- NVIDIA GeForce GTX 1070

The proprietary NVIDIA driver belongs in this image so the GTX 1070 can accept offload later. The driver is not part of the agent crate.

## Agent service

[`open-compute-agent.service`](open-compute-agent.service) is the unit the image should install. The binary is the same `ocos-agent` built for `x86_64-unknown-linux-gnu`.

On first boot the image runs the equivalent of:

```sh
ocos-agent init --data-dir /var/lib/ocos --profile desktop
```

Then it writes the GTX 1070 into `/var/lib/ocos/node.json`:

```json
{
  "gpu": {
    "vendor": "NVIDIA",
    "model": "GeForce GTX 1070",
    "memory_bytes": 8589934592
  }
}
```

`memory_bytes` must be greater than zero. A missing GPU object means the fabric will not offload to this machine.

Wake-on-USB can resume a sleeping machine on some motherboards. A machine that is fully powered off is not part of this phase.

## Shell

The desktop shell is not in the 0.1 crates. It should:

- start at graphical login and wait for a phone
- on `desktop_mode`, open the windows from `session_offer`
- send input back to the phone
- on disconnect, close those windows and leave the processes on the phone

A shell that only renders a remote desktop of the entire phone screen is a possible first painting path. It still has to obey the session rules in [docs/spec/SESSION.md](../../docs/spec/SESSION.md): do not drop the draft, and do not relaunch the app on x86.
