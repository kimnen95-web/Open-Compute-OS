# Pixel 8 image (`ocos-mobile-arm64`)

The phone build is an Android image for Pixel 8, codename `shiba`, with Open Compute OS inside it. It is not a sideloaded app on stock Android.

## What the image contains

- The AOSP userspace and the Pixel kernel and drivers, so the Tensor G3, display, modem, and USB controller work
- `ocos-agent` as a system service that starts at boot
- A USB gadget configuration that exposes CDC-NCM when a host is detected, without a tethering prompt
- The mobile shell as the system UI

The agent crate in this repository is the piece that can be developed without compiling AOSP. The image build is a packaging step: cross-compile `ocos-agent` for `aarch64-linux-android` (or for a Linux chroot on the device, if the image uses one) and install it into the system partition.

## What is intentionally not in this folder

A full AOSP tree. Google's documented workstation for that build is 64 GiB of RAM and about 400 GiB of free disk. The agent itself builds on a normal 16 GiB machine. Keep the ROM build on a machine or a cloud builder that meets Google's requirements, then flash the result.

Flashing `shiba` requires an unlocked bootloader and wipes the phone. That is a property of Pixel images, not of the agent.

## Boot integration sketch

The service should be equivalent to:

```text
service ocos-agent /system/bin/ocos-agent serve --data-dir /data/ocos
    class main
    user system
    group system inet
    disabled
```

Enable it from the USB state machine when the port is configured, or enable it at boot and let it wait. The pairing code has to be visible in the system UI the first time a desktop is attached. After the desktop's key is in `/data/ocos/trust.json`, attaching it goes straight to Desktop Mode.

Do not ship this as a Magisk module or a Play Store app. Those can be used in private experiments. They are not the OS.
