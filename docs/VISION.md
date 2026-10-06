# Vision

Open Compute OS is one personal computer that can wear different hardware.

The phone in a pocket and the desktop on a desk are not two computers that must be kept in sync. They are two shapes of the same environment. The user, their identity, their data, their applications, and their open work stay. The CPU, GPU, memory, storage, display, and battery can change when a different node is connected.

## The problem

A phone and a desktop are still two products:

- a separate operating system
- a separate set of applications
- a separate copy of the data
- a separate account
- a separate session

People sync files, sign in again, install the same app twice, and keep a cloud account just to feel like they have one computer. The hardware vendors then become the owners of that feeling.

## The model

The stable object is the **personal computing environment**:

```text
User
Identity
Data
Applications
Sessions
Security policy
```

The replaceable object is the **compute node**. A node contributes some of:

```text
CPU    GPU    RAM    Storage
Display    Input    Network
Camera    Sensors    Battery
```

A Compute Fabric sits between them. It knows what each node can do, which nodes the user trusts, and which node should run a given piece of work. It does not melt the nodes into one motherboard.

```text
                    Open Compute OS
                          │
        ┌─────────────────┼─────────────────┐
        │                 │                 │
   Identity          Secure data      Applications
        │                 │                 │
        └─────────────────┼─────────────────┘
                          │
                   Session manager
                          │
                    Compute fabric
                          │
          ┌───────────────┼───────────────┐
          │               │               │
     Phone node      Desktop node     Later nodes
       ARM64           x86-64
     Mobile UI       Desktop UI
```

## One platform, more than one image

There is one platform and more than one boot image. Pixel 8 (Tensor G3, ARM64) and a Ryzen PC (x86-64, discrete GPU) cannot load the same kernel. They can load the same identity model, the same data model, the same permission model, the same application API, the same session model, and the same node protocol.

The kernel, the CPU, the GPU driver, and the board drivers stay architecture-specific. Linux is the first kernel. This project does not start by writing another one.

## Desktop Mode

The important everyday gesture is physical:

1. The phone is the computer the user is already using.
2. They plug USB-C into a desktop node that already runs Open Compute OS.
3. The two nodes recognize each other.
4. The shell on the desktop becomes Desktop Mode.
5. Applications that were open on the phone are still open. Their session is the same session.
6. Unplugging returns those applications to the phone.

"The same session" means the process was not killed and the user was not sent through a login wall. In the first implementation the process remains on the phone, because a live migration from ARM to x86 is a different problem. The desktop is where the windows are shown and where new heavy work can run.

## Compute is a resource

The desktop is not a monitor with a keyboard. It is a node with its own cores, its own RAM, and its own GPU. The fabric may place a workload there:

- inference
- video encoding
- compilation
- rendering

The phone can keep input, the session, sensors, and the user's identity.

The fabric looks at each node separately. Eight gibibytes on the phone plus sixteen on the desktop is not twenty-four gibibytes of unified memory. An ARM CPU plus an x86 CPU is not one SMP system. Those shortcuts are out of scope until the fabric itself is boring and reliable.

## Data and trust

Data belongs to the user, not to the phone and not to the desktop. The preferred design is local-first: the environment works when no cloud is reachable.

A cable is not permission. A new node authenticates, the user approves it once, and only then does it receive a capability exchange and a session. Device keys should eventually be backed by a secure element or TPM when the hardware has one. The 0.1 agent uses an Ed25519 key on disk and a pairing code.

## What this is not

The first release does not try to be:

- one kernel binary for every board
- unified physical RAM across ARM and x86
- live migration of every process
- a custom CPU, GPU, connector, or kernel
- a full cloud replica
- a robot or VR shell

Those can be researched after a phone and a desktop can find each other, trust each other, keep a session, and place a workload.

## A short definition

Open Compute OS is a Linux-based, multi-architecture operating platform that separates personal identity, data, applications, and sessions from physical compute hardware. Phones and desktops operate as compute nodes in one fabric. Compute can be added and removed while the user's identity, data, policy, and application sessions remain.

The hardware changes. The user's computer does not.
