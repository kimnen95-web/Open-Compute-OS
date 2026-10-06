# Pixel 8 image (`ocos-mobile-arm64`)

Open Compute OS 0.1.0 for Pixel 8 (`shiba`). This is a **Linux phone image**, not an AOSP ROM with an extra service.

The kernel remains Google's Android kernel, because that is what drives Tensor. Everything above it is Ubuntu: systemd, apt, the Open Compute OS shell, and `ocos-agent` as a systemd unit — the same layout as [the desktop image](../desktop-x86_64/README.md). The glue is Halium / libhybris. See [docs/BASES.md](../../docs/BASES.md) and [ocos-release.json](ocos-release.json).

## What the image contains

- Android `boot` / `vendor` from a Pixel factory image (kernel + HALs)
- Ubuntu rootfs (`system.img` / Halium root)
- `ocos-agent` as `open-compute-agent.service`
- USB CDC-NCM gadget started by the OS, not by an Android tethering dialog
- Linux apps (Notes and the rest), not APKs

## Bring-up (not done in this repository yet)

1. Unlock `shiba`.
2. Choose a Halium Android version that can bind this kernel.
3. Build hybris and a minimal Ubuntu root that reaches a framebuffer or DRM display.
4. Cross-compile `ocos-agent` for `aarch64-unknown-linux-gnu` (glibc, not Android NDK).
5. Enable the same systemd unit the desktop uses.
6. Flash and test USB NCM, then the session handshake with the Ryzen node.

A first boot that only gets a Linux prompt over USB is a valid milestone. Modem and camera are later.

## What this is not

- Stock Android plus `ocos-agent.apk`
- GrapheneOS / LineageOS with a privileged service
- Mainline Linux on Tensor (not available for Pixel 8)
