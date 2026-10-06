//! What a compute node is, in a form any other node can read.
//!
//! A phone and a desktop publish the same descriptor shape. The fabric uses it
//! to place work on one node. It never adds the two memory sizes together.

use serde::{Deserialize, Serialize};

/// 1 KiB.
pub const KIB: u64 = 1024;
/// 1 MiB.
pub const MIB: u64 = 1024 * KIB;
/// 1 GiB.
pub const GIB: u64 = 1024 * MIB;

/// CPU family this node can execute natively.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Architecture {
    #[serde(rename = "aarch64")]
    Aarch64,
    #[serde(rename = "x86_64")]
    X86_64,
    #[serde(rename = "riscv64")]
    Riscv64,
}

impl Architecture {
    /// Architecture of the process that is running now.
    pub fn host() -> Self {
        #[cfg(target_arch = "aarch64")]
        {
            Self::Aarch64
        }
        #[cfg(target_arch = "x86_64")]
        {
            Self::X86_64
        }
        #[cfg(target_arch = "riscv64")]
        {
            Self::Riscv64
        }
        #[cfg(not(any(
            target_arch = "aarch64",
            target_arch = "x86_64",
            target_arch = "riscv64"
        )))]
        {
            Self::X86_64
        }
    }
}

/// Role a node is willing to play in the personal computing environment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Role {
    /// Holds the user identity. The phone is the anchor in the first release.
    Identity,
    Input,
    Display,
    Sensors,
    /// Can run offloaded workloads on its own CPU and memory.
    Compute,
    Storage,
}

/// Where the node's power comes from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum PowerSource {
    Ac,
    Battery { percent: u8 },
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CpuInfo {
    pub model: String,
    pub cores: u32,
    pub threads: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct MemoryInfo {
    pub total_bytes: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GpuInfo {
    pub vendor: String,
    pub model: String,
    /// Dedicated framebuffer size. `0` means a shared mobile GPU, which is not
    /// an offload target.
    pub memory_bytes: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct StorageInfo {
    pub nvme_bytes: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct DisplayInfo {
    pub outputs: u32,
}

/// Published description of one compute node.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Capability {
    pub node_id: String,
    pub name: String,
    pub architecture: Architecture,
    pub roles: Vec<Role>,
    pub cpu: CpuInfo,
    pub memory: MemoryInfo,
    pub gpu: Option<GpuInfo>,
    pub storage: Option<StorageInfo>,
    pub display: DisplayInfo,
    pub power: PowerSource,
}

impl Capability {
    pub fn has_role(&self, role: Role) -> bool {
        self.roles.contains(&role)
    }

    /// Phone (or any node) that owns the user identity.
    pub fn is_identity_anchor(&self) -> bool {
        self.has_role(Role::Identity)
    }

    /// A node that can show Desktop Mode and run heavy work on its own hardware.
    pub fn is_desktop_target(&self) -> bool {
        self.has_role(Role::Compute) && self.has_role(Role::Display)
    }

    /// Discrete GPU that can accept an offloaded workload.
    pub fn has_offload_gpu(&self) -> bool {
        self.gpu.as_ref().is_some_and(|gpu| gpu.memory_bytes > 0)
    }

    /// Reference Pixel 8 profile used by the local demo and by tests.
    ///
    /// The real phone image probes its own hardware. This descriptor exists so
    /// the protocol can be exercised on any developer machine.
    pub fn reference_pixel8(node_id: impl Into<String>) -> Self {
        Self {
            node_id: node_id.into(),
            name: "Pixel 8".into(),
            architecture: Architecture::Aarch64,
            roles: vec![Role::Identity, Role::Input, Role::Display, Role::Sensors],
            cpu: CpuInfo {
                model: "Google Tensor G3".into(),
                cores: 9,
                threads: 9,
            },
            memory: MemoryInfo {
                total_bytes: 8 * GIB,
            },
            gpu: Some(GpuInfo {
                vendor: "ARM".into(),
                model: "Mali-G715".into(),
                memory_bytes: 0,
            }),
            storage: None,
            display: DisplayInfo { outputs: 1 },
            power: PowerSource::Battery { percent: 70 },
        }
    }

    /// Reference desktop: Ryzen 5 5600X, 16 GiB RAM, GTX 1070, 2 TB SSD.
    pub fn reference_ryzen_desktop(node_id: impl Into<String>) -> Self {
        Self {
            node_id: node_id.into(),
            name: "Ryzen Desktop".into(),
            architecture: Architecture::X86_64,
            roles: vec![Role::Compute, Role::Display, Role::Storage],
            cpu: CpuInfo {
                model: "AMD Ryzen 5 5600X".into(),
                cores: 6,
                threads: 12,
            },
            memory: MemoryInfo {
                total_bytes: 16 * GIB,
            },
            gpu: Some(GpuInfo {
                vendor: "NVIDIA".into(),
                model: "GeForce GTX 1070".into(),
                memory_bytes: 8 * GIB,
            }),
            storage: Some(StorageInfo {
                nvme_bytes: 2_000_000_000_000,
            }),
            display: DisplayInfo { outputs: 1 },
            power: PowerSource::Ac,
        }
    }
}

/// Build a capability from the machine that is running the agent, then apply
/// the operator's roles, power, and optional GPU description.
pub fn from_host(
    node_id: &str,
    name: &str,
    roles: Vec<Role>,
    power: PowerSource,
    gpu: Option<GpuInfo>,
    storage: Option<StorageInfo>,
    display_outputs: u32,
) -> Capability {
    Capability {
        node_id: node_id.to_string(),
        name: name.to_string(),
        architecture: Architecture::host(),
        roles,
        cpu: probe_cpu(),
        memory: MemoryInfo {
            total_bytes: probe_memory_bytes().unwrap_or(0),
        },
        gpu,
        storage,
        display: DisplayInfo {
            outputs: display_outputs,
        },
        power,
    }
}

fn probe_cpu() -> CpuInfo {
    let threads = std::thread::available_parallelism()
        .map(|n| n.get() as u32)
        .unwrap_or(1);
    let cores = probe_physical_cores().unwrap_or(threads);
    CpuInfo {
        model: probe_cpu_model().unwrap_or_else(|| "unknown".into()),
        cores,
        threads,
    }
}

fn probe_physical_cores() -> Option<u32> {
    #[cfg(target_os = "macos")]
    {
        sysctl_u32("hw.physicalcpu")
    }
    #[cfg(target_os = "linux")]
    {
        let text = std::fs::read_to_string("/proc/cpuinfo").ok()?;
        let count = text
            .lines()
            .filter(|line| line.starts_with("processor"))
            .count();
        if count == 0 {
            None
        } else {
            Some(count as u32)
        }
    }
    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    {
        None
    }
}

fn probe_cpu_model() -> Option<String> {
    #[cfg(target_os = "macos")]
    {
        sysctl_string("machdep.cpu.brand_string")
    }
    #[cfg(target_os = "linux")]
    {
        let text = std::fs::read_to_string("/proc/cpuinfo").ok()?;
        for line in text.lines() {
            if let Some(rest) = line.strip_prefix("model name") {
                return rest.split(':').nth(1).map(|s| s.trim().to_string());
            }
        }
        None
    }
    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    {
        None
    }
}

fn probe_memory_bytes() -> Option<u64> {
    #[cfg(target_os = "macos")]
    {
        sysctl_u64("hw.memsize")
    }
    #[cfg(target_os = "linux")]
    {
        let text = std::fs::read_to_string("/proc/meminfo").ok()?;
        for line in text.lines() {
            if let Some(rest) = line.strip_prefix("MemTotal:") {
                let kb: u64 = rest.split_whitespace().next()?.parse().ok()?;
                return Some(kb * KIB);
            }
        }
        None
    }
    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    {
        None
    }
}

#[cfg(target_os = "macos")]
fn sysctl_string(key: &str) -> Option<String> {
    let output = std::process::Command::new("sysctl")
        .args(["-n", key])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let text = String::from_utf8(output.stdout).ok()?;
    let trimmed = text.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

#[cfg(target_os = "macos")]
fn sysctl_u64(key: &str) -> Option<u64> {
    sysctl_string(key)?.parse().ok()
}

#[cfg(target_os = "macos")]
fn sysctl_u32(key: &str) -> Option<u32> {
    sysctl_string(key)?.parse().ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reference_machines_match_the_first_hardware_targets() {
        let phone = Capability::reference_pixel8("phone");
        let desk = Capability::reference_ryzen_desktop("desk");
        assert_eq!(phone.architecture, Architecture::Aarch64);
        assert_eq!(phone.memory.total_bytes, 8 * GIB);
        assert!(phone.is_identity_anchor());
        assert!(!phone.has_offload_gpu());
        assert_eq!(desk.architecture, Architecture::X86_64);
        assert_eq!(desk.cpu.cores, 6);
        assert_eq!(desk.memory.total_bytes, 16 * GIB);
        assert!(desk.has_offload_gpu());
        assert!(desk.is_desktop_target());
        assert_eq!(desk.gpu.unwrap().model, "GeForce GTX 1070");
    }

    #[test]
    fn host_probe_reports_some_memory() {
        let cap = from_host(
            "node_test",
            "dev",
            vec![Role::Compute],
            PowerSource::Unknown,
            None,
            None,
            1,
        );
        assert!(cap.memory.total_bytes > 0);
        assert!(cap.cpu.threads >= 1);
    }
}
