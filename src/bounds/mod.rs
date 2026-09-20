// SPDX-License-Identifier: AGPL-3.0-or-later
//! Source-bound caller-budget accounting and monotonic request deadlines.
//!
//! [`crate::bounds::inventory::ResourceAccountant`] is the sole supported request accountant.
//! Static work is charged at reconciled source sites before it begins; elapsed
//! time is observed with a monotonic clock around each declared indivisible unit.
//! Owner-specific accounting remains with the owner crate and is never mirrored
//! into this caller-facing budget vocabulary.

/// Resource-site inventory and mutant-kill assignment. Owner chunk: **C-30**.
///
/// Will own the bidirectional reconciliation between caller-controlled
/// allocation, retention, queue, iteration and traversal sites and their
/// inventory entries.
pub mod inventory;

use std::fmt;
use std::time::{Duration, Instant};

// ---------------------------------------------------------------------------
// Sites — the inventory hook C-30 reconciles against
// ---------------------------------------------------------------------------

/// The stable name of one caller-controlled resource site.
///
/// A site name is a compile-time constant, not caller text: an inventory keyed
/// on runtime strings could not be reconciled against the source at all.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct SiteId(&'static str);

impl SiteId {
    /// Name a site.
    #[must_use]
    pub const fn new(name: &'static str) -> Self {
        Self(name)
    }

    /// The site's stable name.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        self.0
    }
}

impl fmt::Display for SiteId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.0)
    }
}

/// What a site does with caller-controlled data.
///
/// NFR-002's Verification clause enumerates exactly these five; C-30 inventories
/// every source site under one of them.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum SiteKind {
    /// Reserves memory sized by caller-controlled data.
    Allocation,
    /// Retains caller-controlled data beyond the call that supplied it.
    Retention,
    /// Enqueues caller-controlled work for later.
    Queue,
    /// Iterates a caller-controlled sequence.
    Iteration,
    /// Walks a caller-controlled graph.
    Traversal,
}

// ---------------------------------------------------------------------------
// Mechanism 2 — monotonic deadline observation
// ---------------------------------------------------------------------------

/// The name of one declared indivisible unit of work.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct UnitLabel(&'static str);

impl UnitLabel {
    /// Name an indivisible unit.
    #[must_use]
    pub const fn new(name: &'static str) -> Self {
        Self(name)
    }

    /// The unit's stable name.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        self.0
    }
}

impl fmt::Display for UnitLabel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.0)
    }
}

/// Which of the two mandatory observations around an indivisible unit refused.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum Phase {
    /// Observed before the unit started; no later unit was started.
    BeforeUnit,
    /// Observed after the unit finished; the unit itself was indivisible and
    /// ran to completion, so its overshoot is reported as measured.
    AfterUnit,
}

pub(crate) const REQUEST_DEADLINE_SOURCE: inventory::SourceSite = inventory::SourceSite::new(
    SiteId::new("request/unit/deadline"),
    "bounds::inventory::ResourceAccountant::run_unit",
    inventory::BudgetDimension::ElapsedTime,
    SiteKind::Iteration,
);

/// A monotonic deadline observation refused the run.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum DeadlineRefusal {
    /// Observed elapsed time passed the budget.
    Expired {
        /// The unit being entered or left.
        unit: UnitLabel,
        /// Which mandatory observation refused.
        phase: Phase,
        /// The caller's budget.
        budget: Duration,
        /// Elapsed time as measured, never clamped to the budget.
        observed: Duration,
        /// `observed - budget`, always strictly positive.
        overshoot: Duration,
    },
    /// The clock read earlier than a previous reading in the same run.
    ClockRegressed {
        /// The unit being entered or left.
        unit: UnitLabel,
        /// Which mandatory observation refused.
        phase: Phase,
        /// The last elapsed time observed before the regression.
        previous: Duration,
    },
}

/// The one clock reading this module accepts.
///
/// There is deliberately no `wall_clock` method and no signature anywhere in
/// this module that takes a `std::time::SystemTime`: an operator or NTP step of
/// the wall clock must be unable to manufacture or mask a deadline refusal, and
/// the cheapest way to guarantee that is to make wall-clock time unrepresentable
/// in the guard rather than to promise it is ignored.
pub trait Clock: Send {
    /// A reading that never moves backwards.
    fn monotonic(&self) -> Instant;
}

/// The host monotonic clock.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct SystemMonotonicClock;

impl Clock for SystemMonotonicClock {
    fn monotonic(&self) -> Instant {
        Instant::now()
    }
}

/// Monotonic deadline observation around declared indivisible units.
///
/// The budget is an **inclusive** allowance, matching the inclusive count
/// maxima: elapsed time exactly equal to the budget has not expired, and expiry
/// begins strictly beyond it. One boundary convention for the whole module, and
/// it makes an `Expired` with a zero overshoot unrepresentable.
#[derive(Clone, Copy, Debug)]
pub(crate) struct DeadlineGuard {
    start: Instant,
    budget: Duration,
    observed: Duration,
}

impl DeadlineGuard {
    /// Begin observing, reading `clock` once for the origin.
    pub fn start<C: Clock + ?Sized>(clock: &C, budget: Duration) -> Self {
        Self {
            start: clock.monotonic(),
            budget,
            observed: Duration::ZERO,
        }
    }

    /// The greatest elapsed time observed so far, as measured.
    #[must_use]
    pub const fn observed(&self) -> Duration {
        self.observed
    }

    fn observe<C: Clock + ?Sized>(
        &mut self,
        clock: &C,
        unit: UnitLabel,
        phase: Phase,
    ) -> Result<Duration, DeadlineRefusal> {
        let regressed = DeadlineRefusal::ClockRegressed {
            unit,
            phase,
            previous: self.observed,
        };
        let elapsed = clock
            .monotonic()
            .checked_duration_since(self.start)
            .ok_or(regressed)?;
        if elapsed < self.observed {
            return Err(regressed);
        }
        self.observed = elapsed;
        if elapsed > self.budget {
            return Err(DeadlineRefusal::Expired {
                unit,
                phase,
                budget: self.budget,
                observed: elapsed,
                overshoot: elapsed - self.budget,
            });
        }
        Ok(elapsed)
    }

    /// Observe before starting `unit`.
    ///
    /// No later unit starts once the budget has been passed.
    ///
    /// # Errors
    ///
    /// [`DeadlineRefusal::Expired`] when the budget is already gone, or
    /// [`DeadlineRefusal::ClockRegressed`] when the reading moved backwards.
    pub fn begin_unit<C: Clock + ?Sized>(
        &mut self,
        clock: &C,
        unit: UnitLabel,
    ) -> Result<UnitToken, DeadlineRefusal> {
        self.observe(clock, unit, Phase::BeforeUnit)?;
        Ok(UnitToken { unit })
    }

    /// Observe after `token`'s unit finished.
    ///
    /// The unit was indivisible, so it ran to completion; any overshoot is
    /// reported exactly as measured rather than clamped to the budget.
    ///
    /// # Errors
    ///
    /// [`DeadlineRefusal::Expired`] with the measured overshoot, or
    /// [`DeadlineRefusal::ClockRegressed`] when the reading moved backwards.
    pub fn end_unit<C: Clock + ?Sized>(
        &mut self,
        clock: &C,
        token: UnitToken,
    ) -> Result<Duration, DeadlineRefusal> {
        self.observe(clock, token.unit, Phase::AfterUnit)
    }
}

/// Proof that an indivisible unit was entered through [`DeadlineGuard::begin_unit`].
///
/// Not `Copy` and not constructible elsewhere, so the "before" observation
/// cannot be skipped and the "after" observation cannot name a unit that was
/// never entered.
#[derive(Debug, Eq, PartialEq)]
#[must_use = "an entered unit must be closed by end_unit so its overshoot is observed"]
pub(crate) struct UnitToken {
    unit: UnitLabel,
}
