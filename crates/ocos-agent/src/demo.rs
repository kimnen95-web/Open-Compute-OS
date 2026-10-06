//! Local Pixel 8 + Ryzen scenario.
//!
//! Both processes can run on a developer laptop. The capability descriptors are
//! the reference machines, so the demo does not depend on the laptop's CPU.

use crate::config::RunningNode;
use crate::link::{connect_peer, handle_incoming};
use crate::NodeConfig;
use anyhow::{Context, Result};
use ocos_capability::{Capability, GIB};
use ocos_fabric::{Fabric, Workload};
use ocos_identity::new_pairing_code;
use ocos_session::{SessionSnapshot, UiMode};
use std::fmt::Write;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::net::TcpListener;

pub const DEMO_NOTES_DRAFT: &str = "Written on the phone. Still here on the desktop.";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DemoReport {
    pub phone_node_id: String,
    pub desktop_node_id: String,
    pub mutual_trust: bool,
    pub desktop_mode: bool,
    pub phone_ui_mode: UiMode,
    pub desktop_ui_mode: UiMode,
    pub desktop_runtime_node_id: String,
    pub notes_draft: String,
    pub notes_cursor: u64,
    pub placement_node_id: String,
    pub placement_memory_bytes: u64,
    pub pooled_memory_rejected: bool,
}

impl std::fmt::Display for DemoReport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Open Compute OS demo")?;
        writeln!(f, "  phone   {}", self.phone_node_id)?;
        writeln!(f, "  desktop {}", self.desktop_node_id)?;
        writeln!(f, "  mutual trust: {}", self.mutual_trust)?;
        writeln!(f, "  desktop mode: {}", self.desktop_mode)?;
        writeln!(
            f,
            "  phone runtime was {:?}; desktop runtime is {:?} on {}",
            self.phone_ui_mode, self.desktop_ui_mode, self.desktop_runtime_node_id
        )?;
        writeln!(
            f,
            "  notes draft: {:?} (cursor {})",
            self.notes_draft, self.notes_cursor
        )?;
        writeln!(
            f,
            "  video-encode placed on {} using {} bytes of that node",
            self.placement_node_id, self.placement_memory_bytes
        )?;
        write!(
            f,
            "  20 GiB job rejected because RAM is not pooled: {}",
            self.pooled_memory_rejected
        )
    }
}

struct TempDir(PathBuf);

impl Drop for TempDir {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).ok();
    }
}

pub async fn run_demo() -> Result<DemoReport> {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .context("clock")?
        .as_nanos();
    let root = TempDir(std::env::temp_dir().join(format!("ocos-demo-{nanos}")));
    let phone_dir = root.0.join("phone");
    let desktop_dir = root.0.join("desktop");
    let mut phone = RunningNode::init(&phone_dir, NodeConfig::phone())?;
    let mut desktop = RunningNode::init(&desktop_dir, NodeConfig::desktop())?;
    let code = new_pairing_code();
    phone.set_pairing_code(code.clone())?;

    let phone_cap = Capability::reference_pixel8(&phone.identity.node_id);
    let desktop_cap = Capability::reference_ryzen_desktop(&desktop.identity.node_id);
    let mut session = SessionSnapshot::new("ses_demo", &phone.identity.node_id);
    session.open_notes("welcome", DEMO_NOTES_DRAFT, 42);

    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let address = listener.local_addr()?;
    let phone_for_task = phone;
    let session_for_task = session.clone();
    let phone_cap_for_task = phone_cap.clone();
    let server = tokio::spawn(async move {
        let (stream, _) = listener.accept().await?;
        let mut phone = phone_for_task;
        handle_incoming(stream, &mut phone, &phone_cap_for_task, &session_for_task).await?;
        phone.save_trust()?;
        Ok::<_, anyhow::Error>(phone)
    });

    let client = connect_peer(&mut desktop, address, Some(&code), &desktop_cap).await?;
    let phone = server.await.context("phone task")??;

    let phone_trust_has_desktop = phone.trust.contains(&desktop.identity.node_id);
    let desktop_trust_has_phone = desktop.trust.contains(&phone.identity.node_id);
    let notes = client
        .session
        .as_ref()
        .and_then(|session| session.notes())
        .context("desktop did not receive the notes window")?;

    let mut fabric = Fabric::new();
    fabric.upsert(phone_cap, true, true);
    fabric.upsert(desktop_cap.clone(), true, true);
    let placement = fabric.schedule(&Workload::video_encode())?;
    let pooled = Workload {
        name: "would-only-fit-if-ram-were-pooled".into(),
        needs_gpu: false,
        min_memory_bytes: 20 * GIB,
        prefer_ac: false,
    };
    let pooled_memory_rejected = fabric.schedule(&pooled).is_err();

    let report = DemoReport {
        phone_node_id: phone.identity.node_id,
        desktop_node_id: desktop.identity.node_id,
        mutual_trust: phone_trust_has_desktop && desktop_trust_has_phone,
        desktop_mode: client.desktop_mode,
        phone_ui_mode: session.mode,
        desktop_ui_mode: client
            .session
            .as_ref()
            .map(|session| session.mode)
            .unwrap_or(UiMode::Mobile),
        desktop_runtime_node_id: client
            .session
            .as_ref()
            .map(|session| session.runtime_node_id.clone())
            .unwrap_or_default(),
        notes_draft: notes.draft.clone().unwrap_or_default(),
        notes_cursor: notes.cursor.unwrap_or(0),
        placement_node_id: placement.node_id,
        placement_memory_bytes: placement.memory_bytes,
        pooled_memory_rejected,
    };

    if report.placement_node_id != report.desktop_node_id {
        anyhow::bail!("video encode was not placed on the desktop node");
    }
    Ok(report)
}

pub fn render_link(report: &crate::link::LinkReport) -> String {
    let mut text = String::new();
    let _ = writeln!(text, "peer {} ({})", report.peer_name, report.peer_node_id);
    let _ = writeln!(text, "trusted: {}", report.trusted);
    let _ = writeln!(text, "desktop mode: {}", report.desktop_mode);
    if let Some(session) = &report.session {
        let _ = writeln!(text, "ui mode: {:?}", session.mode);
        for window in &session.windows {
            let _ = writeln!(
                text,
                "window {} {:?} cursor {:?}",
                window.title, window.draft, window.cursor
            );
        }
    }
    text
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn reference_demo_preserves_notes_and_refuses_pooled_ram() {
        let report = run_demo().await.unwrap();
        assert!(report.mutual_trust);
        assert!(report.desktop_mode);
        assert_eq!(report.phone_ui_mode, UiMode::Mobile);
        assert_eq!(report.desktop_ui_mode, UiMode::Desktop);
        assert_eq!(report.desktop_runtime_node_id, report.desktop_node_id);
        assert_eq!(report.notes_draft, DEMO_NOTES_DRAFT);
        assert_eq!(report.notes_cursor, 42);
        assert_eq!(report.placement_node_id, report.desktop_node_id);
        assert!(report.pooled_memory_rejected);
        assert_eq!(report.placement_memory_bytes, 16 * GIB);
    }
}
