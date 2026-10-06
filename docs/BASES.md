# Image bases

Open Compute OS is **one Linux userspace** on both machines. It is not an Android service talking to an Ubuntu service.

Pixel 8 cannot boot a mainline Ubuntu kernel. The Tensor G3 still needs Google's Android kernel and vendor HALs. The way to put a real Linux OS on that phone is **Halium**: keep the Android kernel, replace the Android framework with Ubuntu userspace (glibc, systemd, the same packages as the desktop). Ubuntu Touch uses this model. After a successful port, the phone is not Android. It is Open Compute OS / Ubuntu. Apps are Linux apps, not APKs.

The desktop is Ubuntu 24.04 on the Ryzen kernel. Both images ship Open Compute OS 0.1.0. A hello with a different `ocos_version` is rejected.

## Pixel 8 — Halium + Ubuntu userspace (`shiba`)

```text
Open Compute OS shell + agent + Linux apps (aarch64 debs)
                    │
            Ubuntu userspace (glibc)
                    │
               libhybris
                    │
     Android kernel + vendor HALs (Tensor)
```

Bring-up, in order:

1. Unlock the Pixel 8 bootloader.
2. Extract the kernel, `vendor`, and HALs from a Google factory image (the Android version Halium can bind to).
3. Build a Halium init and `system.img` that starts Ubuntu, not Zygote / the Android framework.
4. Install the same `ocos-agent` `.deb` used on the desktop, built for `aarch64`.
5. Flash. The phone boots Open Compute OS. Settings, Notes, and other OCOS apps are Linux programs.

This port for Tensor / Pixel 8 is not known to be complete upstream. Display, USB gadget, GPU, modem, and camera each have to be made to work through hybris. Expect missing radios and cameras in early boots. The point of the port is a Linux phone, not a daily Android replacement on day one.

Do **not** ship AOSP userspace, a Magisk module, or a Play Store agent. Those are the model this file replaces.

## Desktop — Ubuntu 24.04 LTS

Same userspace family: apt, systemd, the Open Compute OS shell, `ocos-agent`, NVIDIA driver for the GTX 1070. Apps are the x86_64 build of the same source that produced the phone `.deb`.

## What must match

| | Pixel image | Desktop image |
| --- | --- | --- |
| OS name | Open Compute OS 0.1.0 | Open Compute OS 0.1.0 |
| Userspace | Ubuntu (Halium) | Ubuntu 24.04 |
| Apps | Linux, `aarch64` | Linux, `x86_64` |
| Agent | `ocos-agent` | `ocos-agent` |
| Kernel | Android kernel for `shiba` | Ubuntu Linux kernel |
| Not used | Android framework, APK, Play | Android-x86 |

Plugging in USB-C is two Linux nodes of one OS exchanging a session. The desktop opens its build of the same app. It is not mirroring an Android activity.
