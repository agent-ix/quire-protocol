// SPDX-License-Identifier: AGPL-3.0-or-later
//! Monotonic deadline observations and resource-site identities.
//!
//! inventory::ResourceAccountant is the sole supported Protocol request
//! accountant. It owns caller-budget charging, inventory reconciliation, and
//! the translation of a deadline observation into a Protocol outcome. This
//! module deliberately contains only the reusable monotonic deadline kernel;
//! it does not offer a second count or allocation guard.

/// Resource-site inventory and request accounting. Owner chunk: C-30.
pub mod inventory;

use std::fmt;
use std::time::{Duration, Instant};

/// The stable name of one caller-controlled resource site.
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

/// What a source site does with caller-controlled data.
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

/// The stable code of a deadline observation refusal.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum DeadlineCode {
    /// The elapsed budget ran out.
    Expired,
    /// The clock read backwards. Not an expiry — a fault.
    ClockRegressed,
}
impl DeadlineCode {
    /// Every deadline code.
    pub const ALL: [Self; 2] = [Self::Expired, Self::ClockRegressed];
    /// The code's stable string.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Expired => "bounds.deadline.expired",
            Self::ClockRegressed => "bounds.deadline.clock-regressed",
        }
    }
}
impl fmt::Display for DeadlineCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

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

/// Which mandatory observation around an indivisible unit refused.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum Phase {
    /// Observed before the unit started; no later unit was started.
    BeforeUnit,
    /// Observed after the unit finished; overshoot is measured.
    AfterUnit,
}
/// Every phase, for coverage reconciliation.
pub const ALL_PHASES: [Phase; 2] = [Phase::BeforeUnit, Phase::AfterUnit];

pub(crate) const REQUEST_DEADLINE_SOURCE: inventory::SourceSite = inventory::SourceSite::new(
    SiteId::new("request/unit/deadline"),
    "bounds::inventory::ResourceAccountant::run_unit",
    inventory::BudgetDimension::ElapsedTime,
    SiteKind::Iteration,
);

/// Rendering an observed duration as whole nanoseconds did not fit u64.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DurationOverflow {
    /// The duration that does not fit.
    pub observed: Duration,
}
impl fmt::Display for DurationOverflow {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "observed duration {:?} does not fit u64 nanoseconds",
            self.observed
        )
    }
}
impl std::error::Error for DurationOverflow {}

/// A monotonic deadline observation refused the run.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DeadlineRefusal {
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
        /// observed minus budget, always strictly positive.
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
impl DeadlineRefusal {
    /// This refusal's stable code.
    #[must_use]
    pub const fn code(&self) -> DeadlineCode {
        match self {
            Self::Expired { .. } => DeadlineCode::Expired,
            Self::ClockRegressed { .. } => DeadlineCode::ClockRegressed,
        }
    }
    /// The unit this refusal surrounds.
    #[must_use]
    pub const fn unit(&self) -> UnitLabel {
        match self {
            Self::Expired { unit, .. } | Self::ClockRegressed { unit, .. } => *unit,
        }
    }
    /// Which mandatory observation refused.
    #[must_use]
    pub const fn phase(&self) -> Phase {
        match self {
            Self::Expired { phase, .. } | Self::ClockRegressed { phase, .. } => *phase,
        }
    }
    /// The observed elapsed time rendered for a wire or result record.
    ///
    /// # Errors
    ///
    /// DurationOverflow when the observation does not fit u64 nanoseconds.
    pub fn observed_nanos(&self) -> Result<u64, DurationOverflow> {
        let observed = match self {
            Self::Expired { observed, .. } => *observed,
            Self::ClockRegressed { previous, .. } => *previous,
        };
        u64::try_from(observed.as_nanos()).map_err(|_| DurationOverflow { observed })
    }
}
impl fmt::Display for DeadlineRefusal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Expired { unit, phase, budget, observed, overshoot } => write!(
                f,
                "{}: unit {unit} {phase:?} budget {budget:?}: observed {observed:?}, overshoot {overshoot:?}",
                self.code()
            ),
            Self::ClockRegressed { unit, phase, previous } => write!(
                f,
                "{}: unit {unit} {phase:?}: clock read earlier than {previous:?}",
                self.code()
            ),
        }
    }
}
impl std::error::Error for DeadlineRefusal {}

/// The one clock reading deadline accounting accepts.
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
#[derive(Clone, Copy, Debug)]
pub struct DeadlineGuard {
    start: Instant,
    budget: Duration,
    observed: Duration,
}
impl DeadlineGuard {
    /// Begin observing, reading clock once for the origin.
    pub fn start<C: Clock + ?Sized>(clock: &C, budget: Duration) -> Self {
        Self {
            start: clock.monotonic(),
            budget,
            observed: Duration::ZERO,
        }
    }
    /// The caller's budget.
    #[must_use]
    pub const fn budget(&self) -> Duration {
        self.budget
    }
    /// The greatest elapsed time observed so far.
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
    /// Observe before starting a unit.
    ///
    /// # Errors
    ///
    /// DeadlineRefusal::Expired when the budget is gone, or ClockRegressed
    /// when the reading moved backwards.
    pub fn begin_unit<C: Clock + ?Sized>(
        &mut self,
        clock: &C,
        unit: UnitLabel,
    ) -> Result<UnitToken, DeadlineRefusal> {
        self.observe(clock, unit, Phase::BeforeUnit)?;
        Ok(UnitToken { unit })
    }
    /// Observe after a unit finished.
    ///
    /// # Errors
    ///
    /// DeadlineRefusal::Expired carries the measured overshoot.
    pub fn end_unit<C: Clock + ?Sized>(
        &mut self,
        clock: &C,
        token: UnitToken,
    ) -> Result<Duration, DeadlineRefusal> {
        self.observe(clock, token.unit, Phase::AfterUnit)
    }
}

/// Proof that an indivisible unit was entered through DeadlineGuard::begin_unit.
#[derive(Debug, Eq, PartialEq)]
#[must_use = "an entered unit must be closed by end_unit so its overshoot is observed"]
pub struct UnitToken {
    unit: UnitLabel,
}
impl UnitToken {
    /// The unit this token was issued for.
    #[must_use]
    pub const fn unit(&self) -> UnitLabel {
        self.unit
    }
}
