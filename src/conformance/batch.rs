// SPDX-License-Identifier: AGPL-3.0-or-later
//! Bounded batch conformance over exact QObs views and authored obligations.

#![deny(clippy::wildcard_enum_match_arm)]
#![allow(
    dead_code,
    reason = "the crate-private standalone evaluator is exercised by TC-004 unit tests; production enters through activation-derived frontends"
)]

use std::collections::{BTreeMap, BTreeSet, VecDeque};

use quire_observation::authority::{completeness, observation, OpenClosed};

use crate::assessment::{AxisInputs, GlobalConformanceInput};
use crate::bounds::inventory::{
    BudgetDimension, ChargeError, OperationOutcome, ResourceAccountant, SourceSite,
};
use crate::bounds::{SiteId, SiteKind, UnitLabel};
use crate::closure::{AssessmentExecution, GlobalConformanceClosure};

use super::{ErrorCode, Refusal, Result};

pub(crate) const OBSERVATION_SOURCE: SourceSite = SourceSite::new(
    SiteId::new("conformance/observation/consume"),
    "conformance::batch::assess",
    BudgetDimension::Observation,
    SiteKind::Iteration,
);
pub(crate) const OPERATION_SOURCE: SourceSite = SourceSite::new(
    SiteId::new("conformance/obligation/assess"),
    "conformance::batch::assess",
    BudgetDimension::Operation,
    SiteKind::Iteration,
);
pub(crate) const PREDECESSOR_SOURCE: SourceSite = SourceSite::new(
    SiteId::new("conformance/predecessor/traverse"),
    "conformance::batch::assess",
    BudgetDimension::Operation,
    SiteKind::Traversal,
);
pub(crate) const RETAINED_BYTE_SOURCE: SourceSite = SourceSite::new(
    SiteId::new("conformance/owner-byte/retain"),
    "conformance::batch::assess",
    BudgetDimension::InputByte,
    SiteKind::Retention,
);
pub(crate) const ADVERSE_SOURCE: SourceSite = SourceSite::new(
    SiteId::new("conformance/adverse/retain"),
    "conformance::batch::assess",
    BudgetDimension::ResultItem,
    SiteKind::Retention,
);
pub(crate) const ALLOCATION_SOURCE: SourceSite = SourceSite::new(
    SiteId::new("conformance/report/allocate"),
    "conformance::batch::assess",
    BudgetDimension::AccountedAllocation,
    SiteKind::Allocation,
);

/// Immutable owner maxima. Callers may lower but never raise them.
pub const OWNER_MAX: Limits = Limits {
    max_obligations: 10_000,
    max_bindings: 10_000,
    max_predecessors: 100_000,
    max_adverse: 100_000,
    max_retained_bytes: 8 * 1024 * 1024,
};

/// Independent ceilings for one conformance request.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Limits {
    /// Maximum authored obligations.
    pub max_obligations: usize,
    /// Maximum admitted observation bindings.
    pub max_bindings: usize,
    /// Maximum authored predecessor edges.
    pub max_predecessors: usize,
    /// Maximum adverse reports.
    pub max_adverse: usize,
    /// Maximum exact owner bytes retained in the report.
    pub max_retained_bytes: usize,
}

impl Limits {
    /// Immutable owner maxima.
    #[must_use]
    pub const fn owner_max() -> Self {
        OWNER_MAX
    }

    /// Clamp caller ceilings to immutable owner maxima.
    #[must_use]
    pub const fn effective(self) -> Self {
        Self {
            max_obligations: min(self.max_obligations, OWNER_MAX.max_obligations),
            max_bindings: min(self.max_bindings, OWNER_MAX.max_bindings),
            max_predecessors: min(self.max_predecessors, OWNER_MAX.max_predecessors),
            max_adverse: min(self.max_adverse, OWNER_MAX.max_adverse),
            max_retained_bytes: min(self.max_retained_bytes, OWNER_MAX.max_retained_bytes),
        }
    }
}

const fn min(left: usize, right: usize) -> usize {
    if left < right {
        left
    } else {
        right
    }
}

impl Default for Limits {
    fn default() -> Self {
        OWNER_MAX
    }
}

/// Work observed by one bounded assessment.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Usage {
    /// Obligations visited.
    pub obligations: usize,
    /// Observation bindings visited.
    pub bindings: usize,
    /// Authored predecessor edges visited.
    pub predecessors: usize,
    /// Adverse reports produced.
    pub adverse: usize,
    /// Exact owner bytes retained.
    pub retained_bytes: usize,
}

/// Exact, display-independent protocol-obligation identity.
#[derive(Clone, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct ObligationId(Box<str>);

impl ObligationId {
    /// Retain an offered exact identity. [`Model::new`] refuses an empty value.
    #[must_use]
    pub fn retained(value: impl Into<Box<str>>) -> Self {
        Self(value.into())
    }

    /// Exact retained identity.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// One explicitly selected protocol obligation and its authored predecessors.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ObservationObligation {
    /// Exact obligation identity.
    pub identity: ObligationId,
    /// Exact opaque producer-local subject-kind bytes selected by the obligation.
    ///
    /// QObs owns this vocabulary. It is deliberately bytes rather than text so
    /// a runtime subject cannot be normalized through an assumed UTF-8 boundary.
    pub subject_kind: Box<[u8]>,
    /// Exact opaque producer-local subject-identity bytes selected by the obligation.
    ///
    /// The authority-qualified runtime-reference contract owns the identity
    /// representation; this consumer retains and compares its bytes exactly.
    pub subject_identity: Box<[u8]>,
    /// Exact workflow-instance identity.
    pub workflow_identity: Box<str>,
    /// Exact business-effect identity, when the obligation owns an effect.
    pub effect_identity: Option<Box<str>>,
    /// Authored predecessor obligations. Their order is retained and not sorted.
    pub predecessors: Vec<ObligationId>,
    /// Whether an explicit valuation is required.
    pub valuation_required: bool,
}

/// Complete explicit assessment model.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Model {
    protocol_identity: Box<str>,
    obligations: Vec<ObservationObligation>,
}

impl Model {
    /// Validate and retain one complete finite obligation model.
    pub fn new(
        protocol_identity: impl Into<Box<str>>,
        obligations: Vec<ObservationObligation>,
        limits: Limits,
    ) -> Result<Self> {
        let protocol_identity = protocol_identity.into();
        let effective = limits.effective();
        let predecessors = match obligations.iter().try_fold(0usize, |sum, obligation| {
            sum.checked_add(obligation.predecessors.len())
        }) {
            Some(predecessors) => predecessors,
            None => {
                return Err(Refusal::new(
                    ErrorCode::ResourceIncomplete,
                    "obligation predecessor population cannot be represented",
                    None,
                    Usage {
                        obligations: obligations.len(),
                        predecessors: usize::MAX,
                        ..Usage::default()
                    },
                    protocol_identity.as_bytes(),
                ))
            }
        };
        let usage = Usage {
            obligations: obligations.len(),
            predecessors,
            ..Usage::default()
        };
        if obligations.len() > effective.max_obligations
            || predecessors > effective.max_predecessors
        {
            return Err(Refusal::new(
                ErrorCode::ResourceIncomplete,
                "obligation model exceeds an effective ceiling",
                None,
                usage,
                protocol_identity.as_bytes(),
            ));
        }
        if protocol_identity.is_empty() || obligations.is_empty() {
            return Err(invalid_model(
                "protocol identity and obligation population must be explicit",
                None,
                usage,
                protocol_identity.as_bytes(),
            ));
        }
        let by_id: BTreeMap<_, _> = obligations
            .iter()
            .enumerate()
            .map(|(index, obligation)| (obligation.identity.clone(), index))
            .collect();
        if by_id.len() != obligations.len()
            || obligations.iter().any(|obligation| {
                obligation.identity.as_str().is_empty()
                    || obligation.subject_kind.is_empty()
                    || obligation.subject_identity.is_empty()
                    || obligation.workflow_identity.is_empty()
                    || obligation
                        .effect_identity
                        .as_deref()
                        .is_some_and(str::is_empty)
                    || obligation
                        .predecessors
                        .iter()
                        .any(|predecessor| !by_id.contains_key(predecessor))
                    || !all_distinct(&obligation.predecessors)
            })
        {
            return Err(invalid_model(
                "obligation identities, references, subjects, workflows, and effects must be exact",
                None,
                usage,
                protocol_identity.as_bytes(),
            ));
        }
        if !acyclic(&obligations, &by_id) {
            return Err(invalid_model(
                "authored obligation predecessors contain a cycle",
                None,
                usage,
                protocol_identity.as_bytes(),
            ));
        }
        Ok(Self {
            protocol_identity,
            obligations,
        })
    }

    /// Exact selected protocol identity.
    #[must_use]
    pub fn protocol_identity(&self) -> &str {
        &self.protocol_identity
    }

    /// Obligations in authored reporting order.
    #[must_use]
    pub fn obligations(&self) -> &[ObservationObligation] {
        &self.obligations
    }
}

fn all_distinct(values: &[ObligationId]) -> bool {
    let mut found = BTreeSet::new();
    values.iter().all(|value| found.insert(value))
}

fn acyclic(obligations: &[ObservationObligation], by_id: &BTreeMap<ObligationId, usize>) -> bool {
    let mut indegree: Vec<usize> = obligations
        .iter()
        .map(|obligation| obligation.predecessors.len())
        .collect();
    let mut successors = vec![Vec::new(); obligations.len()];
    for (dependent, obligation) in obligations.iter().enumerate() {
        for predecessor in &obligation.predecessors {
            if let Some(predecessor) = by_id.get(predecessor) {
                successors[*predecessor].push(dependent);
            }
        }
    }
    let mut ready: VecDeque<_> = indegree
        .iter()
        .enumerate()
        .filter_map(|(index, degree)| (*degree == 0).then_some(index))
        .collect();
    let mut visited = 0usize;
    while let Some(index) = ready.pop_front() {
        let Some(next_visited) = visited.checked_add(1) else {
            return false;
        };
        visited = next_visited;
        for successor in &successors[index] {
            let Some(degree) = indegree[*successor].checked_sub(1) else {
                return false;
            };
            indegree[*successor] = degree;
            if indegree[*successor] == 0 {
                ready.push_back(*successor);
            }
        }
    }
    visited == obligations.len()
}

/// One admitted QObs observation correlated to an exact obligation.
#[derive(Clone, Debug)]
pub struct BoundObservation<'a> {
    /// Exact affected obligation identity selected by correlation.
    pub obligation: ObligationId,
    /// Exact validated QObs record view.
    pub observation: &'a observation::View,
    /// Exact correlated workflow-instance identity.
    pub workflow_identity: Box<str>,
    /// Exact business-effect identity, when one is claimed.
    pub effect_identity: Option<Box<str>>,
}

/// Complete conformance request.
#[derive(Clone, Debug)]
pub struct Request<'a> {
    /// Complete explicit obligation model.
    pub model: &'a Model,
    /// Admitted observations in presentation order.
    pub observations: &'a [BoundObservation<'a>],
    /// Four independent producer-owner axes.
    pub axes: AxisInputs<'a>,
    /// Exact QObs completeness authority view.
    pub completeness: &'a completeness::View,
    /// Complete-global-conformance state and premise population.
    pub global_conformance: &'a GlobalConformanceInput,
    /// Independent assessment-execution disposition.
    pub execution: AssessmentExecution,
}

/// Adverse observation class. No variant is a catch-all for another.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub enum AdverseKind {
    /// The correlated observation is not the obligation's expected observation.
    Unexpected,
    /// The obligation's required observation is absent at a complete boundary.
    Missing,
    /// A dependent observation preceded an authored predecessor.
    CausallyMisordered,
    /// More than one observation claims the same business effect.
    DuplicateEffect,
    /// The correlation names another workflow instance.
    CrossWorkflow,
}

/// One adverse report against an exact obligation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AdverseObservation {
    /// Exact affected obligation identity.
    pub obligation: ObligationId,
    /// Distinct adverse kind.
    pub kind: AdverseKind,
    /// Exact admitted observation identity, absent only for a missing record.
    pub observation_identity: Option<Box<str>>,
}

/// Typed incompleteness that is never coerced to a violation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum IncompleteCause {
    /// A required valuation is explicitly absent in this exact observation.
    MissingValuation {
        /// Exact affected obligation.
        obligation: ObligationId,
        /// Exact admitted observation identity.
        observation_identity: Box<str>,
    },
}

/// Exact complete-global-conformance premise-set defect.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub enum GlobalPremiseDefectKind {
    /// One exact required premise is absent.
    Missing,
    /// One exact premise occurs more than once.
    Duplicate,
    /// A carried premise identity is not in the required population.
    Unexpected,
    /// A retained identity carries another kind, scope, or authority.
    CrossWired,
}

/// One global premise defect that cannot alter per-obligation truth.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GlobalPremiseDefect {
    /// Exact affected premise identity.
    pub premise_identity: Box<str>,
    /// Distinct defect kind.
    pub kind: GlobalPremiseDefectKind,
}

/// One obligation's independent settled-or-pending state.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum ObligationDisposition {
    /// Every fact this obligation requires is present and conformant.
    Conforms,
    /// A required fact may still arrive.
    Pending,
    /// One or more exact adverse reports affect the obligation.
    Violates,
    /// A required valuation is absent.
    Incomplete,
    /// The selected assessment execution did not complete.
    Unavailable,
}

impl ObligationDisposition {
    pub(crate) const fn tag(self) -> &'static str {
        match self {
            Self::Conforms => "conforms",
            Self::Pending => "pending",
            Self::Violates => "violates",
            Self::Incomplete => "incomplete",
            Self::Unavailable => "unavailable",
        }
    }
}

/// Result for one exact obligation, in model order.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ObligationResult {
    /// Exact obligation identity.
    pub obligation: ObligationId,
    /// Independent state.
    pub disposition: ObligationDisposition,
}

/// Aggregate status without erasing any independent input axis.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum GlobalDisposition {
    /// Every obligation and every exact global premise is complete.
    Conforms,
    /// No violation is known and an open producer-owned fact may still advance.
    Pending,
    /// An adverse observation or contradiction prevents conformance.
    Violates,
    /// Execution or required input is unavailable/incomplete.
    Unavailable,
}

/// Exact immutable owner bytes retained by the report.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RetainedDocument {
    /// Exact owner document identity.
    pub identity: Box<str>,
    /// Exact canonical bytes as admitted by the owner reader.
    pub bytes: Vec<u8>,
}

/// One admitted observation retained without repair in presentation order.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RetainedObservation {
    /// Exact correlation target.
    pub obligation: ObligationId,
    /// Exact workflow correlation.
    pub workflow_identity: Box<str>,
    /// Exact effect claim.
    pub effect_identity: Option<Box<str>>,
    /// Exact owner view bytes and identity.
    pub document: RetainedDocument,
}

/// Independently retained assessment inputs.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PremiseSnapshot {
    /// Decision-scope progress view.
    pub decision_scope_progress: RetainedDocument,
    /// Decision-scope closure view.
    pub decision_scope_closure: RetainedDocument,
    /// Surrounding-execution progress view.
    pub surrounding_execution_progress: RetainedDocument,
    /// Surrounding-execution closure view.
    pub surrounding_execution_closure: RetainedDocument,
    /// Input-completeness view.
    pub completeness: RetainedDocument,
    /// Complete global closure state and exact premise sets.
    pub global_conformance: GlobalConformanceInput,
    /// Assessment execution, retained independently.
    pub execution: AssessmentExecution,
}

/// Complete bounded conformance report.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Report {
    /// Selected protocol identity.
    pub protocol_identity: Box<str>,
    /// Per-obligation results in authored model order.
    pub obligations: Vec<ObligationResult>,
    /// Adverse observations in model/kind/presentation order.
    pub adverse: Vec<AdverseObservation>,
    /// Explicit incompleteness records.
    pub incomplete: Vec<IncompleteCause>,
    /// Exact global premise-set defects, independent of obligation results.
    pub global_premise_defects: Vec<GlobalPremiseDefect>,
    /// Global state without folding its premises.
    pub global: GlobalDisposition,
    /// Every admitted observation, unchanged and in presentation order.
    pub observations: Vec<RetainedObservation>,
    /// Every independently supplied producer-owned premise.
    pub premises: PremiseSnapshot,
    /// Bounded work and retention actually observed.
    pub usage: Usage,
}

/// Assess under the shared QProtocol caller budget.
///
/// The complete observation and obligation populations are charged
/// transactionally before any validation, traversal, or result allocation.
/// The conformance owner's immutable maxima remain independently enforced by
/// [`assess`].
pub fn assess_bounded(
    request: Request<'_>,
    limits: Limits,
    resources: &mut ResourceAccountant<'_>,
) -> OperationOutcome<Report, Refusal> {
    resources.run_unit(
        UnitLabel::new("conformance::batch::assess_bounded"),
        |resources| assess_bounded_unit(request, limits, resources),
    )
}

fn assess_bounded_unit(
    request: Request<'_>,
    limits: Limits,
    resources: &mut ResourceAccountant<'_>,
) -> OperationOutcome<Report, Refusal> {
    let effective = limits.effective();
    let predecessors = match predecessor_count(&request) {
        Ok(predecessors) => predecessors,
        Err(error) => return OperationOutcome::from_charge(error),
    };
    let retained_bytes = match retained_byte_count(&request) {
        Ok(retained_bytes) => retained_bytes,
        Err(error) => return OperationOutcome::from_charge(error),
    };
    let charges = [
        (ALLOCATION_SOURCE.id(), 1, None),
        (
            OBSERVATION_SOURCE.id(),
            request.observations.len(),
            Some(effective.max_bindings),
        ),
        (
            OPERATION_SOURCE.id(),
            request.model.obligations.len(),
            Some(effective.max_obligations),
        ),
        (
            PREDECESSOR_SOURCE.id(),
            predecessors,
            Some(effective.max_predecessors),
        ),
        (
            RETAINED_BYTE_SOURCE.id(),
            retained_bytes,
            Some(effective.max_retained_bytes),
        ),
    ];
    if let Err(error) = resources.charge_all_mixed(&charges) {
        return OperationOutcome::from_charge(error);
    }
    match assess_inner(request, effective, Some(resources)) {
        Ok(report) => OperationOutcome::Completed(report),
        Err(AssessError::Domain(error)) => OperationOutcome::DomainError(error),
        Err(AssessError::Charge(error)) => OperationOutcome::from_charge(error),
    }
}

fn predecessor_count(request: &Request<'_>) -> core::result::Result<usize, ChargeError> {
    checked_sum(
        request
            .model
            .obligations
            .iter()
            .map(|obligation| obligation.predecessors.len()),
        PREDECESSOR_SOURCE.id(),
    )
}

fn retained_byte_count(request: &Request<'_>) -> core::result::Result<usize, ChargeError> {
    checked_sum(
        request
            .observations
            .iter()
            .map(|binding| binding.observation.bytes().len())
            .chain([
                request.axes.decision_scope_progress.bytes().len(),
                request.axes.decision_scope_closure.bytes().len(),
                request.axes.surrounding_execution_progress.bytes().len(),
                request.axes.surrounding_execution_closure.bytes().len(),
                request.completeness.bytes().len(),
            ]),
        RETAINED_BYTE_SOURCE.id(),
    )
}

fn checked_sum(
    values: impl IntoIterator<Item = usize>,
    site: SiteId,
) -> core::result::Result<usize, ChargeError> {
    values.into_iter().try_fold(0usize, |consumed, requested| {
        consumed
            .checked_add(requested)
            .ok_or(ChargeError::CounterOverflow {
                site,
                consumed,
                requested,
            })
    })
}

/// Assess one exact admitted history without repairing or reordering it.
pub fn assess(request: Request<'_>, limits: Limits) -> Result<Report> {
    match assess_inner(request, limits.effective(), None) {
        Ok(report) => Ok(report),
        Err(AssessError::Domain(error)) => Err(error),
        Err(AssessError::Charge(_)) => Err(resource_refusal(
            "bounded accounting failed on an unbounded assessment path",
            Usage::default(),
            &[],
        )),
    }
}

enum AssessError {
    Domain(Refusal),
    Charge(ChargeError),
}

impl From<Refusal> for AssessError {
    fn from(error: Refusal) -> Self {
        Self::Domain(error)
    }
}

impl From<ChargeError> for AssessError {
    fn from(error: ChargeError) -> Self {
        Self::Charge(error)
    }
}

fn assess_inner(
    request: Request<'_>,
    limits: Limits,
    mut resources: Option<&mut ResourceAccountant<'_>>,
) -> core::result::Result<Report, AssessError> {
    let effective = limits.effective();
    let predecessors = predecessor_count(&request)?;
    let mut usage = Usage {
        obligations: request.model.obligations.len(),
        bindings: request.observations.len(),
        predecessors,
        ..Usage::default()
    };
    if usage.obligations > effective.max_obligations
        || usage.bindings > effective.max_bindings
        || usage.predecessors > effective.max_predecessors
    {
        return Err(resource_refusal(
            "conformance request exceeds an effective population ceiling",
            usage,
            request.model.protocol_identity.as_bytes(),
        )
        .into());
    }
    validate_owner_axes_and_required_premises(&request, usage)?;
    usage.retained_bytes = retained_byte_count(&request)?;
    if usage.retained_bytes > effective.max_retained_bytes {
        return Err(resource_refusal(
            "retained owner bytes exceed an effective ceiling",
            usage,
            request.model.protocol_identity.as_bytes(),
        )
        .into());
    }

    let by_id: BTreeMap<_, _> = request
        .model
        .obligations
        .iter()
        .enumerate()
        .map(|(index, obligation)| (&obligation.identity, index))
        .collect();
    if let Some(binding) = request
        .observations
        .iter()
        .find(|binding| !by_id.contains_key(&binding.obligation))
    {
        return Err(Refusal::new(
            ErrorCode::InvalidBinding,
            "observation binding names no exact model obligation",
            Some(&binding.obligation),
            usage,
            binding.observation.identity().as_str().as_bytes(),
        )
        .into());
    }

    let mut positions: BTreeMap<&ObligationId, usize> = BTreeMap::new();
    let mut effect_counts: BTreeMap<&str, usize> = BTreeMap::new();
    for (position, binding) in request.observations.iter().enumerate() {
        positions.entry(&binding.obligation).or_insert(position);
        if let Some(effect) = binding.effect_identity.as_deref() {
            *effect_counts.entry(effect).or_default() += 1;
        }
    }

    let boundary_complete = request.completeness.payload().state() == completeness::State::Complete
        && request.axes.decision_scope_closure.payload().state() == OpenClosed::Closed;
    let mut adverse = Vec::new();
    let mut incomplete = Vec::new();
    let mut obligations = Vec::with_capacity(request.model.obligations.len());
    for obligation in &request.model.obligations {
        let bindings: Vec<_> = request
            .observations
            .iter()
            .enumerate()
            .filter(|(_, binding)| binding.obligation == obligation.identity)
            .collect();
        if bindings.is_empty() {
            if boundary_complete {
                retain_adverse(
                    &mut adverse,
                    AdverseObservation {
                        obligation: obligation.identity.clone(),
                        kind: AdverseKind::Missing,
                        observation_identity: None,
                    },
                    &mut resources,
                    effective.max_adverse,
                )?;
                obligations.push(ObligationResult {
                    obligation: obligation.identity.clone(),
                    disposition: match request.execution {
                        AssessmentExecution::Completed => ObligationDisposition::Violates,
                        AssessmentExecution::Refused
                        | AssessmentExecution::Unsupported
                        | AssessmentExecution::ResourceIncomplete
                        | AssessmentExecution::Failed => ObligationDisposition::Unavailable,
                    },
                });
            } else {
                obligations.push(ObligationResult {
                    obligation: obligation.identity.clone(),
                    disposition: ObligationDisposition::Pending,
                });
            }
            continue;
        }

        let mut has_adverse = false;
        let mut has_incomplete = false;
        let predecessor_missing = obligation
            .predecessors
            .iter()
            .any(|predecessor| !positions.contains_key(predecessor));
        for (position, binding) in &bindings {
            let view = binding.observation.payload();
            let observation_identity = view.observation_identity().into();
            if view.subject().kind != obligation.subject_kind.as_ref()
                || view.subject().identity != obligation.subject_identity.as_ref()
            {
                has_adverse = true;
                retain_adverse(
                    &mut adverse,
                    AdverseObservation {
                        obligation: obligation.identity.clone(),
                        kind: AdverseKind::Unexpected,
                        observation_identity: Some(observation_identity),
                    },
                    &mut resources,
                    effective.max_adverse,
                )?;
            }
            if binding.workflow_identity != obligation.workflow_identity {
                has_adverse = true;
                retain_adverse(
                    &mut adverse,
                    AdverseObservation {
                        obligation: obligation.identity.clone(),
                        kind: AdverseKind::CrossWorkflow,
                        observation_identity: Some(view.observation_identity().into()),
                    },
                    &mut resources,
                    effective.max_adverse,
                )?;
            }
            if binding.effect_identity != obligation.effect_identity {
                has_adverse = true;
                retain_adverse(
                    &mut adverse,
                    AdverseObservation {
                        obligation: obligation.identity.clone(),
                        kind: AdverseKind::Unexpected,
                        observation_identity: Some(view.observation_identity().into()),
                    },
                    &mut resources,
                    effective.max_adverse,
                )?;
            } else if binding
                .effect_identity
                .as_deref()
                .is_some_and(|effect| effect_counts.get(effect).copied().unwrap_or(0) > 1)
            {
                has_adverse = true;
                retain_adverse(
                    &mut adverse,
                    AdverseObservation {
                        obligation: obligation.identity.clone(),
                        kind: AdverseKind::DuplicateEffect,
                        observation_identity: Some(view.observation_identity().into()),
                    },
                    &mut resources,
                    effective.max_adverse,
                )?;
            }
            if obligation.predecessors.iter().any(|predecessor| {
                positions
                    .get(predecessor)
                    .is_some_and(|predecessor| *predecessor > *position)
            }) {
                has_adverse = true;
                retain_adverse(
                    &mut adverse,
                    AdverseObservation {
                        obligation: obligation.identity.clone(),
                        kind: AdverseKind::CausallyMisordered,
                        observation_identity: Some(view.observation_identity().into()),
                    },
                    &mut resources,
                    effective.max_adverse,
                )?;
            }
            if obligation.valuation_required
                && matches!(view.value(), observation::ValueRef::Missing)
            {
                has_incomplete = true;
                incomplete.push(IncompleteCause::MissingValuation {
                    obligation: obligation.identity.clone(),
                    observation_identity: view.observation_identity().into(),
                });
            }
        }
        let disposition = match request.execution {
            AssessmentExecution::Refused
            | AssessmentExecution::Unsupported
            | AssessmentExecution::ResourceIncomplete
            | AssessmentExecution::Failed => ObligationDisposition::Unavailable,
            AssessmentExecution::Completed if has_incomplete => ObligationDisposition::Incomplete,
            AssessmentExecution::Completed if has_adverse => ObligationDisposition::Violates,
            AssessmentExecution::Completed if predecessor_missing && boundary_complete => {
                ObligationDisposition::Violates
            }
            AssessmentExecution::Completed if predecessor_missing => ObligationDisposition::Pending,
            AssessmentExecution::Completed => ObligationDisposition::Conforms,
        };
        obligations.push(ObligationResult {
            obligation: obligation.identity.clone(),
            disposition,
        });
    }

    usage.adverse = adverse.len();
    if usage.adverse > effective.max_adverse {
        return Err(resource_refusal(
            "adverse report population exceeds an effective ceiling",
            usage,
            request.model.protocol_identity.as_bytes(),
        )
        .into());
    }
    let observations = retain_observations(request.observations);
    let premises = retain_premises(&request);
    let global_premise_defects = global_premise_defects(request.global_conformance);
    let global = global_disposition(
        &request,
        &obligations,
        &adverse,
        &incomplete,
        &global_premise_defects,
    );
    Ok(Report {
        protocol_identity: request.model.protocol_identity.clone(),
        obligations,
        adverse,
        incomplete,
        global_premise_defects,
        global,
        observations,
        premises,
        usage,
    })
}

fn retain_adverse(
    adverse: &mut Vec<AdverseObservation>,
    item: AdverseObservation,
    resources: &mut Option<&mut ResourceAccountant<'_>>,
    owner_limit: usize,
) -> core::result::Result<(), ChargeError> {
    if let Some(resources) = resources.as_deref_mut() {
        resources.charge_with_limit(ADVERSE_SOURCE.id(), 1, owner_limit)?;
    }
    adverse.push(item);
    Ok(())
}

fn validate_owner_axes_and_required_premises(request: &Request<'_>, usage: Usage) -> Result<()> {
    let axes = request.axes;
    let paired = axes.decision_scope_progress.subject() == axes.decision_scope_closure.subject()
        && axes.surrounding_execution_progress.subject()
            == axes.surrounding_execution_closure.subject();
    let distinct = axes.decision_scope_progress.subject().scope_identity
        != axes.surrounding_execution_progress.subject().scope_identity;
    let completeness_matches = request.completeness.payload().population_identity()
        == axes
            .decision_scope_progress
            .subject()
            .population_identity
            .as_str();
    let global = request.global_conformance;
    let valid_required = global.state != GlobalConformanceClosure::NotRequired
        && !global.required_premises.is_empty()
        && {
            let mut keys = BTreeSet::new();
            global.required_premises.iter().all(|premise| {
                !premise.identity.is_empty()
                    && !premise.scope_identity.is_empty()
                    && !premise.authority_identity.is_empty()
                    && keys.insert((
                        premise.kind,
                        premise.identity.as_str(),
                        premise.scope_identity.as_str(),
                        premise.authority_identity.as_str(),
                    ))
            }) && keys
                .iter()
                .map(|(kind, _, _, _)| *kind)
                .collect::<BTreeSet<_>>()
                .len()
                == 3
        };
    if paired && distinct && completeness_matches && valid_required {
        return Ok(());
    }
    Err(Refusal::new(
        ErrorCode::InvalidPremises,
        "scope owner views or complete-global-conformance premises are cross-wired",
        None,
        usage,
        request.model.protocol_identity.as_bytes(),
    ))
}

fn global_premise_defects(global: &GlobalConformanceInput) -> Vec<GlobalPremiseDefect> {
    let required_by_identity: BTreeMap<_, _> = global
        .required_premises
        .iter()
        .map(|premise| (premise.identity.as_str(), premise))
        .collect();
    let mut defects = Vec::new();
    for required in &global.required_premises {
        let mut matching = global
            .premises
            .iter()
            .filter(|carried| carried.identity == required.identity);
        let first = matching.next();
        let duplicate = matching.next().is_some();
        if duplicate {
            defects.push(GlobalPremiseDefect {
                premise_identity: required.identity.as_str().into(),
                kind: GlobalPremiseDefectKind::Duplicate,
            });
        } else if first.is_some_and(|carried| carried != required) {
            defects.push(GlobalPremiseDefect {
                premise_identity: required.identity.as_str().into(),
                kind: GlobalPremiseDefectKind::CrossWired,
            });
        } else if first.is_none() {
            defects.push(GlobalPremiseDefect {
                premise_identity: required.identity.as_str().into(),
                kind: GlobalPremiseDefectKind::Missing,
            });
        }
    }
    for carried in &global.premises {
        if !required_by_identity.contains_key(carried.identity.as_str()) {
            defects.push(GlobalPremiseDefect {
                premise_identity: carried.identity.as_str().into(),
                kind: GlobalPremiseDefectKind::Unexpected,
            });
        }
    }
    defects
}

fn global_disposition(
    request: &Request<'_>,
    obligations: &[ObligationResult],
    adverse: &[AdverseObservation],
    incomplete: &[IncompleteCause],
    global_premise_defects: &[GlobalPremiseDefect],
) -> GlobalDisposition {
    match request.execution {
        AssessmentExecution::Refused
        | AssessmentExecution::Unsupported
        | AssessmentExecution::ResourceIncomplete
        | AssessmentExecution::Failed => return GlobalDisposition::Unavailable,
        AssessmentExecution::Completed => {}
    }
    if !incomplete.is_empty()
        || !global_premise_defects.is_empty()
        || request.completeness.payload().state() != completeness::State::Complete
    {
        return GlobalDisposition::Unavailable;
    }
    if !adverse.is_empty()
        || request.global_conformance.state == GlobalConformanceClosure::Contradicted
        || obligations
            .iter()
            .any(|result| result.disposition == ObligationDisposition::Violates)
    {
        return GlobalDisposition::Violates;
    }
    let axes_closed = request.axes.decision_scope_progress.payload().state() == OpenClosed::Closed
        && request.axes.decision_scope_closure.payload().state() == OpenClosed::Closed
        && request
            .axes
            .surrounding_execution_progress
            .payload()
            .state()
            == OpenClosed::Closed
        && request.axes.surrounding_execution_closure.payload().state() == OpenClosed::Closed;
    let obligations_complete = obligations
        .iter()
        .all(|result| result.disposition == ObligationDisposition::Conforms);
    if axes_closed
        && obligations_complete
        && request.global_conformance.state == GlobalConformanceClosure::Closed
    {
        GlobalDisposition::Conforms
    } else {
        GlobalDisposition::Pending
    }
}

fn retain_observations(observations: &[BoundObservation<'_>]) -> Vec<RetainedObservation> {
    observations
        .iter()
        .map(|binding| RetainedObservation {
            obligation: binding.obligation.clone(),
            workflow_identity: binding.workflow_identity.clone(),
            effect_identity: binding.effect_identity.clone(),
            document: retained_parts(
                binding.observation.identity().as_str(),
                binding.observation.bytes(),
            ),
        })
        .collect()
}

fn retain_premises(request: &Request<'_>) -> PremiseSnapshot {
    PremiseSnapshot {
        decision_scope_progress: retained_parts(
            request.axes.decision_scope_progress.identity().as_str(),
            request.axes.decision_scope_progress.bytes(),
        ),
        decision_scope_closure: retained_parts(
            request.axes.decision_scope_closure.identity().as_str(),
            request.axes.decision_scope_closure.bytes(),
        ),
        surrounding_execution_progress: retained_parts(
            request
                .axes
                .surrounding_execution_progress
                .identity()
                .as_str(),
            request.axes.surrounding_execution_progress.bytes(),
        ),
        surrounding_execution_closure: retained_parts(
            request
                .axes
                .surrounding_execution_closure
                .identity()
                .as_str(),
            request.axes.surrounding_execution_closure.bytes(),
        ),
        completeness: retained_parts(
            request.completeness.identity().as_str(),
            request.completeness.bytes(),
        ),
        global_conformance: request.global_conformance.clone(),
        execution: request.execution,
    }
}

fn retained_parts(identity: &str, bytes: &[u8]) -> RetainedDocument {
    RetainedDocument {
        identity: identity.into(),
        bytes: bytes.to_vec(),
    }
}

fn invalid_model(
    detail: &'static str,
    affected: Option<&ObligationId>,
    usage: Usage,
    discriminator: &[u8],
) -> Refusal {
    Refusal::new(
        ErrorCode::InvalidModel,
        detail,
        affected,
        usage,
        discriminator,
    )
}

fn resource_refusal(detail: &'static str, usage: Usage, discriminator: &[u8]) -> Refusal {
    Refusal::new(
        ErrorCode::ResourceIncomplete,
        detail,
        None,
        usage,
        discriminator,
    )
}
