//! Decide which compute node should run a workload.
//!
//! Every node keeps its own CPU and memory. This scheduler never adds those
//! sizes together and never pretends two architectures are one SMP machine.

use ocos_capability::{Capability, PowerSource};
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NodeState {
    pub capability: Capability,
    pub trusted: bool,
    pub connected: bool,
}

#[derive(Debug, Clone)]
pub struct Fabric {
    nodes: BTreeMap<String, NodeState>,
}

/// A job that must fit on one node.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Workload {
    pub name: String,
    pub needs_gpu: bool,
    pub min_memory_bytes: u64,
    pub prefer_ac: bool,
}

impl Workload {
    /// Sample offload used by the demo: encode video on a discrete GPU.
    pub fn video_encode() -> Self {
        Self {
            name: "video-encode".into(),
            needs_gpu: true,
            min_memory_bytes: 4 * ocos_capability::GIB,
            prefer_ac: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Placement {
    pub node_id: String,
    pub node_name: String,
    pub memory_bytes: u64,
    pub reason: String,
}

impl Fabric {
    pub fn new() -> Self {
        Self {
            nodes: BTreeMap::new(),
        }
    }

    pub fn upsert(&mut self, capability: Capability, trusted: bool, connected: bool) {
        let node_id = capability.node_id.clone();
        self.nodes.insert(
            node_id,
            NodeState {
                capability,
                trusted,
                connected,
            },
        );
    }

    pub fn nodes(&self) -> impl Iterator<Item = &NodeState> {
        self.nodes.values()
    }

    /// Place `job` on one trusted, connected node.
    ///
    /// Memory is compared per node. A job that fits only if two nodes' RAM
    /// were added together is rejected.
    pub fn schedule(&self, job: &Workload) -> Result<Placement, ScheduleError> {
        let available: Vec<&Capability> = self
            .nodes
            .values()
            .filter(|node| node.trusted && node.connected)
            .map(|node| &node.capability)
            .collect();
        if available.is_empty() {
            return Err(ScheduleError::NoNodes);
        }

        let fitting: Vec<&Capability> = available
            .iter()
            .copied()
            .filter(|node| {
                node.memory.total_bytes >= job.min_memory_bytes
                    && (!job.needs_gpu || node.has_offload_gpu())
            })
            .collect();

        if fitting.is_empty() {
            if job.needs_gpu && !available.iter().any(|node| node.has_offload_gpu()) {
                return Err(ScheduleError::NoGpu);
            }
            let largest = available
                .iter()
                .map(|node| node.memory.total_bytes)
                .max()
                .unwrap_or(0);
            return Err(ScheduleError::NotEnoughMemory {
                needed: job.min_memory_bytes,
                largest_node_bytes: largest,
            });
        }

        let mut chosen_pool = fitting;
        if job.prefer_ac {
            let on_ac: Vec<&Capability> = chosen_pool
                .iter()
                .copied()
                .filter(|node| matches!(node.power, PowerSource::Ac))
                .collect();
            if !on_ac.is_empty() {
                chosen_pool = on_ac;
            }
        }

        chosen_pool.sort_by(|left, right| {
            right
                .memory
                .total_bytes
                .cmp(&left.memory.total_bytes)
                .then_with(|| gpu_memory(right).cmp(&gpu_memory(left)))
                .then_with(|| left.node_id.cmp(&right.node_id))
        });
        let chosen = chosen_pool[0];
        Ok(Placement {
            node_id: chosen.node_id.clone(),
            node_name: chosen.name.clone(),
            memory_bytes: chosen.memory.total_bytes,
            reason: format!(
                "placed on {} alone ({} bytes); other nodes' memory is not added",
                chosen.name, chosen.memory.total_bytes
            ),
        })
    }
}

impl Default for Fabric {
    fn default() -> Self {
        Self::new()
    }
}

fn gpu_memory(capability: &Capability) -> u64 {
    capability
        .gpu
        .as_ref()
        .map(|gpu| gpu.memory_bytes)
        .unwrap_or(0)
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum ScheduleError {
    #[error("no trusted node is connected")]
    NoNodes,
    #[error("no connected node has a discrete GPU")]
    NoGpu,
    #[error(
        "no single node has {needed} bytes (largest connected node has {largest_node_bytes} bytes); memory is not pooled"
    )]
    NotEnoughMemory {
        needed: u64,
        largest_node_bytes: u64,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    use ocos_capability::{Capability, GIB};

    fn fabric() -> Fabric {
        let mut fabric = Fabric::new();
        fabric.upsert(Capability::reference_pixel8("phone"), true, true);
        fabric.upsert(Capability::reference_ryzen_desktop("desk"), true, true);
        fabric
    }

    #[test]
    fn gpu_work_goes_to_the_desktop_node() {
        let placement = fabric().schedule(&Workload::video_encode()).unwrap();
        assert_eq!(placement.node_id, "desk");
        assert_eq!(placement.memory_bytes, 16 * GIB);
    }

    #[test]
    fn memory_is_not_pooled_across_phone_and_desktop() {
        let job = Workload {
            name: "too-big-for-one-node".into(),
            needs_gpu: false,
            min_memory_bytes: 20 * GIB,
            prefer_ac: false,
        };
        let error = fabric().schedule(&job).unwrap_err();
        assert_eq!(
            error,
            ScheduleError::NotEnoughMemory {
                needed: 20 * GIB,
                largest_node_bytes: 16 * GIB,
            }
        );
    }

    #[test]
    fn untrusted_desktop_cannot_receive_work() {
        let mut fabric = Fabric::new();
        fabric.upsert(Capability::reference_pixel8("phone"), true, true);
        fabric.upsert(Capability::reference_ryzen_desktop("desk"), false, true);
        assert_eq!(
            fabric.schedule(&Workload::video_encode()),
            Err(ScheduleError::NoGpu)
        );
    }
}
