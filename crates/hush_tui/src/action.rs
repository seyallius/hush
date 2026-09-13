//! Defines internal actions that mutate the TUI state.

use hush_core::Error;

/// Internal actions that describe intent and state mutations.
///
/// Components translate `Event`s into `Action`s. The main application loop
/// then applies these actions to the `AppState`.
#[derive(Debug)]
pub enum Action {
    /// Exit the application.
    Quit,
    /// Move selection up.
    NavigateUp,
    /// Move selection down.
    NavigateDown,
    /// Confirm or open the selected item.
    Select,
    /// Return to the previous screen.
    Back,
    /// Open the help overlay.
    ShowHelp,
    /// Process a periodic tick.
    Tick,
    /// Handle a terminal resize.
    Resize(u16, u16),
    /// Surface a domain error to the user.
    Error(Error),
    /// Dismiss the active error modal.
    ClearError,
    /// No operation.
    None,
}
