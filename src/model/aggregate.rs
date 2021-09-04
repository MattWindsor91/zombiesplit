/*! Events and storage for sending information about aggregate times on splits.

zombiesplit tracks two [Scope]s of aggregate (total across all times in the
split, and cumulative totals over all splits up to and including the split),
and two [Source]s (from the current attempt, and from the database comparison).

This module contains both the event building blocks used to transmit
observations of aggregate changes from the module, as well as structures for
storing those aggregates.
*/

use std::ops::{Index, IndexMut};

use super::Time;

/// The kind ([Source] and [Scope]) of an aggregate time.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Kind {
    /// The source of this aggregate time.
    pub source: Source,
    /// The scope of this aggregate time.
    pub scope: Scope,
}

impl Kind {
    /// A cumulative time from the current attempt.
    pub const ATTEMPT_CUMULATIVE: Self = Source::Attempt.with(Scope::Cumulative);

    /// A split time from the current attempt.
    pub const ATTEMPT_SPLIT: Self = Source::Attempt.with(Scope::Split);

    /// A cumulative time from the comparison.
    pub const COMPARISON_CUMULATIVE: Self = Source::Comparison.with(Scope::Cumulative);

    /// A split time from the comparison.
    pub const COMPARISON_SPLIT: Self = Source::Comparison.with(Scope::Split);
}

/// Enumeration of sources for aggregate times.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Source {
    /// This time comes from the run attempt.
    Attempt,
    /// This time comes from the comparison; ie, it is the time to which we are
    /// comparing.
    Comparison,
}

impl Source {
    /// Creates a `Kind` using this `Source` and a given `Scope` `scope`.
    ///
    /// # Examples
    ///
    /// ```
    /// use zombiesplit::model::attempt::observer::aggregate::{Source, Scope, Kind};
    ///
    /// let x = Source::Attempt.with(Scope::Cumulative);
    /// assert_eq!(Source::Attempt, x.source);
    /// assert_eq!(Scope::Cumulative, x.scope);
    /// assert_eq!(Kind::ATTEMPT_CUMULATIVE, x);

    /// ```
    #[must_use]
    pub const fn with(self, scope: Scope) -> Kind {
        Kind {
            source: self,
            scope,
        }
    }
}

/// Enumeration of scopes for aggregate times.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Scope {
    /// This time is the total of all times logged on the split.
    Split,
    /// This time is the cumulative run time at this split; ie, the total plus
    /// all totals of all preceding runs.
    Cumulative,
}

/// A persistent set of aggregate times, organised by [Source].
///
/// Use these and [Pair]s to cache observations.
#[derive(Debug, Default, Clone, Copy)]
pub struct Set {
    /// Times for the current attempt.
    pub attempt: Pair,
    /// Times for the comparison.
    pub comparison: Pair,
}

impl Index<Source> for Set {
    type Output = Pair;

    fn index(&self, index: Source) -> &Self::Output {
        match index {
            Source::Attempt => &self.attempt,
            Source::Comparison => &self.comparison,
        }
    }
}

impl IndexMut<Source> for Set {
    fn index_mut(&mut self, index: Source) -> &mut Self::Output {
        match index {
            Source::Attempt => &mut self.attempt,
            Source::Comparison => &mut self.comparison,
        }
    }
}

/// A pair of persisted aggregate times, organised by [Scope].
#[derive(Debug, Default, Clone, Copy)]
pub struct Pair {
    /// Single time for this split only.
    pub split: Option<Time>,
    /// Cumulative time for all splits up to and including this split.
    pub cumulative: Option<Time>,
}

impl Index<Scope> for Pair {
    type Output = Option<Time>;

    fn index(&self, index: Scope) -> &Self::Output {
        match index {
            Scope::Cumulative => &self.cumulative,
            Scope::Split => &self.split,
        }
    }
}

impl IndexMut<Scope> for Pair {
    fn index_mut(&mut self, index: Scope) -> &mut Self::Output {
        match index {
            Scope::Cumulative => &mut self.cumulative,
            Scope::Split => &mut self.split,
        }
    }
}
