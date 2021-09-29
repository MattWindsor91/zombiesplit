//! The [Nav] struct and its implementations.

use super::{
    super::{
        cursor::{self, Cursor},
        State,
    },
    editor::Editor,
    event::{self, Modal},
    EventContext, EventResult, Mode,
};
use crate::model::time::position;

/// Mode for when we are navigating splits.
pub struct Nav {
    /// The cursor.
    cur: Cursor,
}

impl Mode for Nav {
    fn on_entry(&mut self, state: &mut State) {
        state.set_cursor(Some(self.cur.position()));
    }

    fn on_event(&mut self, ctx: EventContext) -> EventResult {
        match ctx.event {
            Modal::Cursor(c) => self.move_cursor(c, ctx.state),
            Modal::EnterField(f) => self.enter_field(f),
            Modal::Undo => self.undo(),
            Modal::Delete => self.delete(),
            _ => EventResult::Handled,
        }
    }

    fn on_exit(&mut self, _state: &mut crate::ui::presenter::State) -> Option<event::Attempt> {
        // Don't clear the cursor, it'll probably be used by the new state.
        None
    }
}

impl Nav {
    /// Creates a new nav mode using a given cursor.
    #[must_use]
    pub fn new(cur: Cursor) -> Self {
        Self { cur }
    }

    /// Creates a transition to a navigation from the given cursor.
    #[must_use]
    pub fn transition(cur: Cursor) -> EventResult {
        EventResult::transition(Self::new(cur))
    }

    /// Performs an undo on the current split, if any.
    fn undo(&mut self) -> EventResult {
        EventResult::Expanded(event::Attempt::Pop(self.cur.position()))
    }

    /// Performs a delete on the current split, if any.
    fn delete(&mut self) -> EventResult {
        EventResult::Expanded(event::Attempt::Clear(self.cur.position()))
    }

    /// Moves the state cursor according to `c`, if possible.
    fn move_cursor(
        &mut self,
        motion: cursor::Motion,
        state: &mut super::super::State,
    ) -> EventResult {
        // TODO(@MattWindsor91): cursor multiplier
        self.cur.move_by(motion, 1);
        state.set_cursor(Some(self.cur.position()));

        EventResult::Handled
    }

    /// Constructs an editor entering the given field.
    fn enter_field(&self, field: position::Name) -> EventResult {
        let editor = Editor::new(self.cur, Some(field));
        EventResult::transition(editor)
    }
}
