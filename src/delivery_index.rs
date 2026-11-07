// SPDX-License-Identifier: AGPL-3.0-or-later
//! FR-018: consumer-authored evidence for one already-pinned delivery.
//!
//! This module is deliberately an in-memory value boundary.  It neither reads a
//! filesystem nor accepts a version-lock manifest or resolver capability, so an
//! index cannot discover, supply, select, or repair a version-lock entry.

use core::fmt;
use core::hash::{Hash, Hasher};
use std::collections::BTreeSet;

use crate::bounds::inventory::{BudgetDimension, OperationOutcome, ResourceAccountant, SourceSite};
use crate::bounds::{SiteId, SiteKind};
use crate::version_lock::{EntryKey, ExactBytes};

/// Exact-byte retention is charged before this consumer keeps a delivery index.
pub(crate) const INPUT_SOURCE: SourceSite = SourceSite::new(
    SiteId::new("delivery-index/member/retain"),
    "delivery_index::DeliveryIndex::new_bounded",
    BudgetDimension::InputByte,
    SiteKind::Retention,
);
/// The bounded assembly creates one retained consumer-owned index value.
pub(crate) const ALLOCATION_SOURCE: SourceSite = SourceSite::new(
    SiteId::new("delivery-index/index/allocate"),
    "delivery_index::DeliveryIndex::new_bounded",
    BudgetDimension::AccountedAllocation,
    SiteKind::Allocation,
);

/// Exact consumer-selected root for a delivered producer revision.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Hash)]
pub struct DeliveryRoot(ExactBytes);

impl DeliveryRoot {
    /// Retain a non-empty root exactly as selected.
    #[must_use]
    pub fn new(bytes: &[u8]) -> Self {
        Self(ExactBytes::new(bytes))
    }

    /// Exact selected root bytes.
    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        self.0.as_bytes()
    }
}

/// Exact immutable producer revision copied into an index.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Hash)]
pub struct ProducerRevision(ExactBytes);

impl ProducerRevision {
    /// Retain the producer revision without normalization.
    #[must_use]
    pub fn new(bytes: &[u8]) -> Self {
        Self(ExactBytes::new(bytes))
    }

    /// Exact producer revision bytes.
    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        self.0.as_bytes()
    }
}

/// Componentwise identity of one consumer-authored delivery index.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Hash)]
pub struct DeliveryIndexKey {
    delivery_root: DeliveryRoot,
    producer_revision: ProducerRevision,
}

impl DeliveryIndexKey {
    /// Build the only identity an index owns.
    #[must_use]
    pub const fn new(delivery_root: DeliveryRoot, producer_revision: ProducerRevision) -> Self {
        Self {
            delivery_root,
            producer_revision,
        }
    }

    /// The selected delivery root.
    #[must_use]
    pub const fn delivery_root(&self) -> &DeliveryRoot {
        &self.delivery_root
    }

    /// The copied producer revision.
    #[must_use]
    pub const fn producer_revision(&self) -> &ProducerRevision {
        &self.producer_revision
    }
}

/// One byte-bearing member as observed under the delivery root.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IndexedMember {
    path: ExactBytes,
    bytes: ExactBytes,
}

impl IndexedMember {
    /// Retain a member path and its exact delivered bytes.
    #[must_use]
    pub fn new(path: &[u8], bytes: &[u8]) -> Self {
        Self {
            path: ExactBytes::new(path),
            bytes: ExactBytes::new(bytes),
        }
    }

    /// Convert an adapter's member-read result without treating an absence as
    /// empty bytes. The adapter remains responsible for I/O.
    pub fn observed(
        path: &[u8],
        result: Result<&[u8], MemberReadError>,
    ) -> Result<Self, DeliveryIndexError> {
        match result {
            Ok(bytes) => Ok(Self::new(path, bytes)),
            Err(MemberReadError::Missing) => {
                Err(DeliveryIndexError::MissingMember(ExactBytes::new(path)))
            }
            Err(MemberReadError::Unreadable(reason)) => Err(DeliveryIndexError::UnreadableMember {
                path: ExactBytes::new(path),
                reason,
            }),
        }
    }

    /// Path relative to the delivery root.
    #[must_use]
    pub const fn path(&self) -> &ExactBytes {
        &self.path
    }

    /// Exact submitted bytes. No digest is synthesized here.
    #[must_use]
    pub const fn bytes(&self) -> &ExactBytes {
        &self.bytes
    }
}

/// An adapter-reported member-read state. This module performs no read itself.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MemberReadError {
    /// The indexed member was absent under the selected root.
    Missing,
    /// The member existed but its bytes could not be read.
    Unreadable(ExactBytes),
}

/// A producer input role as named by the selected producer delivery.
///
/// The producer owns the role vocabulary. This consumer retains its exact
/// spelling and only owns duplicate detection in one index; it does not create
/// a parallel role enumeration.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Hash)]
pub struct InputRole(ExactBytes);

impl InputRole {
    /// Retain the producer role spelling exactly.
    #[must_use]
    pub fn new(bytes: &[u8]) -> Self {
        Self(ExactBytes::new(bytes))
    }

    /// Exact producer role bytes.
    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        self.0.as_bytes()
    }
}

/// An exact input-role member.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IndexedInput {
    role: InputRole,
    member: IndexedMember,
}

impl IndexedInput {
    /// Associate one producer input role with its observed member.
    #[must_use]
    pub const fn new(role: InputRole, member: IndexedMember) -> Self {
        Self { role, member }
    }

    /// The exact producer role.
    #[must_use]
    pub const fn role(&self) -> &InputRole {
        &self.role
    }

    /// The retained member.
    #[must_use]
    pub const fn member(&self) -> &IndexedMember {
        &self.member
    }
}

/// One producer temporal binding retained under its owner, declaration and path.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TemporalBindingRecord {
    owner: ExactBytes,
    declaration: u32,
    path: ExactBytes,
}

impl TemporalBindingRecord {
    /// Retain the producer's binding record; its order is validated by [`DeliveryIndex`].
    #[must_use]
    pub fn new(owner: &[u8], declaration: u32, path: &[u8]) -> Self {
        Self {
            owner: ExactBytes::new(owner),
            declaration,
            path: ExactBytes::new(path),
        }
    }

    /// Exact owner spelling.
    #[must_use]
    pub const fn owner(&self) -> &ExactBytes {
        &self.owner
    }
    /// Producer declaration handle.
    #[must_use]
    pub const fn declaration(&self) -> u32 {
        self.declaration
    }
    /// Retained binding path.
    #[must_use]
    pub const fn path(&self) -> &ExactBytes {
        &self.path
    }
}

/// A compiled-protocol reader dimension, named by the delivery index.
///
/// Protocol owns this closed vocabulary; it is the index's key for a
/// reader-side limit, not a re-export of the reader's accounting type.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CompiledDimension {
    /// Compiled payload bytes.
    PayloadBytes,
    /// Output bytes.
    OutputBytes,
    /// Source bytes.
    SourceBytes,
    /// Content bytes.
    ContentBytes,
    /// Source count.
    Sources,
    /// Dependency count.
    Dependencies,
    /// Definition count.
    Definitions,
    /// Model count.
    Models,
    /// Declaration count.
    Declarations,
    /// Entry count.
    Entries,
    /// Reference count.
    References,
    /// Byte work.
    ByteWork,
    /// Nesting depth.
    Depth,
}

/// A dimension owned either by Protocol or the compiled-protocol reader.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LimitDimension {
    /// A caller-facing Protocol dimension.
    Protocol(BudgetDimension),
    /// A compiled-protocol reader dimension.
    CompiledProtocol(CompiledDimension),
}

impl LimitDimension {
    const fn order_key(self) -> (u8, u8) {
        match self {
            Self::Protocol(value) => (0, protocol_dimension_order(value)),
            Self::CompiledProtocol(value) => (1, compiled_dimension_order(value)),
        }
    }
}

const fn protocol_dimension_order(value: BudgetDimension) -> u8 {
    match value {
        BudgetDimension::InputByte => 0,
        BudgetDimension::Definition => 1,
        BudgetDimension::Node => 2,
        BudgetDimension::Edge => 3,
        BudgetDimension::Role => 4,
        BudgetDimension::Message => 5,
        BudgetDimension::Branch => 6,
        BudgetDimension::Iteration => 7,
        BudgetDimension::ActiveInstance => 8,
        BudgetDimension::Observation => 9,
        BudgetDimension::Lineage => 10,
        BudgetDimension::ResultItem => 11,
        BudgetDimension::DerivedArtifact => 12,
        BudgetDimension::OutputByte => 13,
        BudgetDimension::Operation => 14,
        BudgetDimension::AccountedAllocation => 15,
        BudgetDimension::ElapsedTime => 16,
        BudgetDimension::ResolutionEntryCount => 17,
        BudgetDimension::ResolutionBytesDigested => 18,
        BudgetDimension::ResolutionDeadline => 19,
    }
}

const fn compiled_dimension_order(value: CompiledDimension) -> u8 {
    match value {
        CompiledDimension::PayloadBytes => 0,
        CompiledDimension::OutputBytes => 1,
        CompiledDimension::SourceBytes => 2,
        CompiledDimension::ContentBytes => 3,
        CompiledDimension::Sources => 4,
        CompiledDimension::Dependencies => 5,
        CompiledDimension::Definitions => 6,
        CompiledDimension::Models => 7,
        CompiledDimension::Declarations => 8,
        CompiledDimension::Entries => 9,
        CompiledDimension::References => 10,
        CompiledDimension::ByteWork => 11,
        CompiledDimension::Depth => 12,
    }
}

impl Ord for LimitDimension {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.order_key().cmp(&other.order_key())
    }
}

impl PartialOrd for LimitDimension {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Hash for LimitDimension {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.order_key().hash(state);
    }
}

/// One caller-supplied limit retained by exact accountant key and dimension.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IndexedLimit {
    accountant: EntryKey,
    dimension: LimitDimension,
    bound: u64,
}

impl IndexedLimit {
    /// Construct one exact typed limit entry.
    #[must_use]
    pub const fn new(accountant: EntryKey, dimension: LimitDimension, bound: u64) -> Self {
        Self {
            accountant,
            dimension,
            bound,
        }
    }
    /// Exact accountant owner and contract version.
    #[must_use]
    pub const fn accountant(&self) -> &EntryKey {
        &self.accountant
    }
    /// The non-substitutable owner vocabulary and member.
    #[must_use]
    pub const fn dimension(&self) -> LimitDimension {
        self.dimension
    }
    /// Caller-provided bound.
    #[must_use]
    pub const fn bound(&self) -> u64 {
        self.bound
    }
}

/// One producer mutation fixture indexed by scenario.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IndexedMutation {
    fixture: ExactBytes,
    scenario: ExactBytes,
    path: ExactBytes,
}

impl IndexedMutation {
    /// Retain a producer-provided fixture record exactly.
    #[must_use]
    pub fn new(fixture: &[u8], scenario: &[u8], path: &[u8]) -> Self {
        Self {
            fixture: ExactBytes::new(fixture),
            scenario: ExactBytes::new(scenario),
            path: ExactBytes::new(path),
        }
    }
    /// Fixture name.
    #[must_use]
    pub const fn fixture(&self) -> &ExactBytes {
        &self.fixture
    }
    /// Scenario key.
    #[must_use]
    pub const fn scenario(&self) -> &ExactBytes {
        &self.scenario
    }
    /// Fixture member path.
    #[must_use]
    pub const fn path(&self) -> &ExactBytes {
        &self.path
    }
}

/// The exact non-identity state retained for one index.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DeliveryIndexContents {
    offer: IndexedMember,
    artifact_reference: ExactBytes,
    inputs: Vec<IndexedInput>,
    temporal_profile: ExactBytes,
    temporal_bindings: Vec<TemporalBindingRecord>,
    limits: Vec<IndexedLimit>,
    mutations: Vec<IndexedMutation>,
}

/// An adapter-reported index-read state. It is input, never authority for a
/// version lock or a producer selection.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum IndexRead {
    /// The selected delivery root did not exist.
    MissingRoot,
    /// The root existed but carried no consumer index.
    MissingIndex,
    /// The index could not be deserialized or otherwise read.
    Unreadable(ExactBytes),
    /// Complete contents read by an outer adapter.
    Present(DeliveryIndexContents),
}

impl DeliveryIndexContents {
    /// Build the retained delivery state before validation.
    #[must_use]
    pub fn new(
        offer: IndexedMember,
        artifact_reference: &[u8],
        inputs: Vec<IndexedInput>,
        temporal_profile: &[u8],
        temporal_bindings: Vec<TemporalBindingRecord>,
        limits: Vec<IndexedLimit>,
        mutations: Vec<IndexedMutation>,
    ) -> Self {
        Self {
            offer,
            artifact_reference: ExactBytes::new(artifact_reference),
            inputs,
            temporal_profile: ExactBytes::new(temporal_profile),
            temporal_bindings,
            limits,
            mutations,
        }
    }
}

/// A distinct failure while assembling consumer-owned delivery evidence.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DeliveryIndexError {
    /// The selected delivery root was empty and cannot identify an index.
    EmptyDeliveryRoot,
    /// The producer revision was empty and cannot identify an immutable delivery.
    EmptyProducerRevision,
    /// The selected root was not available.
    MissingRoot(DeliveryRoot),
    /// No index was available under the selected root.
    MissingIndex(DeliveryRoot),
    /// The index record could not be read.
    UnreadableIndex {
        /// The delivery root the unreadable index was selected under.
        root: DeliveryRoot,
        /// The reason the index record could not be read.
        reason: ExactBytes,
    },
    /// A named member was absent.
    MissingMember(ExactBytes),
    /// A named member could not be read.
    UnreadableMember {
        /// The path of the member that could not be read.
        path: ExactBytes,
        /// The reason the member could not be read.
        reason: ExactBytes,
    },
    /// A requested input role has no index entry.
    UnindexedInputRole(InputRole),
    /// Two input entries name the same role.
    RepeatedInputRole(InputRole),
    /// Two temporal records name one declaration.
    RepeatedDeclaration(u32),
    /// A temporal declaration expected by the selected producer package is absent.
    MissingDeclaration(u32),
    /// An indexed temporal record names no selected temporal declaration.
    SurplusDeclaration(u32),
    /// Temporal declarations are not in producer-required ascending order.
    UnsortedBindings {
        /// The declaration number that preceded the out-of-order entry.
        previous: u32,
        /// The out-of-order declaration number found after `previous`.
        declaration: u32,
    },
    /// More than one mutation fixture names one scenario.
    AmbiguousMutation(ExactBytes),
    /// More than one limit names one exact accountant and typed dimension.
    RepeatedLimit {
        /// The accountant key that more than one limit named.
        accountant: EntryKey,
        /// The typed dimension that more than one limit named for `accountant`.
        dimension: LimitDimension,
    },
    /// Retained byte lengths cannot be represented in the request counter.
    RetainedBytesOverflow,
}

impl fmt::Display for DeliveryIndexError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "delivery index assembly failed: {self:?}")
    }
}
impl std::error::Error for DeliveryIndexError {}

/// A validated consumer-owned delivery record.
///
/// Equality is solely [`DeliveryIndexKey`], intentionally excluding every
/// retained state field.
#[derive(Clone, Debug)]
pub struct DeliveryIndex {
    key: DeliveryIndexKey,
    contents: DeliveryIndexContents,
}

impl PartialEq for DeliveryIndex {
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key
    }
}
impl Eq for DeliveryIndex {}

impl DeliveryIndex {
    /// Convert a selected root's adapter-reported state into a validated index.
    ///
    /// No filesystem, transport, manifest, resolver, or version-lock value is
    /// accepted here. Consequently this conversion cannot affect resolution.
    pub fn observed(
        key: DeliveryIndexKey,
        read: IndexRead,
        resources: &mut ResourceAccountant<'_>,
    ) -> OperationOutcome<Self, DeliveryIndexError> {
        if let Err(error) = validate_key(&key) {
            return OperationOutcome::DomainError(error);
        }
        match read {
            IndexRead::MissingRoot => OperationOutcome::DomainError(
                DeliveryIndexError::MissingRoot(key.delivery_root().clone()),
            ),
            IndexRead::MissingIndex => OperationOutcome::DomainError(
                DeliveryIndexError::MissingIndex(key.delivery_root().clone()),
            ),
            IndexRead::Unreadable(reason) => {
                OperationOutcome::DomainError(DeliveryIndexError::UnreadableIndex {
                    root: key.delivery_root().clone(),
                    reason,
                })
            }
            IndexRead::Present(contents) => Self::new_bounded(key, contents, resources),
        }
    }

    /// Validate and retain one index under a caller-owned resource accountant.
    ///
    /// Domain validation happens without mutating accounting. The exact byte
    /// count and final index allocation are then charged as one transaction
    /// before the completed consumer record is returned. There is no public
    /// unbounded final constructor.
    pub fn new_bounded(
        key: DeliveryIndexKey,
        contents: DeliveryIndexContents,
        resources: &mut ResourceAccountant<'_>,
    ) -> OperationOutcome<Self, DeliveryIndexError> {
        let candidate = match Self::validated(key, contents) {
            Ok(value) => value,
            Err(error) => return OperationOutcome::DomainError(error),
        };
        let bytes = match retained_bytes(&candidate.key, &candidate.contents) {
            Ok(value) => value,
            Err(error) => return OperationOutcome::DomainError(error),
        };
        if let Err(error) =
            resources.charge_all(&[(INPUT_SOURCE.id(), bytes), (ALLOCATION_SOURCE.id(), 1)])
        {
            return OperationOutcome::from_charge(error);
        }
        OperationOutcome::Completed(candidate)
    }

    fn validated(
        key: DeliveryIndexKey,
        contents: DeliveryIndexContents,
    ) -> Result<Self, DeliveryIndexError> {
        validate_key(&key)?;
        let mut input_roles = BTreeSet::new();
        for input in &contents.inputs {
            if !input_roles.insert(input.role.clone()) {
                return Err(DeliveryIndexError::RepeatedInputRole(input.role.clone()));
            }
        }
        let mut prior = None;
        for binding in &contents.temporal_bindings {
            if let Some(previous) = prior {
                if binding.declaration == previous {
                    return Err(DeliveryIndexError::RepeatedDeclaration(binding.declaration));
                }
                if binding.declaration < previous {
                    return Err(DeliveryIndexError::UnsortedBindings {
                        previous,
                        declaration: binding.declaration,
                    });
                }
            }
            prior = Some(binding.declaration);
        }
        let mut mutation_scenarios = BTreeSet::new();
        for mutation in &contents.mutations {
            if !mutation_scenarios.insert(mutation.scenario.clone()) {
                return Err(DeliveryIndexError::AmbiguousMutation(
                    mutation.scenario.clone(),
                ));
            }
        }
        let mut limit_keys = BTreeSet::new();
        for limit in &contents.limits {
            let limit_key = (limit.accountant.clone(), limit.dimension);
            if !limit_keys.insert(limit_key) {
                return Err(DeliveryIndexError::RepeatedLimit {
                    accountant: limit.accountant.clone(),
                    dimension: limit.dimension,
                });
            }
        }
        Ok(Self { key, contents })
    }

    /// Consumer-authored index identity.
    #[must_use]
    pub const fn key(&self) -> &DeliveryIndexKey {
        &self.key
    }
    /// Exact offered artifact member.
    #[must_use]
    pub const fn offer(&self) -> &IndexedMember {
        &self.contents.offer
    }
    /// Producer's verbatim external artifact reference.
    #[must_use]
    pub const fn artifact_reference(&self) -> &ExactBytes {
        &self.contents.artifact_reference
    }
    /// Indexed input members.
    #[must_use]
    pub fn inputs(&self) -> &[IndexedInput] {
        &self.contents.inputs
    }
    /// Look up a role without defaulting an absent entry.
    pub fn input(&self, role: &InputRole) -> Result<&IndexedMember, DeliveryIndexError> {
        self.contents
            .inputs
            .iter()
            .find(|input| input.role == *role)
            .map(IndexedInput::member)
            .ok_or_else(|| DeliveryIndexError::UnindexedInputRole(role.clone()))
    }
    /// Indexed temporal profile path.
    #[must_use]
    pub const fn temporal_profile(&self) -> &ExactBytes {
        &self.contents.temporal_profile
    }
    /// Producer-ordered temporal binding records.
    #[must_use]
    pub fn temporal_bindings(&self) -> &[TemporalBindingRecord] {
        &self.contents.temporal_bindings
    }

    /// Check the producer's exact cardinality rule against selected temporal declarations.
    ///
    /// This method deliberately compares only declaration handles: it does not
    /// invent a stronger binding identity or an order beyond the producer's
    /// already-validated ascending declaration order.
    pub fn validate_temporal_declarations(
        &self,
        selected: &[u32],
    ) -> Result<(), DeliveryIndexError> {
        let actual: BTreeSet<_> = self
            .contents
            .temporal_bindings
            .iter()
            .map(TemporalBindingRecord::declaration)
            .collect();
        let mut expected = BTreeSet::new();
        for declaration in selected {
            if !expected.insert(*declaration) {
                return Err(DeliveryIndexError::RepeatedDeclaration(*declaration));
            }
        }
        if let Some(declaration) = expected.difference(&actual).next() {
            return Err(DeliveryIndexError::MissingDeclaration(*declaration));
        }
        if let Some(declaration) = actual.difference(&expected).next() {
            return Err(DeliveryIndexError::SurplusDeclaration(*declaration));
        }
        Ok(())
    }
    /// Deterministically ordered, non-causal limits.
    #[must_use]
    pub fn limits(&self) -> Vec<&IndexedLimit> {
        let mut ordered: Vec<_> = self.contents.limits.iter().collect();
        ordered.sort_by_key(|limit| (limit.accountant.clone(), limit.dimension));
        ordered
    }
    /// One-at-most mutation fixture per scenario.
    #[must_use]
    pub fn mutations(&self) -> &[IndexedMutation] {
        &self.contents.mutations
    }
}

fn validate_key(key: &DeliveryIndexKey) -> Result<(), DeliveryIndexError> {
    if key.delivery_root().as_bytes().is_empty() {
        return Err(DeliveryIndexError::EmptyDeliveryRoot);
    }
    if key.producer_revision().as_bytes().is_empty() {
        return Err(DeliveryIndexError::EmptyProducerRevision);
    }
    Ok(())
}

fn retained_bytes(
    key: &DeliveryIndexKey,
    contents: &DeliveryIndexContents,
) -> Result<usize, DeliveryIndexError> {
    let mut total = key.delivery_root().as_bytes().len();
    for bytes in [
        key.producer_revision().as_bytes(),
        contents.offer.path.as_bytes(),
    ] {
        total = total
            .checked_add(bytes.len())
            .ok_or(DeliveryIndexError::RetainedBytesOverflow)?;
    }
    for bytes in [
        contents.offer.bytes.as_bytes(),
        contents.artifact_reference.as_bytes(),
        contents.temporal_profile.as_bytes(),
    ] {
        total = total
            .checked_add(bytes.len())
            .ok_or(DeliveryIndexError::RetainedBytesOverflow)?;
    }
    for input in &contents.inputs {
        for bytes in [
            input.role.as_bytes(),
            input.member.path.as_bytes(),
            input.member.bytes.as_bytes(),
        ] {
            total = total
                .checked_add(bytes.len())
                .ok_or(DeliveryIndexError::RetainedBytesOverflow)?;
        }
    }
    for binding in &contents.temporal_bindings {
        for bytes in [binding.owner.as_bytes(), binding.path.as_bytes()] {
            total = total
                .checked_add(bytes.len())
                .ok_or(DeliveryIndexError::RetainedBytesOverflow)?;
        }
    }
    for limit in &contents.limits {
        for bytes in [
            limit.accountant.owner().as_bytes(),
            limit.accountant.contract_version().as_bytes(),
        ] {
            total = total
                .checked_add(bytes.len())
                .ok_or(DeliveryIndexError::RetainedBytesOverflow)?;
        }
    }
    for mutation in &contents.mutations {
        for bytes in [
            mutation.fixture.as_bytes(),
            mutation.scenario.as_bytes(),
            mutation.path.as_bytes(),
        ] {
            total = total
                .checked_add(bytes.len())
                .ok_or(DeliveryIndexError::RetainedBytesOverflow)?;
        }
    }
    Ok(total)
}
