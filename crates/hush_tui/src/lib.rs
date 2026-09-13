//! Root crate for hush_tui, defining the TUI contracts, state, and component model.
//! It defines the contracts and state management for the Terminal User Interface.
//! It remains completely decoupled from the CLI argument parsing logic.

pub mod action;
pub mod component;
pub mod event;
pub mod keymap;
pub mod screen;
pub mod state;

pub use action::Action;
pub use component::Component;
pub use event::Event;
pub use screen::{Focus, Screen};
pub use state::AppState;

// -------------------------------------- Internal Helpers -------------------------------------- //

/// Handles terminal resize events and redraws the UI layout.
#[allow(dead_code)]
fn handle_resize() {
    unimplemented!("Will be implemented using `ratatui` in a later issue.")
}
