# Capability descriptor

Every node publishes one descriptor. The fabric treats it as the truth about **that** node. Combining two descriptors into a single RAM size or a single CPU is not a valid use of this document.

## Fields

| Field | Type | Meaning |
| --- | --- | --- |
| `node_id` | string | Same id as the device key. See the [protocol](PROTOCOL.md) |
| `name` | string | Human name ("Pixel 8", "Ryzen Desktop") |
| `architecture` | `aarch64`, `x86_64`, or `riscv64` | Native execution |
| `roles` | list | `identity`, `input`, `display`, `sensors`, `compute`, `storage` |
| `cpu.model` | string | Free-form model name |
| `cpu.cores` | number | Physical cores when known |
| `cpu.threads` | number | Hardware threads |
| `memory.total_bytes` | number | RAM installed in this node only |
| `gpu` | object or null | See below |
| `storage.nvme_bytes` | number, optional | Local storage |
| `display.outputs` | number | Connected outputs this node can drive |
| `power` | object | `{"type":"ac"}`, `{"type":"battery","percent":0-100}`, or `{"type":"unknown"}` |

### GPU

```json
{
  "vendor": "NVIDIA",
  "model": "GeForce GTX 1070",
  "memory_bytes": 8589934592
}
```

`memory_bytes > 0` means a discrete GPU the fabric may offload to. A phone GPU that shares system RAM is described with `memory_bytes: 0` or omitted. Offload ignores it.

### Power

`prefer_ac` workloads prefer a node whose `power.type` is `ac`. They may still run on battery if no AC node fits. They still may not span two nodes.

## Reference nodes

These two descriptors are the first hardware targets. Tests construct them with `Capability::reference_pixel8` and `Capability::reference_ryzen_desktop`.

Phone:

```json
{
  "name": "Pixel 8",
  "architecture": "aarch64",
  "roles": ["identity", "input", "display", "sensors"],
  "cpu": { "model": "Google Tensor G3", "cores": 9, "threads": 9 },
  "memory": { "total_bytes": 8589934592 },
  "gpu": { "vendor": "ARM", "model": "Mali-G715", "memory_bytes": 0 },
  "display": { "outputs": 1 },
  "power": { "type": "battery", "percent": 70 }
}
```

Desktop:

```json
{
  "name": "Ryzen Desktop",
  "architecture": "x86_64",
  "roles": ["compute", "display", "storage"],
  "cpu": { "model": "AMD Ryzen 5 5600X", "cores": 6, "threads": 12 },
  "memory": { "total_bytes": 17179869184 },
  "gpu": {
    "vendor": "NVIDIA",
    "model": "GeForce GTX 1070",
    "memory_bytes": 8589934592
  },
  "storage": { "nvme_bytes": 2000000000000 },
  "display": { "outputs": 1 },
  "power": { "type": "ac" }
}
```

## Identity anchor and desktop target

- **Identity anchor:** `roles` contains `identity`. This node offers the session.
- **Desktop target:** `roles` contains both `compute` and `display`. This node may enter Desktop Mode and accept offload.

A node can theoretically hold both. The first phone profile does not advertise `compute`, so the fabric will not park a discrete-GPU job on it.

## Host probe

`ocos-agent serve` builds a descriptor from the machine it is running on:

- architecture from the process
- memory from the OS
- CPU model and thread count from the OS
- roles, power, display count, and optional GPU or NVMe size from `node.json`

The probe does not invent a GPU. On the Ryzen machine, put the GTX 1070 in `node.json` or offload will not select it.
