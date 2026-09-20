// SPDX-License-Identifier: AGPL-3.0-or-later
//! Claim-local dispatch with exact backend matching and no fallback.

#![deny(clippy::wildcard_enum_match_arm)]

use crate::closure::AssessmentExecution;
use crate::conformance::{GlobalDisposition, Report as ConformanceReport};
use crate::refusal::catalog::{
    CatalogSelection, CodeEntry, CodeModule, Provenance, RefusalCode, RetainedBytes,
};
use crate::refusal::{
    AffectedIdentity, Prerequisite, PrerequisiteState, ProtocolCause, ProtocolRefusal,
    ProtocolStage, RefusalAxis, RefusalCause, RefusalDomain, ResourceUsage, RetryDecision,
    Unavailability,
};
use crate::ByteDigest;

use crate::bounds::inventory::{
    BudgetDimension, ChargeError, OperationOutcome, ResourceAccountant, SourceSite,
};
use crate::bounds::{SiteId, SiteKind, UnitLabel};

use super::premises::{self, CorrespondenceAuthority, PremiseDefect, PremiseSet};
use super::Claim;

pub(crate) const RESULT_SOURCE: SourceSite = SourceSite::new(
    SiteId::new("claims/disposition/retain"),
    "claims::dispatch::dispatch",
    BudgetDimension::ResultItem,
    SiteKind::Retention,
);
pub(crate) const DERIVED_SOURCE: SourceSite = SourceSite::new(
    SiteId::new("claims/correspondence/derive"),
    "claims::dispatch::dispatch",
    BudgetDimension::DerivedArtifact,
    SiteKind::Retention,
);
pub(crate) const OPERATION_SOURCE: SourceSite = SourceSite::new(
    SiteId::new("claims/conformance/dispatch"),
    "claims::dispatch::dispatch_from_conformance",
    BudgetDimension::Operation,
    SiteKind::Iteration,
);
pub(crate) const COMPONENT_SOURCE: SourceSite = SourceSite::new(
    SiteId::new("claims/premise/component/visit"),
    "claims::dispatch::dispatch",
    BudgetDimension::Node,
    SiteKind::Traversal,
);
pub(crate) const ALLOCATION_SOURCE: SourceSite = SourceSite::new(
    SiteId::new("claims/report/allocate"),
    "claims::dispatch::dispatch",
    BudgetDimension::AccountedAllocation,
    SiteKind::Allocation,
);

/// Immutable dispatch owner maxima.
pub const OWNER_MAX: Limits = Limits {
    max_requests: 64,
    max_components: 100_000,
    max_bytes: 8 * 1024 * 1024,
};

/// Bounded dispatch ceilings. Callers may lower but never raise them.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Limits {
    /// Maximum requested claims.
    pub max_requests: usize,
    /// Maximum retained components, assumptions, and dependency edges.
    pub max_components: usize,
    /// Maximum exact premise bytes retained across the request.
    pub max_bytes: usize,
}

impl Limits {
    /// Immutable owner maxima.
    #[must_use]
    pub const fn owner_max() -> Self {
        OWNER_MAX
    }

    /// Effective caller ceilings.
    #[must_use]
    pub const fn effective(self) -> Self {
        Self {
            max_requests: min(self.max_requests, OWNER_MAX.max_requests),
            max_components: min(self.max_components, OWNER_MAX.max_components),
            max_bytes: min(self.max_bytes, OWNER_MAX.max_bytes),
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

/// The only reviewed local capability admitted by this dispatch slice.
///
/// Other claim kinds retain their selected backend bytes but return explicit
/// `Unsupported`: a caller cannot name an alternate backend and thereby turn
/// an advanced claim into a supported one.
const CONFORMANCE_BACKEND_ID: &[u8] = b"qprotocol:conformance";
const CONFORMANCE_BACKEND_VERSION: &[u8] = b"v1";

impl Default for Limits {
    fn default() -> Self {
        OWNER_MAX
    }
}

/// Bounded work observed before dispatch or refusal.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Usage {
    /// Requested claims visited.
    pub requests: usize,
    /// Exact premise components and dependency edges visited.
    pub components: usize,
    /// Exact retained premise bytes visited.
    pub bytes: usize,
}

/// One exact claim request.
pub struct ClaimRequest<'a> {
    /// Exact request identity bytes.
    pub identity: Box<[u8]>,
    /// Exact claim kind.
    pub claim: Claim,
    /// Exact producer-owned strength bytes; no order is inferred.
    pub strength: Box<[u8]>,
    /// Complete premise selection.
    pub premises: PremiseSet,
    /// Selected producer owner for optional derived correspondence.
    pub correspondence: Option<&'a dyn CorrespondenceAuthority>,
}

impl core::fmt::Debug for ClaimRequest<'_> {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter
            .debug_struct("ClaimRequest")
            .field("identity", &self.identity)
            .field("claim", &self.claim)
            .field("strength", &self.strength)
            .field("premises", &self.premises)
            .field("correspondence_selected", &self.correspondence.is_some())
            .finish()
    }
}

/// Closed dispatcher semantic answer for one exact claim.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum Decision {
    /// This exact claim was established.
    Established,
    /// This exact claim was assessed and was not established.
    NotEstablished,
}

/// Claim-local refusal axis.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum FailureCode {
    /// Request identity, strength, or exact premises are invalid.
    InvalidPremises,
    /// The same exact request identity was offered more than once.
    DuplicateRequest,
}

impl FailureCode {
    /// Stable machine code.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        self.entry().code
    }

    const fn entry(self) -> &'static CodeEntry {
        match self {
            Self::InvalidPremises => &INVALID_PREMISES,
            Self::DuplicateRequest => &DUPLICATE_REQUEST,
        }
    }
}

/// Typed refusal for one claim. Sibling dispositions remain available.
#[derive(Clone, Debug)]
pub struct ClaimFailure {
    code: FailureCode,
    defects: Vec<PremiseDefect>,
    outcome_identity: Box<str>,
    envelope: ProtocolRefusal,
}

impl ClaimFailure {
    fn new(
        code: FailureCode,
        request: &ClaimRequest<'_>,
        defects: Vec<PremiseDefect>,
        retains_independent: bool,
        usage: Usage,
    ) -> Self {
        let mut preimage = Vec::new();
        push_part(&mut preimage, code.as_str().as_bytes());
        push_part(&mut preimage, request.identity.as_ref());
        push_part(&mut preimage, request.claim.code().as_bytes());
        push_part(&mut preimage, request.strength.as_ref());
        for defect in &defects {
            push_part(&mut preimage, premise_axis_tag(defect.axis).as_bytes());
            push_part(&mut preimage, defect_kind_tag(defect.kind).as_bytes());
            push_part(
                &mut preimage,
                defect.identity.as_deref().unwrap_or_default(),
            );
        }
        let outcome_identity: Box<str> = format!("sha256:{:x}", ByteDigest::of(&preimage)).into();
        let refusal_code = RefusalCode::of(code.entry());
        let mut causes = defects.iter().map(defect_cause);
        let first_cause = causes.next().unwrap_or_else(|| {
            ProtocolCause::ConflictingDefinition {
                definition: RetainedBytes::new(request.identity.as_ref()),
                conflicting_definition: RetainedBytes::new(request.identity.as_ref()),
            }
            .into()
        });
        let mut builder = ProtocolRefusal::builder(
            refusal_code,
            CatalogSelection::local(),
            ProtocolStage::ClaimDispatch,
            RefusalAxis::new(&preimage).expect("code-prefixed claim axis is non-empty"),
            first_cause,
            refusal_usage(usage),
        )
        .expect("claim codes belong to claim dispatch")
        .prerequisite(Prerequisite::new(
            ProtocolStage::Conformance,
            PrerequisiteState::Established,
        ))
        .identity(AffectedIdentity::retained(request.identity.as_ref()))
        .unavailable_location(Unavailability::NotApplicableToStage);
        for cause in causes {
            builder = builder.cause(cause);
        }
        let envelope = if retains_independent {
            builder
                .retaining_independent((), Vec::new())
                .refusal()
                .clone()
        } else {
            builder.emitting_nothing()
        };
        Self {
            code,
            defects,
            outcome_identity,
            envelope,
        }
    }

    /// Stable typed failure axis.
    #[must_use]
    pub const fn code(&self) -> FailureCode {
        self.code
    }

    /// Every independently retained premise defect.
    #[must_use]
    pub fn defects(&self) -> &[PremiseDefect] {
        &self.defects
    }

    /// Display-independent typed outcome identity.
    #[must_use]
    pub fn outcome_identity(&self) -> &str {
        &self.outcome_identity
    }

    /// Complete claim-local refusal envelope.
    #[must_use]
    pub fn envelope(&self) -> &ProtocolRefusal {
        &self.envelope
    }
}

/// One returned disposition, exactly one per request.
#[derive(Clone, Debug)]
pub struct Disposition {
    /// Exact request identity.
    pub request_identity: Box<[u8]>,
    /// Exact claim answered.
    pub claim: Claim,
    /// Exact strength retained without promotion.
    pub strength: Box<[u8]>,
    /// Independent execution disposition.
    pub execution: AssessmentExecution,
    /// Semantic answer only when execution completed.
    pub decision: Option<Decision>,
    /// Exact request premises, retained under every execution outcome.
    pub premises: PremiseSet,
    /// Claim-local refusal, only for `refused`.
    pub failure: Option<ClaimFailure>,
}

/// Complete ordered dispatch output.
///
/// Its fields are sealed so an external result consumer cannot construct a
/// caller-authored substitute for the report returned by
/// [`crate::assessment::ResolvedAssessmentEngine::dispatch_activated`].
///
/// ```compile_fail,E0451
/// use quire_protocol::claims::dispatch::{Report, Usage};
/// let _ = Report { dispositions: Vec::new(), usage: Usage::default() };
/// ```
#[derive(Clone, Debug)]
pub struct Report {
    /// Exactly one disposition per requested claim, in request order.
    pub(super) dispositions: Vec<Disposition>,
    /// Bounded work actually observed.
    pub(super) usage: Usage,
}

impl Report {
    /// Exactly one disposition per requested claim, in request order.
    #[must_use]
    pub fn dispositions(&self) -> &[Disposition] {
        &self.dispositions
    }

    /// Bounded work actually observed by the closed dispatcher.
    #[must_use]
    pub const fn usage(&self) -> Usage {
        self.usage
    }

    #[cfg(test)]
    pub(crate) fn empty_for_population_test() -> Self {
        Self {
            dispositions: Vec::new(),
            usage: Usage::default(),
        }
    }
}

/// Fatal whole-request error. No partial report accompanies it.
#[derive(Clone, Debug)]
pub struct Refusal {
    usage: Usage,
    envelope: ProtocolRefusal,
}

impl Refusal {
    /// Work observed before refusal.
    #[must_use]
    pub const fn usage(&self) -> Usage {
        self.usage
    }

    /// Complete no-partial-output refusal envelope.
    #[must_use]
    pub fn envelope(&self) -> &ProtocolRefusal {
        &self.envelope
    }
}

impl core::fmt::Display for Refusal {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter.write_str("claims.resource-incomplete: dispatch limits exceeded")
    }
}

impl std::error::Error for Refusal {}

/// Dispatch return type.
pub type Result<T> = std::result::Result<T, Refusal>;

/// Dispatch the closed locally supported claim set from an authoritative
/// conformance report.
///
/// This production path deliberately exposes no backend callback: finite
/// conformance is answered only from the report produced by QProtocol's own
/// conformance owner, while every other claim remains explicitly unsupported.
/// The legacy backend seam remains only while its migration tests are being
/// replaced; callers of this entry point cannot substitute an implementation.
pub(crate) fn dispatch_from_conformance(
    requests: &[ClaimRequest<'_>],
    conformance: &ConformanceReport,
    limits: Limits,
) -> Result<Report> {
    let effective = limits.effective();
    let usage = preflight(requests);
    if usage.requests > effective.max_requests
        || usage.components > effective.max_components
        || usage.bytes > effective.max_bytes
    {
        return Err(resource_refusal(usage));
    }
    let retains_independent = requests.len() > 1;
    let mut request_identities = std::collections::BTreeMap::new();
    for request in requests {
        *request_identities
            .entry(request.identity.as_ref())
            .or_insert(0usize) += 1;
    }
    let mut dispositions = Vec::with_capacity(requests.len());
    for request in requests {
        if request_identities
            .get(request.identity.as_ref())
            .is_some_and(|count| *count > 1)
        {
            dispositions.push(refused(
                request,
                FailureCode::DuplicateRequest,
                Vec::new(),
                retains_independent,
                usage,
            ));
            continue;
        }
        let defects = request_defects(request);
        if !defects.is_empty() {
            dispositions.push(refused(
                request,
                FailureCode::InvalidPremises,
                defects,
                retains_independent,
                usage,
            ));
            continue;
        }
        match request.claim {
            Claim::Conformance => dispositions.push(Disposition {
                request_identity: request.identity.clone(),
                claim: request.claim,
                strength: request.strength.clone(),
                execution: conformance.premises.execution,
                decision: (conformance.premises.execution == AssessmentExecution::Completed)
                    .then_some(match conformance.global {
                        GlobalDisposition::Conforms => Decision::Established,
                        GlobalDisposition::Pending
                        | GlobalDisposition::Violates
                        | GlobalDisposition::Unavailable => Decision::NotEstablished,
                    }),
                premises: request.premises.clone(),
                failure: None,
            }),
            Claim::Monitorability
            | Claim::Projection
            | Claim::Realizability
            | Claim::Refinement
            | Claim::Composition => {
                dispositions.push(unavailable(request, AssessmentExecution::Unsupported));
            }
        }
    }
    Ok(Report {
        dispositions,
        usage,
    })
}

/// Dispatch the closed capability under the shared QProtocol caller budget.
///
/// The complete input, result, derived-artifact, and operation increments are
/// charged transactionally before result allocation. The
/// module's immutable owner maxima remain independently enforced by
/// [`dispatch_from_conformance`]; caller grants can lower work admitted at this boundary but
/// cannot raise those maxima.
#[allow(
    dead_code,
    reason = "the crate-private bounded evaluator is exercised by the retained TC-005/TC-009 semantic tests; production enters through Engine::dispatch_activated"
)]
pub(crate) fn dispatch_bounded(
    requests: &[ClaimRequest<'_>],
    conformance: &ConformanceReport,
    limits: Limits,
    resources: &mut ResourceAccountant<'_>,
) -> OperationOutcome<Report, Refusal> {
    resources.run_unit(UnitLabel::new("claims::dispatch_bounded"), |resources| {
        dispatch_bounded_unit(requests, conformance, limits, resources)
    })
}

#[allow(
    dead_code,
    reason = "the crate-private bounded evaluator is exercised by the retained TC-005/TC-009 semantic tests; production enters through Engine::dispatch_activated"
)]
fn dispatch_bounded_unit(
    requests: &[ClaimRequest<'_>],
    conformance: &ConformanceReport,
    limits: Limits,
    resources: &mut ResourceAccountant<'_>,
) -> OperationOutcome<Report, Refusal> {
    let effective = limits.effective();
    let preflight = match preflight_checked(requests) {
        Ok(preflight) => preflight,
        Err(error) => return OperationOutcome::from_charge(error),
    };
    let derived = requests
        .iter()
        .filter(|request| request.premises.derived_artifact.is_some())
        .count();
    let charges = [
        (ALLOCATION_SOURCE.id(), 1, None),
        (
            super::premises::INPUT_SOURCE.id(),
            preflight.bytes,
            Some(effective.max_bytes),
        ),
        (
            COMPONENT_SOURCE.id(),
            preflight.components,
            Some(effective.max_components),
        ),
        (
            RESULT_SOURCE.id(),
            requests.len(),
            Some(effective.max_requests),
        ),
        (DERIVED_SOURCE.id(), derived, Some(effective.max_components)),
        (
            OPERATION_SOURCE.id(),
            requests.len(),
            Some(effective.max_requests),
        ),
    ];
    if let Err(error) = resources.charge_all_mixed(&charges) {
        return OperationOutcome::from_charge(error);
    }
    OperationOutcome::from_domain(dispatch_from_conformance(requests, conformance, effective))
}

fn request_defects(request: &ClaimRequest<'_>) -> Vec<PremiseDefect> {
    let mut defects = premises::validate(&request.premises, request.correspondence);
    if request.identity.is_empty() || request.strength.is_empty() {
        defects.push(PremiseDefect {
            axis: premises::PremiseAxis::AdmittedFragment,
            kind: premises::PremiseDefectKind::Empty,
            identity: (!request.identity.is_empty()).then(|| request.identity.clone()),
        });
    }
    if request.claim == Claim::Conformance
        && (request.premises.backend.identity.as_ref() != CONFORMANCE_BACKEND_ID
            || request.premises.backend.version.as_ref() != CONFORMANCE_BACKEND_VERSION)
    {
        defects.push(PremiseDefect {
            axis: premises::PremiseAxis::Backend,
            kind: premises::PremiseDefectKind::CorrespondenceMismatch,
            identity: Some(request.premises.backend.identity.clone()),
        });
    }
    defects
}

fn refused(
    request: &ClaimRequest<'_>,
    code: FailureCode,
    defects: Vec<PremiseDefect>,
    retains_independent: bool,
    usage: Usage,
) -> Disposition {
    Disposition {
        request_identity: request.identity.clone(),
        claim: request.claim,
        strength: request.strength.clone(),
        execution: AssessmentExecution::Refused,
        decision: None,
        premises: request.premises.clone(),
        failure: Some(ClaimFailure::new(
            code,
            request,
            defects,
            retains_independent,
            usage,
        )),
    }
}

fn unavailable(request: &ClaimRequest<'_>, execution: AssessmentExecution) -> Disposition {
    Disposition {
        request_identity: request.identity.clone(),
        claim: request.claim,
        strength: request.strength.clone(),
        execution,
        decision: None,
        premises: request.premises.clone(),
        failure: None,
    }
}

fn preflight(requests: &[ClaimRequest<'_>]) -> Usage {
    preflight_checked(requests).unwrap_or(Usage {
        requests: requests.len(),
        components: usize::MAX,
        bytes: usize::MAX,
    })
}

fn preflight_checked(requests: &[ClaimRequest<'_>]) -> core::result::Result<Usage, ChargeError> {
    let mut usage = Usage {
        requests: requests.len(),
        ..Usage::default()
    };
    for request in requests {
        checked_increment(&mut usage.components, 2, COMPONENT_SOURCE.id())?;
        for bytes in [
            request.identity.len(),
            request.strength.len(),
            request.premises.admitted_fragment.identity.len(),
            request.premises.admitted_fragment.bytes.len(),
            request.premises.backend.identity.len(),
            request.premises.backend.version.len(),
        ] {
            checked_increment(&mut usage.bytes, bytes, super::premises::INPUT_SOURCE.id())?;
        }
        for assumption in &request.premises.assumptions {
            visit_component(&assumption.component, &mut usage)?;
            checked_increment(
                &mut usage.components,
                assumption.dependencies.len(),
                COMPONENT_SOURCE.id(),
            )?;
            for dependency in &assumption.dependencies {
                checked_increment(
                    &mut usage.bytes,
                    dependency.len(),
                    super::premises::INPUT_SOURCE.id(),
                )?;
            }
        }
        for component in request
            .premises
            .communication
            .iter()
            .chain(&request.premises.visibility)
            .chain(&request.premises.environment)
            .chain(&request.premises.sources)
        {
            visit_component(component, &mut usage)?;
        }
        if let Some(derived) = &request.premises.derived_artifact {
            visit_component(&derived.artifact, &mut usage)?;
            for component in &derived.source_tuple {
                visit_component(component, &mut usage)?;
            }
        }
    }
    Ok(usage)
}

fn visit_component(
    component: &premises::Component,
    usage: &mut Usage,
) -> core::result::Result<(), ChargeError> {
    checked_increment(&mut usage.components, 1, COMPONENT_SOURCE.id())?;
    checked_increment(
        &mut usage.bytes,
        component.identity.len(),
        super::premises::INPUT_SOURCE.id(),
    )?;
    checked_increment(
        &mut usage.bytes,
        component.bytes.len(),
        super::premises::INPUT_SOURCE.id(),
    )
}

fn checked_increment(
    consumed: &mut usize,
    requested: usize,
    site: SiteId,
) -> core::result::Result<(), ChargeError> {
    let next = consumed
        .checked_add(requested)
        .ok_or(ChargeError::CounterOverflow {
            site,
            consumed: *consumed,
            requested,
        })?;
    *consumed = next;
    Ok(())
}

fn resource_refusal(usage: Usage) -> Refusal {
    let refusal_code = RefusalCode::of(&RESOURCE_INCOMPLETE);
    let envelope = ProtocolRefusal::builder(
        refusal_code,
        CatalogSelection::local(),
        ProtocolStage::ClaimDispatch,
        RefusalAxis::from_registered_code(refusal_code),
        ProtocolCause::IncompleteEvidence {
            evidence: RetainedBytes::new(b"claim-dispatch-owner-limit"),
        }
        .into(),
        refusal_usage(usage),
    )
    .expect("claim codes belong to claim dispatch")
    .prerequisite(Prerequisite::new(
        ProtocolStage::Conformance,
        PrerequisiteState::Established,
    ))
    .unavailable_location(Unavailability::NotApplicableToStage)
    .emitting_nothing();
    Refusal { usage, envelope }
}

fn refusal_usage(usage: Usage) -> ResourceUsage {
    ResourceUsage::protocol(
        "quire.protocol/claim-dispatch-usage/1",
        &[
            ("requests", usage.requests),
            ("components", usage.components),
            ("bytes", usage.bytes),
        ],
    )
}

fn defect_cause(defect: &PremiseDefect) -> RefusalCause {
    let axis = premise_axis_tag(defect.axis).as_bytes();
    let identity = defect.identity.as_deref().unwrap_or(axis);
    match defect.kind {
        premises::PremiseDefectKind::Missing
        | premises::PremiseDefectKind::Empty
        | premises::PremiseDefectKind::DanglingDependency => ProtocolCause::MissingDefinition {
            definition: RetainedBytes::new(identity),
        },
        premises::PremiseDefectKind::Duplicate
        | premises::PremiseDefectKind::CircularDependency => ProtocolCause::ConflictingDefinition {
            definition: RetainedBytes::new(identity),
            conflicting_definition: RetainedBytes::new(axis),
        },
        premises::PremiseDefectKind::CorrespondenceMismatch => ProtocolCause::IdentityMismatch {
            expected: RetainedBytes::new(axis),
            observed: RetainedBytes::new(identity),
        },
    }
    .into()
}

fn push_part(target: &mut Vec<u8>, value: &[u8]) {
    let length = u64::try_from(value.len()).unwrap_or(u64::MAX);
    target.extend_from_slice(&length.to_be_bytes());
    target.extend_from_slice(value);
}

const fn premise_axis_tag(axis: premises::PremiseAxis) -> &'static str {
    match axis {
        premises::PremiseAxis::AdmittedFragment => "admitted-fragment",
        premises::PremiseAxis::Backend => "backend",
        premises::PremiseAxis::Assumption => "assumption",
        premises::PremiseAxis::Communication => "communication",
        premises::PremiseAxis::Visibility => "visibility",
        premises::PremiseAxis::Environment => "environment",
        premises::PremiseAxis::Source => "source",
        premises::PremiseAxis::DerivedArtifact => "derived-artifact",
    }
}

const fn defect_kind_tag(kind: premises::PremiseDefectKind) -> &'static str {
    match kind {
        premises::PremiseDefectKind::Missing => "missing",
        premises::PremiseDefectKind::Empty => "empty",
        premises::PremiseDefectKind::Duplicate => "duplicate",
        premises::PremiseDefectKind::DanglingDependency => "dangling-dependency",
        premises::PremiseDefectKind::CircularDependency => "circular-dependency",
        premises::PremiseDefectKind::CorrespondenceMismatch => "correspondence-mismatch",
    }
}

const INVALID_PREMISES: CodeEntry = CodeEntry {
    provenance: Provenance::Minted,
    code: "claims.premise-invalid",
    domain: RefusalDomain::Assessment,
    retry: RetryDecision::AfterSelectionChange,
    summary: "one exact claim premise is missing, duplicated, circular, or invalid",
};
const DUPLICATE_REQUEST: CodeEntry = CodeEntry {
    provenance: Provenance::Minted,
    code: "claims.request-duplicate",
    domain: RefusalDomain::Assessment,
    retry: RetryDecision::AfterSelectionChange,
    summary: "the same exact claim request identity was offered more than once",
};
const RESOURCE_INCOMPLETE: CodeEntry = CodeEntry {
    provenance: Provenance::Minted,
    code: "claims.resource-incomplete",
    domain: RefusalDomain::Assessment,
    retry: RetryDecision::AfterResourceRelease,
    summary: "claim dispatch exceeded an effective resource ceiling",
};

/// Claim-dispatch refusal codes registered crate-wide.
pub const CODES: CodeModule = CodeModule {
    root: "claims",
    owner: "C-23/C-24",
    module_path: "quire_protocol::claims::dispatch",
    entries: &[INVALID_PREMISES, DUPLICATE_REQUEST, RESOURCE_INCOMPLETE],
};
