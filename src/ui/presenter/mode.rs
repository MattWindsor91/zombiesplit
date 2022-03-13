//! The Mode trait and associated functionality.

pub mod editor;
pub mod event;
pub mod nav;
pub mod quitting;

pub use editor::Editor;
pub use event::Event;
pub use nav::Nav;
pub use quitting::Quitting;
use std::fmt::Display;

use super::State;
use crate::model::session::action;

/// Trait for presenter modes.
///
/// zombiesplit is heavily modal, so most of the current presenter state
/// depends on the current mode.
///
/// Modes can:
///
/// - interpret a certain subset of UI events, turning them into events on the
///   model or transitions to other modes;
/// - modify the presenter's visual state;
/// - retain their own state, such as a split editor or a cursor.
///
/// They can also be `Display`ed; this should show a condensed form of the mode to fit within eg.
/// a status bar.
pub trait Mode: Display {
    /// Called when the mode has been swapped in.
    ///
    /// The [Mode] can perform any initialisation on the visual `state` here.
    fn on_entry(&mut self, state: &mut State);

    /// Handles the mode-specific event given in `ctx`.
    ///
    /// The mode also receives, in `ctx`, the ability to modify both the view
    /// state being displayed in the UI and parts of the downstream.  Note
    /// however that `commit` will get when this mode transitions out, and any
    /// modifications can be batched until then.
    ///
    /// Note that the presenter also handles some events at the global level.
    fn on_event(&mut self, ctx: event::Context) -> event::Outcome;

    /// Called when the mode is about to be swapped out.
    ///
    /// The [Mode] can perform any last-minute adjustments to the visual
    /// `state`, and optionally return a follow-on [Action] representing the
    /// application of this mode's efforts to the model.
    fn on_exit(&mut self, state: &mut State) -> Option<action::Action>;

    /// Is the client running while this mode is active?
    fn is_running(&self) -> bool {
        true
    }
}
