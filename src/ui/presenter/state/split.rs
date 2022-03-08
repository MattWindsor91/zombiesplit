///! Presenter state for individual splits.
use std::fmt::Display;

use super::{cursor::SplitPosition, editor::Editor};
use crate::model::{
    session::{
        self,
        event::{split, time},
    },
    short,
    timing::{
        aggregate,
        comparison::pace::{self, PacedTime},
    },
};

/// A set of split state data.
///
/// This is opaque so as to preserve the invariant that every split can be found by both shortname
/// and by index, and the indices are in sync.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Set {
    short_map: short::Map<usize>,
    vec: Vec<Split>,
}

/// We can produce a split set from an iterator over split dumps.
impl FromIterator<session::Split> for Set {
    fn from_iter<T: IntoIterator<Item = session::Split>>(iter: T) -> Self {
        let mut result = Self::default();

        for (index, split) in iter.into_iter().enumerate() {
            result.short_map.insert(split.info.short, index);
            result.vec.push(Split::from_dump(&split));
        }

        result
    }
}

impl Set {
    /// Constructs a split state from an attempt dump.
    #[must_use]
    pub fn from_dump(dump: &session::State) -> Self {
        dump.run.splits.iter().cloned().collect()
    }

    /// Handles an event for the split with short name `split`.
    pub fn handle_event(&mut self, split: short::Name, evt: &split::Split) {
        if let Some(s) = self.at_short_mut(split) {
            s.handle_event(evt);
        }
    }

    /// Resets the splits ready for a new run.
    pub fn reset(&mut self) {
        for split in &mut self.vec {
            split.reset();
        }
    }

    /// Refreshes the cursor position information for each split.
    pub fn refresh_cursors(&mut self, cur: &super::cursor::Cursor) {
        for (i, s) in &mut self.vec.iter_mut().enumerate() {
            s.position = cur.split_position(i);
        }
    }

    /// Sets the editor at `position` to `editor`, removing all other open editors.
    pub fn set_editor(&mut self, position: usize, editor: Option<&super::super::Editor>) {
        if let Some(s) = self.vec.get_mut(position) {
            s.set_editor(editor);
        }
    }

    /// Gets the number of splits in the set.
    ///
    /// ```
    /// use zombiesplit::ui::presenter::state::split;
    /// use zombiesplit::model::{game, session::Split};
    ///
    /// let s = split::Set::default();
    /// assert_eq!(0, s.len());
    ///
    /// let vec = vec![
    ///   Split::new(game::Split::new("pp1", "Palmtree Panic 1")),
    ///   Split::new(game::Split::new("pp2", "Palmtree Panic 2")),
    ///   Split::new(game::Split::new("pp3", "Palmtree Panic 3")),
    /// ];
    /// let s2 = split::Set::from_iter(vec);
    /// assert_eq!(3, s2.len());
    /// ```
    #[must_use]
    pub fn len(&self) -> usize {
        self.vec.len()
    }

    /// Gets whether the split set is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.vec.is_empty()
    }

    fn at_short_mut(&mut self, split: short::Name) -> Option<&mut Split> {
        self.index_of(split).and_then(|x| self.vec.get_mut(x))
    }

    /// Gets the last-seen index of the split `split`.
    #[must_use]
    pub fn index_of(&self, split: short::Name) -> Option<usize> {
        self.short_map.get(&split).copied()
    }

    /// Sets the number of splits in the set to `count`.
    ///
    /// Existing splits will be truncated; any new splits will be filled in with the default value.
    pub fn set_split_count(&mut self, count: usize) {
        self.vec.resize_with(count, Default::default);
    }

    /// Tries to get the split at `index`.
    #[must_use]
    pub fn at_index(&self, index: usize) -> Option<&Split> {
        self.vec.get(index)
    }
}

/// Presenter state about one split.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Split {
    /// The number of times that have been logged on this split.
    pub num_times: usize,
    /// The display name of the split.
    pub name: String,
    /// The aggregate times logged for this split.
    pub aggregates: aggregate::Full,
    /// The pace of this split in the run-so-far.
    pub pace_in_run: pace::SplitInRun,
    /// The last logged cursor-relative position for this split.
    pub position: SplitPosition,
    /// Any editor active on this split.
    pub editor: Option<Editor>,
}

impl Split {
    /// Creates a new split with a display name, but no other data logged.
    ///
    /// ```
    /// use zombiesplit::ui::presenter::state;
    ///
    /// let s = state::Split::new("Palmtree Panic 1");
    /// assert_eq!("Palmtree Panic 1", s.name);
    /// assert_eq!(0, s.num_times);
    /// assert_eq!(zombiesplit::model::timing::comparison::pace::SplitInRun::Inconclusive, s.pace_in_run);
    /// ```
    #[must_use]
    pub fn new<N: Display>(name: N) -> Self {
        let name = name.to_string();
        Self {
            name,
            ..Self::default()
        }
    }

    #[must_use]
    pub fn from_dump(dump: &session::split::Split) -> Self {
        Self {
            num_times: dump.times.len(),
            name: dump.info.name.clone(),

            // TODO(@MattWindsor91): do this.
            aggregates: Default::default(),
            pace_in_run: Default::default(),
            position: Default::default(),
            editor: None,
        }
    }

    /// Resets the per-run state of this split.
    ///
    /// This clears the aggregates, pacing information, and time count; it
    /// doesn't reset metadata.
    pub fn reset(&mut self) {
        self.num_times = 0;
        self.aggregates = aggregate::Full::default();
        self.pace_in_run = pace::SplitInRun::default();
    }

    /// Gets the cumulative time at this split along with its pace note.
    #[must_use]
    pub fn paced_cumulative(&self) -> PacedTime {
        let time = self.aggregates[aggregate::Source::Attempt][aggregate::Scope::Cumulative];
        PacedTime {
            pace: self.pace_in_run.overall(),
            time,
        }
    }

    /// Handles an observation for this split.
    pub fn handle_event(&mut self, evt: &split::Split) {
        match evt {
            split::Split::Time(t, time::Time::Aggregate(kind)) => {
                self.aggregates[kind.source][kind.scope] = *t;
            }
            split::Split::Time(_, time::Time::Pushed) => {
                self.num_times += 1;
            }
            split::Split::Time(_, time::Time::Popped) => {
                self.num_times -= 1;
                // Moving the newly popped time to the editor gets handled
                // elsewhere.
            }
            split::Split::Pace(pace) => {
                self.pace_in_run = *pace;
            }
        }
    }

    /// Populates this split state with the current state of `editor`.
    pub fn set_editor(&mut self, editor: Option<&super::super::mode::Editor>) {
        self.editor = editor.map(|e| {
            let mut out = Editor {
                hours: e.time.hours.to_string(),
                mins: e.time.mins.to_string(),
                secs: e.time.secs.to_string(),
                msecs: e.time.millis.to_string(),
                field: None,
            };

            if let Some(ref field) = e.field {
                let pos = field.position();
                out.field = Some(pos);
                out[pos] = field.to_string();
            }

            out
        });
    }
}
