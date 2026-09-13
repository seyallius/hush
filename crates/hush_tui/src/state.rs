//! Defines the global application state for the TUI.

use crate::screen::{Focus, Screen};
use hush_config::Config;
use hush_core::Error;

/// The global state of the TUI application.
///
/// This struct holds all the data needed to render the UI and process events.
/// It is passed to components during event handling and rendering.
pub struct AppState {
    /// The currently active screen.
    pub current_screen: Screen,
    /// The currently focused pane or region.
    pub focus: Focus,
    /// The index of the currently selected item in a list view.
    pub selected_index: usize,
    /// The total number of items in the current list view.
    pub item_count: usize,
    /// A transient status message to display to the user.
    pub status_message: String,
    /// An active error to be displayed in a modal or error bar.
    pub active_error: Option<Error>,
    /// The resolved application configuration.
    pub config: Config,
}
impl AppState {
    /// Creates a new `AppState` with default values.
    pub fn new(config: Config) -> Self {
        Self {
            current_screen: Screen::List,
            focus: Focus::Main,
            selected_index: 0,
            item_count: 0,
            status_message: String::from("Ready"),
            active_error: None,
            config,
        }
    }

    /// Surfaces an error to the user by storing it and shifting focus to the error modal.
    ///
    /// This satisfies the contract that errors are representable without panicking.
    pub fn surface_error(&mut self, err: Error) {
        self.active_error = Some(err);
        self.focus = Focus::Modal;
    }

    /// Clears the active error and returns focus to the main pane.
    pub fn clear_error(&mut self) {
        self.active_error = None;
        self.focus = Focus::Main;
    }
}
