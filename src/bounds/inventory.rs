// SPDX-License-Identifier: AGPL-3.0-or-later
//! QProtocol caller budgets and the source-bound resource-site inventory.
//!
//! This module deliberately does not reuse
//! [`quire_spec_language::protocol_artifact::Dimension`]. That type is the
//! producer's accounting vocabulary while reading a compiled artifact. The
//! closed [`BudgetDimension`] set below is QProtocol's caller-facing vocabulary
//! from FR-016. Similar labels across the two domains do not make them the same
//! authority or the same exhausted resource.
//!
//! A [`SourceSite`] is declared beside production work. An independent
//! [`InventoryEntry`] assigns that site an authored-limit authority, pre-work
//! charge point, finite termination argument, resource-incomplete code, and a
//! discriminating TC-009 boundary case. [`Inventory::reconcile`] compares both
//! populations in both directions and rejects every duplicate, missing, orphan,
//! cross-wired, or boundary-unassigned entry.

use core::fmt;

use super::{SiteId, SiteKind};

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
    /// Manifest bytes digested before producer values are read.
    ResolutionBytesDigested,
    /// Monotonic elapsed time for manifest resolution.
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
}

impl Budget {
    /// Construct a request with every caller grant explicitly absent.
    #[must_use]
    pub const fn nothing_granted() -> Self {
        Self {
            grants: [None; BudgetDimension::ALL.len()],
        }
    }

    /// Add one positive grant without replacing an earlier selection.
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
        *slot = Some(granted);
        Ok(self)
    }

    /// The exact caller grant, retaining absence.
    #[must_use]
    pub const fn grant_for(&self, dimension: BudgetDimension) -> Option<usize> {
        self.grants[dimension.index()]
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
    DuplicateSource { site: SiteId },
    /// An inventory entry was declared twice.
    DuplicateEntry { site: SiteId },
    /// A real source site has no inventory entry.
    MissingEntry { site: SiteId },
    /// An inventory entry names no real source site.
    OrphanEntry { site: SiteId },
    /// An entry changes the source interface, dimension, or kind.
    BindingMismatch { site: SiteId },
    /// An entry has no discriminating TC-009 boundary case.
    BoundaryUnassigned { site: SiteId },
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

    fn entry(&self, site: SiteId) -> Option<&InventoryEntry> {
        self.entries.iter().find(|entry| entry.source.id == site)
    }
}

/// Typed accounting faults that are not resource-incomplete outcomes.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ChargeError {
    /// A positive caller grant was exhausted before work began.
    ResourceIncomplete(ResourceIncomplete),
    /// The source site did not survive inventory reconciliation.
    UnregisteredSite { site: SiteId },
    /// The usage accumulator itself cannot represent the requested total.
    CounterOverflow {
        /// Site being charged.
        site: SiteId,
        /// Usage before the failed increment.
        consumed: usize,
        /// Requested increment.
        requested: usize,
    },
}

/// Internal accounting fault, structurally separate from budget exhaustion.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AccountingFault {
    /// The source site did not survive inventory reconciliation.
    UnregisteredSite { site: SiteId },
    /// The usage accumulator itself cannot represent the requested total.
    CounterOverflow {
        /// Site being charged.
        site: SiteId,
        /// Usage before the failed increment.
        consumed: usize,
        /// Requested increment.
        requested: usize,
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
    /// A caller grant stopped the operation before its excess unit began.
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
        }
    }
}

impl std::error::Error for ChargeError {}

/// A budget-exhausted outcome, distinct from every input refusal.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ResourceIncomplete {
    code: IncompleteCode,
    dimension: BudgetDimension,
    site: SiteId,
    granted: usize,
    consumed: usize,
    requested: usize,
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

    /// Successfully charged usage before this request.
    #[must_use]
    pub const fn consumed(self) -> usize {
        self.consumed
    }

    /// Refused increment.
    #[must_use]
    pub const fn requested(self) -> usize {
        self.requested
    }
}

impl fmt::Display for ResourceIncomplete {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
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

/// Per-request caller-budget accounting over a reconciled inventory.
pub struct ResourceAccountant<'inventory> {
    budget: Budget,
    usage: Usage,
    inventory: &'inventory Inventory,
}

impl<'inventory> ResourceAccountant<'inventory> {
    /// Begin a fresh request with zero consumption.
    #[must_use]
    pub const fn new(budget: Budget, inventory: &'inventory Inventory) -> Self {
        Self {
            budget,
            usage: Usage::ZERO,
            inventory,
        }
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
    /// The effective ceiling is the lower of `authored_limit` and a present
    /// caller grant. An absent caller grant remains absent and therefore cannot
    /// weaken or replace the authored ceiling.
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
        let Some(entry) = self.inventory.entry(site) else {
            return Err(ChargeError::UnregisteredSite { site });
        };
        let dimension = entry.source.dimension;
        let consumed = self.usage.consumed[dimension.index()];
        let Some(next) = consumed.checked_add(requested) else {
            return Err(ChargeError::CounterOverflow {
                site,
                consumed,
                requested,
            });
        };
        let caller_limit = self.budget.grant_for(dimension);
        let effective_limit = match (caller_limit, authored_limit) {
            (Some(caller), Some(authored)) => Some(caller.min(authored)),
            (Some(caller), None) => Some(caller),
            (None, Some(authored)) => Some(authored),
            (None, None) => None,
        };
        if let Some(granted) = effective_limit {
            if next > granted {
                return Err(ChargeError::ResourceIncomplete(ResourceIncomplete {
                    code: entry.incomplete_code,
                    dimension,
                    site,
                    granted,
                    consumed,
                    requested,
                }));
            }
        }
        self.usage.consumed[dimension.index()] = next;
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
        let before = self.usage;
        for (site, requested) in charges {
            if let Err(error) = self.charge(*site, *requested) {
                self.usage = before;
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
        let before = self.usage;
        for (site, requested, authored_limit) in charges {
            if let Err(error) = self.charge_with_limit(*site, *requested, *authored_limit) {
                self.usage = before;
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
const ADMISSION_INPUT: SourceSite = SourceSite::new(
    SiteId::new("admission/offer/decode"),
    "inventory::map::admit_discriminating",
    BudgetDimension::InputByte,
    SiteKind::Iteration,
);
const LINK_DEFINITION: SourceSite = SourceSite::new(
    SiteId::new("link/definition/select"),
    "link::link",
    BudgetDimension::Definition,
    SiteKind::Iteration,
);
const LINK_NODE: SourceSite = SourceSite::new(
    SiteId::new("link/node/retain"),
    "link::link",
    BudgetDimension::Node,
    SiteKind::Retention,
);
const LINK_EDGE: SourceSite = SourceSite::new(
    SiteId::new("link/edge/traverse"),
    "link::link",
    BudgetDimension::Edge,
    SiteKind::Traversal,
);
const LINK_ROLE: SourceSite = SourceSite::new(
    SiteId::new("link/role/retain"),
    "link::link",
    BudgetDimension::Role,
    SiteKind::Retention,
);
const LINK_MESSAGE: SourceSite = SourceSite::new(
    SiteId::new("link/message/retain"),
    "link::link",
    BudgetDimension::Message,
    SiteKind::Retention,
);
const LINK_BRANCH: SourceSite = SourceSite::new(
    SiteId::new("link/interleaving/branch/queue"),
    "link::interleaving",
    BudgetDimension::Branch,
    SiteKind::Queue,
);
const LINK_ITERATION: SourceSite = SourceSite::new(
    SiteId::new("link/repeat/iterate"),
    "link::control_shape",
    BudgetDimension::Iteration,
    SiteKind::Iteration,
);
const LINK_ACTIVE_INSTANCE: SourceSite = SourceSite::new(
    SiteId::new("link/active-instance/retain"),
    "link::link",
    BudgetDimension::ActiveInstance,
    SiteKind::Retention,
);
const CONFORMANCE_OBSERVATION: SourceSite = SourceSite::new(
    SiteId::new("conformance/observation/consume"),
    "conformance::batch::assess",
    BudgetDimension::Observation,
    SiteKind::Iteration,
);
const RESULT_LINEAGE: SourceSite = SourceSite::new(
    SiteId::new("result/lineage/traverse"),
    "result::lineage::validate",
    BudgetDimension::Lineage,
    SiteKind::Traversal,
);
const RESULT_ITEM: SourceSite = SourceSite::new(
    SiteId::new("result/item/retain"),
    "result::record::produce",
    BudgetDimension::ResultItem,
    SiteKind::Retention,
);
const CONTRACT_IR_DERIVED: SourceSite = SourceSite::new(
    SiteId::new("result/contract-ir/derive"),
    "result::contract_ir::map",
    BudgetDimension::DerivedArtifact,
    SiteKind::Retention,
);
const RESULT_OUTPUT: SourceSite = SourceSite::new(
    SiteId::new("result/output/allocate"),
    "result::record::produce",
    BudgetDimension::OutputByte,
    SiteKind::Allocation,
);
const CONFORMANCE_OPERATION: SourceSite = SourceSite::new(
    SiteId::new("conformance/obligation/assess"),
    "conformance::batch::assess",
    BudgetDimension::Operation,
    SiteKind::Iteration,
);
const IDENTITY_ALLOCATION: SourceSite = SourceSite::new(
    SiteId::new("identity/storage/allocate"),
    "ids::*::new",
    BudgetDimension::AccountedAllocation,
    SiteKind::Allocation,
);
const REQUEST_DEADLINE: SourceSite = SourceSite::new(
    SiteId::new("request/unit/deadline"),
    "bounds::DeadlineGuard::begin_unit",
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
const CLAIM_INPUT: SourceSite = SourceSite::new(
    SiteId::new("claims/premise/retain"),
    "claims::premises::PremiseSet",
    BudgetDimension::InputByte,
    SiteKind::Retention,
);
const CLAIM_RESULT: SourceSite = SourceSite::new(
    SiteId::new("claims/disposition/retain"),
    "claims::dispatch::dispatch",
    BudgetDimension::ResultItem,
    SiteKind::Retention,
);
const CLAIM_DERIVED: SourceSite = SourceSite::new(
    SiteId::new("claims/correspondence/derive"),
    "claims::dispatch::dispatch",
    BudgetDimension::DerivedArtifact,
    SiteKind::Retention,
);
const CLAIM_OPERATION: SourceSite = SourceSite::new(
    SiteId::new("claims/backend/invoke"),
    "claims::dispatch::dispatch",
    BudgetDimension::Operation,
    SiteKind::Iteration,
);
const COMPENSATION_OBSERVATION: SourceSite = SourceSite::new(
    SiteId::new("compensation/observation/consume"),
    "compensation::recovery::assess",
    BudgetDimension::Observation,
    SiteKind::Iteration,
);
const COMPENSATION_RESULT: SourceSite = SourceSite::new(
    SiteId::new("compensation/result/retain"),
    "compensation::compose",
    BudgetDimension::ResultItem,
    SiteKind::Retention,
);
const RESULT_INPUT: SourceSite = SourceSite::new(
    SiteId::new("result/input/decode"),
    "result::reader::read",
    BudgetDimension::InputByte,
    SiteKind::Iteration,
);
const RESULT_OPERATION: SourceSite = SourceSite::new(
    SiteId::new("result/field/visit"),
    "result::reader::read",
    BudgetDimension::Operation,
    SiteKind::Traversal,
);

/// Production work sites derived from reviewed Rust interfaces and paths.
///
/// This population is intentionally separate from [`PRODUCTION_ENTRIES`]. A
/// deletion or addition on either side therefore fails bidirectional
/// reconciliation rather than changing both populations by construction.
pub const PRODUCTION_SOURCES: &[SourceSite] = &[
    crate::inventory::map::ADMISSION_INPUT_SOURCE,
    crate::link::DEFINITION_SOURCE,
    crate::link::NODE_SOURCE,
    crate::link::EDGE_SOURCE,
    crate::link::ROLE_SOURCE,
    crate::link::MESSAGE_SOURCE,
    crate::link::BRANCH_SOURCE,
    crate::link::ITERATION_SOURCE,
    crate::link::ACTIVE_INSTANCE_SOURCE,
    crate::conformance::batch::OBSERVATION_SOURCE,
    crate::result::lineage::LINEAGE_SOURCE,
    crate::result::record::ITEM_SOURCE,
    crate::result::contract_ir::DERIVED_SOURCE,
    crate::result::record::OUTPUT_SOURCE,
    crate::conformance::batch::OPERATION_SOURCE,
    crate::ids::ALLOCATION_SOURCE,
    super::REQUEST_DEADLINE_SOURCE,
    crate::repro::INPUT_SOURCE,
    crate::repro::ALLOCATION_SOURCE,
    crate::claims::premises::INPUT_SOURCE,
    crate::claims::dispatch::RESULT_SOURCE,
    crate::claims::dispatch::DERIVED_SOURCE,
    crate::claims::dispatch::OPERATION_SOURCE,
    crate::compensation::recovery::OBSERVATION_SOURCE,
    crate::compensation::RESULT_SOURCE,
    crate::result::reader::INPUT_SOURCE,
    crate::result::reader::OPERATION_SOURCE,
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
        ADMISSION_INPUT,
        LimitAuthority::ConsumerOwner,
        ChargePoint::BeforeDecode,
        Termination::InputLength,
    ),
    entry(
        LINK_DEFINITION,
        LimitAuthority::ProducerOwner,
        ChargePoint::BeforeIterate,
        Termination::CollectionLength,
    ),
    entry(
        LINK_NODE,
        LimitAuthority::ProducerOwner,
        ChargePoint::BeforeRetain,
        Termination::CollectionLength,
    ),
    entry(
        LINK_EDGE,
        LimitAuthority::ProducerOwner,
        ChargePoint::BeforeTraverse,
        Termination::FiniteGraph,
    ),
    entry(
        LINK_ROLE,
        LimitAuthority::ProducerOwner,
        ChargePoint::BeforeRetain,
        Termination::CollectionLength,
    ),
    entry(
        LINK_MESSAGE,
        LimitAuthority::ProducerOwner,
        ChargePoint::BeforeRetain,
        Termination::CollectionLength,
    ),
    entry(
        LINK_BRANCH,
        LimitAuthority::ProtocolAuthor,
        ChargePoint::BeforeQueue,
        Termination::FiniteGraph,
    ),
    entry(
        LINK_ITERATION,
        LimitAuthority::ProtocolAuthor,
        ChargePoint::BeforeIterate,
        Termination::AuthoredBound,
    ),
    entry(
        LINK_ACTIVE_INSTANCE,
        LimitAuthority::ProducerOwner,
        ChargePoint::BeforeRetain,
        Termination::CollectionLength,
    ),
    entry(
        CONFORMANCE_OBSERVATION,
        LimitAuthority::ConsumerOwner,
        ChargePoint::BeforeIterate,
        Termination::CollectionLength,
    ),
    entry(
        RESULT_LINEAGE,
        LimitAuthority::ConsumerOwner,
        ChargePoint::BeforeTraverse,
        Termination::FiniteGraph,
    ),
    entry(
        RESULT_ITEM,
        LimitAuthority::ConsumerOwner,
        ChargePoint::BeforeRetain,
        Termination::CollectionLength,
    ),
    entry(
        CONTRACT_IR_DERIVED,
        LimitAuthority::ConsumerOwner,
        ChargePoint::BeforeRetain,
        Termination::SingleUnit,
    ),
    entry(
        RESULT_OUTPUT,
        LimitAuthority::ConsumerOwner,
        ChargePoint::BeforeAllocate,
        Termination::InputLength,
    ),
    entry(
        CONFORMANCE_OPERATION,
        LimitAuthority::ConsumerOwner,
        ChargePoint::BeforeUnit,
        Termination::CollectionLength,
    ),
    entry(
        IDENTITY_ALLOCATION,
        LimitAuthority::ConsumerOwner,
        ChargePoint::BeforeAllocate,
        Termination::SingleUnit,
    ),
    entry(
        REQUEST_DEADLINE,
        LimitAuthority::ConsumerOwner,
        ChargePoint::BeforeUnit,
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
        CLAIM_INPUT,
        LimitAuthority::ConsumerOwner,
        ChargePoint::BeforeRetain,
        Termination::CollectionLength,
    ),
    entry(
        CLAIM_RESULT,
        LimitAuthority::ConsumerOwner,
        ChargePoint::BeforeRetain,
        Termination::CollectionLength,
    ),
    entry(
        CLAIM_DERIVED,
        LimitAuthority::ConsumerOwner,
        ChargePoint::BeforeRetain,
        Termination::SingleUnit,
    ),
    entry(
        CLAIM_OPERATION,
        LimitAuthority::ConsumerOwner,
        ChargePoint::BeforeUnit,
        Termination::CollectionLength,
    ),
    entry(
        COMPENSATION_OBSERVATION,
        LimitAuthority::ProducerOwner,
        ChargePoint::BeforeIterate,
        Termination::CollectionLength,
    ),
    entry(
        COMPENSATION_RESULT,
        LimitAuthority::ConsumerOwner,
        ChargePoint::BeforeRetain,
        Termination::CollectionLength,
    ),
    entry(
        RESULT_INPUT,
        LimitAuthority::ConsumerOwner,
        ChargePoint::BeforeDecode,
        Termination::InputLength,
    ),
    entry(
        RESULT_OPERATION,
        LimitAuthority::ConsumerOwner,
        ChargePoint::BeforeTraverse,
        Termination::FiniteGraph,
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
