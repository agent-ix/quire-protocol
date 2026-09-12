// SPDX-License-Identifier: AGPL-3.0-or-later
//! Count, allocation and deadline guards. Owner chunk: **C-29**.
//!
//! # Two mechanisms, never one code
//!
//! NFR-002 names two obligations that look alike and are not:
//!
//! 1. **Prechecked counts and accounted allocations.** A ceiling is compared
//!    *before* the next unit is decoded, inserted or allocated. The maximum is
//!    inclusive: the exact boundary is admitted and the smallest representable
//!    excess unit is refused before any memory is reserved. Cause: the caller
//!    asked for more than the budget allows. Remediation: ask for less, or raise
//!    the limit. Truth condition: an arithmetic comparison that does not depend
//!    on when it is evaluated.
//! 2. **Monotonic deadline observation.** Elapsed time is read before and after
//!    each declared indivisible unit. No later unit starts after expiry, and the
//!    observed elapsed time and the indivisible-unit overshoot are reported as
//!    measured. Cause: the work already under way took longer than the budget.
//!    Remediation: retry with a longer budget, or a faster host. Truth
//!    condition: a reading of a clock, which is never reproducible.
//!
//! A caller that cannot tell "you asked for too much" from "it took too long"
//! can act correctly on neither. So the two are not variants of one enum here.
//! They are separate types — [`PrecheckRefusal`] and [`DeadlineRefusal`] — with
//! separate closed code enums — [`PrecheckCode`] and [`DeadlineCode`] — whose
//! stable strings live in disjoint namespaces (`bounds.precheck.*` against
//! `bounds.deadline.*`). No function in this module can return both, and
//! [`RunStop`] — the single place they compose — keeps each on its own arm.
//!
//! # The clock is monotonic by type, not by promise
//!
//! [`Clock`] exposes exactly one reading, [`Clock::monotonic`], of type
//! [`std::time::Instant`]. `std::time::SystemTime` — the wall clock, which
//! jumps when an operator or NTP adjusts it — is not accepted anywhere in this
//! module's signatures, so a wall-clock jump in either direction can neither
//! manufacture nor mask a deadline refusal. A reading that still moves backwards
//! (a `Clock` implementation that lies) is refused as
//! [`DeadlineRefusal::ClockRegressed`], which is *not* the expiry code: a broken
//! clock and an exhausted budget are different facts.
//!
//! # Integer conversions
//!
//! Count accounting stays in A's own `usize` domain end to end, so there is no
//! narrowing conversion in it to get wrong; the one addition that can overflow
//! is `checked_add`, reported as [`PrecheckCode::CounterOverflow`]. The one
//! conversion that does cross a persistence boundary — an observed
//! [`std::time::Duration`] rendered as whole nanoseconds for a wire or result
//! record — is checked by [`DeadlineRefusal::observed_nanos`] and refuses with
//! [`DurationOverflow`] rather than truncating. A truncated duration is a limit
//! that silently stops limiting.
//!
//! # Public enums are closed, deliberately
//!
//! Every enum this module exports is closed — none carries
//! `#[non_exhaustive]`. The reasoning follows C-07 (PR #37) rather than copying
//! it: a `#[non_exhaustive]` enum forces every downstream `match` to carry a
//! wildcard arm, and a wildcard arm silently absorbs a variant added later.
//! That is precisely the failure this chunk exists to remove — a refusal that is
//! not independently assertable because some `_ =>` already caught it. Keeping
//! these enums closed means a new bound, a new site kind or a new clock fault
//! breaks every consumer's `match` at compile time, which is the intended cost
//! of changing what the protocol can refuse. The crate's own matches over these
//! types are exhaustive and carry no catch-all arm.
//!
//! # Reconciliation with A
//!
//! A owns the accounting for work A performs: `Limits`, `Usage`, `Dimension`,
//! `Exhaustion` and `ACCOUNTING_VERSION`. This module does not re-implement it
//! and does not invent a parallel limits type — [`CountGuard`] is built from A's
//! `Limits` (already `bounded()`), charges A's `Usage` fields, names A's
//! `Dimension`, and mirrors A's per-dimension [`Accumulation`] discipline so
//! that B's pre-work refusal and A's in-work `Exhaustion` cannot disagree about
//! which dimension is a peak and which is cumulative. What is B's own is the
//! *site* layer: the guard refuses work at a site that is not in the inventory,
//! which is the hook C-30 reconciles against.
//!
//! ## The accumulation table is an unverifiable mirror, and that is a gap
//!
//! [`accumulation`] states, for each of A's thirteen dimensions, whether A peaks
//! or sums it. A does not publish that fact. `Dimension` is a bare public enum
//! with no accessor, and the discipline lives in a private `Work::fields`, so
//! **no test in this repository can fail when A changes it.** The table was
//! checked by reading A's source at the pinned revision; it agrees there today,
//! and nothing here will notice if it stops agreeing.
//!
//! That is a real exposure, not a stylistic one: if a dimension's discipline
//! flips in A, B's precheck admits a unit A's in-work accounting then refuses
//! (or the reverse), and the two guards disagree about one budget with no
//! failing test on either side.
//!
//! The fix is A's to make, and is not to be worked around here with a local
//! re-derivation. **B asks A to publish `Dimension::accumulation()`** — or any
//! accessor exposing the peak/cumulative discipline — so this table can be
//! replaced by A's own answer. Until then `tc_009_every_a_dimension_has_a_reconciled_accumulation`
//! states the mapping per dimension rather than as a pair of totals, so B's half
//! of the reconciliation is at least explicit and has to be changed
//! deliberately.

/// Resource-site inventory and mutant-kill assignment. Owner chunk: **C-30**.
///
/// Will own the bidirectional reconciliation between caller-controlled
/// allocation, retention, queue, iteration and traversal sites and their
/// inventory entries.
pub mod inventory;

use std::fmt;
use std::time::{Duration, Instant};

use quire_spec_language::protocol_artifact::{Dimension, Limits, Usage};

// ---------------------------------------------------------------------------
// Dimension reconciliation with A
// ---------------------------------------------------------------------------

/// How a dimension accumulates, mirroring A's `Work::charge`.
///
/// A peak dimension retains the largest single candidate; a cumulative
/// dimension sums every charge. Getting this wrong in B would let a pre-work
/// precheck admit a unit that A's in-work accounting then refuses, or the
/// reverse — two guards disagreeing about the same budget.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum Accumulation {
    /// The dimension retains the largest candidate seen.
    Peak,
    /// The dimension sums every charge.
    Cumulative,
}

/// A's accumulation discipline for `dimension`.
#[must_use]
pub const fn accumulation(dimension: Dimension) -> Accumulation {
    match dimension {
        Dimension::PayloadBytes
        | Dimension::OutputBytes
        | Dimension::SourceBytes
        | Dimension::Sources
        | Dimension::Dependencies
        | Dimension::Definitions
        | Dimension::Models
        | Dimension::Declarations
        | Dimension::Depth => Accumulation::Peak,
        Dimension::ContentBytes
        | Dimension::Entries
        | Dimension::References
        | Dimension::ByteWork => Accumulation::Cumulative,
    }
}

/// Every dimension A declares, in A's declaration order.
///
/// Written out rather than derived so that a dimension added to A breaks the
/// exhaustive match in [`accumulation`] and the length assertion in the TC-009
/// suite, instead of quietly going unguarded.
pub const ALL_DIMENSIONS: [Dimension; 13] = [
    Dimension::PayloadBytes,
    Dimension::OutputBytes,
    Dimension::SourceBytes,
    Dimension::ContentBytes,
    Dimension::Sources,
    Dimension::Dependencies,
    Dimension::Definitions,
    Dimension::Models,
    Dimension::Declarations,
    Dimension::Entries,
    Dimension::References,
    Dimension::ByteWork,
    Dimension::Depth,
];

/// The caller's effective ceiling for `dimension`.
#[must_use]
pub const fn limit_for(limits: &Limits, dimension: Dimension) -> usize {
    match dimension {
        Dimension::PayloadBytes => limits.payload_bytes,
        Dimension::OutputBytes => limits.output_bytes,
        Dimension::SourceBytes => limits.source_bytes,
        Dimension::ContentBytes => limits.content_bytes,
        Dimension::Sources => limits.sources,
        Dimension::Dependencies => limits.dependencies,
        Dimension::Definitions => limits.definitions,
        Dimension::Models => limits.models,
        Dimension::Declarations => limits.declarations,
        Dimension::Entries => limits.entries,
        Dimension::References => limits.references,
        Dimension::ByteWork => limits.byte_work,
        Dimension::Depth => limits.depth,
    }
}

/// Usage charged so far against `dimension`.
#[must_use]
pub const fn usage_for(usage: &Usage, dimension: Dimension) -> usize {
    match dimension {
        Dimension::PayloadBytes => usage.payload_bytes,
        Dimension::OutputBytes => usage.output_bytes,
        Dimension::SourceBytes => usage.source_bytes,
        Dimension::ContentBytes => usage.content_bytes,
        Dimension::Sources => usage.sources,
        Dimension::Dependencies => usage.dependencies,
        Dimension::Definitions => usage.definitions,
        Dimension::Models => usage.models,
        Dimension::Declarations => usage.declarations,
        Dimension::Entries => usage.entries,
        Dimension::References => usage.references,
        Dimension::ByteWork => usage.byte_work,
        Dimension::Depth => usage.depth,
    }
}

fn usage_slot(usage: &mut Usage, dimension: Dimension) -> &mut usize {
    match dimension {
        Dimension::PayloadBytes => &mut usage.payload_bytes,
        Dimension::OutputBytes => &mut usage.output_bytes,
        Dimension::SourceBytes => &mut usage.source_bytes,
        Dimension::ContentBytes => &mut usage.content_bytes,
        Dimension::Sources => &mut usage.sources,
        Dimension::Dependencies => &mut usage.dependencies,
        Dimension::Definitions => &mut usage.definitions,
        Dimension::Models => &mut usage.models,
        Dimension::Declarations => &mut usage.declarations,
        Dimension::Entries => &mut usage.entries,
        Dimension::References => &mut usage.references,
        Dimension::ByteWork => &mut usage.byte_work,
        Dimension::Depth => &mut usage.depth,
    }
}

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

/// Every site kind, for reconciliation coverage.
pub const ALL_SITE_KINDS: [SiteKind; 5] = [
    SiteKind::Allocation,
    SiteKind::Retention,
    SiteKind::Queue,
    SiteKind::Iteration,
    SiteKind::Traversal,
];

/// One registered resource site: a name, the dimension that bounds it, and what
/// it does.
///
/// A site is *declared* here rather than checked ad hoc at each call, so that
/// C-30 can reconcile code sites against inventory entries in both directions
/// without every call site inventing its own comparison.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BoundedSite {
    id: SiteId,
    dimension: Dimension,
    kind: SiteKind,
}

impl BoundedSite {
    /// Declare a site bounded by `dimension`.
    #[must_use]
    pub const fn new(id: SiteId, dimension: Dimension, kind: SiteKind) -> Self {
        Self {
            id,
            dimension,
            kind,
        }
    }

    /// The site's name.
    #[must_use]
    pub const fn id(&self) -> SiteId {
        self.id
    }

    /// The dimension whose ceiling bounds this site.
    #[must_use]
    pub const fn dimension(&self) -> Dimension {
        self.dimension
    }

    /// What this site does with caller-controlled data.
    #[must_use]
    pub const fn kind(&self) -> SiteKind {
        self.kind
    }
}

/// Registration faults. Kept out of both refusal enums: failing to *declare* a
/// site is a programming fault in this crate, not a refusal of a caller's
/// request, and collapsing the two would put a B-internal defect on the same
/// code as a caller's over-large input.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RegistryError {
    /// A second entry claimed a name already registered.
    DuplicateSite {
        /// The name registered twice.
        site: SiteId,
    },
}

impl fmt::Display for RegistryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DuplicateSite { site } => {
                write!(f, "resource site {site} is registered twice")
            }
        }
    }
}

impl std::error::Error for RegistryError {}

/// The declared set of caller-controlled resource sites.
///
/// C-30 consumes this: `sites()` is the inventory read out of the code, so a
/// deleted, duplicated or unlisted entry is decidable rather than a matter of
/// grepping.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct SiteRegistry {
    sites: Vec<BoundedSite>,
}

impl SiteRegistry {
    /// An empty registry.
    #[must_use]
    pub const fn new() -> Self {
        Self { sites: Vec::new() }
    }

    /// Declare `site`, refusing a duplicate name.
    ///
    /// # Errors
    ///
    /// [`RegistryError::DuplicateSite`] when the name is already registered.
    pub fn register(&mut self, site: BoundedSite) -> Result<(), RegistryError> {
        if self.sites.iter().any(|known| known.id == site.id) {
            return Err(RegistryError::DuplicateSite { site: site.id });
        }
        self.sites.push(site);
        Ok(())
    }

    /// The registered site named `id`, if any.
    #[must_use]
    pub fn get(&self, id: SiteId) -> Option<&BoundedSite> {
        self.sites.iter().find(|site| site.id == id)
    }

    /// Every registered site, in registration order.
    #[must_use]
    pub fn sites(&self) -> &[BoundedSite] {
        &self.sites
    }
}

// ---------------------------------------------------------------------------
// Mechanism 1 — prechecked counts and accounted allocations
// ---------------------------------------------------------------------------

/// The stable code of a prechecked count or allocation refusal.
///
/// Every string is in the `bounds.precheck.` namespace; no deadline code is.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum PrecheckCode {
    /// A counting, retention, queue, iteration or traversal site exceeded its
    /// inclusive ceiling.
    CountExceeded,
    /// An allocation site would have reserved memory beyond its inclusive
    /// ceiling.
    AllocationExceeded,
    /// Charging the request would overflow the accumulator itself.
    CounterOverflow,
    /// Work was attempted at a site that is not in the inventory.
    UnregisteredSite,
}

impl PrecheckCode {
    /// The shared namespace prefix of every precheck code.
    pub const NAMESPACE: &'static str = "bounds.precheck.";

    /// Every precheck code.
    pub const ALL: [Self; 4] = [
        Self::CountExceeded,
        Self::AllocationExceeded,
        Self::CounterOverflow,
        Self::UnregisteredSite,
    ];

    /// The code's stable string.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::CountExceeded => "bounds.precheck.count-exceeded",
            Self::AllocationExceeded => "bounds.precheck.allocation-exceeded",
            Self::CounterOverflow => "bounds.precheck.counter-overflow",
            Self::UnregisteredSite => "bounds.precheck.unregistered-site",
        }
    }
}

impl fmt::Display for PrecheckCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A bound refused *before* the work was started.
///
/// Nothing was decoded, inserted or allocated for the refused unit, and usage
/// is unchanged: the fields report what would have happened, not what did.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PrecheckRefusal {
    /// A counting site would have passed its inclusive ceiling.
    CountExceeded {
        /// The site that was refused.
        site: SiteId,
        /// The dimension that bounds it.
        dimension: Dimension,
        /// Usage successfully charged before this request.
        used: usize,
        /// The refused increment.
        requested: usize,
        /// The effective ceiling.
        limit: usize,
    },
    /// An allocation site would have reserved beyond its inclusive ceiling.
    AllocationExceeded {
        /// The site that was refused.
        site: SiteId,
        /// The dimension that bounds it.
        dimension: Dimension,
        /// Usage successfully charged before this request.
        used: usize,
        /// The refused reservation.
        requested: usize,
        /// The effective ceiling.
        limit: usize,
    },
    /// `used + requested` does not fit the accumulator.
    CounterOverflow {
        /// The site that was refused.
        site: SiteId,
        /// The dimension that bounds it.
        dimension: Dimension,
        /// Usage successfully charged before this request.
        used: usize,
        /// The refused increment.
        requested: usize,
    },
    /// The site is not in the inventory, so nothing bounds it.
    UnregisteredSite {
        /// The undeclared name.
        site: SiteId,
    },
}

impl PrecheckRefusal {
    /// This refusal's stable code.
    ///
    /// The match is exhaustive and carries no catch-all arm, so a new bound
    /// cannot be introduced without giving it a code of its own.
    #[must_use]
    pub const fn code(&self) -> PrecheckCode {
        match self {
            Self::CountExceeded { .. } => PrecheckCode::CountExceeded,
            Self::AllocationExceeded { .. } => PrecheckCode::AllocationExceeded,
            Self::CounterOverflow { .. } => PrecheckCode::CounterOverflow,
            Self::UnregisteredSite { .. } => PrecheckCode::UnregisteredSite,
        }
    }

    /// The refused site.
    #[must_use]
    pub const fn site(&self) -> SiteId {
        match self {
            Self::CountExceeded { site, .. }
            | Self::AllocationExceeded { site, .. }
            | Self::CounterOverflow { site, .. }
            | Self::UnregisteredSite { site } => *site,
        }
    }
}

impl fmt::Display for PrecheckRefusal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CountExceeded {
                site,
                dimension,
                used,
                requested,
                limit,
            }
            | Self::AllocationExceeded {
                site,
                dimension,
                used,
                requested,
                limit,
            } => write!(
                f,
                "{}: site {site} dimension {dimension:?} limit {limit}: requested {requested} after {used}",
                self.code()
            ),
            Self::CounterOverflow {
                site,
                dimension,
                used,
                requested,
            } => write!(
                f,
                "{}: site {site} dimension {dimension:?}: {used} + {requested} overflows",
                self.code()
            ),
            Self::UnregisteredSite { site } => {
                write!(f, "{}: site {site} is not in the inventory", self.code())
            }
        }
    }
}

impl std::error::Error for PrecheckRefusal {}

/// Permission to reserve `granted` units at a site, issued only by a successful
/// precheck.
///
/// Memory is reserved through [`Reservation::with_capacity`], which exists only
/// on this token, so "precheck then allocate" is the only representable order:
/// there is no way to allocate first and check afterwards.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[must_use = "a reservation that is never used charged the budget for nothing"]
pub struct Reservation {
    site: SiteId,
    dimension: Dimension,
    granted: usize,
    next: usize,
}

impl Reservation {
    /// The site this reservation was granted for.
    #[must_use]
    pub const fn site(&self) -> SiteId {
        self.site
    }

    /// The dimension that bounds the granted site.
    #[must_use]
    pub const fn dimension(&self) -> Dimension {
        self.dimension
    }

    /// The units granted.
    #[must_use]
    pub const fn granted(&self) -> usize {
        self.granted
    }

    /// Reserve exactly the granted capacity.
    #[must_use]
    pub fn with_capacity<T>(&self) -> Vec<T> {
        Vec::with_capacity(self.granted)
    }
}

/// Prechecked count and allocation accounting over A's `Limits` and `Usage`.
///
/// This is mechanism 1. It reads no clock, holds no deadline and cannot return
/// a [`DeadlineRefusal`].
#[derive(Clone, Debug)]
pub struct CountGuard {
    limits: Limits,
    usage: Usage,
    registry: SiteRegistry,
}

impl CountGuard {
    /// A guard over the caller's effective ceilings and the declared sites.
    ///
    /// `limits` is passed through A's own `bounded()` so B's precheck and A's
    /// in-work accounting share one effective ceiling.
    #[must_use]
    pub fn new(limits: Limits, registry: SiteRegistry) -> Self {
        Self {
            limits: limits.bounded(),
            usage: Usage::default(),
            registry,
        }
    }

    /// The effective ceilings.
    #[must_use]
    pub const fn limits(&self) -> &Limits {
        &self.limits
    }

    /// Usage charged so far. A refused request leaves this unchanged.
    #[must_use]
    pub const fn usage(&self) -> &Usage {
        &self.usage
    }

    /// The declared sites.
    #[must_use]
    pub const fn registry(&self) -> &SiteRegistry {
        &self.registry
    }

    /// Decide `requested` units at `site` without charging anything.
    ///
    /// The maximum is inclusive: a request landing exactly on the ceiling is
    /// admitted, and the smallest representable excess — one more unit — is
    /// refused.
    ///
    /// # Errors
    ///
    /// [`PrecheckRefusal::UnregisteredSite`] when the site is not declared,
    /// [`PrecheckRefusal::CounterOverflow`] when the charge would not fit the
    /// accumulator, and [`PrecheckRefusal::AllocationExceeded`] or
    /// [`PrecheckRefusal::CountExceeded`] when the ceiling would be passed —
    /// the former for [`SiteKind::Allocation`], the latter for every other kind.
    pub fn precheck(&self, site: SiteId, requested: usize) -> Result<Reservation, PrecheckRefusal> {
        let Some(declared) = self.registry.get(site) else {
            return Err(PrecheckRefusal::UnregisteredSite { site });
        };
        let dimension = declared.dimension();
        let used = usage_for(&self.usage, dimension);
        let limit = limit_for(&self.limits, dimension);

        let next = match accumulation(dimension) {
            Accumulation::Peak => used.max(requested),
            Accumulation::Cumulative => {
                used.checked_add(requested)
                    .ok_or(PrecheckRefusal::CounterOverflow {
                        site,
                        dimension,
                        used,
                        requested,
                    })?
            }
        };

        if next > limit {
            return Err(match declared.kind() {
                SiteKind::Allocation => PrecheckRefusal::AllocationExceeded {
                    site,
                    dimension,
                    used,
                    requested,
                    limit,
                },
                SiteKind::Retention
                | SiteKind::Queue
                | SiteKind::Iteration
                | SiteKind::Traversal => PrecheckRefusal::CountExceeded {
                    site,
                    dimension,
                    used,
                    requested,
                    limit,
                },
            });
        }

        Ok(Reservation {
            site,
            dimension,
            granted: requested,
            next,
        })
    }

    /// Precheck and, only on success, charge the request.
    ///
    /// # Errors
    ///
    /// As [`CountGuard::precheck`]. On refusal nothing is charged: the
    /// post-charge value is computed by the precheck itself, so there is no
    /// second, unchecked arithmetic path here that could disagree with it.
    pub fn charge(
        &mut self,
        site: SiteId,
        requested: usize,
    ) -> Result<Reservation, PrecheckRefusal> {
        let reservation = self.precheck(site, requested)?;
        *usage_slot(&mut self.usage, reservation.dimension) = reservation.next;
        Ok(reservation)
    }
}

// ---------------------------------------------------------------------------
// Mechanism 2 — monotonic deadline observation
// ---------------------------------------------------------------------------

/// The stable code of a deadline observation refusal.
///
/// Every string is in the `bounds.deadline.` namespace; no precheck code is.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum DeadlineCode {
    /// The elapsed budget ran out.
    Expired,
    /// The clock read backwards. Not an expiry — a fault.
    ClockRegressed,
}

impl DeadlineCode {
    /// The shared namespace prefix of every deadline code.
    pub const NAMESPACE: &'static str = "bounds.deadline.";

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

/// Which of the two mandatory observations around an indivisible unit refused.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum Phase {
    /// Observed before the unit started; no later unit was started.
    BeforeUnit,
    /// Observed after the unit finished; the unit itself was indivisible and
    /// ran to completion, so its overshoot is reported as measured.
    AfterUnit,
}

/// Every phase, for coverage reconciliation.
pub const ALL_PHASES: [Phase; 2] = [Phase::BeforeUnit, Phase::AfterUnit];

/// Rendering an observed duration as whole nanoseconds did not fit `u64`.
///
/// Reported rather than truncated: a truncated elapsed time is a deadline that
/// silently stops being a deadline.
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

impl DeadlineRefusal {
    /// This refusal's stable code.
    ///
    /// Exhaustive, no catch-all arm.
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

    /// The observed elapsed time rendered for a wire or persistence record.
    ///
    /// # Errors
    ///
    /// [`DurationOverflow`] when the observation does not fit `u64`
    /// nanoseconds. The conversion is checked; it never truncates.
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
            Self::Expired {
                unit,
                phase,
                budget,
                observed,
                overshoot,
            } => write!(
                f,
                "{}: unit {unit} {phase:?} budget {budget:?}: observed {observed:?}, overshoot {overshoot:?}",
                self.code()
            ),
            Self::ClockRegressed {
                unit,
                phase,
                previous,
            } => write!(
                f,
                "{}: unit {unit} {phase:?}: clock read earlier than {previous:?}",
                self.code()
            ),
        }
    }
}

impl std::error::Error for DeadlineRefusal {}

/// The one clock reading this module accepts.
///
/// There is deliberately no `wall_clock` method and no signature anywhere in
/// this module that takes a `std::time::SystemTime`: an operator or NTP step of
/// the wall clock must be unable to manufacture or mask a deadline refusal, and
/// the cheapest way to guarantee that is to make wall-clock time unrepresentable
/// in the guard rather than to promise it is ignored.
///
/// ```compile_fail
/// use std::time::{Duration, SystemTime};
/// use quire_protocol::bounds::DeadlineGuard;
/// struct WallClock;
/// impl quire_protocol::bounds::Clock for WallClock {
///     // SystemTime is not an Instant; this trait cannot carry wall-clock time.
///     fn monotonic(&self) -> SystemTime { SystemTime::now() }
/// }
/// let _ = DeadlineGuard::start(&WallClock, Duration::from_secs(1));
/// ```
pub trait Clock {
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
/// This is mechanism 2. It holds no ceiling, charges no dimension and cannot
/// return a [`PrecheckRefusal`].
///
/// The budget is an **inclusive** allowance, matching the inclusive count
/// maxima: elapsed time exactly equal to the budget has not expired, and expiry
/// begins strictly beyond it. One boundary convention for the whole module, and
/// it makes an `Expired` with a zero overshoot unrepresentable.
#[derive(Clone, Copy, Debug)]
pub struct DeadlineGuard {
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

    /// The caller's budget.
    #[must_use]
    pub const fn budget(&self) -> Duration {
        self.budget
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

// ---------------------------------------------------------------------------
// The one place the two mechanisms compose
// ---------------------------------------------------------------------------

/// Why a bounded run stopped.
///
/// The two mechanisms meet here and nowhere else, and they meet on separate
/// arms carrying separate types. There is no constructor that can put a
/// [`DeadlineCode`] on the [`RunStop::Count`] arm or the reverse.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RunStop {
    /// A bound was exceeded and the unit was refused before it started.
    Count(PrecheckRefusal),
    /// Time ran out around work already under way.
    Deadline(DeadlineRefusal),
}

impl RunStop {
    /// The stable code string of whichever refusal stopped the run.
    ///
    /// Exhaustive, no catch-all arm.
    #[must_use]
    pub const fn code_str(&self) -> &'static str {
        match self {
            Self::Count(refusal) => refusal.code().as_str(),
            Self::Deadline(refusal) => refusal.code().as_str(),
        }
    }
}

impl fmt::Display for RunStop {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Count(refusal) => write!(f, "{refusal}"),
            Self::Deadline(refusal) => write!(f, "{refusal}"),
        }
    }
}

impl std::error::Error for RunStop {}

/// A run bounded by both mechanisms at once.
///
/// The deadline is observed before the count is prechecked, so a run that is
/// out of time neither charges the budget nor manufactures a count refusal from
/// the fact that it stopped.
#[derive(Clone, Debug)]
pub struct BoundedRun {
    counts: CountGuard,
    deadline: DeadlineGuard,
}

impl BoundedRun {
    /// Compose a count guard and a deadline guard.
    #[must_use]
    pub const fn new(counts: CountGuard, deadline: DeadlineGuard) -> Self {
        Self { counts, deadline }
    }

    /// The count accounting.
    #[must_use]
    pub const fn counts(&self) -> &CountGuard {
        &self.counts
    }

    /// The deadline observation.
    #[must_use]
    pub const fn deadline(&self) -> &DeadlineGuard {
        &self.deadline
    }

    /// Admit one indivisible unit: observe the deadline, then precheck and
    /// charge `requested` units at `site`.
    ///
    /// # Errors
    ///
    /// [`RunStop::Deadline`] when time ran out, [`RunStop::Count`] when a bound
    /// was exceeded. On either, nothing was allocated for the refused unit.
    pub fn admit_unit<C: Clock + ?Sized>(
        &mut self,
        clock: &C,
        unit: UnitLabel,
        site: SiteId,
        requested: usize,
    ) -> Result<(Reservation, UnitToken), RunStop> {
        let token = self
            .deadline
            .begin_unit(clock, unit)
            .map_err(RunStop::Deadline)?;
        let reservation = self
            .counts
            .charge(site, requested)
            .map_err(RunStop::Count)?;
        Ok((reservation, token))
    }

    /// Close the unit opened by [`BoundedRun::admit_unit`].
    ///
    /// # Errors
    ///
    /// [`RunStop::Deadline`] with the measured overshoot.
    pub fn finish_unit<C: Clock + ?Sized>(
        &mut self,
        clock: &C,
        token: UnitToken,
    ) -> Result<Duration, RunStop> {
        self.deadline
            .end_unit(clock, token)
            .map_err(RunStop::Deadline)
    }
}
