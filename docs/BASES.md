# Image bases

Open Compute OS 0.1.0 is the version both machines run. A hello with a different `ocos_version` is rejected, so a phone and a desktop only synchronize when they are the same release.

There is no single Android build or Ubuntu build that boots both a Pixel 8 and a Ryzen PC. The kernels stay different. The synchronized system is this release: protocol 1, the same identity files, the same session snapshot, and apps built from one source for `aarch64` and `x86_64`.

## Pixel 8 — AOSP `android-latest-release`

Device codename: `shiba`.

Use the newest AOSP branch Google publishes for current Pixel devices (`android-latest-release`). That tree, plus Google's Pixel kernel and vendor blobs, is what makes the display, USB, modem, and camera work. The Open Compute OS agent, shell, and data services are installed into that image. After flashing, the phone boots Open Compute OS.

Do not put Ubuntu, Debian, or postmarketOS on this phone for 0.1. Those systems do not have a working driver set for the Tensor G3.

## Desktop — Ubuntu 24.04 LTS

Use Ubuntu 24.04 LTS on the Ryzen machine, with the NVIDIA driver for the GTX 1070. The same agent is installed as a system service. Ubuntu is the base because it is a normal desktop: package updates, a display server, and a GPU driver. Android-x86 would make the app package look the same and would give up that desktop.

## What must match

| | Pixel image | Desktop image |
| --- | --- | --- |
| `ocos_version` | 0.1.0 | 0.1.0 |
| `protocol_version` | 1 | 1 |
| App source | one repository | the same repository |
| App binary | `aarch64` | `x86_64` |
| Kernel | Pixel Android kernel | Ubuntu Linux kernel |

The manifests in `os/mobile-shiba/ocos-release.json` and `os/desktop-x86_64/ocos-release.json` record this pair. A test reads both files and fails if the versions diverge.
