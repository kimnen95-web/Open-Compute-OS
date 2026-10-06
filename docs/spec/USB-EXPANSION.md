# USB-C expansion

Desktop Mode starts when the phone is plugged into the desktop with a USB-C cable. There is no dock in this phase, and there is no app to install at plug-in time. Both machines already run Open Compute OS.

## Intended path

1. The phone image brings up a USB gadget when it sees a host. The function is CDC-NCM, so the cable becomes a network.
2. The desktop image is the USB host. Its agent is already running; the user does not install a driver or an extension for the basic flow.
3. The two agents open TCP port `9740` on that link and perform the [handshake](PROTOCOL.md).
4. If the desktop's device key is already trusted, pairing is skipped and Desktop Mode starts.
5. If it is not trusted, the phone shows a pairing code. This happens once.
6. Unplugging drops the link. The session remains on the phone.

The phone is the USB device because that matches how a phone port works. The desktop does not need to become a gadget.

## What 0.1 actually does

The agent speaks the handshake over any TCP socket. `ocos-agent demo` uses localhost. That is enough to test identity, trust, sessions, and placement without a phone image.

The gadget configuration, the link-local addresses, and "do not ask the user to enable tethering" belong to the Pixel image. They are described in [os/mobile-shiba/README.md](../../os/mobile-shiba/README.md) and are not implemented in the crates yet.

## What the cable is not

- It is not a request to merge RAM.
- It is not permission by itself. An unknown host still has to pair.
- It is not DisplayPort-only desktop mode. Driving a monitor from the phone SoC is a different product. Here the desktop node has its own CPU and GPU, and the phone keeps the application processes.
- It does not have to wake a PC that is fully powered off. Wake-from-sleep (Wake-on-USB) may work on a given motherboard. A machine in soft-off is out of scope.

## Later

Encryption on the NCM interface, a pixel stream for the windows, and input in the opposite direction (keyboard and pointer on the desktop, process on the phone) are the next protocol pieces. They should be new messages, not a side channel the agent does not know about.
