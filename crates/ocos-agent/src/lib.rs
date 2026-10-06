//! Node agent for Open Compute OS.
//!
//! `ocos-agent` is the process every compute node runs. It publishes a
//! capability, proves its device key, pairs once, and then carries session
//! state to a desktop node. The phone process keeps running.

mod auth;
mod config;
mod demo;
mod link;

pub use config::{NodeConfig, NodePaths, RunningNode};
pub use demo::{render_link, run_demo, DemoReport, DEMO_NOTES_DRAFT};
pub use link::{connect_peer, handle_incoming, listen, LinkReport};
pub use ocos_identity::new_pairing_code;
