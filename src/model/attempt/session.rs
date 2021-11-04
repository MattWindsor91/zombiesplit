//! The [Session] type and related code.

use super::{
    super::{
        aggregate,
        comparison::{self, pace, Comparison},
        game::category,
        history, short, Time,
    },
    observer::{self, split::Observer as SO, time::Observer as TO},
    split, Observer, Run,
};
use crate::model::attempt::action::Action;

/// Holds all data for an attempt session.
///
/// A session consists of category information, the current run being
/// attempted, and any comparison data being worked against.
///
/// It also has zero or more observers attached that can be sent information
/// about the run's progress, and a comparison provider.  The latter feeds into
/// the session's lifetime.
pub struct Session<'cmp> {
    /// Metadata for the game/category currently being run.
    pub metadata: category::Info,
    /// The current run.
    run: Run,
    /// Comparison data for the game/category currently being run.
    comparison: Comparison,
    /// Any observers attached to the session.
    pub observers: observer::Mux,
    /// The function for timestamping outgoing runs.
    timestamper: fn() -> chrono::DateTime<chrono::Utc>,
    /// The comparison provider.
    comparator: Box<dyn comparison::Provider + 'cmp>,
}

impl<'a> Session<'a> {
    /// Starts a new session with a given set of metadata and starting run.
    #[must_use]
    pub fn new(metadata: category::Info, run: Run) -> Self {
        Self {
            metadata,
            run,
            comparison: Comparison::default(),
            observers: observer::Mux::default(),
            timestamper: chrono::Utc::now,
            comparator: Box::new(comparison::NullProvider),
        }
    }

    /// Replaces the session's timestamper with a different function.
    ///
    /// Useful for stubbing out time when testing.
    pub fn set_timestamper(&mut self, ts: fn() -> chrono::DateTime<chrono::Utc>) {
        self.timestamper = ts;
    }

    /// Replaces the session's comparison provider with a different one.
    ///
    /// By default, the session doesn't have comparisons set up, so this will
    /// need to be done to get comparisons working.
    ///
    /// Triggers an immediate comparison reset.
    pub fn set_comparison_provider(&mut self, p: Box<dyn comparison::Provider + 'a>) {
        self.comparator = p;
        self.refresh_comparison();
    }

    /// Converts this session's current run, if any, to a historic run.
    ///
    /// Returns `None` if there is no started run.
    #[must_use]
    pub fn run_as_historic(&self) -> Option<history::run::FullyTimed<category::ShortDescriptor>> {
        self.run
            .status()
            .to_completeness()
            .map(|c| self.run_as_historic_with_completion(c))
    }

    fn run_as_historic_with_completion(
        &self,
        was_completed: bool,
    ) -> history::run::FullyTimed<category::ShortDescriptor> {
        history::run::FullyTimed {
            category_locator: self.metadata.short.clone(),
            was_completed,
            date: (self.timestamper)(),
            timing: self.run.timing_as_historic(),
        }
    }

    /// Asks the comparison provider for an updated comparison.
    ///
    /// This should occur when the run is reset, in case the outgoing run has
    /// changed the comparisons.
    fn refresh_comparison(&mut self) {
        if let Some(c) = self.comparator.comparison() {
            self.comparison = c;
        }
        self.observe_comparison();
    }

    /// Dumps initial session information to the observers.
    ///
    /// This should only be called once, as it might not be idempotent.
    pub fn dump_to_observers(&self) {
        self.observe_game_category();
        self.observe_splits();
        self.observe_attempt();
        self.observe_comparison();
    }

    /// Sends information about each split to the observers.
    fn observe_splits(&self) {
        // TODO(@MattWindsor91): see if I can lifetime this better.
        for split in self.run.splits.iter() {
            self.observers.observe(observer::Event::AddSplit(
                split.info.short,
                split.info.name.clone(),
            ));
        }
    }

    fn observe_reset(&self) {
        self.observers
            .observe(observer::Event::Reset(self.run_as_historic()));
    }

    fn observe_attempt(&self) {
        self.observers
            .observe(observer::Event::Attempt(self.run.attempt));
    }

    fn observe_game_category(&self) {
        self.observers
            .observe(observer::Event::GameCategory(self.metadata.clone()));
    }

    /// Observes all paces and aggregates for each split, notifying all
    /// observers.
    fn observe_paces_and_aggregates(&self) {
        // TODO(@MattWindsor91): start from a particular split, to avoid
        // redundancy?

        let mut total = pace::PacedTime::default();

        for (split, agg) in self.run.splits.aggregates() {
            let pace = self.split_pace(split, agg);
            let short = split.info.short;
            self.observers
                .observe_split(short, observer::split::Event::Pace(pace));
            self.observers
                .observe_aggregate_set(short, agg, aggregate::Source::Attempt);

            if total.time < agg.cumulative {
                total = pace::PacedTime {
                    pace: pace.overall(),
                    time: agg.cumulative,
                }
            }
        }

        self.observers
            .observe(observer::Event::Total(total, aggregate::Source::Attempt));
    }

    fn split_pace(&self, split: &super::Split, agg: aggregate::Set) -> pace::SplitInRun {
        if split.num_times() == 0 {
            pace::SplitInRun::Inconclusive
        } else {
            self.comparison.pace(split.info.short, agg)
        }
    }

    /// Observes the contents of a comparison.
    ///
    /// This lets the user interface know, for each splits, which times we are
    /// running against.
    fn observe_comparison(&self) {
        // TODO(@MattWindsor91): wrapping this in a PacedTime is a bit silly.
        self.observers.observe(observer::Event::Total(
            pace::PacedTime::inconclusive(self.comparison.total()),
            aggregate::Source::Comparison,
        ));
        self.observe_comparison_splits();
    }

    /// Observes comparison data for each split in the run.
    fn observe_comparison_splits(&self) {
        for split in self.run.splits.iter() {
            let short = split.info.short;
            if let Some(s) = self.comparison.aggregate_for(short) {
                self.observers
                    .observe_aggregate_set(short, *s, aggregate::Source::Comparison);
            }
        }
    }

    /// Performs the action `action` on this session's current run.
    pub fn perform(&mut self, action: Action) {
        match action {
            Action::Clear(s) => self.clear_at(s),
            Action::NewRun => self.reset(),
            Action::Pop(s) => self.pop_from(s),
            Action::Push(s, t) => self.push_to(s, t),
        }
    }

    fn reset(&mut self) {
        self.observe_reset();
        self.run.reset();
        self.observe_attempt();
        self.refresh_comparison();
    }

    /// Gets the number of splits in the run.
    #[must_use]
    pub fn num_splits(&self) -> usize {
        // TODO(@MattWindsor91): this delegation is tedious and suggests there
        // are issues in my abstraction here.
        self.run.num_splits()
    }

    /// Gets the position of the split with short name `short`.
    #[must_use]
    pub fn position_of(&self, short: impl Into<short::Name>) -> Option<usize> {
        self.run.position_of(short)
    }

    fn clear_at(&mut self, split: impl split::Locator) {
        if let Some(s) = self.run.splits.get_mut(split) {
            s.clear();
            // TODO(@MattWindsor91): observe
        }
    }

    fn push_to(&mut self, split: impl split::Locator, time: Time) {
        if let Some(s) = self.run.splits.get_mut(split) {
            s.push(time);
            let short = s.info.short;
            self.observers
                .observe_time(short, time, observer::time::Event::Pushed);
            self.observe_paces_and_aggregates();
        }
    }

    fn pop_from(&mut self, split: impl split::Locator) {
        if let Some((short, time)) = self.run.splits.get_mut(split).and_then(|s| {
            let short = s.info.short;
            s.pop().map(|time| (short, time))
        }) {
            self.observers
                .observe_time(short, time, observer::time::Event::Popped);
            self.observe_paces_and_aggregates();
        }
    }
}
