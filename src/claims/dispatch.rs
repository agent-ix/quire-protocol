// SPDX-License-Identifier: AGPL-3.0-or-later
//! Claim-local dispatch with exact backend matching and no fallback.

#![deny(clippy::wildcard_enum_match_arm)]

use crate::closure::AssessmentExecution;
use crate::refusal::catalog::{CatalogSelection, CodeEntry, CodeModule, Provenance, RefusalCode};
use crate::refusal::{
    AffectedIdentity, OriginatingStage, ProtocolRefusal, RefusalDomain, RetryDecision,
    Unavailability,
};
use crate::ByteDigest;

use crate::bounds::inventory::{BudgetDimension, SourceSite};
use crate::bounds::{SiteId, SiteKind};

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
    SiteId::new("claims/backend/invoke"),
    "claims::dispatch::dispatch",
    BudgetDimension::Operation,
    SiteKind::Iteration,
);

/// Immutable dispatch owner maxima.
pub const OWNER_MAX: Limits = Limits {
    max_requests: 64,
    max_backends: 64,
    max_components: 100_000,
    max_bytes: 8 * 1024 * 1024,
};

/// Bounded dispatch ceilings. Callers may lower but never raise them.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Limits {
    /// Maximum requested claims.
    pub max_requests: usize,
    /// Maximum backend implementations inspected.
    pub max_backends: usize,
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
            max_backends: min(self.max_backends, OWNER_MAX.max_backends),
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
    /// Backend implementations inspected.
    pub backends: usize,
    /// Exact premise components and dependency edges visited.
    pub components: usize,
    /// Exact retained premise bytes visited.
    pub bytes: usize,
    /// Backend invocations completed.
    pub backend_calls: usize,
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

/// Backend's completed semantic answer for one exact claim.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum Decision {
    /// This exact claim was established.
    Established,
    /// This exact claim was assessed and was not established.
    NotEstablished,
}

/// Backend execution result. Capability selection happens before this call.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum BackendExecution {
    /// Backend completed with an exact claim answer.
    Completed(Decision),
    /// Backend refused this exact request.
    Refused,
    /// A declared backend budget was exhausted.
    ResourceIncomplete,
    /// Selected backend failed; the dispatcher does not try another.
    Failed,
}

/// Backend answer plus the exact premise evidence it claims to have used.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BackendReply {
    /// Exact execution outcome.
    pub execution: BackendExecution,
    /// Exact backend-retained evidence, checked against the request.
    pub evidence: PremiseSet,
}

/// Analysis backend seam. Dispatch calls at most one exact selected backend.
pub trait Backend {
    /// Exact backend identity and version offered by this implementation.
    fn selection(&self) -> &premises::BackendSelection;

    /// Whether this exact backend implements this exact claim and strength.
    fn supports(&self, claim: Claim, strength: &[u8]) -> bool;

    /// Execute one exact supported request.
    fn assess(&mut self, claim: Claim, strength: &[u8], premises: &PremiseSet) -> BackendReply;
}

/// Claim-local refusal axis.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum FailureCode {
    /// Request identity, strength, or exact premises are invalid.
    InvalidPremises,
    /// Backend identity exists but the selected version does not.
    BackendMismatch,
    /// More than one backend claims the exact selection.
    BackendAmbiguous,
    /// The same exact request identity was offered more than once.
    DuplicateRequest,
    /// Backend returned evidence other than the exact selected premises.
    EvidenceMismatch,
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
            Self::BackendMismatch => &BACKEND_MISMATCH,
            Self::BackendAmbiguous => &BACKEND_AMBIGUOUS,
            Self::DuplicateRequest => &DUPLICATE_REQUEST,
            Self::EvidenceMismatch => &EVIDENCE_MISMATCH,
        }
    }
}

/// Typed refusal for one claim. Sibling dispositions remain available.
#[derive(Clone, Debug)]
pub struct ClaimFailure {
    code: FailureCode,
    defects: Vec<PremiseDefect>,
    outcome_identity: Box<str>,
    envelope: Box<ProtocolRefusal>,
}

impl ClaimFailure {
    fn new(
        code: FailureCode,
        request: &ClaimRequest<'_>,
        defects: Vec<PremiseDefect>,
        retains_independent: bool,
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
        let builder = ProtocolRefusal::builder(
            RefusalCode::of(code.entry()),
            CatalogSelection::local(),
            OriginatingStage::ResourceExecution,
        )
        .identity(AffectedIdentity::retained(request.identity.as_ref()))
        .unavailable_location(Unavailability::NotApplicableToStage);
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
            envelope: Box::new(envelope),
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
        self.envelope.as_ref()
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
#[derive(Clone, Debug)]
pub struct Report {
    /// Exactly one disposition per requested claim, in request order.
    pub dispositions: Vec<Disposition>,
    /// Bounded work actually observed.
    pub usage: Usage,
}

/// Fatal whole-request error. No partial report accompanies it.
#[derive(Clone, Debug)]
pub struct Refusal {
    usage: Usage,
    envelope: Box<ProtocolRefusal>,
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
        self.envelope.as_ref()
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

/// Dispatch every exact request independently and invoke no fallback backend.
pub fn dispatch(
    requests: &[ClaimRequest<'_>],
    backends: &mut [&mut dyn Backend],
    limits: Limits,
) -> Result<Report> {
    let effective = limits.effective();
    let mut usage = preflight(requests, backends.len());
    if usage.requests > effective.max_requests
        || usage.backends > effective.max_backends
        || usage.components > effective.max_components
        || usage.bytes > effective.max_bytes
    {
        return Err(resource_refusal(usage));
    }
    let mut request_identities = std::collections::BTreeMap::new();
    for request in requests {
        *request_identities
            .entry(request.identity.as_ref())
            .or_insert(0usize) += 1;
    }
    let retains_independent = requests.len() > 1;
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
            ));
            continue;
        }
        let exact: Vec<_> = backends
            .iter_mut()
            .enumerate()
            .filter_map(|(index, backend)| {
                (backend.selection() == &request.premises.backend).then_some(index)
            })
            .collect();
        if exact.len() > 1 {
            dispositions.push(refused(
                request,
                FailureCode::BackendAmbiguous,
                Vec::new(),
                retains_independent,
            ));
            continue;
        }
        let Some(index) = exact.first().copied() else {
            let identity_exists = backends
                .iter()
                .any(|backend| backend.selection().identity == request.premises.backend.identity);
            if identity_exists {
                dispositions.push(refused(
                    request,
                    FailureCode::BackendMismatch,
                    Vec::new(),
                    retains_independent,
                ));
            } else {
                dispositions.push(unavailable(request, AssessmentExecution::Unsupported));
            }
            continue;
        };
        let backend = &mut backends[index];
        if !backend.supports(request.claim, &request.strength) {
            dispositions.push(unavailable(request, AssessmentExecution::Unsupported));
            continue;
        }
        usage.backend_calls = usage.backend_calls.saturating_add(1);
        let reply = backend.assess(request.claim, &request.strength, &request.premises);
        if reply.evidence != request.premises {
            dispositions.push(refused(
                request,
                FailureCode::EvidenceMismatch,
                premises::substitutions(&request.premises, &reply.evidence),
                retains_independent,
            ));
            continue;
        }
        let (execution, decision) = match reply.execution {
            BackendExecution::Completed(decision) => {
                (AssessmentExecution::Completed, Some(decision))
            }
            BackendExecution::Refused => (AssessmentExecution::Refused, None),
            BackendExecution::ResourceIncomplete => (AssessmentExecution::ResourceIncomplete, None),
            BackendExecution::Failed => (AssessmentExecution::Failed, None),
        };
        dispositions.push(Disposition {
            request_identity: request.identity.clone(),
            claim: request.claim,
            strength: request.strength.clone(),
            execution,
            decision,
            premises: request.premises.clone(),
            failure: None,
        });
    }
    Ok(Report {
        dispositions,
        usage,
    })
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
    defects
}

fn refused(
    request: &ClaimRequest<'_>,
    code: FailureCode,
    defects: Vec<PremiseDefect>,
    retains_independent: bool,
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

fn preflight(requests: &[ClaimRequest<'_>], backends: usize) -> Usage {
    let mut usage = Usage {
        requests: requests.len(),
        backends,
        ..Usage::default()
    };
    for request in requests {
        usage.components = usage.components.saturating_add(2);
        usage.bytes = usage
            .bytes
            .saturating_add(request.identity.len())
            .saturating_add(request.strength.len())
            .saturating_add(request.premises.admitted_fragment.identity.len())
            .saturating_add(request.premises.admitted_fragment.bytes.len())
            .saturating_add(request.premises.backend.identity.len())
            .saturating_add(request.premises.backend.version.len());
        for assumption in &request.premises.assumptions {
            visit_component(&assumption.component, &mut usage);
            usage.components = usage
                .components
                .saturating_add(assumption.dependencies.len());
            for dependency in &assumption.dependencies {
                usage.bytes = usage.bytes.saturating_add(dependency.len());
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
            visit_component(component, &mut usage);
        }
        if let Some(derived) = &request.premises.derived_artifact {
            visit_component(&derived.artifact, &mut usage);
            for component in &derived.source_tuple {
                visit_component(component, &mut usage);
            }
        }
    }
    usage
}

fn visit_component(component: &premises::Component, usage: &mut Usage) {
    usage.components = usage.components.saturating_add(1);
    usage.bytes = usage
        .bytes
        .saturating_add(component.identity.len())
        .saturating_add(component.bytes.len());
}

fn resource_refusal(usage: Usage) -> Refusal {
    let envelope = ProtocolRefusal::builder(
        RefusalCode::of(&RESOURCE_INCOMPLETE),
        CatalogSelection::local(),
        OriginatingStage::ResourceExecution,
    )
    .unavailable_location(Unavailability::NotApplicableToStage)
    .emitting_nothing();
    Refusal {
        usage,
        envelope: Box::new(envelope),
    }
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
        premises::PremiseDefectKind::Substituted => "substituted",
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
const BACKEND_MISMATCH: CodeEntry = CodeEntry {
    provenance: Provenance::Minted,
    code: "claims.backend-mismatch",
    domain: RefusalDomain::Assessment,
    retry: RetryDecision::AfterSelectionChange,
    summary: "the selected backend identity exists only at another version",
};
const BACKEND_AMBIGUOUS: CodeEntry = CodeEntry {
    provenance: Provenance::Minted,
    code: "claims.backend-ambiguous",
    domain: RefusalDomain::Assessment,
    retry: RetryDecision::AfterSelectionChange,
    summary: "more than one backend claims the exact selected identity and version",
};
const DUPLICATE_REQUEST: CodeEntry = CodeEntry {
    provenance: Provenance::Minted,
    code: "claims.request-duplicate",
    domain: RefusalDomain::Assessment,
    retry: RetryDecision::AfterSelectionChange,
    summary: "the same exact claim request identity was offered more than once",
};
const EVIDENCE_MISMATCH: CodeEntry = CodeEntry {
    provenance: Provenance::Minted,
    code: "claims.evidence-mismatch",
    domain: RefusalDomain::Assessment,
    retry: RetryDecision::Never,
    summary: "the backend returned evidence other than the exact selected premises",
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
    entries: &[
        INVALID_PREMISES,
        BACKEND_MISMATCH,
        BACKEND_AMBIGUOUS,
        DUPLICATE_REQUEST,
        EVIDENCE_MISMATCH,
        RESOURCE_INCOMPLETE,
    ],
};
