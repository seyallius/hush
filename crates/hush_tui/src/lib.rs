//! lib.rs defines the contracts and state management for the Terminal User Interface.
//! It remains completely decoupled from the CLI argument parsing logic.

/// The central state tree for the TUI application.
/// Tracks the current screen, user selections, and error states.
pub struct AppState {
    /// Whether the application should continue running or exit.
    pub running: bool,
}
impl Default for AppState {
    fn default() -> Self {
        Self { running: true }
    }
}

/// Handles terminal resize events and redraws the UI layout.
fn handle_resize() {
    unimplemented!("Will be implemented using `ratatui` in a later issue.")
}
