//! Application sessions that stay alive when a desktop node is attached.
//!
//! The snapshot describes windows that are already running on the identity
//! node (the phone). Desktop Mode is a change of presentation: the same
//! process, the same draft, the same cursor. Plugging in a cable does not
//! relaunch the app and does not migrate an ARM process onto x86.

use serde::{Deserialize, Serialize};

/// Reference app id for the Notes session used by the demo and tests.
pub const NOTES_APP_ID: &str = "os.ocos.notes";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiMode {
    Mobile,
    Desktop,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct WindowBounds {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

/// State of one open window. This is not a new process.
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
    pub owner_node_id: String,
    pub mode: UiMode,
    pub windows: Vec<WindowState>,
}

impl SessionSnapshot {
    pub fn new(session_id: impl Into<String>, owner_node_id: impl Into<String>) -> Self {
        Self {
            session_id: session_id.into(),
            owner_node_id: owner_node_id.into(),
            mode: UiMode::Mobile,
            windows: Vec::new(),
        }
    }

    /// Open Notes inside the existing session. The process stays on the owner node.
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

    /// Same windows, drawn by the desktop shell.
    pub fn present_on_desktop(&self) -> Self {
        let mut next = self.clone();
        next.mode = UiMode::Desktop;
        next
    }

    /// Same windows, drawn by the phone shell after the cable is removed.
    pub fn present_on_mobile(&self) -> Self {
        let mut next = self.clone();
        next.mode = UiMode::Mobile;
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
        let desktop = session.present_on_desktop();
        assert_eq!(desktop.mode, UiMode::Desktop);
        assert_eq!(
            desktop.notes().unwrap().draft.as_deref(),
            Some("half written")
        );
        assert_eq!(desktop.notes().unwrap().cursor, Some(12));
        assert_eq!(session.mode, UiMode::Mobile);

        let phone = desktop.present_on_mobile();
        assert_eq!(phone.mode, UiMode::Mobile);
        assert_eq!(
            phone.notes().unwrap().draft.as_deref(),
            Some("half written")
        );
        assert_eq!(phone.windows.len(), session.windows.len());
    }
}
