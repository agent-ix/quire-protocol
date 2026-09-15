// SPDX-License-Identifier: AGPL-3.0-or-later
//! Shared, producer-owned facts consumed by assessment and result construction.
//!
//! This module is deliberately neutral: conformance assesses these inputs and
//! the result owner serializes them, but neither subsystem owns or re-derives
//! the QObs facts or the declared global-closure premise set.

use quire_observation::authority::{
    closure as observation_closure, completeness, observation as observation_record, position,
    progress,
};
use quire_observation::{AdmittedBundleKey, QualifiedSubject, Relationship};
use quire_spec_language::protocol_artifact::native_temporal::v2 as temporal_v2;
use quire_spec_language::protocol_artifact::wire::{BindingKind, ControlOperation};
use quire_spec_language::protocol_artifact::ProtocolNumber;
use quire_spec_language::state::{
    self as state_evaluation, EvaluationOutcome, EvaluationReport, StateView,
    ValueKind as StateValueKind,
};

use crate::bounds::inventory::{
    production_inventory, AccountingFault, Budget, BudgetDimension,
    OperationOutcome as BoundedOutcome, ResourceAccountant, Usage as ResourceAccountantUsage,
};
use crate::claims::dispatch::{
    self, ClaimRequest, Limits as DispatchLimits, Report as DispatchReport,
};
use crate::claims::Claim;
use crate::closure::GlobalConformanceClosure;
use crate::compensation::registration::Snapshot as CompensationSnapshot;
use crate::compensation::{
    algebra::{AttemptOutcome, EffectDisposition, OrderingViolation},
    graph::{
        MissingEvidence as CompensationMissing, Outcome as CompensationOutcome,
        Refusal as CompensationRefusal, Snapshot as CompensationState,
        Transition as CompensationTransition,
    },
    registration::Registration as CompensationRegistration,
};
use crate::conformance::{
    ActivatedAssessmentError, ActivationFact, Kernel as ConformanceKernel,
    KernelRefusal as ConformanceKernelRefusal, Limits as ConformanceLimits,
};
use crate::model::v2::{
    link as link_v2, CompensationRegistrationInput, DefinitionIdentity, DefinitionRegistry,
    LinkedAssessment, LinkedProtocol, RuntimeBinding, SlotCompatibility, SlotHandle,
};
use crate::outcome::ClaimOutcome;
use crate::producer_contracts::{
    ProducerResolution, ProducerResolutionEvidence, ProducerResolutionRequest,
};
use crate::refusal::catalog::{
    CatalogSelection, CodeEntry, CodeModule, Provenance, RefusalCode, RetainedBytes,
};
use crate::refusal::{
    AffectedIdentity, Prerequisite, PrerequisiteState, ProtocolCause, ProtocolRefusal,
    ProtocolStage, RefusalAxis, RefusalDomain, ResourceUsage, RetryDecision, Unavailability,
};
use crate::repro::ReproductionRecord;
use crate::result::{self, ClaimKind};

/// One claim outcome offered to an assessment bundle.
///
/// The row remains generic because canonical result-row construction belongs to
/// the result owner. The engine owns only the exact linked assessment and the
/// one-row-or-one-refusal cardinality shared by every claim producer.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ClaimRow<Row> {
    /// Exact requested claim this outcome answers.
    pub claim: Claim,
    /// Either the claim's canonical row or its claim-local refusal.
    pub outcome: ClaimOutcome<Row>,
}

/// Independently owned facts needed to construct one canonical claim row.
///
/// This deliberately omits execution, truth, claim identity/strength/backend,
/// assumptions, progress/closure axes, completeness, and global conformance.
/// The Engine derives or reuses those fields from the exact conformance and
/// dispatch stages executed inside the same atomic assessment operation, so a
/// production caller cannot restate a complete result-owner [`result::Input`]
/// and substitute values for the preceding stages.
///
/// ```compile_fail,E0603
/// // The complete row input is an Engine-private transition value.
/// use quire_protocol::assessment::CanonicalRowInput;
/// ```
#[derive(Clone, Debug)]
pub struct CanonicalRowFacts<'a> {
    /// Strict QSL checked-predicate owner view selected for this assessment.
    pub checked_predicate:
        &'a quire_spec_language::protocol_artifact::checked_predicate::ValidatedCheckedPredicate,
    /// Strict QSL temporal-subject owner view selected for this assessment.
    pub temporal_subject:
        &'a quire_spec_language::protocol_artifact::temporal_subject::ValidatedTemporalSubject,
    /// Typed settlement and exact supporting identities.
    pub settlement: result::Settlement,
    /// Typed activation state.
    pub activation: result::Activation,
    /// Typed role participation state.
    pub participation: result::Participation,
    /// Typed observation-adequacy state.
    pub observation_adequacy: result::Adequacy,
    /// Typed protocol-adequacy state.
    pub protocol_adequacy: result::ProtocolAdequacy,
    /// Complete decision-support population.
    pub decision_support: Vec<result::DecisionSupport>,
    /// Exact limitations retained for this claim.
    pub limitations: Vec<result::Limitation>,
    /// Exact selected model artifact; the Engine verifies it belongs to its
    /// immutable semantic selection closure before retaining its identity.
    pub model_artifact: &'a quire_spec_language::protocol_artifact::wire::ArtifactRef,
    /// Exact QObs runtime input; the Engine verifies it is one of its admitted
    /// runtime bindings and derives the reversible result-field carrier.
    pub runtime_input: &'a QualifiedSubject,
    /// Caller-selected deterministic inputs needed to reproduce this run.
    pub reproduction: ReproductionRecord,
    /// Optional validated direct correction edge.
    pub correction: Option<result::CorrectionInput<'a>>,
    /// Result-owner limits, which may lower but never raise owner maxima.
    pub limits: result::Limits,
}

/// The exact QObs event and QSL v2 request selected for one activation site.
///
/// The request is already constructor-private and strict-read by QSL. This
/// boundary only binds its opaque trigger bytes to the QObs-authoritative
/// subject and proves that the temporal declaration belongs to this immutable
/// protocol definition; it does not interpret either owner value.
#[derive(Clone, Copy)]
pub struct ActivationBinding<'a> {
    subject: &'a QualifiedSubject,
    request: &'a temporal_v2::ValidatedRequest,
}

impl core::fmt::Debug for ActivationBinding<'_> {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter
            .debug_struct("ActivationBinding")
            .field("subject", self.subject)
            .field("temporal_declaration", &self.request.declaration())
            .finish_non_exhaustive()
    }
}

impl<'a> ActivationBinding<'a> {
    /// Bind one strict QSL v2 request to the selected QObs subject and static
    /// definition identity.
    pub fn new(
        identity: &DefinitionIdentity,
        subject: &'a QualifiedSubject,
        request: &'a temporal_v2::ValidatedRequest,
    ) -> Result<Self, BundleRefusal> {
        if request.semantic_trigger() != subject.identity().as_bytes() {
            return Err(BundleRefusal::ActivationSubjectMismatch);
        }
        if !identity
            .temporal_bindings()
            .iter()
            .any(|binding| binding.declaration == request.declaration())
        {
            return Err(BundleRefusal::ActivationDeclarationMismatch);
        }
        Ok(Self { subject, request })
    }

    #[must_use]
    pub const fn subject(&self) -> &'a QualifiedSubject {
        self.subject
    }

    #[must_use]
    pub const fn request(&self) -> &'a temporal_v2::ValidatedRequest {
        self.request
    }
}

/// Why an exact linked assessment cannot become one complete claim bundle.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BundleRefusal {
    /// No claim was selected for this assessment request.
    EmptyRequest,
    /// The caller selected one claim more than once.
    DuplicateRequestedClaim,
    /// A row names a claim not selected by this exact engine.
    UnexpectedClaim,
    /// More than one row answers one selected claim.
    DuplicateClaimRow,
    /// A selected claim has neither a canonical row nor a claim-local refusal.
    MissingClaimRow,
    /// The admitted linked context cannot derive its static definition identity.
    InvalidDefinitionIdentity,
    /// The exact linked context cannot produce its static QSL graph.
    InvalidLinkedProtocol,
    /// Linked graph and compensation state cannot form one conformance kernel.
    InvalidConformanceKernel,
    /// A canonical result row was offered under a different requested claim.
    CanonicalClaimMismatch,
    /// A canonical input borrows a QSL view from another admitted package.
    CanonicalPackageMismatch,
    /// The ordered claim-dispatch report differs from this Engine request or its rows.
    DispatchReportMismatch,
    /// QSL v2 opaque trigger bytes differ from the selected QObs subject.
    ActivationSubjectMismatch,
    /// The QSL v2 temporal declaration is not selected by this protocol definition.
    ActivationDeclarationMismatch,
}

macro_rules! bundle_code {
    ($name:ident, $code:literal, $domain:expr) => {
        const $name: CodeEntry = CodeEntry {
            provenance: Provenance::Minted,
            code: $code,
            domain: $domain,
            retry: RetryDecision::AfterSelectionChange,
            summary: "the assessment claim population cannot form one complete bundle",
        };
    };
}

bundle_code!(
    BUNDLE_EMPTY_REQUEST,
    "assessment-input.bundle-empty-request",
    RefusalDomain::Assessment
);
bundle_code!(
    BUNDLE_DUPLICATE_REQUEST,
    "assessment-input.bundle-duplicate-request",
    RefusalDomain::Assessment
);
bundle_code!(
    BUNDLE_UNEXPECTED_CLAIM,
    "assessment-input.bundle-unexpected-claim",
    RefusalDomain::Assessment
);
bundle_code!(
    BUNDLE_DUPLICATE_ROW,
    "assessment-input.bundle-duplicate-row",
    RefusalDomain::Assessment
);
bundle_code!(
    BUNDLE_MISSING_ROW,
    "assessment-input.bundle-missing-row",
    RefusalDomain::Assessment
);
bundle_code!(
    BUNDLE_INVALID_IDENTITY,
    "assessment-representation.bundle-invalid-identity",
    RefusalDomain::Representation
);
bundle_code!(
    BUNDLE_INVALID_LINKED_PROTOCOL,
    "assessment-representation.bundle-invalid-linked-protocol",
    RefusalDomain::Representation
);
bundle_code!(
    BUNDLE_INVALID_CONFORMANCE_KERNEL,
    "assessment-conformance.bundle-invalid-conformance-kernel",
    RefusalDomain::Assessment
);
const ASSESSMENT_ACCOUNTING_FAULT: CodeEntry = CodeEntry {
    provenance: Provenance::Minted,
    code: "assessment-accounting.fault",
    domain: RefusalDomain::Assessment,
    retry: RetryDecision::Never,
    summary: "the Protocol resource accountant could not represent assessment work",
};
bundle_code!(
    BUNDLE_CLAIM_MISMATCH,
    "assessment-result.bundle-claim-mismatch",
    RefusalDomain::Result
);
bundle_code!(
    BUNDLE_PACKAGE_MISMATCH,
    "assessment-result.bundle-package-mismatch",
    RefusalDomain::Result
);
bundle_code!(
    BUNDLE_DISPATCH_MISMATCH,
    "assessment-result.bundle-dispatch-mismatch",
    RefusalDomain::Result
);
const RESULT_ACCOUNTING_FAULT: CodeEntry = CodeEntry {
    provenance: Provenance::Minted,
    code: "assessment-result.accounting-fault",
    domain: RefusalDomain::Result,
    retry: RetryDecision::Never,
    summary: "the Protocol resource accountant could not represent result work",
};
bundle_code!(
    BUNDLE_ACTIVATION_SUBJECT,
    "assessment-activation.subject-mismatch",
    RefusalDomain::Assessment
);
bundle_code!(
    BUNDLE_ACTIVATION_DECLARATION,
    "assessment-activation.declaration-mismatch",
    RefusalDomain::Assessment
);
/// Refusal codes minted by the assessment-bundle orchestration boundary.
pub const CODES: CodeModule = CodeModule {
    root: "assessment-input",
    owner: "Task-006",
    module_path: "quire_protocol::assessment",
    entries: &[
        BUNDLE_EMPTY_REQUEST,
        BUNDLE_DUPLICATE_REQUEST,
        BUNDLE_UNEXPECTED_CLAIM,
        BUNDLE_DUPLICATE_ROW,
        BUNDLE_MISSING_ROW,
    ],
};

/// Representation-stage assessment setup codes.
pub const REPRESENTATION_CODES: CodeModule = CodeModule {
    root: "assessment-representation",
    owner: "Task-006",
    module_path: "quire_protocol::assessment",
    entries: &[BUNDLE_INVALID_IDENTITY, BUNDLE_INVALID_LINKED_PROTOCOL],
};

/// Conformance-stage assessment setup codes.
pub const CONFORMANCE_CODES: CodeModule = CodeModule {
    root: "assessment-conformance",
    owner: "Task-006",
    module_path: "quire_protocol::assessment",
    entries: &[BUNDLE_INVALID_CONFORMANCE_KERNEL],
};

/// Assessment-stage resource-accounting invariant codes.
pub const ACCOUNTING_CODES: CodeModule = CodeModule {
    root: "assessment-accounting",
    owner: "Task-006",
    module_path: "quire_protocol::assessment",
    entries: &[ASSESSMENT_ACCOUNTING_FAULT],
};

/// Result-assembly assessment codes.
pub const RESULT_CODES: CodeModule = CodeModule {
    root: "assessment-result",
    owner: "Task-006",
    module_path: "quire_protocol::assessment",
    entries: &[
        BUNDLE_CLAIM_MISMATCH,
        BUNDLE_PACKAGE_MISMATCH,
        BUNDLE_DISPATCH_MISMATCH,
        RESULT_ACCOUNTING_FAULT,
    ],
};

/// Conformance-stage activation-binding codes.
pub const ACTIVATION_CODES: CodeModule = CodeModule {
    root: "assessment-activation",
    owner: "Task-006",
    module_path: "quire_protocol::assessment",
    entries: &[BUNDLE_ACTIVATION_SUBJECT, BUNDLE_ACTIVATION_DECLARATION],
};

fn outcome_refusal(error: BundleRefusal) -> ProtocolRefusal {
    let (entry, stage) = match error {
        BundleRefusal::EmptyRequest => (&BUNDLE_EMPTY_REQUEST, ProtocolStage::ClaimDispatch),
        BundleRefusal::DuplicateRequestedClaim => {
            (&BUNDLE_DUPLICATE_REQUEST, ProtocolStage::ClaimDispatch)
        }
        BundleRefusal::UnexpectedClaim => (&BUNDLE_UNEXPECTED_CLAIM, ProtocolStage::ClaimDispatch),
        BundleRefusal::DuplicateClaimRow => (&BUNDLE_DUPLICATE_ROW, ProtocolStage::ClaimDispatch),
        BundleRefusal::MissingClaimRow => (&BUNDLE_MISSING_ROW, ProtocolStage::ClaimDispatch),
        BundleRefusal::InvalidDefinitionIdentity => {
            (&BUNDLE_INVALID_IDENTITY, ProtocolStage::Representation)
        }
        BundleRefusal::InvalidLinkedProtocol => (
            &BUNDLE_INVALID_LINKED_PROTOCOL,
            ProtocolStage::Representation,
        ),
        BundleRefusal::InvalidConformanceKernel => (
            &BUNDLE_INVALID_CONFORMANCE_KERNEL,
            ProtocolStage::Conformance,
        ),
        BundleRefusal::CanonicalClaimMismatch => {
            (&BUNDLE_CLAIM_MISMATCH, ProtocolStage::ResultAssembly)
        }
        BundleRefusal::CanonicalPackageMismatch => {
            (&BUNDLE_PACKAGE_MISMATCH, ProtocolStage::ResultAssembly)
        }
        BundleRefusal::DispatchReportMismatch => {
            (&BUNDLE_DISPATCH_MISMATCH, ProtocolStage::ResultAssembly)
        }
        BundleRefusal::ActivationSubjectMismatch => {
            (&BUNDLE_ACTIVATION_SUBJECT, ProtocolStage::Conformance)
        }
        BundleRefusal::ActivationDeclarationMismatch => {
            (&BUNDLE_ACTIVATION_DECLARATION, ProtocolStage::Conformance)
        }
    };
    let code = RefusalCode::of(entry);
    let mut builder = ProtocolRefusal::builder(
        code,
        CatalogSelection::local(),
        stage,
        RefusalAxis::from_registered_code(code),
        bundle_cause(error, code),
        ResourceUsage::protocol("quire.protocol/assessment-usage/1", &[]),
    )
    .expect("assessment bundle code belongs to claim dispatch")
    .unavailable_location(Unavailability::NotApplicableToStage);
    match stage {
        ProtocolStage::Representation => {
            builder = builder.prerequisite(Prerequisite::new(
                ProtocolStage::Admission,
                PrerequisiteState::Established,
            ));
        }
        ProtocolStage::Conformance => {
            builder = builder.prerequisite(Prerequisite::new(
                ProtocolStage::Representation,
                PrerequisiteState::Established,
            ));
        }
        ProtocolStage::ResultAssembly => {
            builder = builder.prerequisite(Prerequisite::new(
                ProtocolStage::ClaimDispatch,
                PrerequisiteState::Established,
            ));
        }
        ProtocolStage::ContractResolution
        | ProtocolStage::Admission
        | ProtocolStage::Compensation
        | ProtocolStage::ClaimDispatch
        | ProtocolStage::ResultRead
        | ProtocolStage::Projection
        | ProtocolStage::EvidenceHandoff => {}
    }
    builder.emitting_nothing()
}

fn bundle_cause(error: BundleRefusal, code: RefusalCode) -> crate::refusal::RefusalCause {
    let code = RetainedBytes::new(code.as_str().as_bytes());
    match error {
        BundleRefusal::EmptyRequest | BundleRefusal::MissingClaimRow => {
            ProtocolCause::MissingDefinition { definition: code }
        }
        BundleRefusal::DuplicateRequestedClaim | BundleRefusal::DuplicateClaimRow => {
            ProtocolCause::ConflictingDefinition {
                definition: code.clone(),
                conflicting_definition: code,
            }
        }
        BundleRefusal::InvalidDefinitionIdentity
        | BundleRefusal::InvalidLinkedProtocol
        | BundleRefusal::InvalidConformanceKernel => {
            ProtocolCause::IncompleteEvidence { evidence: code }
        }
        BundleRefusal::UnexpectedClaim
        | BundleRefusal::CanonicalClaimMismatch
        | BundleRefusal::CanonicalPackageMismatch
        | BundleRefusal::DispatchReportMismatch
        | BundleRefusal::ActivationSubjectMismatch
        | BundleRefusal::ActivationDeclarationMismatch => ProtocolCause::IdentityMismatch {
            expected: RetainedBytes::new(b"engine-bound-assessment-value"),
            observed: code,
        },
    }
    .into()
}

#[derive(Clone, Copy)]
enum AccountingStage {
    Conformance,
    ClaimDispatch,
    ResultAssembly,
    ResultRead,
}

impl AccountingStage {
    const fn protocol_stage(self) -> ProtocolStage {
        match self {
            Self::Conformance => ProtocolStage::Conformance,
            Self::ClaimDispatch => ProtocolStage::ClaimDispatch,
            Self::ResultAssembly => ProtocolStage::ResultAssembly,
            Self::ResultRead => ProtocolStage::ResultRead,
        }
    }

    const fn entry(self) -> &'static CodeEntry {
        match self {
            Self::Conformance | Self::ClaimDispatch => &ASSESSMENT_ACCOUNTING_FAULT,
            Self::ResultAssembly | Self::ResultRead => &RESULT_ACCOUNTING_FAULT,
        }
    }

    const fn prerequisite(self) -> ProtocolStage {
        match self {
            Self::Conformance => ProtocolStage::Representation,
            Self::ClaimDispatch => ProtocolStage::Conformance,
            Self::ResultAssembly => ProtocolStage::ClaimDispatch,
            Self::ResultRead => ProtocolStage::ResultAssembly,
        }
    }
}

fn accounting_refusal(
    fault: AccountingFault,
    stage: AccountingStage,
    usage: ResourceAccountantUsage,
) -> ProtocolRefusal {
    let (affected, axis) = encode_accounting_fault(fault);
    let code = RefusalCode::of(stage.entry());
    let measures = BudgetDimension::ALL
        .iter()
        .map(|dimension| (dimension.as_str(), usage.consumed(*dimension)))
        .collect::<Vec<_>>();
    ProtocolRefusal::builder(
        code,
        CatalogSelection::local(),
        stage.protocol_stage(),
        RefusalAxis::new(&axis).expect("the typed accounting-fault axis is non-empty"),
        ProtocolCause::IncompleteEvidence {
            evidence: RetainedBytes::new(&axis),
        }
        .into(),
        ResourceUsage::protocol("quire.protocol/assessment-usage/1", &measures),
    )
    .expect("the accounting-fault code belongs to its typed stage")
    .identity(AffectedIdentity::retained(&affected))
    .prerequisite(Prerequisite::new(
        stage.prerequisite(),
        PrerequisiteState::Established,
    ))
    .unavailable_location(Unavailability::NotApplicableToStage)
    .emitting_nothing()
}

fn encode_accounting_fault(fault: AccountingFault) -> (Vec<u8>, Vec<u8>) {
    let mut axis = b"accounting-fault\0".to_vec();
    let affected = match fault {
        AccountingFault::UnregisteredSite { site } => {
            axis.extend_from_slice(b"unregistered-site\0");
            site.as_str().as_bytes().to_vec()
        }
        AccountingFault::CounterOverflow {
            site,
            consumed,
            requested,
        } => {
            axis.extend_from_slice(b"counter-overflow\0");
            axis.extend_from_slice(&consumed.to_be_bytes());
            axis.extend_from_slice(&requested.to_be_bytes());
            site.as_str().as_bytes().to_vec()
        }
        AccountingFault::CounterUnderflow {
            site,
            consumed,
            requested,
        } => {
            axis.extend_from_slice(b"counter-underflow\0");
            axis.extend_from_slice(&consumed.to_be_bytes());
            axis.extend_from_slice(&requested.to_be_bytes());
            site.as_str().as_bytes().to_vec()
        }
        AccountingFault::ClockRegressed {
            unit,
            phase,
            previous,
        } => {
            axis.extend_from_slice(b"clock-regressed\0");
            axis.extend_from_slice(unit.as_str().as_bytes());
            axis.push(0);
            axis.push(match phase {
                crate::bounds::Phase::BeforeUnit => 0,
                crate::bounds::Phase::AfterUnit => 1,
            });
            axis.extend_from_slice(&previous.as_nanos().to_be_bytes());
            unit.as_str().as_bytes().to_vec()
        }
        AccountingFault::ElapsedTimeOverflow { observed } => {
            axis.extend_from_slice(b"elapsed-time-overflow\0");
            axis.extend_from_slice(&observed.as_nanos().to_be_bytes());
            b"assessment/request/deadline".to_vec()
        }
        AccountingFault::SiteDimensionMismatch {
            site,
            declared,
            required,
        } => {
            axis.extend_from_slice(b"site-dimension-mismatch\0");
            axis.extend_from_slice(declared.as_str().as_bytes());
            axis.push(0);
            axis.extend_from_slice(required.as_str().as_bytes());
            site.as_str().as_bytes().to_vec()
        }
        AccountingFault::TemporalDimensionChargedStatically { site, dimension } => {
            axis.extend_from_slice(b"temporal-dimension-charged-statically\0");
            axis.extend_from_slice(dimension.as_str().as_bytes());
            site.as_str().as_bytes().to_vec()
        }
    };
    axis.extend_from_slice(&affected);
    (affected, axis)
}

/// One non-retargetable assessment request.
///
/// The engine borrows the constructor-validated linked assessment and retains
/// its derived FR-288 identity at construction. It exposes no setter for either
/// value, so a later row assembly cannot select another protocol while reusing
/// this request's claim state.
#[derive(Clone, Debug)]
pub struct Engine<'a> {
    context: LinkedAssessment<'a>,
    identity: DefinitionIdentity,
    linked: LinkedProtocol<'a>,
    compensation_inputs: Vec<CompensationRegistrationInput<'a>>,
    compensation: CompensationSnapshot,
    compensation_state: CompensationState,
    conformance: ConformanceKernel,
    activation: Option<ActivationBinding<'a>>,
    requested: Vec<Claim>,
}

impl<'a> Engine<'a> {
    /// Start one assessment using the shared public operation outcome algebra.
    pub fn start(
        context: LinkedAssessment<'a>,
        requested: Vec<Claim>,
    ) -> crate::outcome::OperationOutcome<Self> {
        match Self::new(context, requested) {
            Ok(engine) => crate::outcome::OperationOutcome::Completed(engine),
            Err(error) => crate::outcome::OperationOutcome::Refused(outcome_refusal(error)),
        }
    }
    /// Start one assessment over an exact linked context and claim population.
    pub fn new(
        context: LinkedAssessment<'a>,
        requested: Vec<Claim>,
    ) -> Result<Self, BundleRefusal> {
        Self::new_bound(context, requested, None)
    }

    /// Start one assessment with an exact QSL-v2/QObs activation binding.
    pub fn new_activated(
        context: LinkedAssessment<'a>,
        requested: Vec<Claim>,
        subject: &'a QualifiedSubject,
        request: &'a temporal_v2::ValidatedRequest,
    ) -> Result<Self, BundleRefusal> {
        let identity = context
            .definition_identity()
            .map_err(|_| BundleRefusal::InvalidDefinitionIdentity)?;
        let activation = ActivationBinding::new(&identity, subject, request)?;
        Self::new_bound(context, requested, Some(activation))
    }

    fn new_bound(
        context: LinkedAssessment<'a>,
        requested: Vec<Claim>,
        activation: Option<ActivationBinding<'a>>,
    ) -> Result<Self, BundleRefusal> {
        validate_requested(&requested)?;
        let identity = context
            .definition_identity()
            .map_err(|_| BundleRefusal::InvalidDefinitionIdentity)?;
        let mut definitions = DefinitionRegistry::default();
        let linked =
            link_v2(context, &mut definitions).map_err(|_| BundleRefusal::InvalidLinkedProtocol)?;
        let compensation_inputs = context.compensation_registration_inputs();
        // Runtime effect bindings are eligible inputs, not registrations.
        // A registration is committed only after its exact admitted guard has
        // completed true through `activate_compensation`.
        let compensation = CompensationSnapshot::default();
        let conformance = ConformanceKernel::new(&linked, &compensation, &[], &[])
            .map_err(|_| BundleRefusal::InvalidConformanceKernel)?;
        Ok(Self {
            context,
            identity,
            linked,
            compensation_inputs,
            compensation,
            compensation_state: CompensationState::default(),
            conformance,
            activation,
            requested,
        })
    }

    /// Returns every exact QSL-compensation/QObs-effect registration input
    /// captured when this non-retargetable request was created.
    #[must_use]
    pub fn compensation_inputs(&self) -> &[CompensationRegistrationInput<'a>] {
        &self.compensation_inputs
    }

    /// Returns the exact QSL graph and static compensation definitions linked
    /// from this request's immutable context.
    #[must_use]
    pub const fn linked(&self) -> &LinkedProtocol<'a> {
        &self.linked
    }

    /// Returns the one linked-graph/compensation kernel for both conformance
    /// frontends of this assessment.
    #[must_use]
    pub const fn conformance(&self) -> &ConformanceKernel {
        &self.conformance
    }

    /// Returns the replay-idempotent registrations derived for this request.
    #[must_use]
    pub const fn compensation(&self) -> &CompensationSnapshot {
        &self.compensation
    }

    /// Returns the owned graph-derived activation, attempt, effect, commit,
    /// ordering and recovery state committed by this non-retargetable request.
    #[must_use]
    pub const fn compensation_state(&self) -> &CompensationState {
        &self.compensation_state
    }

    /// Returns the immutable activation pair where this assessment is event-triggered.
    #[must_use]
    pub const fn activation(&self) -> Option<ActivationBinding<'a>> {
        self.activation
    }

    /// Derive the exact producer-resolution identity for this immutable request.
    ///
    /// The encoding is componentwise and length-prefixed. Runtime bindings are
    /// ordered by their admitted static slot, so legal arrival permutations do
    /// not change the identity. The activation request remains distinct from
    /// the static/runtime context and is included when this Engine is activated.
    #[must_use]
    pub fn producer_resolution_request(&self) -> ProducerResolutionRequest {
        let mut encoded = Vec::new();
        append_component(&mut encoded, b"quire-protocol.producer-resolution/1");
        append_component(
            &mut encoded,
            format!("{:x}", self.context.package().digest()).as_bytes(),
        );
        append_component(
            &mut encoded,
            &self.identity.declaration_handle().to_be_bytes(),
        );

        let static_selection = self.identity.static_binding();
        append_component(&mut encoded, static_selection.identity.as_bytes());
        append_component(&mut encoded, static_selection.revision.as_bytes());
        append_component(&mut encoded, &static_selection.digest);

        let mut bindings = self.context.runtime().bindings().iter().collect::<Vec<_>>();
        bindings.sort_by_key(|binding| binding.slot());
        append_count(&mut encoded, bindings.len());
        for binding in bindings {
            append_component(&mut encoded, &binding.slot().get().to_be_bytes());
            if let Some(subject) = binding.subject_value() {
                append_component(&mut encoded, b"subject");
                append_subject(&mut encoded, subject);
            } else if let Some(relationship) = binding.relationship_value() {
                append_component(&mut encoded, b"relationship");
                append_relationship(&mut encoded, relationship);
            }
        }

        if let Some(activation) = self.activation {
            append_component(&mut encoded, b"activated");
            append_subject(&mut encoded, activation.subject());
            append_component(
                &mut encoded,
                format!("{:x}", activation.request().document().digest()).as_bytes(),
            );
            append_component(
                &mut encoded,
                activation.request().document().identity().as_bytes(),
            );
            append_component(&mut encoded, activation.request().semantic_trigger());
        } else {
            append_component(&mut encoded, b"not-activated");
        }

        ProducerResolutionRequest::new(&encoded)
            .expect("the versioned producer-resolution identity prefix is nonempty")
    }

    /// Dispatch this Engine's exact requested claims from an admitted
    /// activation-derived conformance assessment.
    ///
    /// This is the production bridge for Task-005: it obtains the report from
    /// the same graph/compensation kernel that owns the request rather than
    /// accepting a report selected by a caller. The report-taking evaluator is
    /// crate-private and exists only for semantic model tests.
    #[allow(
        clippy::too_many_arguments,
        reason = "the private transition receives each independently owned conformance and dispatch input without a shadow aggregate"
    )]
    pub fn dispatch_activated(
        &self,
        requests: &[ClaimRequest<'_>],
        facts: &[ActivationFact<'_>],
        axes: AxisInputs<'_>,
        completeness: &completeness::View,
        global_conformance: &GlobalConformanceInput,
        execution: crate::closure::AssessmentExecution,
        conformance_limits: ConformanceLimits,
        dispatch_limits: DispatchLimits,
        budget: Budget,
    ) -> crate::outcome::OperationOutcome<DispatchReport> {
        let inventory = production_inventory()
            .expect("the compile-time Protocol resource inventory must reconcile");
        let mut resources = ResourceAccountant::new(budget, &inventory);
        match self.dispatch_activated_bounded(
            requests,
            facts,
            axes,
            completeness,
            global_conformance,
            execution,
            conformance_limits,
            dispatch_limits,
            &mut resources,
        ) {
            BoundedOutcome::Completed((_, dispatch)) => {
                crate::outcome::OperationOutcome::Completed(dispatch)
            }
            BoundedOutcome::DomainError(refusal) => {
                crate::outcome::OperationOutcome::Refused(refusal)
            }
            BoundedOutcome::ResourceIncomplete(incomplete) => {
                crate::outcome::OperationOutcome::ResourceIncomplete(incomplete)
            }
            BoundedOutcome::AccountingFault(fault) => crate::outcome::OperationOutcome::Refused(
                accounting_refusal(fault, AccountingStage::ClaimDispatch, resources.usage()),
            ),
        }
    }

    #[allow(
        clippy::too_many_arguments,
        reason = "the public orchestration keeps owner inputs typed and separate instead of introducing a caller-built wire-shaped aggregate"
    )]
    fn dispatch_activated_bounded(
        &self,
        requests: &[ClaimRequest<'_>],
        facts: &[ActivationFact<'_>],
        axes: AxisInputs<'_>,
        completeness: &completeness::View,
        global_conformance: &GlobalConformanceInput,
        execution: crate::closure::AssessmentExecution,
        conformance_limits: ConformanceLimits,
        dispatch_limits: DispatchLimits,
        resources: &mut ResourceAccountant<'_>,
    ) -> BoundedOutcome<(crate::conformance::Report, DispatchReport), ProtocolRefusal> {
        if let Err(error) = validate_dispatch_request(&self.requested, requests) {
            return BoundedOutcome::DomainError(outcome_refusal(error));
        }
        let report = match self.conformance.assess_activated_bounded(
            facts,
            axes,
            completeness,
            global_conformance,
            execution,
            conformance_limits,
            resources,
        ) {
            BoundedOutcome::Completed(report) => report,
            BoundedOutcome::DomainError(error) => {
                return BoundedOutcome::DomainError(activated_assessment_refusal(error));
            }
            BoundedOutcome::ResourceIncomplete(incomplete) => {
                return BoundedOutcome::ResourceIncomplete(incomplete);
            }
            BoundedOutcome::AccountingFault(fault) => {
                return BoundedOutcome::DomainError(accounting_refusal(
                    fault,
                    AccountingStage::Conformance,
                    resources.usage(),
                ));
            }
        };
        match dispatch::dispatch_bounded(requests, &report, dispatch_limits, resources) {
            BoundedOutcome::Completed(dispatch) => BoundedOutcome::Completed((report, dispatch)),
            BoundedOutcome::DomainError(error) => {
                BoundedOutcome::DomainError(error.envelope().clone())
            }
            BoundedOutcome::ResourceIncomplete(incomplete) => {
                BoundedOutcome::ResourceIncomplete(incomplete)
            }
            BoundedOutcome::AccountingFault(fault) => BoundedOutcome::DomainError(
                accounting_refusal(fault, AccountingStage::ClaimDispatch, resources.usage()),
            ),
        }
    }

    /// Evaluate the exact admitted guard and transactionally create every
    /// registration paired with the selected established forward effect.
    ///
    /// False, incomplete, refusing, exhausted, and non-Boolean evaluations
    /// add no registration. The complete QSL report is returned unchanged.
    pub fn activate_compensation(
        &mut self,
        compensation_handle: u32,
        view: &StateView,
        limits: state_evaluation::Limits,
    ) -> CompensationOutcome {
        let Some(definition) = self.linked.compensation(compensation_handle) else {
            return CompensationOutcome::Refused(CompensationRefusal::UnknownCompensation(
                compensation_handle,
            ));
        };
        match evaluate_boolean(
            self.context.package(),
            self.identity.declaration_handle(),
            definition.definition().guard.clone(),
            view,
            limits,
        ) {
            BooleanEvaluation::False(report) => CompensationOutcome::GuardFalse(report),
            BooleanEvaluation::Source(report) => CompensationOutcome::Source(report),
            BooleanEvaluation::Unsupported(report) => CompensationOutcome::Unsupported(report),
            BooleanEvaluation::True(report) => {
                let mut matching = self
                    .compensation_inputs
                    .iter()
                    .filter(|input| input.compensation_handle() == compensation_handle)
                    .copied()
                    .peekable();
                if matching.peek().is_none() {
                    return CompensationOutcome::UnavailableAfterEvaluation {
                        missing: CompensationMissing::ForwardEffect,
                        report,
                    };
                }
                let mut registration_candidate = self.compensation.clone();
                let mut state_candidate = self.compensation_state.clone();
                let activation = match state_candidate.retain_activation(
                    compensation_handle,
                    view.clone(),
                    report,
                ) {
                    Ok(transition) => transition,
                    Err(error) => return CompensationOutcome::Refused(error),
                };
                let mut created = false;
                for input in matching {
                    created |= registration_candidate.ensure(&self.identity, &input).1;
                }
                // The candidate registrations are derived exclusively from
                // this Engine's immutable linked/runtime capability. Replace
                // only that owned population before either state is committed,
                // so the conformance and compensation snapshots cannot diverge.
                let conformance = self
                    .conformance
                    .with_engine_compensation(&registration_candidate);
                self.compensation = registration_candidate;
                self.compensation_state = state_candidate;
                self.conformance = conformance;
                CompensationOutcome::Completed(
                    if created || activation == CompensationTransition::Applied {
                        CompensationTransition::Applied
                    } else {
                        CompensationTransition::Replay
                    },
                )
            }
        }
    }

    /// Admit one exact QObs compensation-attempt subject under QSL's authored
    /// signed-64 bound and retry expression.
    ///
    /// The initial attempt counts as position one and needs no retry relation.
    /// Every later distinct attempt must make the admitted retry expression
    /// complete true. An offered related effect must already be retained as an
    /// independently established compensation effect for this registration.
    /// Exact replay is a no-op; changed replay and one-over-bound candidates
    /// leave the prior snapshot unchanged.
    pub fn admit_compensation_attempt(
        &mut self,
        compensation_handle: u32,
        binding: &RuntimeBinding,
        established_effect: Option<&RuntimeBinding>,
        outcome: Option<AttemptOutcome>,
        retry_view: Option<&StateView>,
        limits: state_evaluation::Limits,
    ) -> CompensationOutcome {
        let Some(definition) = self.linked.compensation(compensation_handle) else {
            return CompensationOutcome::Refused(CompensationRefusal::UnknownCompensation(
                compensation_handle,
            ));
        };
        let Some(registration) = self.registration(compensation_handle) else {
            return CompensationOutcome::Unavailable(CompensationMissing::ForwardEffect);
        };
        let slot = SlotHandle::new(definition.definition().attempt_instance);
        let subject = match self.runtime_subject(slot, BindingKind::CompensationAttempt, binding) {
            Ok(subject) => subject,
            Err(error) => return CompensationOutcome::Refused(error),
        };
        let established_effect = match established_effect {
            Some(binding) => {
                let slot = SlotHandle::new(definition.definition().effect_instance);
                let subject =
                    match self.runtime_subject(slot, BindingKind::CompensationEffect, binding) {
                        Ok(subject) => subject,
                        Err(error) => return CompensationOutcome::Refused(error),
                    };
                let Some(effect) = self.compensation_state.effect(&subject) else {
                    return CompensationOutcome::Unavailable(
                        CompensationMissing::CompensationEffect,
                    );
                };
                if effect.registration() != &registration {
                    return CompensationOutcome::Refused(CompensationRefusal::ConflictingReplay);
                }
                Some(subject)
            }
            None => None,
        };
        if let Some(existing) = self.compensation_state.attempt(&subject) {
            if existing.registration() != &registration
                || existing.established_effect() != established_effect.as_ref()
                || existing.outcome() != outcome
            {
                return CompensationOutcome::Refused(CompensationRefusal::ConflictingReplay);
            }
            if existing.retry().is_none() {
                return CompensationOutcome::Completed(CompensationTransition::Replay);
            }
            let empty = StateView::default();
            let view = retry_view.unwrap_or(&empty);
            return match evaluate_boolean(
                self.context.package(),
                self.identity.declaration_handle(),
                definition.definition().retry.clone(),
                view,
                limits,
            ) {
                BooleanEvaluation::True(report)
                    if existing.retry_view() == Some(view) && existing.retry() == Some(&report) =>
                {
                    CompensationOutcome::Completed(CompensationTransition::Replay)
                }
                BooleanEvaluation::True(_) => {
                    CompensationOutcome::Refused(CompensationRefusal::ConflictingReplay)
                }
                BooleanEvaluation::False(report) => CompensationOutcome::Refused(
                    CompensationRefusal::RetryNotAuthorized(Box::new(report)),
                ),
                BooleanEvaluation::Source(report) => CompensationOutcome::Source(report),
                BooleanEvaluation::Unsupported(report) => CompensationOutcome::Unsupported(report),
            };
        }

        let retained = self.compensation_state.attempts_for(&registration);
        let attempted = match retained.checked_add(1) {
            Some(value) => value,
            None => {
                return CompensationOutcome::Refused(CompensationRefusal::RetryBoundExceeded {
                    bound: i64::MAX,
                    attempted: usize::MAX,
                })
            }
        };
        let bound = match definition.definition().maximum_attempts.checked() {
            Ok(ProtocolNumber::Integer(value)) if value.value() > 0 => value.value(),
            Ok(ProtocolNumber::Integer(_)) | Ok(ProtocolNumber::Rational(_)) | Err(_) => {
                return CompensationOutcome::Refused(CompensationRefusal::InvalidRetryBound)
            }
        };
        if i128::try_from(attempted).unwrap_or(i128::MAX) > i128::from(bound) {
            return CompensationOutcome::Refused(CompensationRefusal::RetryBoundExceeded {
                bound,
                attempted,
            });
        }

        let (retry_view, retry) = if retained == 0 {
            (None, None)
        } else {
            let empty = StateView::default();
            let view = retry_view.unwrap_or(&empty);
            match evaluate_boolean(
                self.context.package(),
                self.identity.declaration_handle(),
                definition.definition().retry.clone(),
                view,
                limits,
            ) {
                BooleanEvaluation::True(report) => (Some(view.clone()), Some(report)),
                BooleanEvaluation::False(report) => {
                    return CompensationOutcome::Refused(CompensationRefusal::RetryNotAuthorized(
                        Box::new(report),
                    ))
                }
                BooleanEvaluation::Source(report) => return CompensationOutcome::Source(report),
                BooleanEvaluation::Unsupported(report) => {
                    return CompensationOutcome::Unsupported(report)
                }
            }
        };

        let mut candidate = self.compensation_state.clone();
        candidate.push_attempt(
            registration,
            subject,
            established_effect,
            outcome,
            retry_view,
            retry,
        );
        self.compensation_state = candidate;
        CompensationOutcome::Completed(CompensationTransition::Applied)
    }

    /// Retain one independently established compensation effect.
    ///
    /// Successful sends, deliveries, attempts, and operation invocations cannot
    /// enter this method because the linked slot must be exactly QSL's
    /// `compensation_effect` requirement.
    pub fn record_compensation_effect(
        &mut self,
        compensation_handle: u32,
        binding: &RuntimeBinding,
        disposition: Option<EffectDisposition>,
    ) -> CompensationOutcome {
        let Some(definition) = self.linked.compensation(compensation_handle) else {
            return CompensationOutcome::Refused(CompensationRefusal::UnknownCompensation(
                compensation_handle,
            ));
        };
        let Some(registration) = self.registration(compensation_handle) else {
            return CompensationOutcome::Unavailable(CompensationMissing::ForwardEffect);
        };
        let slot = SlotHandle::new(definition.definition().effect_instance);
        let subject = match self.runtime_subject(slot, BindingKind::CompensationEffect, binding) {
            Ok(subject) => subject,
            Err(error) => return CompensationOutcome::Refused(error),
        };
        if let Some(existing) = self.compensation_state.effect(&subject) {
            return if existing.registration() == &registration
                && existing.disposition() == disposition
            {
                CompensationOutcome::Completed(CompensationTransition::Replay)
            } else {
                CompensationOutcome::Refused(CompensationRefusal::ConflictingReplay)
            };
        }
        let mut candidate = self.compensation_state.clone();
        candidate.push_effect(registration, subject, disposition);
        self.compensation_state = candidate;
        CompensationOutcome::Completed(CompensationTransition::Applied)
    }

    /// Retain only the exact commit node selected by this compensation.
    pub fn record_compensation_commit(
        &mut self,
        compensation_handle: u32,
        binding: &RuntimeBinding,
    ) -> CompensationOutcome {
        let Some(definition) = self.linked.compensation(compensation_handle) else {
            return CompensationOutcome::Refused(CompensationRefusal::UnknownCompensation(
                compensation_handle,
            ));
        };
        let Some(registration) = self.registration(compensation_handle) else {
            return CompensationOutcome::Unavailable(CompensationMissing::ForwardEffect);
        };
        let Some(commit) = definition.definition().commit.0.as_ref() else {
            return CompensationOutcome::Refused(CompensationRefusal::CommitNotSelected);
        };
        let Some(control) = self
            .conformance
            .obligation(commit.index)
            .filter(|_| commit.declaration == self.identity.declaration_handle())
        else {
            return CompensationOutcome::Refused(CompensationRefusal::InvalidCommitSelection);
        };
        let ControlOperation::Commit { instance, .. } = control.operation() else {
            return CompensationOutcome::Refused(CompensationRefusal::InvalidCommitSelection);
        };
        let slot = SlotHandle::new(*instance);
        let subject = match self.runtime_subject(slot, BindingKind::Commit, binding) {
            Ok(subject) => subject,
            Err(error) => return CompensationOutcome::Refused(error),
        };
        if let Some(existing) = self.compensation_state.commit(&registration) {
            return if existing.subject() == &subject {
                CompensationOutcome::Completed(CompensationTransition::Replay)
            } else {
                CompensationOutcome::Refused(CompensationRefusal::ConflictingReplay)
            };
        }
        let mut candidate = self.compensation_state.clone();
        candidate.push_commit(registration, subject);
        self.compensation_state = candidate;
        CompensationOutcome::Completed(CompensationTransition::Applied)
    }

    /// Evaluate and retain only the selected admitted QSL recovery expression.
    ///
    /// There is deliberately no callback parameter. QSL's complete report,
    /// including its limits, usage, owner accounting version, and terminal
    /// outcome, is preserved without conversion.
    pub fn evaluate_compensation_recovery(
        &mut self,
        compensation_handle: u32,
        view: &StateView,
        limits: state_evaluation::Limits,
    ) -> CompensationOutcome {
        let Some(definition) = self.linked.compensation(compensation_handle) else {
            return CompensationOutcome::Refused(CompensationRefusal::UnknownCompensation(
                compensation_handle,
            ));
        };
        let Some(registration) = self.registration(compensation_handle) else {
            return CompensationOutcome::Unavailable(CompensationMissing::ForwardEffect);
        };
        if self
            .compensation_state
            .effects()
            .iter()
            .all(|effect| effect.registration() != &registration)
        {
            return CompensationOutcome::Unavailable(CompensationMissing::CompensationEffect);
        }
        if definition.definition().commit.0.is_some()
            && self.compensation_state.commit(&registration).is_none()
        {
            return CompensationOutcome::Unavailable(CompensationMissing::Commit);
        }
        let report = match evaluate_boolean(
            self.context.package(),
            self.identity.declaration_handle(),
            definition.definition().recover.clone(),
            view,
            limits,
        ) {
            BooleanEvaluation::True(report) | BooleanEvaluation::False(report) => report,
            BooleanEvaluation::Source(report) => return CompensationOutcome::Source(report),
            BooleanEvaluation::Unsupported(report) => {
                return CompensationOutcome::Unsupported(report)
            }
        };
        let mut candidate = self.compensation_state.clone();
        match candidate.retain_recovery(registration, view.clone(), report) {
            Ok(transition) => {
                self.compensation_state = candidate;
                CompensationOutcome::Completed(transition)
            }
            Err(error) => CompensationOutcome::Refused(error),
        }
    }

    /// Assess the three commit-ordering laws only from QSL's selected nodes and
    /// QObs's strict observation/position authority.
    pub fn assess_compensation_ordering(
        &mut self,
        compensation_handle: u32,
        forward: &observation_record::View,
        effect: &observation_record::View,
        commit: Option<&observation_record::View>,
        positions: &position::View,
    ) -> CompensationOutcome {
        let Some(definition) = self.linked.compensation(compensation_handle) else {
            return CompensationOutcome::Refused(CompensationRefusal::UnknownCompensation(
                compensation_handle,
            ));
        };
        let Some(registration) = self.registration(compensation_handle) else {
            return CompensationOutcome::Unavailable(CompensationMissing::ForwardEffect);
        };
        if !observation_matches(registration.key().forward_effect(), forward) {
            return CompensationOutcome::Refused(CompensationRefusal::ObservationSubjectMismatch);
        }
        let Some(effect_fact) = self
            .compensation_state
            .effects()
            .iter()
            .find(|candidate| observation_matches(candidate.subject(), effect))
        else {
            return CompensationOutcome::Refused(CompensationRefusal::ObservationSubjectMismatch);
        };
        if effect_fact.registration() != &registration {
            return CompensationOutcome::Refused(CompensationRefusal::ObservationSubjectMismatch);
        }
        let selected_commit = definition.definition().commit.0.is_some();
        let commit_fact = self.compensation_state.commit(&registration);
        let commit = match (selected_commit, commit_fact, commit) {
            (true, Some(fact), Some(view)) if observation_matches(fact.subject(), view) => {
                Some(view)
            }
            (true, _, None) => {
                return CompensationOutcome::Unavailable(CompensationMissing::Commit)
            }
            (true, _, Some(_)) => {
                return CompensationOutcome::Refused(
                    CompensationRefusal::ObservationSubjectMismatch,
                )
            }
            (false, _, None) => None,
            (false, _, Some(_)) => {
                return CompensationOutcome::Refused(CompensationRefusal::CommitNotSelected)
            }
        };
        for observation in [Some(forward), Some(effect), commit].into_iter().flatten() {
            let clock = observation.payload().clock();
            if clock.identity != positions.payload().clock_identity()
                || clock.revision != positions.payload().clock_revision()
            {
                return CompensationOutcome::Refused(
                    CompensationRefusal::PositionAuthorityMismatch,
                );
            }
        }
        let Some(forward_position) = observation_position(positions, forward) else {
            return CompensationOutcome::Unavailable(CompensationMissing::Position);
        };
        let Some(effect_position) = observation_position(positions, effect) else {
            return CompensationOutcome::Unavailable(CompensationMissing::Position);
        };
        let commit_position = match commit {
            Some(observation) => match observation_position(positions, observation) {
                Some(value) => Some(value),
                None => return CompensationOutcome::Unavailable(CompensationMissing::Position),
            },
            None => None,
        };

        let mut violations = Vec::new();
        if commit_position.is_some_and(|position| position < forward_position) {
            violations.push(OrderingViolation::RegistrationAfterCommit);
        }
        if effect_position < forward_position {
            violations.push(OrderingViolation::CompensationBeforeForwardEffect);
        }
        if commit_position.is_some_and(|position| position < effect_position) {
            violations.push(OrderingViolation::CompensationAfterCommit);
        }
        let mut candidate = self.compensation_state.clone();
        match candidate.retain_ordering(
            registration,
            forward.clone(),
            effect.clone(),
            commit.cloned(),
            positions.clone(),
            violations,
        ) {
            Ok(transition) => {
                self.compensation_state = candidate;
                CompensationOutcome::Completed(transition)
            }
            Err(error) => CompensationOutcome::Refused(error),
        }
    }

    fn registration(&self, compensation_handle: u32) -> Option<CompensationRegistration> {
        self.compensation
            .find(&self.identity, compensation_handle)
            .cloned()
    }

    fn runtime_subject(
        &self,
        expected: SlotHandle,
        expected_kind: BindingKind,
        binding: &RuntimeBinding,
    ) -> Result<QualifiedSubject, CompensationRefusal> {
        if binding.slot() != expected {
            return Err(CompensationRefusal::UnexpectedSlot {
                expected,
                actual: binding.slot(),
            });
        }
        let Some(slot) = self
            .context
            .definition()
            .slots()
            .iter()
            .find(|slot| slot.handle == expected)
        else {
            return Err(CompensationRefusal::UnselectedSlot(expected));
        };
        let Some(requirement) = self
            .linked
            .declaration()
            .bindings
            .get(usize::try_from(expected.get()).unwrap_or(usize::MAX))
        else {
            return Err(CompensationRefusal::UnselectedSlot(expected));
        };
        if requirement.kind != expected_kind {
            return Err(CompensationRefusal::UnexpectedBindingKind(expected));
        }
        let Some(subject) = binding.subject_value() else {
            return Err(CompensationRefusal::WrongRuntimeValue(expected));
        };
        if subject.authority() != self.context.definition().producer() {
            return Err(CompensationRefusal::ForeignAuthority(expected));
        }
        match &slot.compatibility {
            SlotCompatibility::Subject { kind } if subject.kind() == kind => Ok(subject.clone()),
            SlotCompatibility::Subject { .. } => {
                Err(CompensationRefusal::IncompatibleSubjectKind(expected))
            }
            SlotCompatibility::Relationship { .. } => {
                Err(CompensationRefusal::WrongRuntimeValue(expected))
            }
        }
    }

    /// Retain one independent outcome for every selected claim.
    ///
    /// Generic assembly is internal support for the typed result stage. The
    /// public path is [`Self::assess_activated`], which owns production and
    /// strict reading from the Engine's exact linked assessment.
    pub(crate) fn assemble<Row>(
        self,
        rows: Vec<ClaimRow<Row>>,
    ) -> Result<AssessmentBundle<'a, Row>, BundleRefusal> {
        validate_rows(&self.requested, &rows)?;
        Ok(AssessmentBundle {
            context: self.context,
            identity: self.identity,
            linked: self.linked,
            compensation_inputs: self.compensation_inputs,
            compensation: self.compensation,
            compensation_state: self.compensation_state,
            conformance: self.conformance,
            budget: None,
            usage: ResourceAccountantUsage::default(),
            producer_resolution: None,
            rows,
        })
    }

    /// Assemble result-owner rows while verifying their retained FR-290 claim.
    ///
    /// A canonical row may not be moved to another claim merely because both
    /// rows share an assessment context. Claim-local refusals have no result
    /// row and therefore remain valid independent outcomes.
    pub(crate) fn assemble_canonical(
        self,
        rows: Vec<ClaimRow<result::ValidatedResult>>,
    ) -> Result<AssessmentBundle<'a, result::ValidatedResult>, BundleRefusal> {
        for row in &rows {
            if let ClaimOutcome::Canonical(result) = &row.outcome {
                if claim_from_result_kind(result.claim().kind) != row.claim {
                    return Err(BundleRefusal::CanonicalClaimMismatch);
                }
            }
        }
        self.assemble(rows)
    }

    #[allow(
        clippy::too_many_arguments,
        reason = "the private transition retains each independently owned stage value at the atomic Engine boundary"
    )]
    fn construct_canonical_bounded(
        self,
        dispatch: &DispatchReport,
        conformance: &crate::conformance::Report,
        axes: AxisInputs<'a>,
        completeness: &'a completeness::View,
        global_conformance: &GlobalConformanceInput,
        inputs: Vec<Option<CanonicalRowFacts<'a>>>,
        resources: &mut ResourceAccountant<'_>,
    ) -> crate::outcome::OperationOutcome<AssessmentBundle<'a, result::ValidatedResult>> {
        if let Err(error) = validate_dispatch_rows(&self.requested, dispatch, &inputs) {
            return crate::outcome::OperationOutcome::Refused(outcome_refusal(error));
        }
        let mut rows = Vec::with_capacity(inputs.len());
        for (selected, disposition) in inputs.into_iter().zip(dispatch.dispositions()) {
            let outcome = if let Some(failure) = &disposition.failure {
                ClaimOutcome::Refused(failure.envelope().clone())
            } else {
                let Some(selected) = selected else {
                    return crate::outcome::OperationOutcome::Refused(outcome_refusal(
                        BundleRefusal::MissingClaimRow,
                    ));
                };
                if let Err(error) = validate_input_package(
                    self.context.package().digest(),
                    selected.checked_predicate.package_digest(),
                    selected.temporal_subject.package_digest(),
                ) {
                    return crate::outcome::OperationOutcome::Refused(outcome_refusal(error));
                }
                match result_input_from_dispatch(
                    disposition,
                    conformance,
                    &self.identity,
                    &self.context,
                    axes,
                    completeness,
                    global_conformance,
                    selected,
                ) {
                    Err(refusal) => ClaimOutcome::Refused(refusal.envelope().clone()),
                    Ok((input, limits)) => {
                        match result::produce_bounded(input.clone(), limits, resources) {
                            BoundedOutcome::Completed(document) => match result::read_bounded(
                                document.bytes(),
                                result::Expected {
                                    input,
                                    enclosing_digest: document.digest(),
                                },
                                limits,
                                resources,
                            ) {
                                BoundedOutcome::Completed(validated) => {
                                    ClaimOutcome::Canonical(validated)
                                }
                                BoundedOutcome::DomainError(refusal) => {
                                    ClaimOutcome::Refused(refusal.envelope().clone())
                                }
                                BoundedOutcome::ResourceIncomplete(incomplete) => {
                                    return crate::outcome::OperationOutcome::ResourceIncomplete(
                                        incomplete,
                                    );
                                }
                                BoundedOutcome::AccountingFault(fault) => {
                                    return crate::outcome::OperationOutcome::Refused(
                                        accounting_refusal(
                                            fault,
                                            AccountingStage::ResultRead,
                                            resources.usage(),
                                        ),
                                    );
                                }
                            },
                            BoundedOutcome::DomainError(refusal) => {
                                ClaimOutcome::Refused(refusal.envelope().clone())
                            }
                            BoundedOutcome::ResourceIncomplete(incomplete) => {
                                return crate::outcome::OperationOutcome::ResourceIncomplete(
                                    incomplete,
                                );
                            }
                            BoundedOutcome::AccountingFault(fault) => {
                                return crate::outcome::OperationOutcome::Refused(
                                    accounting_refusal(
                                        fault,
                                        AccountingStage::ResultAssembly,
                                        resources.usage(),
                                    ),
                                );
                            }
                        }
                    }
                }
            };
            rows.push(ClaimRow {
                claim: disposition.claim,
                outcome,
            });
        }
        if rows
            .iter()
            .any(|row| matches!(row.outcome, ClaimOutcome::Canonical(_)))
        {
            for row in &mut rows {
                if let ClaimOutcome::Refused(refusal) = &mut row.outcome {
                    *refusal = refusal.clone().with_independent_results_retained();
                }
            }
        }
        match self.assemble_canonical(rows) {
            Ok(mut bundle) => {
                bundle.usage = resources.usage();
                crate::outcome::OperationOutcome::Completed(bundle)
            }
            Err(error) => crate::outcome::OperationOutcome::Refused(outcome_refusal(error)),
        }
    }

    /// Run conformance, claim dispatch, canonical result production, and strict
    /// reading as one non-retargetable assessment operation.
    ///
    /// Callers provide `Some` result-owner facts only for dispositions that can
    /// reach row construction and `None` for claim-local dispatch refusals. The
    /// Engine derives truth, execution, claim selection, and assumptions and
    /// reuses the exact progress, closure, completeness, and global-conformance
    /// values it assessed. One resource accountant spans every stage.
    ///
    /// ```compile_fail,E0599
    /// use quire_protocol::assessment::{CanonicalRowFacts, Engine};
    /// use quire_protocol::bounds::inventory::Budget;
    /// use quire_protocol::claims::dispatch::Report;
    ///
    /// fn retarget<'a>(
    ///     engine: Engine<'a>,
    ///     foreign_report: &Report,
    ///     facts: Vec<Option<CanonicalRowFacts<'a>>>,
    /// ) {
    ///     // Result construction accepts no report produced by another Engine.
    ///     let _ = engine.construct_canonical_outcome(
    ///         foreign_report,
    ///         facts,
    ///         Budget::nothing_granted(),
    ///     );
    /// }
    /// ```
    #[allow(
        clippy::too_many_arguments,
        reason = "the public orchestration keeps independently owned inputs typed and separate"
    )]
    pub fn assess_activated(
        self,
        producers: &ProducerResolution,
        requests: &[ClaimRequest<'_>],
        facts: &[ActivationFact<'_>],
        axes: AxisInputs<'a>,
        completeness: &'a completeness::View,
        global_conformance: &GlobalConformanceInput,
        execution: crate::closure::AssessmentExecution,
        conformance_limits: ConformanceLimits,
        dispatch_limits: DispatchLimits,
        inputs: Vec<Option<CanonicalRowFacts<'a>>>,
        budget: Budget,
    ) -> crate::outcome::OperationOutcome<AssessmentBundle<'a, result::ValidatedResult>> {
        match producers.authorize(&self.producer_resolution_request()) {
            crate::outcome::OperationOutcome::Completed(()) => {}
            crate::outcome::OperationOutcome::Refused(refusal) => {
                return crate::outcome::OperationOutcome::Refused(refusal);
            }
            crate::outcome::OperationOutcome::ResourceIncomplete(incomplete) => {
                return crate::outcome::OperationOutcome::ResourceIncomplete(incomplete);
            }
        }
        match self.assess_activated_authorized(
            requests,
            facts,
            axes,
            completeness,
            global_conformance,
            execution,
            conformance_limits,
            dispatch_limits,
            inputs,
            budget,
        ) {
            crate::outcome::OperationOutcome::Completed(mut bundle) => {
                bundle.producer_resolution = Some(producers.evidence());
                crate::outcome::OperationOutcome::Completed(bundle)
            }
            crate::outcome::OperationOutcome::Refused(refusal) => {
                crate::outcome::OperationOutcome::Refused(refusal)
            }
            crate::outcome::OperationOutcome::ResourceIncomplete(incomplete) => {
                crate::outcome::OperationOutcome::ResourceIncomplete(incomplete)
            }
        }
    }

    #[allow(
        clippy::too_many_arguments,
        reason = "the authorized transition retains each independently owned stage value at the atomic Engine boundary"
    )]
    pub(crate) fn assess_activated_authorized(
        self,
        requests: &[ClaimRequest<'_>],
        facts: &[ActivationFact<'_>],
        axes: AxisInputs<'a>,
        completeness: &'a completeness::View,
        global_conformance: &GlobalConformanceInput,
        execution: crate::closure::AssessmentExecution,
        conformance_limits: ConformanceLimits,
        dispatch_limits: DispatchLimits,
        inputs: Vec<Option<CanonicalRowFacts<'a>>>,
        budget: Budget,
    ) -> crate::outcome::OperationOutcome<AssessmentBundle<'a, result::ValidatedResult>> {
        let granted_budget = budget.clone();
        let inventory = production_inventory()
            .expect("the compile-time Protocol resource inventory must reconcile");
        let mut resources = ResourceAccountant::new(budget, &inventory);
        let (conformance, dispatch) = match self.dispatch_activated_bounded(
            requests,
            facts,
            axes,
            completeness,
            global_conformance,
            execution,
            conformance_limits,
            dispatch_limits,
            &mut resources,
        ) {
            BoundedOutcome::Completed(value) => value,
            BoundedOutcome::DomainError(refusal) => {
                return crate::outcome::OperationOutcome::Refused(refusal);
            }
            BoundedOutcome::ResourceIncomplete(incomplete) => {
                return crate::outcome::OperationOutcome::ResourceIncomplete(incomplete);
            }
            BoundedOutcome::AccountingFault(fault) => {
                return crate::outcome::OperationOutcome::Refused(accounting_refusal(
                    fault,
                    AccountingStage::ClaimDispatch,
                    resources.usage(),
                ));
            }
        };
        match self.construct_canonical_bounded(
            &dispatch,
            &conformance,
            axes,
            completeness,
            global_conformance,
            inputs,
            &mut resources,
        ) {
            crate::outcome::OperationOutcome::Completed(mut bundle) => {
                bundle.budget = Some(granted_budget);
                crate::outcome::OperationOutcome::Completed(bundle)
            }
            crate::outcome::OperationOutcome::Refused(refusal) => {
                crate::outcome::OperationOutcome::Refused(refusal)
            }
            crate::outcome::OperationOutcome::ResourceIncomplete(incomplete) => {
                crate::outcome::OperationOutcome::ResourceIncomplete(incomplete)
            }
        }
    }
}

fn append_component(encoded: &mut Vec<u8>, component: &[u8]) {
    encoded.extend_from_slice(&(component.len() as u128).to_be_bytes());
    encoded.extend_from_slice(component);
}

fn append_count(encoded: &mut Vec<u8>, count: usize) {
    append_component(encoded, &(count as u128).to_be_bytes());
}

fn append_authority(encoded: &mut Vec<u8>, authority: &AdmittedBundleKey) {
    append_component(encoded, authority.bundle_identity.as_bytes());
    append_component(encoded, authority.bundle_revision.namespace.as_bytes());
    append_component(encoded, authority.bundle_revision.value.as_bytes());
    append_component(encoded, authority.digest.algorithm.as_bytes());
    append_component(encoded, authority.digest.domain.as_bytes());
    append_component(encoded, authority.digest.version.as_bytes());
    append_component(encoded, authority.digest.value.as_bytes());
}

fn append_subject(encoded: &mut Vec<u8>, subject: &QualifiedSubject) {
    append_authority(encoded, subject.authority());
    append_component(encoded, subject.kind().as_bytes());
    append_component(encoded, subject.identity().as_bytes());
}

fn append_relationship(encoded: &mut Vec<u8>, relationship: &Relationship) {
    append_authority(encoded, relationship.authority());
    append_component(encoded, relationship.interface_version().as_bytes());
    append_component(encoded, relationship.declaration_identity().as_bytes());
    append_component(encoded, relationship.identity().as_bytes());
    append_subject(encoded, relationship.source());
    append_subject(encoded, relationship.target());
}

fn validate_dispatch_request(
    requested: &[Claim],
    requests: &[ClaimRequest<'_>],
) -> Result<(), BundleRefusal> {
    if requests.len() != requested.len()
        || requests
            .iter()
            .any(|request| !requested.contains(&request.claim))
        || requested.iter().any(|claim| {
            requests
                .iter()
                .filter(|request| request.claim == *claim)
                .count()
                != 1
        })
    {
        Err(BundleRefusal::UnexpectedClaim)
    } else {
        Ok(())
    }
}

fn activated_assessment_refusal(error: ActivatedAssessmentError) -> ProtocolRefusal {
    match error {
        ActivatedAssessmentError::Kernel(error) => kernel_refusal(error),
        ActivatedAssessmentError::Conformance(refusal) => refusal.envelope().clone(),
    }
}

fn kernel_refusal(error: ConformanceKernelRefusal) -> ProtocolRefusal {
    let tag = match error {
        ConformanceKernelRefusal::ForeignDefinition => b"foreign-definition".as_slice(),
        ConformanceKernelRefusal::UnknownCompensation => b"unknown-compensation".as_slice(),
        ConformanceKernelRefusal::UnknownRegistration => b"unknown-registration".as_slice(),
        ConformanceKernelRefusal::UnrepresentableControl => b"unrepresentable-control".as_slice(),
        ConformanceKernelRefusal::ForeignActivationPackage => {
            b"foreign-activation-package".as_slice()
        }
        ConformanceKernelRefusal::MissingActivationMapping => {
            b"missing-activation-mapping".as_slice()
        }
        ConformanceKernelRefusal::InvalidActivationControl => {
            b"invalid-activation-control".as_slice()
        }
        ConformanceKernelRefusal::ActivationSubjectMismatch => {
            b"activation-subject-mismatch".as_slice()
        }
        ConformanceKernelRefusal::ActivationResultMismatch => {
            b"activation-result-mismatch".as_slice()
        }
        ConformanceKernelRefusal::InvalidActivationTemporal => {
            b"invalid-activation-temporal".as_slice()
        }
        ConformanceKernelRefusal::ActivationOccurrenceMismatch => {
            b"activation-occurrence-mismatch".as_slice()
        }
        ConformanceKernelRefusal::ConflictingActivation => b"conflicting-activation".as_slice(),
        ConformanceKernelRefusal::ForeignActivatedOccurrence => {
            b"foreign-activated-occurrence".as_slice()
        }
        ConformanceKernelRefusal::DuplicateActivatedObligation => {
            b"duplicate-activated-obligation".as_slice()
        }
        ConformanceKernelRefusal::InvalidDerivedModel => b"invalid-derived-model".as_slice(),
        ConformanceKernelRefusal::UnexpectedActivatedOccurrence => {
            b"unexpected-activated-occurrence".as_slice()
        }
    };
    let code = RefusalCode::of(&BUNDLE_INVALID_CONFORMANCE_KERNEL);
    let cause = match error {
        ConformanceKernelRefusal::UnknownCompensation
        | ConformanceKernelRefusal::UnknownRegistration
        | ConformanceKernelRefusal::MissingActivationMapping => ProtocolCause::MissingDefinition {
            definition: RetainedBytes::new(tag),
        },
        ConformanceKernelRefusal::ConflictingActivation
        | ConformanceKernelRefusal::DuplicateActivatedObligation => {
            ProtocolCause::ConflictingDefinition {
                definition: RetainedBytes::new(tag),
                conflicting_definition: RetainedBytes::new(tag),
            }
        }
        ConformanceKernelRefusal::ForeignDefinition
        | ConformanceKernelRefusal::ForeignActivationPackage
        | ConformanceKernelRefusal::ActivationSubjectMismatch
        | ConformanceKernelRefusal::ActivationResultMismatch
        | ConformanceKernelRefusal::ActivationOccurrenceMismatch
        | ConformanceKernelRefusal::ForeignActivatedOccurrence
        | ConformanceKernelRefusal::UnexpectedActivatedOccurrence => {
            ProtocolCause::IdentityMismatch {
                expected: RetainedBytes::new(b"engine-bound-activation"),
                observed: RetainedBytes::new(tag),
            }
        }
        ConformanceKernelRefusal::UnrepresentableControl
        | ConformanceKernelRefusal::InvalidActivationControl
        | ConformanceKernelRefusal::InvalidActivationTemporal
        | ConformanceKernelRefusal::InvalidDerivedModel => ProtocolCause::IncompleteEvidence {
            evidence: RetainedBytes::new(tag),
        },
    };
    ProtocolRefusal::builder(
        code,
        CatalogSelection::local(),
        ProtocolStage::Conformance,
        RefusalAxis::new(tag).expect("kernel refusal tags are non-empty"),
        cause.into(),
        ResourceUsage::protocol("quire.protocol/assessment-usage/1", &[]),
    )
    .expect("kernel refusal code belongs to conformance")
    .prerequisite(Prerequisite::new(
        ProtocolStage::Representation,
        PrerequisiteState::Established,
    ))
    .unavailable_location(Unavailability::NotApplicableToStage)
    .emitting_nothing()
}

enum BooleanEvaluation {
    True(EvaluationReport),
    False(EvaluationReport),
    Source(EvaluationReport),
    Unsupported(EvaluationReport),
}

fn evaluate_boolean(
    package: &quire_spec_language::protocol_artifact::v2::AdmittedPackage,
    declaration: u32,
    value: quire_spec_language::protocol_artifact::wire::Handle,
    view: &StateView,
    limits: state_evaluation::Limits,
) -> BooleanEvaluation {
    let report = state_evaluation::evaluate_v2(
        package,
        state_evaluation::EvaluationRequest { declaration, value },
        view,
        limits,
    );
    enum Classification {
        True,
        False,
        Unsupported,
        Source,
    }
    let classification = match report.outcome() {
        EvaluationOutcome::Completed(value) => match value.kind() {
            StateValueKind::Boolean(true) => Classification::True,
            StateValueKind::Boolean(false) => Classification::False,
            StateValueKind::Number(_)
            | StateValueKind::Text(_)
            | StateValueKind::Enum(_)
            | StateValueKind::Record(_)
            | StateValueKind::Option(_)
            | StateValueKind::Sequence(_)
            | StateValueKind::Reference(_)
            | StateValueKind::Object(_) => Classification::Unsupported,
        },
        EvaluationOutcome::Incomplete(_)
        | EvaluationOutcome::Refused(_)
        | EvaluationOutcome::Exhausted(_) => Classification::Source,
    };
    match classification {
        Classification::True => BooleanEvaluation::True(report),
        Classification::False => BooleanEvaluation::False(report),
        Classification::Unsupported => BooleanEvaluation::Unsupported(report),
        Classification::Source => BooleanEvaluation::Source(report),
    }
}

fn observation_matches(subject: &QualifiedSubject, observation: &observation_record::View) -> bool {
    let observed = observation.payload().subject();
    let authority = subject.authority();
    authority.bundle_identity == observed.authority.identity
        && authority.bundle_revision == *observed.authority.revision
        && authority.digest == *observed.authority.digest
        && subject.kind().as_bytes() == observed.kind
        && subject.identity().as_bytes() == observed.identity
}

fn observation_position(
    positions: &position::View,
    observation: &observation_record::View,
) -> Option<usize> {
    positions
        .payload()
        .positions()
        .position(|(_, identity)| identity == observation.payload().observation_identity())
}

#[allow(
    clippy::too_many_arguments,
    reason = "the private transition receives exact owner outputs separately so no caller-built result aggregate can substitute for them"
)]
fn result_input_from_dispatch<'a>(
    disposition: &dispatch::Disposition,
    conformance: &crate::conformance::Report,
    definition: &DefinitionIdentity,
    context: &LinkedAssessment<'_>,
    axes: AxisInputs<'a>,
    completeness: &'a completeness::View,
    global_conformance: &GlobalConformanceInput,
    selected: CanonicalRowFacts<'a>,
) -> result::Result<(result::Input<'a>, result::Limits)> {
    let strength = match disposition.strength.as_ref() {
        b"proven" => result::ClaimStrength::Proven,
        b"bounded-checked" => result::ClaimStrength::BoundedChecked,
        b"tested" => result::ClaimStrength::Tested,
        b"observed" => result::ClaimStrength::Observed,
        _ => {
            return Err(dispatch_value_refusal(
                "dispatch selected an unknown result claim strength",
            ));
        }
    };
    let admitted_fragment_identity =
        retained_text(disposition.premises.admitted_fragment.identity.as_ref())?;
    let backend_identity = retained_text(disposition.premises.backend.identity.as_ref())?;
    let assumptions = disposition
        .premises
        .assumptions
        .iter()
        .map(|assumption| {
            Ok(result::Assumption {
                identity: retained_text(assumption.component.identity.as_ref())?,
                state: match assumption
                    .state
                    .ok_or_else(|| dispatch_value_refusal("dispatch omitted an assumption state"))?
                {
                    crate::claims::premises::AssumptionState::Met => result::AssumptionState::Met,
                    crate::claims::premises::AssumptionState::NotMet => {
                        result::AssumptionState::NotMet
                    }
                    crate::claims::premises::AssumptionState::Unknown => {
                        result::AssumptionState::Unknown
                    }
                    crate::claims::premises::AssumptionState::NotAssessed => {
                        result::AssumptionState::NotAssessed
                    }
                },
            })
        })
        .collect::<result::Result<Vec<_>>>()?;
    let provenance = provenance_from_dispatch(disposition, definition, context, &selected)?;
    let limits = selected.limits;
    Ok((
        result::Input {
            checked_predicate: selected.checked_predicate,
            temporal_subject: selected.temporal_subject,
            axes,
            completeness,
            execution: disposition.execution,
            truth: truth_from_dispatch(disposition, conformance),
            settlement: selected.settlement,
            activation: selected.activation,
            participation: selected.participation,
            assumptions,
            observation_adequacy: selected.observation_adequacy,
            protocol_adequacy: selected.protocol_adequacy,
            claim: result::Claim {
                kind: result_claim_kind(disposition.claim),
                strength: Some(strength),
                admitted_fragment_identity: Some(admitted_fragment_identity),
                backend_identity: Some(backend_identity),
            },
            decision_support: selected.decision_support,
            global_conformance: global_conformance.clone(),
            limitations: selected.limitations,
            provenance,
            correction: selected.correction,
        },
        limits,
    ))
}

fn provenance_from_dispatch(
    disposition: &dispatch::Disposition,
    definition: &DefinitionIdentity,
    context: &LinkedAssessment<'_>,
    selected: &CanonicalRowFacts<'_>,
) -> result::Result<result::ProvenanceRecord> {
    let model_selected = definition.closure().models().iter().any(|model| {
        context
            .package()
            .inherited()
            .dependencies
            .get(usize::try_from(model.value().artifact).unwrap_or(usize::MAX))
            .is_some_and(|dependency| dependency.artifact == *selected.model_artifact)
    });
    if !model_selected {
        return Err(dispatch_value_refusal(
            "result model provenance is outside the Engine selection closure",
        ));
    }
    let runtime_selected = context.runtime().bindings().iter().any(|binding| {
        binding
            .subject_value()
            .is_some_and(|subject| subject == selected.runtime_input)
    });
    if !runtime_selected {
        return Err(dispatch_value_refusal(
            "result runtime input is outside the Engine binding set",
        ));
    }
    let input_identity = result::runtime_reference::encode_subject(selected.runtime_input)
        .map_err(|_| dispatch_value_refusal("result runtime input cannot be represented"))?;
    Ok(result::ProvenanceRecord {
        source_identity: definition.source_document().identity.clone(),
        profile_identity: definition.closure().profile().value().identity.clone(),
        model_identity: selected.model_artifact.identity.clone(),
        protocol_identity: format!(
            "{}:{}",
            definition.source_document().identity,
            definition.declaration_handle()
        ),
        request_identity: retained_text(disposition.request_identity.as_ref())?,
        binding_identity: definition.static_binding().identity.to_string(),
        input_identity,
        backend_identity: retained_text(disposition.premises.backend.identity.as_ref())?,
        producer_identity: context.definition().producer().bundle_identity.clone(),
        reproduction: selected.reproduction.clone(),
    })
}

fn retained_text(bytes: &[u8]) -> result::Result<String> {
    String::from_utf8(bytes.to_vec()).map_err(|_| {
        dispatch_value_refusal(
            "dispatch identity cannot be represented in the selected result contract",
        )
    })
}

fn dispatch_value_refusal(detail: &'static str) -> result::Refusal {
    result::Refusal::assembly(
        result::ErrorCode::InvalidSelection,
        detail,
        result::Usage::default(),
        None,
    )
}

const fn truth_from_dispatch(
    disposition: &dispatch::Disposition,
    conformance: &crate::conformance::Report,
) -> result::Truth {
    use crate::closure::AssessmentExecution;
    use crate::conformance::GlobalDisposition;

    if !matches!(disposition.execution, AssessmentExecution::Completed) {
        return result::Truth::Unavailable;
    }
    if matches!(disposition.claim, Claim::Conformance) {
        return match conformance.global {
            GlobalDisposition::Conforms => result::Truth::Satisfied,
            GlobalDisposition::Pending => result::Truth::Pending,
            GlobalDisposition::Violates => result::Truth::Violated,
            GlobalDisposition::Unavailable => result::Truth::Unavailable,
        };
    }
    match disposition.decision {
        Some(dispatch::Decision::Established) => result::Truth::Satisfied,
        Some(dispatch::Decision::NotEstablished) => result::Truth::Violated,
        None => result::Truth::Unavailable,
    }
}

const fn result_claim_kind(claim: Claim) -> ClaimKind {
    match claim {
        Claim::Conformance => ClaimKind::GlobalConformance,
        Claim::Monitorability => ClaimKind::Monitorability,
        Claim::Projection => ClaimKind::LocalProjection,
        Claim::Realizability => ClaimKind::Realizability,
        Claim::Refinement => ClaimKind::Refinement,
        Claim::Composition => ClaimKind::Composition,
    }
}

/// Require both strict QSL source views used by a result owner to answer this
/// Engine's one admitted package. A result reader may validate each view on its
/// own, but only this boundary can prevent cross-assessment retargeting.
fn validate_input_package(
    expected: crate::ByteDigest,
    predicate: crate::ByteDigest,
    temporal: crate::ByteDigest,
) -> Result<(), BundleRefusal> {
    if expected == predicate && expected == temporal {
        Ok(())
    } else {
        Err(BundleRefusal::CanonicalPackageMismatch)
    }
}

/// Require the preceding closed dispatch stage to answer this Engine's exact
/// claim population in the same declared order as canonical-row construction.
fn validate_dispatch_rows(
    requested: &[Claim],
    dispatch: &DispatchReport,
    inputs: &[Option<CanonicalRowFacts<'_>>],
) -> Result<(), BundleRefusal> {
    validate_dispatch_population(requested, dispatch)?;
    if inputs.len() != requested.len() {
        return Err(BundleRefusal::MissingClaimRow);
    }
    for (disposition, input) in dispatch.dispositions().iter().zip(inputs) {
        if disposition.failure.is_some() == input.is_some() {
            return Err(BundleRefusal::DispatchReportMismatch);
        }
    }
    Ok(())
}

fn validate_dispatch_population(
    requested: &[Claim],
    dispatch: &DispatchReport,
) -> Result<(), BundleRefusal> {
    if dispatch.dispositions().len() != requested.len() {
        return Err(BundleRefusal::DispatchReportMismatch);
    }
    for (requested, disposition) in requested.iter().zip(dispatch.dispositions()) {
        if disposition.claim != *requested {
            return Err(BundleRefusal::DispatchReportMismatch);
        }
    }
    Ok(())
}

const fn claim_from_result_kind(kind: ClaimKind) -> Claim {
    match kind {
        ClaimKind::GlobalConformance => Claim::Conformance,
        ClaimKind::Monitorability => Claim::Monitorability,
        ClaimKind::LocalProjection => Claim::Projection,
        ClaimKind::Realizability => Claim::Realizability,
        ClaimKind::Refinement => Claim::Refinement,
        ClaimKind::Composition => Claim::Composition,
    }
}

fn validate_requested(requested: &[Claim]) -> Result<(), BundleRefusal> {
    if requested.is_empty() {
        return Err(BundleRefusal::EmptyRequest);
    }
    for (index, claim) in requested.iter().enumerate() {
        if requested[..index].contains(claim) {
            return Err(BundleRefusal::DuplicateRequestedClaim);
        }
    }
    Ok(())
}

fn validate_rows<Row>(requested: &[Claim], rows: &[ClaimRow<Row>]) -> Result<(), BundleRefusal> {
    for row in rows {
        if !requested.contains(&row.claim) {
            return Err(BundleRefusal::UnexpectedClaim);
        }
        if rows.iter().filter(|other| other.claim == row.claim).count() > 1 {
            return Err(BundleRefusal::DuplicateClaimRow);
        }
    }
    if requested
        .iter()
        .any(|claim| !rows.iter().any(|row| row.claim == *claim))
    {
        return Err(BundleRefusal::MissingClaimRow);
    }
    Ok(())
}

/// Complete, non-retargetable claim population for one linked assessment.
#[derive(Clone, Debug)]
pub struct AssessmentBundle<'a, Row> {
    context: LinkedAssessment<'a>,
    identity: DefinitionIdentity,
    linked: LinkedProtocol<'a>,
    compensation_inputs: Vec<CompensationRegistrationInput<'a>>,
    compensation: CompensationSnapshot,
    compensation_state: CompensationState,
    conformance: ConformanceKernel,
    budget: Option<Budget>,
    usage: ResourceAccountantUsage,
    producer_resolution: Option<ProducerResolutionEvidence>,
    rows: Vec<ClaimRow<Row>>,
}

impl<Row> AssessmentBundle<'_, Row> {
    /// The exact linked assessment the rows answer.
    #[must_use]
    pub const fn context(&self) -> &LinkedAssessment<'_> {
        &self.context
    }
    /// The fixed FR-288 definition identity for all rows.
    #[must_use]
    pub const fn identity(&self) -> &DefinitionIdentity {
        &self.identity
    }
    /// Exact linked QSL graph retained for all result rows.
    #[must_use]
    pub const fn linked(&self) -> &LinkedProtocol<'_> {
        &self.linked
    }
    /// The non-retargetable conformance kernel retained with this bundle.
    #[must_use]
    pub const fn conformance(&self) -> &ConformanceKernel {
        &self.conformance
    }
    /// Exact static-compensation/runtime-effect pairs available to the pure
    /// compensation stage for this one linked assessment.
    #[must_use]
    pub fn compensation_inputs(&self) -> &[CompensationRegistrationInput<'_>] {
        &self.compensation_inputs
    }
    /// Replay-idempotent registrations derived from this bundle's exact inputs.
    #[must_use]
    pub const fn compensation(&self) -> &CompensationSnapshot {
        &self.compensation
    }
    /// Exact graph-derived compensation state retained with this bundle.
    #[must_use]
    pub const fn compensation_state(&self) -> &CompensationState {
        &self.compensation_state
    }
    /// Total shared-accountant usage across conformance, dispatch, result
    /// production and strict reading for this completed operation.
    #[must_use]
    pub const fn usage(&self) -> ResourceAccountantUsage {
        self.usage
    }
    /// Exact caller grants for the composed operation, when it passed through
    /// the bounded assessment path.
    #[must_use]
    pub const fn budget(&self) -> Option<&Budget> {
        self.budget.as_ref()
    }
    /// Exact producer-resolution attribution retained by the public composed
    /// assessment path. Final FR-017 record composition remains Task-018-owned.
    #[must_use]
    pub const fn producer_resolution(&self) -> Option<&ProducerResolutionEvidence> {
        self.producer_resolution.as_ref()
    }
    /// One ordered outcome for every requested claim.
    #[must_use]
    pub fn rows(&self) -> &[ClaimRow<Row>] {
        &self.rows
    }
}

#[cfg(test)]
mod tests {
    use super::{
        accounting_refusal, claim_from_result_kind, encode_accounting_fault, kernel_refusal,
        outcome_refusal, validate_dispatch_population, validate_input_package, validate_requested,
        validate_rows, AccountingStage, BundleRefusal, ClaimRow,
    };
    use crate::bounds::inventory::{
        production_inventory, AccountingFault, Budget, BudgetDimension, ResourceAccountant,
    };
    use crate::bounds::{Phase, SiteId, UnitLabel};
    use crate::claims::{dispatch::Report as DispatchReport, Claim};
    use crate::conformance::KernelRefusal;
    use crate::outcome::ClaimOutcome;
    use crate::refusal::{AffectedIdentity, PartialOutput, ProtocolStage, RefusalDomain};
    use crate::result::ClaimKind;
    use crate::ByteDigest;
    use ix_trace_rs::trace;
    use std::collections::BTreeSet;
    use std::time::Duration;

    #[test]
    fn assessment_request_requires_one_claim() {
        assert_eq!(validate_requested(&[]), Err(BundleRefusal::EmptyRequest));
    }

    #[test]
    fn assessment_request_refuses_duplicate_claims() {
        assert_eq!(
            validate_requested(&[Claim::Conformance, Claim::Conformance]),
            Err(BundleRefusal::DuplicateRequestedClaim)
        );
    }

    #[test]
    fn assessment_request_keeps_distinct_claims() {
        assert_eq!(
            validate_requested(&[Claim::Conformance, Claim::Monitorability]),
            Ok(())
        );
    }

    fn row(claim: Claim) -> ClaimRow<u8> {
        ClaimRow {
            claim,
            outcome: ClaimOutcome::Canonical(1),
        }
    }

    #[test]
    fn assessment_rows_refuse_unknown_duplicate_and_missing_claims() {
        let requested = [Claim::Conformance, Claim::Monitorability];
        assert_eq!(
            validate_rows(&requested, &[row(Claim::Projection)]),
            Err(BundleRefusal::UnexpectedClaim)
        );
        assert_eq!(
            validate_rows(
                &requested,
                &[row(Claim::Conformance), row(Claim::Conformance)]
            ),
            Err(BundleRefusal::DuplicateClaimRow)
        );
        assert_eq!(
            validate_rows(&requested, &[row(Claim::Conformance)]),
            Err(BundleRefusal::MissingClaimRow)
        );
    }

    #[test]
    fn assessment_rows_keep_one_outcome_per_requested_claim() {
        let requested = [Claim::Conformance, Claim::Monitorability];
        assert_eq!(
            validate_rows(
                &requested,
                &[row(Claim::Conformance), row(Claim::Monitorability)]
            ),
            Ok(())
        );
    }

    /// Every whole-assessment refusal carries a source-bound envelope rather
    /// than using the absence of a bundle as its identity.
    #[test]
    #[trace(
        "TC-011",
        "FR-008-AC-1",
        "FR-008-AC-4",
        "FR-008-AC-5",
        "FR-036-AC-1",
        "FR-039-AC-1",
        "FR-040-AC-1"
    )]
    fn tc_011_assessment_bundle_refusals_are_typed_and_distinct() {
        let reasons = [
            (
                BundleRefusal::EmptyRequest,
                RefusalDomain::Assessment,
                ProtocolStage::ClaimDispatch,
            ),
            (
                BundleRefusal::DuplicateRequestedClaim,
                RefusalDomain::Assessment,
                ProtocolStage::ClaimDispatch,
            ),
            (
                BundleRefusal::UnexpectedClaim,
                RefusalDomain::Assessment,
                ProtocolStage::ClaimDispatch,
            ),
            (
                BundleRefusal::DuplicateClaimRow,
                RefusalDomain::Assessment,
                ProtocolStage::ClaimDispatch,
            ),
            (
                BundleRefusal::MissingClaimRow,
                RefusalDomain::Assessment,
                ProtocolStage::ClaimDispatch,
            ),
            (
                BundleRefusal::InvalidDefinitionIdentity,
                RefusalDomain::Representation,
                ProtocolStage::Representation,
            ),
            (
                BundleRefusal::InvalidLinkedProtocol,
                RefusalDomain::Representation,
                ProtocolStage::Representation,
            ),
            (
                BundleRefusal::InvalidConformanceKernel,
                RefusalDomain::Assessment,
                ProtocolStage::Conformance,
            ),
            (
                BundleRefusal::CanonicalClaimMismatch,
                RefusalDomain::Result,
                ProtocolStage::ResultAssembly,
            ),
            (
                BundleRefusal::CanonicalPackageMismatch,
                RefusalDomain::Result,
                ProtocolStage::ResultAssembly,
            ),
            (
                BundleRefusal::DispatchReportMismatch,
                RefusalDomain::Result,
                ProtocolStage::ResultAssembly,
            ),
            (
                BundleRefusal::ActivationSubjectMismatch,
                RefusalDomain::Assessment,
                ProtocolStage::Conformance,
            ),
            (
                BundleRefusal::ActivationDeclarationMismatch,
                RefusalDomain::Assessment,
                ProtocolStage::Conformance,
            ),
        ];
        let mut codes = BTreeSet::new();
        let mut axes = BTreeSet::new();
        for (reason, domain, stage) in reasons {
            let refusal = outcome_refusal(reason);
            assert_eq!(refusal.domain(), domain);
            assert_eq!(refusal.stage(), stage);
            assert_eq!(refusal.partial_output(), PartialOutput::NoneEmitted);
            assert!(!refusal.causes().is_empty());
            assert!(codes.insert(refusal.code().as_str()));
            assert!(axes.insert(refusal.axis().as_bytes().to_vec()));
        }
    }

    #[test]
    #[trace("TC-011", "FR-008-AC-1", "FR-036-AC-1", "FR-039-AC-1")]
    fn tc_011_accounting_faults_retain_exact_stage_identity_and_interior() {
        let site = SiteId::new("assessment/test/accounting");
        let faults = [
            AccountingFault::UnregisteredSite { site },
            AccountingFault::CounterOverflow {
                site,
                consumed: 2,
                requested: 3,
            },
            AccountingFault::CounterUnderflow {
                site,
                consumed: 5,
                requested: 7,
            },
            AccountingFault::ClockRegressed {
                unit: UnitLabel::new("assessment-test"),
                phase: Phase::AfterUnit,
                previous: Duration::from_nanos(11),
            },
            AccountingFault::ElapsedTimeOverflow {
                observed: Duration::from_nanos(13),
            },
            AccountingFault::SiteDimensionMismatch {
                site,
                declared: BudgetDimension::InputByte,
                required: BudgetDimension::OutputByte,
            },
            AccountingFault::TemporalDimensionChargedStatically {
                site,
                dimension: BudgetDimension::ElapsedTime,
            },
        ];
        let mut interiors = BTreeSet::new();
        for fault in faults {
            let (affected, axis) = encode_accounting_fault(fault);
            assert!(!affected.is_empty());
            assert!(interiors.insert(axis));
        }

        let inventory = production_inventory().expect("production inventory reconciles");
        let usage = ResourceAccountant::new(Budget::nothing_granted(), &inventory).usage();
        let stages = [
            (
                AccountingStage::Conformance,
                RefusalDomain::Assessment,
                ProtocolStage::Conformance,
                ProtocolStage::Representation,
            ),
            (
                AccountingStage::ClaimDispatch,
                RefusalDomain::Assessment,
                ProtocolStage::ClaimDispatch,
                ProtocolStage::Conformance,
            ),
            (
                AccountingStage::ResultAssembly,
                RefusalDomain::Result,
                ProtocolStage::ResultAssembly,
                ProtocolStage::ClaimDispatch,
            ),
            (
                AccountingStage::ResultRead,
                RefusalDomain::Result,
                ProtocolStage::ResultRead,
                ProtocolStage::ResultAssembly,
            ),
        ];
        for (accounting_stage, domain, stage, prerequisite) in stages {
            let refusal = accounting_refusal(
                AccountingFault::CounterOverflow {
                    site,
                    consumed: 2,
                    requested: 3,
                },
                accounting_stage,
                usage,
            );
            assert_eq!(refusal.domain(), domain);
            assert_eq!(refusal.stage(), stage);
            assert_eq!(refusal.prerequisites()[0].stage(), prerequisite);
            let AffectedIdentity::Retained(identity) = refusal.identity() else {
                panic!("accounting fault must retain its affected site")
            };
            assert_eq!(identity.as_bytes(), site.as_str().as_bytes());
            assert!(refusal
                .axis()
                .as_bytes()
                .starts_with(b"accounting-fault\0counter-overflow\0"));
            assert_eq!(
                refusal.resource_usage().protocol_record().measures().len(),
                BudgetDimension::ALL.len()
            );
        }
    }

    #[test]
    #[trace("TC-011", "FR-008-AC-1", "FR-036-AC-1", "FR-039-AC-1")]
    fn tc_011_kernel_refusal_axes_retain_the_exact_activation_failure() {
        let failures = [
            KernelRefusal::ForeignDefinition,
            KernelRefusal::UnknownCompensation,
            KernelRefusal::UnknownRegistration,
            KernelRefusal::UnrepresentableControl,
            KernelRefusal::ForeignActivationPackage,
            KernelRefusal::MissingActivationMapping,
            KernelRefusal::InvalidActivationControl,
            KernelRefusal::ActivationSubjectMismatch,
            KernelRefusal::ActivationResultMismatch,
            KernelRefusal::InvalidActivationTemporal,
            KernelRefusal::ActivationOccurrenceMismatch,
            KernelRefusal::ConflictingActivation,
            KernelRefusal::ForeignActivatedOccurrence,
            KernelRefusal::DuplicateActivatedObligation,
            KernelRefusal::InvalidDerivedModel,
            KernelRefusal::UnexpectedActivatedOccurrence,
        ];
        let mut axes = BTreeSet::new();
        for failure in failures {
            let refusal = kernel_refusal(failure);
            assert_eq!(refusal.stage(), ProtocolStage::Conformance);
            assert!(!refusal.causes().is_empty());
            assert!(axes.insert(refusal.axis().as_bytes().to_vec()));
        }
        assert_eq!(axes.len(), failures.len());
    }

    #[test]
    fn canonical_result_claim_kinds_map_to_their_exact_dispatch_claims() {
        assert_eq!(
            claim_from_result_kind(ClaimKind::GlobalConformance),
            Claim::Conformance
        );
        assert_eq!(
            claim_from_result_kind(ClaimKind::Monitorability),
            Claim::Monitorability
        );
        assert_eq!(
            claim_from_result_kind(ClaimKind::LocalProjection),
            Claim::Projection
        );
        assert_eq!(
            claim_from_result_kind(ClaimKind::Realizability),
            Claim::Realizability
        );
        assert_eq!(
            claim_from_result_kind(ClaimKind::Refinement),
            Claim::Refinement
        );
        assert_eq!(
            claim_from_result_kind(ClaimKind::Composition),
            Claim::Composition
        );
    }

    #[test]
    fn canonical_input_views_must_borrow_the_engines_admitted_package() {
        let selected = ByteDigest::of(b"selected-package");
        assert_eq!(validate_input_package(selected, selected, selected), Ok(()));
        assert_eq!(
            validate_input_package(selected, ByteDigest::of(b"foreign"), selected),
            Err(BundleRefusal::CanonicalPackageMismatch)
        );
        assert_eq!(
            validate_input_package(selected, selected, ByteDigest::of(b"foreign")),
            Err(BundleRefusal::CanonicalPackageMismatch)
        );
    }

    #[test]
    fn dispatch_report_cannot_omit_or_reorder_the_engines_claim_population() {
        let missing = DispatchReport::empty_for_population_test();
        assert_eq!(
            validate_dispatch_population(&[Claim::Conformance], &missing),
            Err(BundleRefusal::DispatchReportMismatch)
        );
    }

    #[test]
    #[trace("TC-006", "FR-006-AC-1", "FR-006-AC-9")]
    fn dispatch_report_substitution_is_a_compile_time_error() {
        let source = include_str!("claims/dispatch.rs");
        assert!(source.contains("```compile_fail,E0451"));
        assert!(source.contains("let _ = Report { dispositions:"));
        let assessment = include_str!(concat!("assessment", ".rs"));
        assert!(assessment.contains("```compile_fail,E0599"));
        assert!(assessment.contains("engine.construct_canonical_outcome("));
        let forbidden_public_constructor = ["pub fn construct_", "canonical_outcome("].concat();
        assert!(!assessment.contains(&forbidden_public_constructor));
        let result_owner = include_str!("result/record.rs");
        assert!(result_owner.contains("pub(crate) fn produce("));
        assert!(result_owner.contains("pub(crate) fn produce_bounded("));
        assert!(!result_owner.contains("pub fn produce("));
    }

    #[test]
    #[trace("TC-011", "FR-008-AC-9")]
    fn tc_011_refusal_parent_and_children_form_one_complete_decomposition() {
        let parent = include_str!("../spec/functional/FR-008-return-typed-refusals.md");
        let children = [
            (
                "FR-036",
                include_str!("../spec/functional/FR-036-carry-complete-refusal-envelope.md"),
            ),
            (
                "FR-037",
                include_str!("../spec/functional/FR-037-derive-no-refusal-fact-from-text.md"),
            ),
            (
                "FR-038",
                include_str!("../spec/functional/FR-038-retain-unknown-source-contracts.md"),
            ),
            (
                "FR-039",
                include_str!("../spec/functional/FR-039-preserve-typed-causes-distinctly.md"),
            ),
            (
                "FR-040",
                include_str!("../spec/functional/FR-040-emit-no-success-beside-fatal-refusal.md"),
            ),
        ];

        let parent_frontmatter = parent
            .split_once("---\n")
            .and_then(|(_, remainder)| remainder.split_once("---\n"))
            .map(|(frontmatter, _)| frontmatter)
            .expect("FR-008 has YAML frontmatter");
        let behavior = parent
            .split_once("## Behavior")
            .and_then(|(_, remainder)| remainder.split_once("The load-bearing property"))
            .map(|(behavior, _)| behavior)
            .expect("FR-008 has its decomposition table");
        let criteria = parent
            .split_once("## Acceptance Criteria")
            .and_then(|(_, remainder)| remainder.split_once("## Owned and adopted vocabulary"))
            .map(|(criteria, _)| criteria)
            .expect("FR-008 has acceptance criteria");

        for (child_id, child) in children {
            let contains =
                format!("target: ix://agent-ix/quire-protocol/{child_id}\n    type: contains");
            assert_eq!(parent_frontmatter.matches(&contains).count(), 1);

            let child_frontmatter = child
                .split_once("---\n")
                .and_then(|(_, remainder)| remainder.split_once("---\n"))
                .map(|(frontmatter, _)| frontmatter)
                .expect("child requirement has YAML frontmatter");
            assert_eq!(
                child_frontmatter
                    .matches("target: ix://agent-ix/quire-protocol/FR-008\n    type: part_of",)
                    .count(),
                1
            );
            assert_eq!(behavior.matches(&format!("[{child_id}]")).count(), 1);

            for criterion in criteria
                .split_whitespace()
                .filter(|word| word.starts_with(child_id) && word.contains("-AC-"))
            {
                let criterion = criterion.trim_matches(|character: char| {
                    !character.is_ascii_alphanumeric() && character != '-'
                });
                assert!(
                    child.contains(&format!("| {criterion} |")),
                    "{criterion} must be stated by {child_id}"
                );
            }
        }

        for parent_criterion in 1..=8 {
            let row = criteria
                .lines()
                .find(|line| line.starts_with(&format!("| FR-008-AC-{parent_criterion} |")))
                .expect("each aggregated FR-008 criterion is present");
            assert!(
                (36..=40).any(|child| row.contains(&format!("FR-{child:03}-AC-"))),
                "FR-008-AC-{parent_criterion} must identify its atomic child criterion"
            );
        }
    }
}

/// Required global-closure premise kind.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub enum PremiseKind {
    /// Exact execution-closure premise.
    Execution,
    /// Exact branch-closure premise.
    Branch,
    /// Exact workflow-closure premise.
    Workflow,
}

impl PremiseKind {
    pub(crate) const fn label(self) -> &'static str {
        match self {
            Self::Execution => "execution",
            Self::Branch => "branch",
            Self::Workflow => "workflow",
        }
    }
}

/// One exact required global-closure premise.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct GlobalPremise {
    /// Premise axis.
    pub kind: PremiseKind,
    /// Exact premise identity.
    pub identity: String,
    /// Exact scope identity.
    pub scope_identity: String,
    /// Exact producer authority identity.
    pub authority_identity: String,
}

/// Global closure state plus independently selected required and carried sets.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GlobalConformanceInput {
    /// Independently supplied FR-247 state.
    pub state: GlobalConformanceClosure,
    /// Complete required premise population in authored order.
    pub required_premises: Vec<GlobalPremise>,
    /// Exact carried premise population in authored order.
    pub premises: Vec<GlobalPremise>,
}

/// Four independent QObs owner views retained without re-derivation.
#[derive(Clone, Copy, Debug)]
pub struct AxisInputs<'a> {
    /// Decision-scope progress authority view.
    pub decision_scope_progress: &'a progress::View,
    /// Decision-scope closure authority view.
    pub decision_scope_closure: &'a observation_closure::View,
    /// Surrounding-execution progress authority view.
    pub surrounding_execution_progress: &'a progress::View,
    /// Surrounding-execution closure authority view.
    pub surrounding_execution_closure: &'a observation_closure::View,
}
