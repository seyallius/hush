//! Translates raw terminal key events into domain Actions.

use crate::{
    action::Action,
    screen::{Focus, Screen},
};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

/// Resolves a raw key event into a domain action based to the current context.
///
/// This centralizes keybinding logic so that adding a new screen or focus area
/// forces a deliberate decision about how keys map to actions.
pub fn resolve_key(key: &KeyEvent, screen: &Screen, focus: &Focus) -> Action {
    // Global bindings that work anywhere
    if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
        return Action::Quit;
    }

    if key.code == KeyCode::Char('?') {
        return Action::ShowHelp;
    }

    // Context-specific bindings
    match focus {
        Focus::Modal => match key.code {
            KeyCode::Esc | KeyCode::Enter => Action::ClearError,
            _ => Action::None,
        },
        Focus::Main => match screen {
            Screen::List => match key.code {
                KeyCode::Up | KeyCode::Char('k') => Action::NavigateUp,
                KeyCode::Down | KeyCode::Char('j') => Action::NavigateDown,
                KeyCode::Enter => Action::Select,
                KeyCode::Char('q') => Action::Quit,
                _ => Action::None,
            },
            Screen::Detail | Screen::Config | Screen::Help => match key.code {
                KeyCode::Esc | KeyCode::Char('q') => Action::Back,
                _ => Action::None,
            },
        },
        Focus::Sidebar => {
            // Sidebar keybindings will be defined when sidebar components are added.
            Action::None
        }
    }
}
