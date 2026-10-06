//! Application sessions shared by the phone and desktop images.
//!
//! Plugging in hands the open work to the desktop runtime. That runtime is the
//! x86 build of the same app, opened at the same document and cursor. Unplugging
//! hands the latest state back to the phone runtime. An ARM process is not
//! moved onto the x86 CPU.

use serde::{Deserialize, Serialize};

/// Reference app id for the Notes session used by the demo and tests.
pub const NOTES_APP_ID: &str = "os.ocos.notes";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiMode {
    Mobile,
    Desktop,
}

/// Which build of an Open Compute OS app is hosting the session.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RuntimeArch {
    #[serde(rename = "aarch64")]
    Aarch64,
    #[serde(rename = "x86_64")]
    X86_64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct WindowBounds {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

/// State of one open window, independent of which CPU is running it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WindowState {
    pub app_id: String,
    pub title: String,
    pub document_id: Option<String>,
    pub cursor: Option<u64>,
    pub draft: Option<String>,
    pub bounds: Option<WindowBounds>,
}

/// Everything the shell needs to show the user's current work.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionSnapshot {
    pub session_id: String,
    /// Node that owns the user's data. On the first hardware pair this is the phone.
    pub owner_node_id: String,
    /// Node whose app runtime currently has the windows open.
    pub runtime_node_id: String,
    pub runtime_arch: RuntimeArch,
    pub mode: UiMode,
    pub windows: Vec<WindowState>,
}

impl SessionSnapshot {
    pub fn new(session_id: impl Into<String>, owner_node_id: impl Into<String>) -> Self {
        let owner_node_id = owner_node_id.into();
        Self {
            session_id: session_id.into(),
            runtime_node_id: owner_node_id.clone(),
            runtime_arch: RuntimeArch::Aarch64,
            owner_node_id,
            mode: UiMode::Mobile,
            windows: Vec::new(),
        }
    }

    /// Open Notes inside the existing session.
    pub fn open_notes(
        &mut self,
        document_id: impl Into<String>,
        draft: impl Into<String>,
        cursor: u64,
    ) {
        let document_id = document_id.into();
        self.windows.push(WindowState {
            app_id: NOTES_APP_ID.into(),
            title: format!("Notes — {document_id}"),
            document_id: Some(document_id),
            cursor: Some(cursor),
            draft: Some(draft.into()),
            bounds: None,
        });
    }

    pub fn notes(&self) -> Option<&WindowState> {
        self.windows
            .iter()
            .find(|window| window.app_id == NOTES_APP_ID)
    }

    /// Open this work in the desktop node's own app runtime.
    pub fn open_on_desktop(&self, desktop_node_id: &str) -> Self {
        self.open_on(desktop_node_id, RuntimeArch::X86_64, UiMode::Desktop)
    }

    /// Return this work to the phone node's own app runtime.
    pub fn resume_on_phone(&self, phone_node_id: &str) -> Self {
        self.open_on(phone_node_id, RuntimeArch::Aarch64, UiMode::Mobile)
    }

    fn open_on(&self, node_id: &str, arch: RuntimeArch, mode: UiMode) -> Self {
        let mut next = self.clone();
        next.runtime_node_id = node_id.to_string();
        next.runtime_arch = arch;
        next.mode = mode;
        next
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn desktop_mode_keeps_the_draft_and_cursor() {
        let mut session = SessionSnapshot::new("ses_demo", "node_phone");
        session.open_notes("welcome", "half written", 12);
        let desktop = session.open_on_desktop("node_desk");
        assert_eq!(desktop.mode, UiMode::Desktop);
        assert_eq!(desktop.runtime_arch, RuntimeArch::X86_64);
        assert_eq!(desktop.runtime_node_id, "node_desk");
        assert_eq!(desktop.owner_node_id, "node_phone");
        assert_eq!(
            desktop.notes().unwrap().draft.as_deref(),
            Some("half written")
        );
        assert_eq!(desktop.notes().unwrap().cursor, Some(12));
        assert_eq!(session.runtime_arch, RuntimeArch::Aarch64);

        let phone = desktop.resume_on_phone("node_phone");
        assert_eq!(phone.mode, UiMode::Mobile);
        assert_eq!(phone.runtime_arch, RuntimeArch::Aarch64);
        assert_eq!(phone.runtime_node_id, "node_phone");
        assert_eq!(
            phone.notes().unwrap().draft.as_deref(),
            Some("half written")
        );
    }
}
