// SPDX-License-Identifier: AGPL-3.0-or-later
//! NFR-005 scenario evidence, composed as a value for external handoff.
//!
//! This module has no persistence or read-back operation. It validates and
//! returns one value; Quoin and Engineering Assurance own everything after the
//! handoff boundary.

use super::{EntryKey, ExactBytes, ManifestEntry, ResolutionRefusal, ResolvedManifest};

/// The two integrations NFR-005 places in scope.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Hash)]
pub enum IntegrationId {
    /// Observation and temporal facts into the protocol evaluator.
    It002,
    /// Assurance and E02 result publication.
    It003,
}

/// Identity of one owed integration scenario.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Hash)]
pub struct ScenarioKey {
    integration: IntegrationId,
    scenario: ExactBytes,
}

impl ScenarioKey {
    /// Build a non-empty scenario identity.
    ///
    /// # Errors
    ///
    /// [`EvidenceError::EmptyScenario`] when `scenario` is empty.
    pub fn new(integration: IntegrationId, scenario: &[u8]) -> Result<Self, EvidenceError> {
        if scenario.is_empty() {
            return Err(EvidenceError::EmptyScenario);
        }
        Ok(Self {
            integration,
            scenario: ExactBytes::new(scenario),
        })
    }

    /// Owning integration.
    #[must_use]
    pub const fn integration(&self) -> IntegrationId {
        self.integration
    }

    /// Exact scenario identity.
    #[must_use]
    pub const fn scenario(&self) -> &ExactBytes {
        &self.scenario
    }
}

/// Resolution state retained by a scenario.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ResolutionEvidence {
    /// The full set of resolved entries. Verified bytes remain on the live
    /// resolver result and are not duplicated into the handoff record.
    Complete(Vec<ManifestEntry>),
    /// Entries resolved before a refusal, plus the entry that caused it.
    Partial {
        /// Entries resolved before the causing entry.
        resolved_before: Vec<ManifestEntry>,
        /// Exact key of the causing entry.
        causing_entry: EntryKey,
    },
}

impl ResolutionEvidence {
    /// Snapshot a completed resolution state.
    #[must_use]
    pub fn complete(resolved: &ResolvedManifest) -> Self {
        Self::Complete(resolved.entries().to_vec())
    }

    /// Snapshot an entry-caused refusal's partial state.
    ///
    /// # Errors
    ///
    /// [`EvidenceError::RefusalHasNoEntryCause`] for a delivery-surface
    /// refusal, because FR-017 admits only a partial state plus causing entry.
    pub fn from_refusal(refusal: &ResolutionRefusal) -> Result<Self, EvidenceError> {
        let Some(causing_entry) = refusal.key() else {
            return Err(EvidenceError::RefusalHasNoEntryCause);
        };
        Ok(Self::Partial {
            resolved_before: refusal.resolved_before().to_vec(),
            causing_entry: causing_entry.clone(),
        })
    }
}

/// One producer value and the exact owner copied from its lock entry.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValueOwner {
    value: ExactBytes,
    owner: ExactBytes,
}

impl ValueOwner {
    /// Build a per-value owner record.
    ///
    /// # Errors
    ///
    /// [`EvidenceError::EmptyValue`] or [`EvidenceError::EmptyOwner`] when a
    /// required component is absent.
    pub fn new(value: &[u8], owner: &[u8]) -> Result<Self, EvidenceError> {
        if value.is_empty() {
            return Err(EvidenceError::EmptyValue);
        }
        if owner.is_empty() {
            return Err(EvidenceError::EmptyOwner);
        }
        Ok(Self {
            value: ExactBytes::new(value),
            owner: ExactBytes::new(owner),
        })
    }

    /// Exact producer value identity.
    #[must_use]
    pub const fn value(&self) -> &ExactBytes {
        &self.value
    }

    /// Exact producer owner identity.
    #[must_use]
    pub const fn owner(&self) -> &ExactBytes {
        &self.owner
    }
}

/// Every caller-facing budget dimension FR-016 declares.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Hash)]
pub enum BudgetDimension {
    InputByte,
    Definition,
    Node,
    Edge,
    Role,
    Message,
    Branch,
    Iteration,
    ActiveInstance,
    Observation,
    Lineage,
    ResultItem,
    DerivedArtifact,
    OutputByte,
    Operation,
    AccountedAllocation,
    ElapsedTime,
    ResolutionEntryCount,
    ResolutionBytesDigested,
    ResolutionDeadline,
}

/// Granted and consumed figures for one dimension.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BudgetFigure {
    dimension: BudgetDimension,
    granted: u64,
    consumed: u64,
}

impl BudgetFigure {
    /// Record one dimension independently of every other.
    #[must_use]
    pub const fn new(dimension: BudgetDimension, granted: u64, consumed: u64) -> Self {
        Self {
            dimension,
            granted,
            consumed,
        }
    }

    /// Dimension identity.
    #[must_use]
    pub const fn dimension(self) -> BudgetDimension {
        self.dimension
    }

    /// Caller grant.
    #[must_use]
    pub const fn granted(self) -> u64 {
        self.granted
    }

    /// Measured consumption.
    #[must_use]
    pub const fn consumed(self) -> u64 {
        self.consumed
    }

    /// Elapsed figures are retained but excluded from reproducible comparison.
    #[must_use]
    pub const fn is_reproducible(self) -> bool {
        !matches!(
            self.dimension,
            BudgetDimension::ElapsedTime | BudgetDimension::ResolutionDeadline
        )
    }
}

/// Typed identity of the outcome a scenario reached.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EvidenceOutcome {
    /// Exact canonical protocol-result identity.
    CanonicalResult(ExactBytes),
    /// Exact stable typed-refusal identity.
    Refusal(ExactBytes),
    /// Exact resource-incomplete dimension identity.
    ResourceIncomplete(BudgetDimension),
}

/// Exact acceptance criterion carried by this evidence record.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TraceBinding {
    criterion: ExactBytes,
}

impl TraceBinding {
    /// Retain a non-empty criterion identity.
    ///
    /// The test that constructs this value carries the corresponding bare
    /// `ix_trace_rs::trace` attribute; this runtime value does not pretend to
    /// replace that compiler-checked binding.
    ///
    /// # Errors
    ///
    /// [`EvidenceError::EmptyTraceBinding`] when the criterion is absent.
    pub fn new(criterion: &[u8]) -> Result<Self, EvidenceError> {
        if criterion.is_empty() {
            return Err(EvidenceError::EmptyTraceBinding);
        }
        Ok(Self {
            criterion: ExactBytes::new(criterion),
        })
    }

    /// Exact criterion identity.
    #[must_use]
    pub const fn criterion(&self) -> &ExactBytes {
        &self.criterion
    }
}

/// An invalid or incomplete evidence record.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EvidenceError {
    EmptyScenario,
    EmptyValue,
    EmptyOwner,
    EmptyOutcomeIdentity,
    EmptyTraceBinding,
    MissingRefusalAxis,
    UnexpectedRefusalAxis,
    DuplicateValue(ExactBytes),
    DuplicateBudgetDimension(BudgetDimension),
    RefusalHasNoEntryCause,
}

/// One scenario's complete value at the external handoff boundary.
#[derive(Clone, Debug)]
pub struct ScenarioEvidenceRecord {
    key: ScenarioKey,
    resolution: ResolutionEvidence,
    value_owners: Vec<ValueOwner>,
    outcome: EvidenceOutcome,
    refusal_axis: Option<ExactBytes>,
    budget: Vec<BudgetFigure>,
    trace: TraceBinding,
}

impl ScenarioEvidenceRecord {
    /// Compose a complete record without retaining it locally.
    ///
    /// # Errors
    ///
    /// [`EvidenceError`] when a required outcome component is absent or a
    /// per-value/per-dimension identity is duplicated.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        key: ScenarioKey,
        resolution: ResolutionEvidence,
        value_owners: Vec<ValueOwner>,
        outcome: EvidenceOutcome,
        refusal_axis: Option<ExactBytes>,
        budget: Vec<BudgetFigure>,
        trace: TraceBinding,
    ) -> Result<Self, EvidenceError> {
        match &outcome {
            EvidenceOutcome::CanonicalResult(identity) | EvidenceOutcome::Refusal(identity)
                if identity.as_bytes().is_empty() =>
            {
                return Err(EvidenceError::EmptyOutcomeIdentity);
            }
            EvidenceOutcome::Refusal(_) => {
                if refusal_axis
                    .as_ref()
                    .is_none_or(|axis| axis.as_bytes().is_empty())
                {
                    return Err(EvidenceError::MissingRefusalAxis);
                }
            }
            EvidenceOutcome::CanonicalResult(_) | EvidenceOutcome::ResourceIncomplete(_) => {
                if refusal_axis.is_some() {
                    return Err(EvidenceError::UnexpectedRefusalAxis);
                }
            }
        }

        for (index, owner) in value_owners.iter().enumerate() {
            if value_owners[index + 1..]
                .iter()
                .any(|other| other.value == owner.value)
            {
                return Err(EvidenceError::DuplicateValue(owner.value.clone()));
            }
        }
        for (index, figure) in budget.iter().enumerate() {
            if budget[index + 1..]
                .iter()
                .any(|other| other.dimension == figure.dimension)
            {
                return Err(EvidenceError::DuplicateBudgetDimension(figure.dimension));
            }
        }

        Ok(Self {
            key,
            resolution,
            value_owners,
            outcome,
            refusal_axis,
            budget,
            trace,
        })
    }

    pub const fn key(&self) -> &ScenarioKey {
        &self.key
    }

    pub const fn resolution(&self) -> &ResolutionEvidence {
        &self.resolution
    }

    pub fn value_owners(&self) -> &[ValueOwner] {
        &self.value_owners
    }

    pub const fn outcome(&self) -> &EvidenceOutcome {
        &self.outcome
    }

    pub const fn refusal_axis(&self) -> Option<&ExactBytes> {
        self.refusal_axis.as_ref()
    }

    pub fn budget(&self) -> &[BudgetFigure] {
        &self.budget
    }

    pub const fn trace(&self) -> &TraceBinding {
        &self.trace
    }
}

impl PartialEq for ScenarioEvidenceRecord {
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key
    }
}

impl Eq for ScenarioEvidenceRecord {}

/// Two distinct refusal axes retained the same typed outcome.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RefusalAxisCollision {
    integration: IntegrationId,
    first_axis: ExactBytes,
    second_axis: ExactBytes,
    outcome: ExactBytes,
}

impl RefusalAxisCollision {
    pub const fn integration(&self) -> IntegrationId {
        self.integration
    }

    pub const fn first_axis(&self) -> &ExactBytes {
        &self.first_axis
    }

    pub const fn second_axis(&self) -> &ExactBytes {
        &self.second_axis
    }

    pub const fn outcome(&self) -> &ExactBytes {
        &self.outcome
    }
}

/// Enforce refusal-axis distinctness across a whole integration.
///
/// Two scenarios on the same axis may share an outcome. Two different axes in
/// the same integration may not. Records from different integrations are
/// compared independently.
pub fn validate_refusal_axis_distinctness(
    records: &[ScenarioEvidenceRecord],
) -> Result<(), RefusalAxisCollision> {
    for (index, first) in records.iter().enumerate() {
        let (Some(first_axis), EvidenceOutcome::Refusal(first_outcome)) =
            (&first.refusal_axis, &first.outcome)
        else {
            continue;
        };
        for second in &records[index + 1..] {
            let (Some(second_axis), EvidenceOutcome::Refusal(second_outcome)) =
                (&second.refusal_axis, &second.outcome)
            else {
                continue;
            };
            if first.key.integration == second.key.integration
                && first_axis != second_axis
                && first_outcome == second_outcome
            {
                return Err(RefusalAxisCollision {
                    integration: first.key.integration,
                    first_axis: first_axis.clone(),
                    second_axis: second_axis.clone(),
                    outcome: first_outcome.clone(),
                });
            }
        }
    }
    Ok(())
}
