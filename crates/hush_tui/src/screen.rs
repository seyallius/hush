//! Defines the available screens and focus models for the TUI.

/// Represents the top-level screens in the TUI application.
///
/// Adding a new screen forces a compile-time decision about how it is rendered
/// and how it integrates into the navigation keymap.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    /// The main file browser list.
    List,
    /// Detailed view of a single encrypted file.
    Detail,
    /// Configuration management screen.
    Config,
    /// Help and keybindings overlay.
    Help,
}

/// Represents the currently focused region within a screen.
///
/// This enables multi-pane screens where different components can intercept
/// keyboard events based on the current focus.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Focus {
    /// The primary content area (e.g., the file list).
    Main,
    /// A secondary sidebar or preview pane.
    Sidebar,
    /// A modal overlay (e.g., error dialogs, confirmation prompts).
    Modal,
}
