// SPDX-License-Identifier: AGPL-3.0-or-later
//! QProtocol caller budgets and the source-bound resource-site inventory.
//!
//! This module deliberately does not reuse
//! [`quire_spec_language::protocol_artifact::Dimension`]. That type is the
//! producer's accounting vocabulary while reading a compiled artifact. The
//! closed [`crate::bounds::inventory::BudgetDimension`] set below is QProtocol's
//! caller-facing vocabulary from FR-016. Similar labels across the two domains
//! do not make them the same authority or the same exhausted resource.
//!
//! A [`crate::bounds::inventory::SourceSite`] is declared beside production
//! work. An independent [`crate::bounds::inventory::InventoryEntry`] assigns
//! that site an authored-limit authority, pre-work charge point, finite
//! termination argument, resource-incomplete code, and a discriminating
//! TC-009 boundary case. [`crate::bounds::inventory::Inventory::reconcile`] compares both
//! populations in both directions and rejects every duplicate, missing, orphan,
//! cross-wired, or boundary-unassigned entry.

use core::fmt;
use std::time::Duration;

use super::{
    Clock, DeadlineRefusal, Phase, SiteId, SiteKind, SystemMonotonicClock, UnitLabel, UnitToken,
};

/// QProtocol's complete caller-facing resource-budget vocabulary.
///
/// The first seventeen values are declared by NFR-002. The final three are the
/// separate manifest-resolution budget declared by FR-009. This is not the
/// producer reader's thirteen-dimension vocabulary.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum BudgetDimension {
    /// Caller-supplied bytes decoded or retained.
    InputByte,
    /// Protocol definitions admitted.
    Definition,
    /// Protocol nodes admitted.
    Node,
    /// Protocol edges admitted.
    Edge,
    /// Roles admitted.
    Role,
    /// Messages admitted.
    Message,
    /// Branches explored.
    Branch,
    /// Replay iterations executed.
    Iteration,
    /// Concurrent active instances retained.
    ActiveInstance,
    /// Observation facts consumed.
    Observation,
    /// Support-lineage entries validated.
    Lineage,
    /// Items constructed into a result.
    ResultItem,
    /// Derived artifacts constructed.
    DerivedArtifact,
    /// Constructed output bytes.
    OutputByte,
    /// Accounted operations performed.
    Operation,
    /// Accounted allocations made.
    AccountedAllocation,
    /// Monotonic elapsed time for request work.
    ElapsedTime,
    /// Manifest entries resolved before producer values are read.
    ResolutionEntryCount,
    /// Manifest bytes and request-selected loaded values digested by resolution.
    ResolutionBytesDigested,
    /// Monotonic elapsed time through contract and loaded-selection resolution.
    ResolutionDeadline,
}

impl BudgetDimension {
    /// The seventeen request dimensions declared directly by NFR-002.
    pub const NFR_002: [Self; 17] = [
        Self::InputByte,
        Self::Definition,
        Self::Node,
        Self::Edge,
        Self::Role,
        Self::Message,
        Self::Branch,
        Self::Iteration,
        Self::ActiveInstance,
        Self::Observation,
        Self::Lineage,
        Self::ResultItem,
        Self::DerivedArtifact,
        Self::OutputByte,
        Self::Operation,
        Self::AccountedAllocation,
        Self::ElapsedTime,
    ];

    /// Every dimension in specification order.
    pub const ALL: [Self; 20] = [
        Self::InputByte,
        Self::Definition,
        Self::Node,
        Self::Edge,
        Self::Role,
        Self::Message,
        Self::Branch,
        Self::Iteration,
        Self::ActiveInstance,
        Self::Observation,
        Self::Lineage,
        Self::ResultItem,
        Self::DerivedArtifact,
        Self::OutputByte,
        Self::Operation,
        Self::AccountedAllocation,
        Self::ElapsedTime,
        Self::ResolutionEntryCount,
        Self::ResolutionBytesDigested,
        Self::ResolutionDeadline,
    ];

    /// The exact FR-016 label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::InputByte => "input-byte",
            Self::Definition => "definition",
            Self::Node => "node",
            Self::Edge => "edge",
            Self::Role => "role",
            Self::Message => "message",
            Self::Branch => "branch",
            Self::Iteration => "iteration",
            Self::ActiveInstance => "active-instance",
            Self::Observation => "observation",
            Self::Lineage => "lineage",
            Self::ResultItem => "result-item",
            Self::DerivedArtifact => "derived-artifact",
            Self::OutputByte => "output-byte",
            Self::Operation => "operation",
            Self::AccountedAllocation => "accounted-allocation",
            Self::ElapsedTime => "elapsed-time",
            Self::ResolutionEntryCount => "resolution-entry-count",
            Self::ResolutionBytesDigested => "resolution-bytes-digested",
            Self::ResolutionDeadline => "resolution-deadline",
        }
    }

    const fn index(self) -> usize {
        match self {
            Self::InputByte => 0,
            Self::Definition => 1,
            Self::Node => 2,
            Self::Edge => 3,
            Self::Role => 4,
            Self::Message => 5,
            Self::Branch => 6,
            Self::Iteration => 7,
            Self::ActiveInstance => 8,
            Self::Observation => 9,
            Self::Lineage => 10,
            Self::ResultItem => 11,
            Self::DerivedArtifact => 12,
            Self::OutputByte => 13,
            Self::Operation => 14,
            Self::AccountedAllocation => 15,
            Self::ElapsedTime => 16,
            Self::ResolutionEntryCount => 17,
            Self::ResolutionBytesDigested => 18,
            Self::ResolutionDeadline => 19,
        }
    }
}

impl fmt::Display for BudgetDimension {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

/// Invalid construction of a caller budget.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BudgetError {
    /// NFR-002 accepts only positive caller grants.
    NonPositiveGrant {
        /// Dimension offered a zero grant.
        dimension: BudgetDimension,
    },
    /// A selected grant may not be silently replaced.
    DuplicateGrant {
        /// Dimension selected more than once.
        dimension: BudgetDimension,
    },
    /// A nanosecond deadline does not fit [`Duration`]'s `u64` input.
    UnrepresentableDuration {
        /// Deadline dimension whose grant cannot be represented.
        dimension: BudgetDimension,
        /// Offered nanosecond count.
        granted: usize,
    },
}

impl fmt::Display for BudgetError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NonPositiveGrant { dimension } => {
                write!(formatter, "resource grant for {dimension} is not positive")
            }
            Self::DuplicateGrant { dimension } => {
                write!(
                    formatter,
                    "resource grant for {dimension} was selected twice"
                )
            }
            Self::UnrepresentableDuration { dimension, granted } => write!(
                formatter,
                "resource grant {granted} for {dimension} does not fit a nanosecond duration"
            ),
        }
    }
}

impl std::error::Error for BudgetError {}

/// One request's independently selected caller grants.
///
/// There is intentionally no [`Default`] implementation. Absence is authored
/// explicitly with [`Self::nothing_granted`] and remains `None`; it is never
/// converted into zero, unbounded, or another dimension's value.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Budget {
    grants: [Option<usize>; BudgetDimension::ALL.len()],
    elapsed_time: Option<(Duration, usize)>,
    resolution_deadline: Option<(Duration, usize)>,
}

impl Budget {
    /// Construct a request with every caller grant explicitly absent.
    #[must_use]
    pub const fn nothing_granted() -> Self {
        Self {
            grants: [None; BudgetDimension::ALL.len()],
            elapsed_time: None,
            resolution_deadline: None,
        }
    }

    /// Add one positive grant without replacing an earlier selection.
    /// `elapsed-time` and `resolution-deadline` are expressed as exact
    /// nanosecond counts and retained as checked [`Duration`] values for
    /// monotonic observation.
    ///
    /// # Errors
    ///
    /// Returns [`BudgetError::NonPositiveGrant`] for zero and
    /// [`BudgetError::DuplicateGrant`] when the dimension was already selected.
    pub fn grant(
        mut self,
        dimension: BudgetDimension,
        granted: usize,
    ) -> Result<Self, BudgetError> {
        if granted == 0 {
            return Err(BudgetError::NonPositiveGrant { dimension });
        }
        let slot = &mut self.grants[dimension.index()];
        if slot.is_some() {
            return Err(BudgetError::DuplicateGrant { dimension });
        }
        if matches!(
            dimension,
            BudgetDimension::ElapsedTime | BudgetDimension::ResolutionDeadline
        ) {
            let nanos = u64::try_from(granted)
                .map_err(|_| BudgetError::UnrepresentableDuration { dimension, granted })?;
            let value = Some((Duration::from_nanos(nanos), granted));
            match dimension {
                BudgetDimension::ElapsedTime => self.elapsed_time = value,
                BudgetDimension::ResolutionDeadline => self.resolution_deadline = value,
                _ => unreachable!("the temporal-dimension match above is exhaustive"),
            }
        }
        *slot = Some(granted);
        Ok(self)
    }

    /// The exact caller grant, retaining absence.
    #[must_use]
    pub const fn grant_for(&self, dimension: BudgetDimension) -> Option<usize> {
        self.grants[dimension.index()]
    }

    const fn elapsed_time(&self) -> Option<(Duration, usize)> {
        self.elapsed_time
    }

    const fn resolution_deadline(&self) -> Option<(Duration, usize)> {
        self.resolution_deadline
    }
}

/// Work consumed independently on every caller-budget dimension.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Usage {
    consumed: [usize; BudgetDimension::ALL.len()],
}

impl Usage {
    const ZERO: Self = Self {
        consumed: [0; BudgetDimension::ALL.len()],
    };

    /// Work charged on `dimension`.
    #[must_use]
    pub const fn consumed(self, dimension: BudgetDimension) -> usize {
        self.consumed[dimension.index()]
    }
}

/// The owner of the authored ceiling applied in addition to a caller grant.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum LimitAuthority {
    /// QProtocol owns an immutable maximum for the operation.
    ConsumerOwner,
    /// The admitted protocol authors the semantic maximum.
    ProtocolAuthor,
    /// A producer-owned public contract enforces its own vocabulary and limit.
    ProducerOwner,
}

/// The exact point at which the requested increment must be charged.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum ChargePoint {
    /// Before allocating caller-sized memory.
    BeforeAllocate,
    /// Before decoding caller bytes.
    BeforeDecode,
    /// Before inserting a caller-controlled item.
    BeforeInsert,
    /// Before retaining caller-controlled data.
    BeforeRetain,
    /// Before enqueueing caller-controlled work.
    BeforeQueue,
    /// Before starting the next iteration.
    BeforeIterate,
    /// Before traversing the next graph unit.
    BeforeTraverse,
    /// Before starting a declared indivisible unit.
    BeforeUnit,
    /// Monotonic observations both before and after an indivisible unit.
    AroundUnit,
}

/// Why work at a source site is finite.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum Termination {
    /// A finite input byte length decreases to zero.
    InputLength,
    /// A finite collection length bounds the loop.
    CollectionLength,
    /// A visited set bounds traversal of a finite admitted graph.
    FiniteGraph,
    /// An authored semantic maximum bounds the work.
    AuthoredBound,
    /// The operation consists of one indivisible unit.
    SingleUnit,
}

/// Stable resource-incomplete code; deliberately not a refusal-catalog code.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum IncompleteCode {
    /// A requested increment would exceed a positive caller grant.
    BudgetExceeded,
}

impl IncompleteCode {
    /// Stable machine-readable code.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            // FR-016 owns this outcome and names it `resource-incomplete`.
            // The dimension and site remain separate typed identity fields;
            // they are not parsed back out of an invented dotted code.
            Self::BudgetExceeded => "resource-incomplete",
        }
    }
}

/// Stable identity of one resource-incomplete outcome.
///
/// This is a distinct Rust type from [`crate::refusal::catalog::RefusalCode`]
/// and its code is the FR-016 outcome label rather than a refusal-catalog
/// spelling. The site remains in the identity so
/// two independently exhausted charge points on one dimension do not collapse.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct ResourceIncompleteIdentity {
    code: IncompleteCode,
    dimension: BudgetDimension,
    site: SiteId,
}

impl ResourceIncompleteIdentity {
    /// Stable code family.
    #[must_use]
    pub const fn code(self) -> IncompleteCode {
        self.code
    }

    /// Exhausted caller-budget dimension.
    #[must_use]
    pub const fn dimension(self) -> BudgetDimension {
        self.dimension
    }

    /// Exact production charge point.
    #[must_use]
    pub const fn site(self) -> SiteId {
        self.site
    }
}

/// The traced boundary case assigned to one inventory entry.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct BoundaryCase(&'static str);

impl BoundaryCase {
    /// Sentinel used only to prove that reconciliation rejects an unassigned
    /// boundary. It is never admitted into an [`Inventory`].
    pub const UNASSIGNED: Self = Self("");

    /// Name a bare traced TC-009 boundary case.
    #[must_use]
    pub const fn new(symbol: &'static str) -> Self {
        Self(symbol)
    }

    /// The Rust test symbol.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        self.0
    }
}

/// One caller-controlled production source site.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct SourceSite {
    id: SiteId,
    interface: &'static str,
    dimension: BudgetDimension,
    kind: SiteKind,
}

impl SourceSite {
    /// Declare a source site beside the operation that performs the work.
    #[must_use]
    pub const fn new(
        id: SiteId,
        interface: &'static str,
        dimension: BudgetDimension,
        kind: SiteKind,
    ) -> Self {
        Self {
            id,
            interface,
            dimension,
            kind,
        }
    }

    /// Stable site name.
    #[must_use]
    pub const fn id(self) -> SiteId {
        self.id
    }

    /// Owning public or internal Rust interface.
    #[must_use]
    pub const fn interface(self) -> &'static str {
        self.interface
    }

    /// Caller-budget dimension charged here.
    #[must_use]
    pub const fn dimension(self) -> BudgetDimension {
        self.dimension
    }

    /// Operation performed on caller-controlled data.
    #[must_use]
    pub const fn kind(self) -> SiteKind {
        self.kind
    }
}

/// The independently authored inventory row for one [`SourceSite`].
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct InventoryEntry {
    source: SourceSite,
    authority: LimitAuthority,
    charge_point: ChargePoint,
    termination: Termination,
    incomplete_code: IncompleteCode,
    boundary_case: BoundaryCase,
}

impl InventoryEntry {
    /// Construct one complete inventory row.
    #[must_use]
    pub const fn new(
        source: SourceSite,
        authority: LimitAuthority,
        charge_point: ChargePoint,
        termination: Termination,
        incomplete_code: IncompleteCode,
        boundary_case: BoundaryCase,
    ) -> Self {
        Self {
            source,
            authority,
            charge_point,
            termination,
            incomplete_code,
            boundary_case,
        }
    }

    /// Source-side binding.
    #[must_use]
    pub const fn source(self) -> SourceSite {
        self.source
    }

    /// Authored limit authority.
    #[must_use]
    pub const fn authority(self) -> LimitAuthority {
        self.authority
    }

    /// Pre-work charge point.
    #[must_use]
    pub const fn charge_point(self) -> ChargePoint {
        self.charge_point
    }

    /// Finite termination argument.
    #[must_use]
    pub const fn termination(self) -> Termination {
        self.termination
    }

    /// Typed incomplete code.
    #[must_use]
    pub const fn incomplete_code(self) -> IncompleteCode {
        self.incomplete_code
    }

    /// Assigned bare traced boundary case.
    #[must_use]
    pub const fn boundary_case(self) -> BoundaryCase {
        self.boundary_case
    }
}

/// Failure to reconcile production source sites with inventory entries.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InventoryError {
    /// A source site identifier was declared twice.
    DuplicateSource {
        /// The site identifier that was declared more than once.
        site: SiteId,
    },
    /// An inventory entry was declared twice.
    DuplicateEntry {
        /// The site identifier whose inventory entry was declared more than once.
        site: SiteId,
    },
    /// A real source site has no inventory entry.
    MissingEntry {
        /// The site identifier with no inventory entry.
        site: SiteId,
    },
    /// An inventory entry names no real source site.
    OrphanEntry {
        /// The site identifier named by the entry with no matching source.
        site: SiteId,
    },
    /// An entry changes the source interface, dimension, or kind.
    BindingMismatch {
        /// The site identifier whose entry disagrees with its source declaration.
        site: SiteId,
    },
    /// An entry has no discriminating TC-009 boundary case.
    BoundaryUnassigned {
        /// The site identifier whose entry has no assigned boundary case.
        site: SiteId,
    },
}

impl fmt::Display for InventoryError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (kind, site) = match self {
            Self::DuplicateSource { site } => ("duplicate source", site),
            Self::DuplicateEntry { site } => ("duplicate entry", site),
            Self::MissingEntry { site } => ("missing entry", site),
            Self::OrphanEntry { site } => ("orphan entry", site),
            Self::BindingMismatch { site } => ("cross-wired entry", site),
            Self::BoundaryUnassigned { site } => ("unassigned boundary", site),
        };
        write!(formatter, "resource inventory {kind}: {site}")
    }
}

impl std::error::Error for InventoryError {}

/// A successfully reconciled source-bound inventory.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Inventory {
    entries: Vec<InventoryEntry>,
}

impl Inventory {
    /// Reconcile source declarations and inventory rows bidirectionally.
    ///
    /// # Errors
    ///
    /// Returns the first deterministic duplicate, missing, orphan, cross-wired,
    /// or boundary-unassigned site. No partial inventory is returned.
    pub fn reconcile(
        sources: &[SourceSite],
        entries: &[InventoryEntry],
    ) -> Result<Self, InventoryError> {
        for (index, source) in sources.iter().enumerate() {
            if sources[..index].iter().any(|known| known.id == source.id) {
                return Err(InventoryError::DuplicateSource { site: source.id });
            }
        }
        for (index, entry) in entries.iter().enumerate() {
            if entries[..index]
                .iter()
                .any(|known| known.source.id == entry.source.id)
            {
                return Err(InventoryError::DuplicateEntry {
                    site: entry.source.id,
                });
            }
        }
        for source in sources {
            let Some(entry) = entries.iter().find(|entry| entry.source.id == source.id) else {
                return Err(InventoryError::MissingEntry { site: source.id });
            };
            if entry.source != *source {
                return Err(InventoryError::BindingMismatch { site: source.id });
            }
            if entry.boundary_case == BoundaryCase::UNASSIGNED {
                return Err(InventoryError::BoundaryUnassigned { site: source.id });
            }
        }
        for entry in entries {
            if !sources.iter().any(|source| source.id == entry.source.id) {
                return Err(InventoryError::OrphanEntry {
                    site: entry.source.id,
                });
            }
        }
        Ok(Self {
            entries: entries.to_vec(),
        })
    }

    /// Every reconciled row, in inventory order.
    #[must_use]
    pub fn entries(&self) -> &[InventoryEntry] {
        &self.entries
    }

    fn entry(&self, site: SiteId) -> Option<(usize, &InventoryEntry)> {
        self.entries
            .iter()
            .enumerate()
            .find(|(_, entry)| entry.source.id == site)
    }
}

/// Typed accounting faults that are not resource-incomplete outcomes.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ChargeError {
    /// A positive caller grant was exhausted before work began.
    ResourceIncomplete(ResourceIncomplete),
    /// The source site did not survive inventory reconciliation.
    UnregisteredSite {
        /// The site identifier that did not survive inventory reconciliation.
        site: SiteId,
    },
    /// The usage accumulator itself cannot represent the requested total.
    CounterOverflow {
        /// Site being charged.
        site: SiteId,
        /// Usage before the failed increment.
        consumed: usize,
        /// Requested increment.
        requested: usize,
    },
    /// Transactional rollback found less usage than its successful charge.
    CounterUnderflow {
        /// Site being rolled back.
        site: SiteId,
        /// Usage present when rollback was attempted.
        consumed: usize,
        /// Previously charged increment being removed.
        requested: usize,
    },
    /// A supplied clock moved backwards within this request.
    ClockRegressed {
        /// Unit whose boundary observed the regression.
        unit: UnitLabel,
        /// Boundary at which the regression was observed.
        phase: Phase,
        /// Greatest earlier elapsed observation.
        previous: Duration,
    },
    /// An elapsed observation cannot be retained in the caller usage domain.
    ElapsedTimeOverflow {
        /// Exact duration that did not fit `usize` nanoseconds.
        observed: Duration,
    },
    /// A reconciled site was wired to the wrong budget dimension.
    SiteDimensionMismatch {
        /// Site whose production use disagrees with the inventory row.
        site: SiteId,
        /// Dimension found in the inventory.
        declared: BudgetDimension,
        /// Dimension required by this production mechanism.
        required: BudgetDimension,
    },
    /// A deadline dimension was passed through static additive accounting.
    TemporalDimensionChargedStatically {
        /// Static charge site.
        site: SiteId,
        /// Deadline dimension that requires monotonic observation.
        dimension: BudgetDimension,
    },
}

/// Internal accounting fault, structurally separate from budget exhaustion.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AccountingFault {
    /// The source site did not survive inventory reconciliation.
    UnregisteredSite {
        /// The site identifier that did not survive inventory reconciliation.
        site: SiteId,
    },
    /// The usage accumulator itself cannot represent the requested total.
    CounterOverflow {
        /// Site being charged.
        site: SiteId,
        /// Usage before the failed increment.
        consumed: usize,
        /// Requested increment.
        requested: usize,
    },
    /// Transactional rollback found less usage than its successful charge.
    CounterUnderflow {
        /// Site being rolled back.
        site: SiteId,
        /// Usage present when rollback was attempted.
        consumed: usize,
        /// Previously charged increment being removed.
        requested: usize,
    },
    /// A supplied clock moved backwards within this request.
    ClockRegressed {
        /// Unit whose boundary observed the regression.
        unit: UnitLabel,
        /// Boundary at which the regression was observed.
        phase: Phase,
        /// Greatest earlier elapsed observation.
        previous: Duration,
    },
    /// An elapsed observation cannot be retained in the caller usage domain.
    ElapsedTimeOverflow {
        /// Exact duration that did not fit `usize` nanoseconds.
        observed: Duration,
    },
    /// A reconciled site was wired to the wrong budget dimension.
    SiteDimensionMismatch {
        /// Site whose production use disagrees with the inventory row.
        site: SiteId,
        /// Dimension found in the inventory.
        declared: BudgetDimension,
        /// Dimension required by this production mechanism.
        required: BudgetDimension,
    },
    /// A deadline dimension was passed through static additive accounting.
    TemporalDimensionChargedStatically {
        /// Static charge site.
        site: SiteId,
        /// Deadline dimension that requires monotonic observation.
        dimension: BudgetDimension,
    },
}

/// One bounded operation's typed outcome.
///
/// Resource exhaustion is neither a domain refusal nor an internal accounting
/// defect. Keeping all four arms at the type boundary prevents a caller from
/// treating a budget-short run as an assertion about admissibility.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum OperationOutcome<T, E> {
    /// The operation completed and returned its value.
    Completed(T),
    /// The operation reached its own typed domain error or refusal.
    DomainError(E),
    /// A caller grant stopped excess static work or an elapsed deadline ended
    /// the request around an indivisible unit.
    ResourceIncomplete(ResourceIncomplete),
    /// The inventory or accounting mechanism itself is inconsistent.
    AccountingFault(AccountingFault),
}

impl<T, E> OperationOutcome<T, E> {
    pub(crate) fn from_domain(result: Result<T, E>) -> Self {
        match result {
            Ok(value) => Self::Completed(value),
            Err(error) => Self::DomainError(error),
        }
    }

    pub(crate) const fn from_charge(error: ChargeError) -> Self {
        match error {
            ChargeError::ResourceIncomplete(incomplete) => Self::ResourceIncomplete(incomplete),
            ChargeError::UnregisteredSite { site } => {
                Self::AccountingFault(AccountingFault::UnregisteredSite { site })
            }
            ChargeError::CounterOverflow {
                site,
                consumed,
                requested,
            } => Self::AccountingFault(AccountingFault::CounterOverflow {
                site,
                consumed,
                requested,
            }),
            ChargeError::CounterUnderflow {
                site,
                consumed,
                requested,
            } => Self::AccountingFault(AccountingFault::CounterUnderflow {
                site,
                consumed,
                requested,
            }),
            ChargeError::ClockRegressed {
                unit,
                phase,
                previous,
            } => Self::AccountingFault(AccountingFault::ClockRegressed {
                unit,
                phase,
                previous,
            }),
            ChargeError::ElapsedTimeOverflow { observed } => {
                Self::AccountingFault(AccountingFault::ElapsedTimeOverflow { observed })
            }
            ChargeError::SiteDimensionMismatch {
                site,
                declared,
                required,
            } => Self::AccountingFault(AccountingFault::SiteDimensionMismatch {
                site,
                declared,
                required,
            }),
            ChargeError::TemporalDimensionChargedStatically { site, dimension } => {
                Self::AccountingFault(AccountingFault::TemporalDimensionChargedStatically {
                    site,
                    dimension,
                })
            }
        }
    }
}

impl fmt::Display for ChargeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ResourceIncomplete(incomplete) => incomplete.fmt(formatter),
            Self::UnregisteredSite { site } => {
                write!(formatter, "unregistered resource site {site}")
            }
            Self::CounterOverflow {
                site,
                consumed,
                requested,
            } => write!(
                formatter,
                "resource counter overflow at {site}: {consumed} + {requested}"
            ),
            Self::CounterUnderflow {
                site,
                consumed,
                requested,
            } => write!(
                formatter,
                "resource counter underflow at {site}: {consumed} - {requested}"
            ),
            Self::ClockRegressed {
                unit,
                phase,
                previous,
            } => write!(
                formatter,
                "clock regressed around unit {unit} at {phase:?} after {previous:?}"
            ),
            Self::ElapsedTimeOverflow { observed } => write!(
                formatter,
                "elapsed observation {observed:?} does not fit usize nanoseconds"
            ),
            Self::SiteDimensionMismatch {
                site,
                declared,
                required,
            } => write!(
                formatter,
                "resource site {site} declares {declared} but production requires {required}"
            ),
            Self::TemporalDimensionChargedStatically { site, dimension } => write!(
                formatter,
                "deadline dimension {dimension} at {site} requires monotonic observation"
            ),
        }
    }
}

impl std::error::Error for ChargeError {}

/// A static or elapsed-time budget outcome, distinct from every input refusal.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ResourceIncomplete {
    code: IncompleteCode,
    dimension: BudgetDimension,
    site: SiteId,
    granted: usize,
    consumed: usize,
    requested: usize,
    deadline: Option<DeadlineObservation>,
}

/// Exact monotonic observation carried by elapsed-time resource exhaustion.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DeadlineObservation {
    unit: UnitLabel,
    phase: Phase,
    budget: Duration,
    observed: Duration,
    overshoot: Duration,
}

impl DeadlineObservation {
    /// Indivisible unit whose boundary exhausted the deadline.
    #[must_use]
    pub const fn unit(self) -> UnitLabel {
        self.unit
    }

    /// Boundary observed before or after the indivisible unit.
    #[must_use]
    pub const fn phase(self) -> Phase {
        self.phase
    }

    /// Inclusive caller deadline.
    #[must_use]
    pub const fn budget(self) -> Duration {
        self.budget
    }

    /// Exact monotonic elapsed observation, never clamped.
    #[must_use]
    pub const fn observed(self) -> Duration {
        self.observed
    }

    /// Exact indivisible-unit overshoot.
    #[must_use]
    pub const fn overshoot(self) -> Duration {
        self.overshoot
    }
}

impl ResourceIncomplete {
    /// Stable typed identity, distinct from every input-refusal identity.
    #[must_use]
    pub const fn identity(self) -> ResourceIncompleteIdentity {
        ResourceIncompleteIdentity {
            code: self.code,
            dimension: self.dimension,
            site: self.site,
        }
    }

    /// Stable resource-incomplete code.
    #[must_use]
    pub const fn code(self) -> IncompleteCode {
        self.code
    }

    /// Exact exhausted caller dimension.
    #[must_use]
    pub const fn dimension(self) -> BudgetDimension {
        self.dimension
    }

    /// Exact source site whose next unit did not start.
    #[must_use]
    pub const fn site(self) -> SiteId {
        self.site
    }

    /// Positive caller grant.
    #[must_use]
    pub const fn granted(self) -> usize {
        self.granted
    }

    /// Successfully charged static usage, or the exact elapsed observation
    /// including overshoot for deadline exhaustion.
    #[must_use]
    pub const fn consumed(self) -> usize {
        self.consumed
    }

    /// Refused static increment. This is zero for elapsed-time exhaustion,
    /// whose exact overshoot is carried by [`Self::deadline`].
    #[must_use]
    pub const fn requested(self) -> usize {
        self.requested
    }

    /// Deadline detail for elapsed-time exhaustion; absent for static limits.
    #[must_use]
    pub const fn deadline(self) -> Option<DeadlineObservation> {
        self.deadline
    }
}

impl fmt::Display for ResourceIncomplete {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(deadline) = self.deadline {
            return write!(
                formatter,
                "{}: {} at {} unit {} {:?} observed {:?} of {:?}, overshoot {:?}",
                self.code.as_str(),
                self.dimension,
                self.site,
                deadline.unit,
                deadline.phase,
                deadline.observed,
                deadline.budget,
                deadline.overshoot
            );
        }
        write!(
            formatter,
            "{}: {} at {} consumed {} of {}, next unit {} did not start",
            self.code.as_str(),
            self.dimension,
            self.site,
            self.consumed,
            self.granted,
            self.requested
        )
    }
}

struct RequestDeadline {
    clock: Box<dyn Clock + Send>,
    guard: super::DeadlineGuard,
    granted: usize,
}

struct DeadlineTarget<'deadline> {
    runtime: &'deadline mut Option<RequestDeadline>,
    site: SiteId,
    dimension: BudgetDimension,
}

/// Per-request caller-budget accounting over a reconciled inventory.
pub struct ResourceAccountant<'inventory> {
    budget: Budget,
    usage: Usage,
    site_consumed: Box<[usize]>,
    in_owner_unit: bool,
    inventory: &'inventory Inventory,
    request_deadline: Option<RequestDeadline>,
    resolution_deadline: Option<RequestDeadline>,
}

impl<'inventory> ResourceAccountant<'inventory> {
    /// Begin a fresh request with zero consumption.
    #[must_use]
    pub fn new(budget: Budget, inventory: &'inventory Inventory) -> Self {
        Self::with_deadline_clocks(
            budget,
            inventory,
            SystemMonotonicClock,
            SystemMonotonicClock,
        )
    }

    /// Begin a fresh request with an injected monotonic clock.
    ///
    /// This constructor exists so deadline behaviour is exactly testable. The
    /// production constructor uses [`SystemMonotonicClock`]. When no
    /// `elapsed-time` grant is selected, the clock is retained by no runtime
    /// guard and no deadline observation is performed.
    #[must_use]
    pub fn with_clock<C>(budget: Budget, inventory: &'inventory Inventory, clock: C) -> Self
    where
        C: Clock + 'static,
    {
        Self::with_deadline_clocks(budget, inventory, clock, SystemMonotonicClock)
    }

    /// Begin a fresh resolution request with an injected resolution clock.
    #[must_use]
    pub(crate) fn with_resolution_clock<C>(
        budget: Budget,
        inventory: &'inventory Inventory,
        clock: C,
    ) -> Self
    where
        C: Clock + 'static,
    {
        Self::with_deadline_clocks(budget, inventory, SystemMonotonicClock, clock)
    }

    fn with_deadline_clocks<RequestClock, ResolutionClock>(
        budget: Budget,
        inventory: &'inventory Inventory,
        request_clock: RequestClock,
        resolution_clock: ResolutionClock,
    ) -> Self
    where
        RequestClock: Clock + 'static,
        ResolutionClock: Clock + 'static,
    {
        let request_deadline = Self::deadline_runtime(budget.elapsed_time(), request_clock);
        let resolution_deadline =
            Self::deadline_runtime(budget.resolution_deadline(), resolution_clock);
        Self {
            budget,
            usage: Usage::ZERO,
            site_consumed: vec![0; inventory.entries.len()].into_boxed_slice(),
            in_owner_unit: false,
            inventory,
            request_deadline,
            resolution_deadline,
        }
    }

    fn deadline_runtime<C>(
        selection: Option<(Duration, usize)>,
        clock: C,
    ) -> Option<RequestDeadline>
    where
        C: Clock + 'static,
    {
        selection.map(|(duration, granted)| {
            let clock: Box<dyn Clock + Send> = Box::new(clock);
            let guard = super::DeadlineGuard::start(clock.as_ref(), duration);
            RequestDeadline {
                clock,
                guard,
                granted,
            }
        })
    }

    /// Usage charged so far.
    #[must_use]
    pub const fn usage(&self) -> Usage {
        self.usage
    }

    /// Charge one increment before starting work at `site`.
    ///
    /// An absent caller grant remains absent and does not invent a ceiling;
    /// the site's separately inventoried authored authority remains responsible
    /// for its owner or protocol maximum. A positive caller grant is inclusive.
    ///
    /// # Errors
    ///
    /// Returns [`ChargeError::ResourceIncomplete`] before changing usage when
    /// the next unit exceeds a positive grant. Internal inventory and counter
    /// faults are separate error variants and are never reported as exhausted
    /// caller budgets.
    pub fn charge(&mut self, site: SiteId, requested: usize) -> Result<(), ChargeError> {
        self.charge_effective(site, requested, None)
    }

    /// Charge one increment against both its authored and caller ceilings.
    ///
    /// A present caller ceiling applies to aggregate usage on the dimension;
    /// `authored_limit` applies only to this source site's usage in the current
    /// outer owner unit. This keeps two sites on one dimension from consuming
    /// each other's authored allowance and keeps repeated result rows from
    /// consuming one another's per-document maxima. The caller ceiling remains
    /// cumulative across every unit in the request.
    /// An absent caller grant remains absent and cannot weaken or replace the
    /// authored ceiling.
    ///
    /// # Errors
    ///
    /// Returns [`ChargeError::ResourceIncomplete`] before changing usage when
    /// the next unit exceeds either ceiling. Inventory and counter faults remain
    /// structurally separate.
    pub fn charge_with_limit(
        &mut self,
        site: SiteId,
        requested: usize,
        authored_limit: usize,
    ) -> Result<(), ChargeError> {
        self.charge_effective(site, requested, Some(authored_limit))
    }

    fn charge_effective(
        &mut self,
        site: SiteId,
        requested: usize,
        authored_limit: Option<usize>,
    ) -> Result<(), ChargeError> {
        let Some((entry_index, entry)) = self.inventory.entry(site) else {
            return Err(ChargeError::UnregisteredSite { site });
        };
        let dimension = entry.source.dimension;
        if matches!(
            dimension,
            BudgetDimension::ElapsedTime | BudgetDimension::ResolutionDeadline
        ) {
            return Err(ChargeError::TemporalDimensionChargedStatically { site, dimension });
        }
        let consumed = self.usage.consumed[dimension.index()];
        let Some(next) = consumed.checked_add(requested) else {
            return Err(ChargeError::CounterOverflow {
                site,
                consumed,
                requested,
            });
        };
        if let Some(granted) = self.budget.grant_for(dimension) {
            if next > granted {
                return Err(ChargeError::ResourceIncomplete(ResourceIncomplete {
                    code: entry.incomplete_code,
                    dimension,
                    site,
                    granted,
                    consumed,
                    requested,
                    deadline: None,
                }));
            }
        }
        let site_consumed = self.site_consumed[entry_index];
        let Some(site_next) = site_consumed.checked_add(requested) else {
            return Err(ChargeError::CounterOverflow {
                site,
                consumed: site_consumed,
                requested,
            });
        };
        if let Some(granted) = authored_limit {
            if site_next > granted {
                return Err(ChargeError::ResourceIncomplete(ResourceIncomplete {
                    code: entry.incomplete_code,
                    dimension,
                    site,
                    granted,
                    consumed: site_consumed,
                    requested,
                    deadline: None,
                }));
            }
        }
        self.usage.consumed[dimension.index()] = next;
        self.site_consumed[entry_index] = site_next;
        Ok(())
    }

    fn observe_deadline_at(
        inventory: &Inventory,
        usage: &mut Usage,
        target: DeadlineTarget<'_>,
        unit: UnitLabel,
        phase: Phase,
        token: Option<UnitToken>,
    ) -> Result<Option<UnitToken>, ChargeError> {
        let Some(runtime) = target.runtime.as_mut() else {
            return Ok(None);
        };
        let Some((_, entry)) = inventory.entry(target.site) else {
            return Err(ChargeError::UnregisteredSite { site: target.site });
        };
        if entry.source.dimension != target.dimension {
            return Err(ChargeError::SiteDimensionMismatch {
                site: target.site,
                declared: entry.source.dimension,
                required: target.dimension,
            });
        }
        let observation = match (phase, token) {
            (Phase::BeforeUnit, None) => runtime
                .guard
                .begin_unit(runtime.clock.as_ref(), unit)
                .map(Some),
            (Phase::AfterUnit, Some(token)) => runtime
                .guard
                .end_unit(runtime.clock.as_ref(), token)
                .map(|_| None),
            (Phase::BeforeUnit, Some(_)) | (Phase::AfterUnit, None) => {
                return Err(ChargeError::ClockRegressed {
                    unit,
                    phase,
                    previous: runtime.guard.observed(),
                })
            }
        };
        let observed = runtime.guard.observed();
        let consumed = usize::try_from(observed.as_nanos())
            .map_err(|_| ChargeError::ElapsedTimeOverflow { observed })?;
        usage.consumed[target.dimension.index()] = consumed;
        match observation {
            Ok(token) => Ok(token),
            Err(DeadlineRefusal::Expired {
                unit,
                phase,
                budget,
                observed,
                overshoot,
            }) => Err(ChargeError::ResourceIncomplete(ResourceIncomplete {
                code: IncompleteCode::BudgetExceeded,
                dimension: target.dimension,
                site: target.site,
                granted: runtime.granted,
                consumed,
                requested: 0,
                deadline: Some(DeadlineObservation {
                    unit,
                    phase,
                    budget,
                    observed,
                    overshoot,
                }),
            })),
            Err(DeadlineRefusal::ClockRegressed {
                unit,
                phase,
                previous,
            }) => Err(ChargeError::ClockRegressed {
                unit,
                phase,
                previous,
            }),
        }
    }

    fn observe_request_deadline(
        &mut self,
        unit: UnitLabel,
        phase: Phase,
        token: Option<UnitToken>,
    ) -> Result<Option<UnitToken>, ChargeError> {
        Self::observe_deadline_at(
            self.inventory,
            &mut self.usage,
            DeadlineTarget {
                runtime: &mut self.request_deadline,
                site: super::REQUEST_DEADLINE_SOURCE.id(),
                dimension: BudgetDimension::ElapsedTime,
            },
            unit,
            phase,
            token,
        )
    }

    fn observe_resolution_deadline(
        &mut self,
        unit: UnitLabel,
        phase: Phase,
        token: Option<UnitToken>,
    ) -> Result<Option<UnitToken>, ChargeError> {
        Self::observe_deadline_at(
            self.inventory,
            &mut self.usage,
            DeadlineTarget {
                runtime: &mut self.resolution_deadline,
                site: crate::version_lock::RESOLUTION_DEADLINE_SOURCE.id(),
                dimension: BudgetDimension::ResolutionDeadline,
            },
            unit,
            phase,
            token,
        )
    }

    fn enter_owner_unit(&mut self) -> bool {
        let was_in_owner_unit = self.in_owner_unit;
        if !was_in_owner_unit {
            self.site_consumed.fill(0);
            self.in_owner_unit = true;
        }
        was_in_owner_unit
    }

    fn leave_owner_unit(&mut self, was_in_owner_unit: bool) {
        self.in_owner_unit = was_in_owner_unit;
    }

    /// Execute one declared indivisible owner unit under the request deadline.
    ///
    /// The before observation occurs before `operation`. A successful or
    /// domain-complete operation is replaced by resource-incomplete when the
    /// after observation detects overshoot. A static resource or accounting
    /// stop remains the primary outcome, while the opened deadline token is
    /// still closed to preserve observation discipline.
    pub(crate) fn run_unit<T, E>(
        &mut self,
        unit: UnitLabel,
        operation: impl FnOnce(&mut Self) -> OperationOutcome<T, E>,
    ) -> OperationOutcome<T, E> {
        let token = match self.observe_request_deadline(unit, Phase::BeforeUnit, None) {
            Ok(token) => token,
            Err(error) => return OperationOutcome::from_charge(error),
        };
        let was_in_owner_unit = self.enter_owner_unit();
        let outcome = operation(self);
        self.leave_owner_unit(was_in_owner_unit);
        let stopped = matches!(
            &outcome,
            OperationOutcome::ResourceIncomplete(_) | OperationOutcome::AccountingFault(_)
        );
        let finished = self.observe_request_deadline(unit, Phase::AfterUnit, token);
        if stopped {
            return outcome;
        }
        match finished {
            Ok(_) => outcome,
            Err(error) => OperationOutcome::from_charge(error),
        }
    }

    /// Execute one resolution unit under the distinct FR-009 deadline.
    pub(crate) fn run_resolution_unit<T, E>(
        &mut self,
        unit: UnitLabel,
        operation: impl FnOnce(&mut Self) -> OperationOutcome<T, E>,
    ) -> OperationOutcome<T, E> {
        let token = match self.observe_resolution_deadline(unit, Phase::BeforeUnit, None) {
            Ok(token) => token,
            Err(error) => return OperationOutcome::from_charge(error),
        };
        let was_in_owner_unit = self.enter_owner_unit();
        let outcome = operation(self);
        self.leave_owner_unit(was_in_owner_unit);
        let stopped = matches!(
            &outcome,
            OperationOutcome::ResourceIncomplete(_) | OperationOutcome::AccountingFault(_)
        );
        let finished = self.observe_resolution_deadline(unit, Phase::AfterUnit, token);
        if stopped {
            return outcome;
        }
        match finished {
            Ok(_) => outcome,
            Err(error) => OperationOutcome::from_charge(error),
        }
    }

    fn rollback_charge(&mut self, site: SiteId, requested: usize) -> Result<(), ChargeError> {
        let Some((entry_index, entry)) = self.inventory.entry(site) else {
            return Err(ChargeError::UnregisteredSite { site });
        };
        let dimension = entry.source.dimension;
        let consumed = self.usage.consumed[dimension.index()];
        let Some(next) = consumed.checked_sub(requested) else {
            return Err(ChargeError::CounterUnderflow {
                site,
                consumed,
                requested,
            });
        };
        let site_consumed = self.site_consumed[entry_index];
        let Some(site_next) = site_consumed.checked_sub(requested) else {
            return Err(ChargeError::CounterUnderflow {
                site,
                consumed: site_consumed,
                requested,
            });
        };
        self.usage.consumed[dimension.index()] = next;
        self.site_consumed[entry_index] = site_next;
        Ok(())
    }

    /// Transactionally charge all pre-work increments for one operation.
    ///
    /// If any dimension cannot admit its increment, every earlier tentative
    /// charge is rolled back. This is accounting rollback only: callers must
    /// invoke it before any represented work begins.
    ///
    /// # Errors
    ///
    /// Returns the first charge error in declared order with usage unchanged.
    pub fn charge_all(&mut self, charges: &[(SiteId, usize)]) -> Result<(), ChargeError> {
        for (index, (site, requested)) in charges.iter().enumerate() {
            if let Err(error) = self.charge(*site, *requested) {
                for (charged_site, charged_request) in charges[..index].iter().rev() {
                    self.rollback_charge(*charged_site, *charged_request)?;
                }
                return Err(error);
            }
        }
        Ok(())
    }

    /// Transactionally charge increments against authored and caller ceilings.
    ///
    /// # Errors
    ///
    /// Returns the first resource-incomplete or accounting fault with the
    /// complete pre-call usage restored.
    pub fn charge_all_with_limits(
        &mut self,
        charges: &[(SiteId, usize, usize)],
    ) -> Result<(), ChargeError> {
        for (index, (site, requested, authored_limit)) in charges.iter().enumerate() {
            if let Err(error) = self.charge_with_limit(*site, *requested, *authored_limit) {
                for (charged_site, charged_request, _) in charges[..index].iter().rev() {
                    self.rollback_charge(*charged_site, *charged_request)?;
                }
                return Err(error);
            }
        }
        Ok(())
    }

    /// Transactionally charge one operation whose sites mix caller-only and
    /// authored ceilings.
    ///
    /// `None` preserves the absence of an authored maximum; it is not replaced
    /// by `usize::MAX` or another site's authority. This is crate-private so
    /// owner operations can compose one atomic pre-work batch without exposing
    /// optional ceiling mechanics as part of the public caller API.
    pub(crate) fn charge_all_mixed(
        &mut self,
        charges: &[(SiteId, usize, Option<usize>)],
    ) -> Result<(), ChargeError> {
        for (index, (site, requested, authored_limit)) in charges.iter().enumerate() {
            if let Err(error) = self.charge_effective(*site, *requested, *authored_limit) {
                for (charged_site, charged_request, _) in charges[..index].iter().rev() {
                    self.rollback_charge(*charged_site, *charged_request)?;
                }
                return Err(error);
            }
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Production source population and independent inventory population
// ---------------------------------------------------------------------------

const PRODUCTION_BOUNDARY: BoundaryCase =
    BoundaryCase::new("tc_009_every_production_site_has_below_at_above_boundary");

// Inventory-side bindings are spelled independently from the source-side
// declarations in their owning modules. Reusing the source constants here
// would let a cross-wire mutate both populations at once and evade comparison.
const IDENTITY_ALLOCATION: SourceSite = SourceSite::new(
    SiteId::new("identity/storage/allocate"),
    "ids::*::new",
    BudgetDimension::AccountedAllocation,
    SiteKind::Allocation,
);
const IDENTITY_INPUT: SourceSite = SourceSite::new(
    SiteId::new("identity/storage/retain"),
    "ids::*::new",
    BudgetDimension::InputByte,
    SiteKind::Retention,
);
const REQUEST_DEADLINE: SourceSite = SourceSite::new(
    SiteId::new("request/unit/deadline"),
    "bounds::inventory::ResourceAccountant::run_unit",
    BudgetDimension::ElapsedTime,
    SiteKind::Iteration,
);
const REPRO_INPUT: SourceSite = SourceSite::new(
    SiteId::new("reproduction/identity/retain"),
    "repro::ToolchainIdentity::new/repro::CorpusIdentity::new",
    BudgetDimension::InputByte,
    SiteKind::Retention,
);
const REPRO_ALLOCATION: SourceSite = SourceSite::new(
    SiteId::new("reproduction/identity/allocate"),
    "repro::ToolchainIdentity::new/repro::CorpusIdentity::new",
    BudgetDimension::AccountedAllocation,
    SiteKind::Allocation,
);
const COMPENSATION_OBSERVATION: SourceSite = SourceSite::new(
    SiteId::new("compensation/observation/consume"),
    "compensation::recovery::assess",
    BudgetDimension::Observation,
    SiteKind::Iteration,
);
const COMPENSATION_ACTIVATION_RESULT: SourceSite = SourceSite::new(
    SiteId::new("compensation/result/activation/retain"),
    "compensation::compose",
    BudgetDimension::ResultItem,
    SiteKind::Retention,
);
const COMPENSATION_REGISTRATION_RESULT: SourceSite = SourceSite::new(
    SiteId::new("compensation/result/registration/retain"),
    "compensation::compose",
    BudgetDimension::ResultItem,
    SiteKind::Retention,
);
const COMPENSATION_ORDERING_RESULT: SourceSite = SourceSite::new(
    SiteId::new("compensation/result/ordering/retain"),
    "compensation::compose",
    BudgetDimension::ResultItem,
    SiteKind::Retention,
);
const COMPENSATION_ATTEMPT_RESULT: SourceSite = SourceSite::new(
    SiteId::new("compensation/result/attempt/retain"),
    "compensation::compose",
    BudgetDimension::ResultItem,
    SiteKind::Retention,
);
const COMPENSATION_EFFECT_RESULT: SourceSite = SourceSite::new(
    SiteId::new("compensation/result/effect/retain"),
    "compensation::compose",
    BudgetDimension::ResultItem,
    SiteKind::Retention,
);
const COMPENSATION_RETRY_RESULT: SourceSite = SourceSite::new(
    SiteId::new("compensation/result/retry/retain"),
    "compensation::compose",
    BudgetDimension::ResultItem,
    SiteKind::Retention,
);
const COMPENSATION_RECOVERY_RESULT: SourceSite = SourceSite::new(
    SiteId::new("compensation/result/recovery/retain"),
    "compensation::compose",
    BudgetDimension::ResultItem,
    SiteKind::Retention,
);
const COMPENSATION_ALLOCATION: SourceSite = SourceSite::new(
    SiteId::new("compensation/snapshot/allocate"),
    "compensation::compose",
    BudgetDimension::AccountedAllocation,
    SiteKind::Allocation,
);
const RESOLUTION_ENTRY: SourceSite = SourceSite::new(
    SiteId::new("resolution/entry/validate"),
    "version_lock::resolve",
    BudgetDimension::ResolutionEntryCount,
    SiteKind::Iteration,
);
const RESOLUTION_BYTES: SourceSite = SourceSite::new(
    SiteId::new("resolution/bytes/digest"),
    "version_lock::resolve",
    BudgetDimension::ResolutionBytesDigested,
    SiteKind::Iteration,
);
const RESOLUTION_DEADLINE: SourceSite = SourceSite::new(
    SiteId::new("resolution/unit/deadline"),
    "version_lock::resolve",
    BudgetDimension::ResolutionDeadline,
    SiteKind::Iteration,
);
const DELIVERY_INDEX_INPUT: SourceSite = SourceSite::new(
    SiteId::new("delivery-index/member/retain"),
    "delivery_index::DeliveryIndex::new_bounded",
    BudgetDimension::InputByte,
    SiteKind::Retention,
);
const DELIVERY_INDEX_ALLOCATION: SourceSite = SourceSite::new(
    SiteId::new("delivery-index/index/allocate"),
    "delivery_index::DeliveryIndex::new_bounded",
    BudgetDimension::AccountedAllocation,
    SiteKind::Allocation,
);

/// Production work sites derived from reviewed Rust interfaces and paths.
///
/// This population is intentionally separate from [`PRODUCTION_ENTRIES`]. A
/// deletion or addition on either side therefore fails bidirectional
/// reconciliation rather than changing both populations by construction.
pub const PRODUCTION_SOURCES: &[SourceSite] = &[
    crate::delivery_index::INPUT_SOURCE,
    crate::delivery_index::ALLOCATION_SOURCE,
    crate::version_lock::RESOLUTION_ENTRY_SOURCE,
    crate::version_lock::RESOLUTION_BYTES_SOURCE,
    crate::version_lock::RESOLUTION_DEADLINE_SOURCE,
    crate::ids::ALLOCATION_SOURCE,
    crate::ids::INPUT_SOURCE,
    super::REQUEST_DEADLINE_SOURCE,
    crate::repro::INPUT_SOURCE,
    crate::repro::ALLOCATION_SOURCE,
    crate::compensation::recovery::OBSERVATION_SOURCE,
    crate::compensation::ACTIVATION_RESULT_SOURCE,
    crate::compensation::REGISTRATION_RESULT_SOURCE,
    crate::compensation::ORDERING_RESULT_SOURCE,
    crate::compensation::ATTEMPT_RESULT_SOURCE,
    crate::compensation::EFFECT_RESULT_SOURCE,
    crate::compensation::RETRY_RESULT_SOURCE,
    crate::compensation::RECOVERY_RESULT_SOURCE,
    crate::compensation::ALLOCATION_SOURCE,
];

const fn entry(
    source: SourceSite,
    authority: LimitAuthority,
    charge_point: ChargePoint,
    termination: Termination,
) -> InventoryEntry {
    InventoryEntry::new(
        source,
        authority,
        charge_point,
        termination,
        IncompleteCode::BudgetExceeded,
        PRODUCTION_BOUNDARY,
    )
}

/// Independently authored production inventory rows.
pub const PRODUCTION_ENTRIES: &[InventoryEntry] = &[
    entry(
        DELIVERY_INDEX_INPUT,
        LimitAuthority::ConsumerOwner,
        ChargePoint::BeforeRetain,
        Termination::InputLength,
    ),
    entry(
        DELIVERY_INDEX_ALLOCATION,
        LimitAuthority::ConsumerOwner,
        ChargePoint::BeforeAllocate,
        Termination::SingleUnit,
    ),
    entry(
        RESOLUTION_ENTRY,
        LimitAuthority::ConsumerOwner,
        ChargePoint::BeforeIterate,
        Termination::CollectionLength,
    ),
    entry(
        RESOLUTION_BYTES,
        LimitAuthority::ConsumerOwner,
        ChargePoint::BeforeIterate,
        Termination::InputLength,
    ),
    entry(
        RESOLUTION_DEADLINE,
        LimitAuthority::ConsumerOwner,
        ChargePoint::AroundUnit,
        Termination::CollectionLength,
    ),
    entry(
        IDENTITY_ALLOCATION,
        LimitAuthority::ConsumerOwner,
        ChargePoint::BeforeAllocate,
        Termination::SingleUnit,
    ),
    entry(
        IDENTITY_INPUT,
        LimitAuthority::ConsumerOwner,
        ChargePoint::BeforeRetain,
        Termination::InputLength,
    ),
    entry(
        REQUEST_DEADLINE,
        LimitAuthority::ConsumerOwner,
        ChargePoint::AroundUnit,
        Termination::SingleUnit,
    ),
    entry(
        REPRO_INPUT,
        LimitAuthority::ConsumerOwner,
        ChargePoint::BeforeRetain,
        Termination::InputLength,
    ),
    entry(
        REPRO_ALLOCATION,
        LimitAuthority::ConsumerOwner,
        ChargePoint::BeforeAllocate,
        Termination::SingleUnit,
    ),
    entry(
        COMPENSATION_OBSERVATION,
        LimitAuthority::ProducerOwner,
        ChargePoint::BeforeIterate,
        Termination::CollectionLength,
    ),
    entry(
        COMPENSATION_ACTIVATION_RESULT,
        LimitAuthority::ConsumerOwner,
        ChargePoint::BeforeRetain,
        Termination::SingleUnit,
    ),
    entry(
        COMPENSATION_REGISTRATION_RESULT,
        LimitAuthority::ConsumerOwner,
        ChargePoint::BeforeRetain,
        Termination::SingleUnit,
    ),
    entry(
        COMPENSATION_ORDERING_RESULT,
        LimitAuthority::ConsumerOwner,
        ChargePoint::BeforeRetain,
        Termination::SingleUnit,
    ),
    entry(
        COMPENSATION_ATTEMPT_RESULT,
        LimitAuthority::ConsumerOwner,
        ChargePoint::BeforeRetain,
        Termination::CollectionLength,
    ),
    entry(
        COMPENSATION_EFFECT_RESULT,
        LimitAuthority::ConsumerOwner,
        ChargePoint::BeforeRetain,
        Termination::CollectionLength,
    ),
    entry(
        COMPENSATION_RETRY_RESULT,
        LimitAuthority::ConsumerOwner,
        ChargePoint::BeforeRetain,
        Termination::CollectionLength,
    ),
    entry(
        COMPENSATION_RECOVERY_RESULT,
        LimitAuthority::ConsumerOwner,
        ChargePoint::BeforeRetain,
        Termination::CollectionLength,
    ),
    entry(
        COMPENSATION_ALLOCATION,
        LimitAuthority::ConsumerOwner,
        ChargePoint::BeforeAllocate,
        Termination::SingleUnit,
    ),
];

/// Reviewed production source sites.
#[must_use]
pub const fn production_sources() -> &'static [SourceSite] {
    PRODUCTION_SOURCES
}

/// Reviewed production inventory entries.
#[must_use]
pub const fn production_entries() -> &'static [InventoryEntry] {
    PRODUCTION_ENTRIES
}

/// Reconcile the complete production populations.
///
/// # Errors
///
/// Returns the exact inventory mutation found before any request work begins.
pub fn production_inventory() -> Result<Inventory, InventoryError> {
    Inventory::reconcile(PRODUCTION_SOURCES, PRODUCTION_ENTRIES)
}
