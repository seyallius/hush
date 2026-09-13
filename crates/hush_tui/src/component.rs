//! Defines the Component trait for UI composition.

use crate::{action::Action, event::Event, screen::Focus, state::AppState};
use ratatui::{Frame, layout::Rect};

/// The core trait for all TUI UI elements.
///
/// Components are responsible for handling events that occur while they are
/// focused, and rendering themselves into a given area of the terminal.
pub trait Component {
    /// Processes an incoming event and returns an action to mutate the state.
    ///
    /// Components should only process events if they are the currently focused
    /// component, or if the event is global (like `Ctrl+C`).
    fn handle_event(&mut self, event: &Event, state: &mut AppState) -> Action;
    /// Renders the component into the provided terminal frame and area.
    ///
    /// This method must not mutate the application state.
    fn render(&self, state: &AppState, frame: &mut Frame, area: Rect);
    /// Returns the focus region this component occupies.
    fn focus(&self) -> Focus;
}

/// A stub component used as a placeholder during the skeleton phase.
///
/// This satisfies the `todo!()` requirement for Issue #31 while allowing
/// the architecture to compile before actual widgets are built in Issue #6.
pub struct StubComponent;
impl Component for StubComponent {
    fn handle_event(&mut self, _event: &Event, _state: &mut AppState) -> Action {
        todo!("Implement event handling for this component in Issue #6")
    }

    fn render(&self, _state: &AppState, _frame: &mut Frame, _area: Rect) {
        todo!("Implement rendering for this component in Issue #6")
    }

    fn focus(&self) -> Focus {
        Focus::Main
    }
}
