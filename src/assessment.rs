// SPDX-License-Identifier: AGPL-3.0-or-later
//! Assessment orchestration over shared, producer-owned facts.
//!
//! Conformance assesses authenticated QSL/QObs inputs and the Engine carries
//! those stage outputs into private result construction. It does not parse or
//! repair owner bytes, and callers cannot restate the semantic result facts.

use quire_observation::authority::{
    closure as observation_closure, completeness, observation as observation_record, position,
    progress,
};
use quire_observation::QualifiedSubject;
use quire_spec_language::protocol_artifact::native_temporal::v2 as temporal_v2;
use quire_spec_language::protocol_artifact::v3;
use quire_spec_language::protocol_artifact::wire::{BindingKind, ControlOperation};
use quire_spec_language::protocol_artifact::ProtocolNumber;
use quire_spec_language::protocol_artifact::{
    checked_predicate::ValidatedCheckedPredicate, temporal_subject::ValidatedTemporalSubject,
};
use quire_spec_language::state::{
    self as state_evaluation, EvaluationOutcome, EvaluationReport, StateView,
    ValueKind as StateValueKind,
};

use crate::bounds::inventory::{
    production_inventory, AccountingFault, Budget, BudgetDimension,
    OperationOutcome as BoundedOutcome, ResourceAccountant, Usage as ResourceAccountantUsage,
};
use crate::bounds::Clock;
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
    ActivatedAssessmentError, ActivationBinding as ConformanceActivationBinding, ActivationFact,
    Kernel as ConformanceKernel, KernelRefusal as ConformanceKernelRefusal,
    Limits as ConformanceLimits,
};
use crate::model::v2::{
    link as link_v2, CompensationRegistrationInput, DefinitionIdentity, DefinitionRegistry,
    LinkedAssessment, LinkedProtocol, RuntimeBinding, SlotCompatibility, SlotHandle,
};
use crate::outcome::ClaimOutcome;
use crate::producer_contracts::{
    resolve_before_producer_read, ProducerReadiness, ProducerResolution, ProducerResolutionRequest,
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
use crate::version_lock::{ResolutionBudget, VersionLockManifest};

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

/// Caller-owned controls for one canonical claim row.
///
/// Every semantic result value is derived from the Engine's authenticated QSL,
/// QObs, conformance, and dispatch outputs. This type deliberately exposes only
/// the local reproduction selection and caller-lowered result-owner limits.
/// Correction semantics remain absent until an owner-authenticated
/// contradiction capability can supply them; a caller cannot restate one.
///
/// ```compile_fail,E0603
/// use quire_protocol::assessment::CanonicalRowFacts;
/// ```
#[derive(Clone, Debug)]
pub struct CanonicalRowControls {
    reproduction: ReproductionRecord,
    limits: result::Limits,
}

impl CanonicalRowControls {
    /// Select deterministic local-run provenance and result-owner ceilings.
    #[must_use]
    pub const fn new(reproduction: ReproductionRecord, limits: result::Limits) -> Self {
        Self {
            reproduction,
            limits,
        }
    }
}

/// Engine-derived transition value consumed only by canonical result assembly.
#[derive(Clone, Debug)]
struct CanonicalRowFacts<'a> {
    checked_predicate: &'a ValidatedCheckedPredicate,
    temporal_subject: &'a ValidatedTemporalSubject,
    truth: result::Truth,
    settlement: result::Settlement,
    activation: result::Activation,
    participation: result::Participation,
    observation_adequacy: result::Adequacy,
    protocol_adequacy: result::ProtocolAdequacy,
    decision_support: Vec<result::DecisionSupport>,
    limitations: Vec<result::Limitation>,
    model_artifact: &'a quire_spec_language::protocol_artifact::wire::ArtifactRef,
    runtime_input: &'a QualifiedSubject,
    reproduction: ReproductionRecord,
    limits: result::Limits,
}

#[derive(Clone, Copy, Debug)]
struct ResultSelection<'a> {
    checked_predicate: &'a ValidatedCheckedPredicate,
    temporal_subject: &'a ValidatedTemporalSubject,
}

#[derive(Clone, Debug)]
struct SelectedActivation<'a> {
    owner: ConformanceActivationBinding<'a>,
    subject: &'a QualifiedSubject,
}

#[derive(Clone, Copy, Debug)]
enum StartupRefusal {
    Bundle(BundleRefusal),
    Kernel(ConformanceKernelRefusal),
}

impl From<BundleRefusal> for StartupRefusal {
    fn from(value: BundleRefusal) -> Self {
        Self::Bundle(value)
    }
}

impl From<ConformanceKernelRefusal> for StartupRefusal {
    fn from(value: ConformanceKernelRefusal) -> Self {
        Self::Kernel(value)
    }
}

/// Why an exact linked assessment cannot become one complete claim bundle.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum BundleRefusal {
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
    /// The ordered claim-dispatch report differs from this Engine request or its rows.
    DispatchReportMismatch,
    /// Canonical production was requested without authenticated result-owner views.
    MissingResultSelection,
    /// The checked predicate belongs to another admitted package.
    CheckedPredicatePackageMismatch,
    /// The temporal subject belongs to another admitted package.
    TemporalSubjectPackageMismatch,
    /// The checked predicate names another temporal declaration.
    CheckedPredicateDeclarationMismatch,
    /// The strict temporal request names another declaration.
    ResultRequestDeclarationMismatch,
    /// The strict temporal request names another temporal-subject document.
    ResultRequestSubjectMismatch,
    /// The linked definition does not select the request's temporal declaration.
    ResultTemporalBindingMissing,
    /// The checked predicate is not reachable from the selected temporal subject.
    ResultPredicateUnreachable,
    /// The static closure does not select exactly one result model artifact.
    AmbiguousResultModel,
    /// Activated dispatch was attempted without one Engine-selected activation.
    MissingActivationSelection,
    /// The activation subject is not one of the linked runtime bindings.
    ActivationRuntimeBindingMismatch,
    /// An activation fact belongs to another Engine-selected activation.
    ActivationFactMismatch,
}

#[derive(Clone, Copy, Debug)]
enum RowAssemblyRefusal<'a> {
    UnknownClaimStrength(&'a [u8]),
    MissingAssumptionState(&'a [u8]),
    NonTextRequestIdentity(&'a [u8]),
    NonTextAdmittedFragmentIdentity(&'a [u8]),
    NonTextBackendIdentity(&'a [u8]),
    NonTextAssumptionIdentity(&'a [u8]),
    ForeignModelProvenance,
    ForeignRuntimeInput,
    UnrepresentableRuntimeInput,
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
    BUNDLE_DISPATCH_MISMATCH,
    "assessment-result.bundle-dispatch-mismatch",
    RefusalDomain::Result
);
bundle_code!(
    BUNDLE_MISSING_RESULT_SELECTION,
    "assessment-result.missing-owner-selection",
    RefusalDomain::Result
);
bundle_code!(
    BUNDLE_CHECKED_PREDICATE_PACKAGE_MISMATCH,
    "assessment-result.checked-predicate-package-mismatch",
    RefusalDomain::Result
);
bundle_code!(
    BUNDLE_TEMPORAL_SUBJECT_PACKAGE_MISMATCH,
    "assessment-result.temporal-subject-package-mismatch",
    RefusalDomain::Result
);
bundle_code!(
    BUNDLE_CHECKED_PREDICATE_DECLARATION_MISMATCH,
    "assessment-result.checked-predicate-declaration-mismatch",
    RefusalDomain::Result
);
bundle_code!(
    BUNDLE_RESULT_REQUEST_DECLARATION_MISMATCH,
    "assessment-result.request-declaration-mismatch",
    RefusalDomain::Result
);
bundle_code!(
    BUNDLE_RESULT_REQUEST_SUBJECT_MISMATCH,
    "assessment-result.request-subject-mismatch",
    RefusalDomain::Result
);
bundle_code!(
    BUNDLE_RESULT_TEMPORAL_BINDING_MISSING,
    "assessment-result.temporal-binding-missing",
    RefusalDomain::Result
);
bundle_code!(
    BUNDLE_RESULT_PREDICATE_UNREACHABLE,
    "assessment-result.predicate-unreachable",
    RefusalDomain::Result
);
bundle_code!(
    BUNDLE_AMBIGUOUS_RESULT_MODEL,
    "assessment-result.ambiguous-model-selection",
    RefusalDomain::Result
);
bundle_code!(
    ROW_UNKNOWN_CLAIM_STRENGTH,
    "assessment-result.unknown-claim-strength",
    RefusalDomain::Result
);
bundle_code!(
    ROW_MISSING_ASSUMPTION_STATE,
    "assessment-result.missing-assumption-state",
    RefusalDomain::Result
);
bundle_code!(
    ROW_NON_TEXT_REQUEST_IDENTITY,
    "assessment-result.non-text-request-identity",
    RefusalDomain::Result
);
bundle_code!(
    ROW_NON_TEXT_ADMITTED_FRAGMENT_IDENTITY,
    "assessment-result.non-text-admitted-fragment-identity",
    RefusalDomain::Result
);
bundle_code!(
    ROW_NON_TEXT_BACKEND_IDENTITY,
    "assessment-result.non-text-backend-identity",
    RefusalDomain::Result
);
bundle_code!(
    ROW_NON_TEXT_ASSUMPTION_IDENTITY,
    "assessment-result.non-text-assumption-identity",
    RefusalDomain::Result
);
bundle_code!(
    ROW_FOREIGN_MODEL_PROVENANCE,
    "assessment-result.foreign-model-provenance",
    RefusalDomain::Result
);
bundle_code!(
    ROW_FOREIGN_RUNTIME_INPUT,
    "assessment-result.foreign-runtime-input",
    RefusalDomain::Result
);
bundle_code!(
    ROW_UNREPRESENTABLE_RUNTIME_INPUT,
    "assessment-result.unrepresentable-runtime-input",
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
    BUNDLE_MISSING_ACTIVATION_SELECTION,
    "assessment-activation.missing-selection",
    RefusalDomain::Assessment
);
bundle_code!(
    BUNDLE_ACTIVATION_RUNTIME_BINDING,
    "assessment-activation.runtime-binding-mismatch",
    RefusalDomain::Assessment
);
bundle_code!(
    BUNDLE_ACTIVATION_FACT,
    "assessment-activation.fact-mismatch",
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
        BUNDLE_DISPATCH_MISMATCH,
        BUNDLE_MISSING_RESULT_SELECTION,
        BUNDLE_CHECKED_PREDICATE_PACKAGE_MISMATCH,
        BUNDLE_TEMPORAL_SUBJECT_PACKAGE_MISMATCH,
        BUNDLE_CHECKED_PREDICATE_DECLARATION_MISMATCH,
        BUNDLE_RESULT_REQUEST_DECLARATION_MISMATCH,
        BUNDLE_RESULT_REQUEST_SUBJECT_MISMATCH,
        BUNDLE_RESULT_TEMPORAL_BINDING_MISSING,
        BUNDLE_RESULT_PREDICATE_UNREACHABLE,
        BUNDLE_AMBIGUOUS_RESULT_MODEL,
        ROW_UNKNOWN_CLAIM_STRENGTH,
        ROW_MISSING_ASSUMPTION_STATE,
        ROW_NON_TEXT_REQUEST_IDENTITY,
        ROW_NON_TEXT_ADMITTED_FRAGMENT_IDENTITY,
        ROW_NON_TEXT_BACKEND_IDENTITY,
        ROW_NON_TEXT_ASSUMPTION_IDENTITY,
        ROW_FOREIGN_MODEL_PROVENANCE,
        ROW_FOREIGN_RUNTIME_INPUT,
        ROW_UNREPRESENTABLE_RUNTIME_INPUT,
        RESULT_ACCOUNTING_FAULT,
    ],
};

/// Conformance-stage activation-binding codes.
pub const ACTIVATION_CODES: CodeModule = CodeModule {
    root: "assessment-activation",
    owner: "Task-006",
    module_path: "quire_protocol::assessment",
    entries: &[
        BUNDLE_MISSING_ACTIVATION_SELECTION,
        BUNDLE_ACTIVATION_RUNTIME_BINDING,
        BUNDLE_ACTIVATION_FACT,
    ],
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
        BundleRefusal::DispatchReportMismatch => {
            (&BUNDLE_DISPATCH_MISMATCH, ProtocolStage::ResultAssembly)
        }
        BundleRefusal::MissingResultSelection => (
            &BUNDLE_MISSING_RESULT_SELECTION,
            ProtocolStage::ResultAssembly,
        ),
        BundleRefusal::CheckedPredicatePackageMismatch => (
            &BUNDLE_CHECKED_PREDICATE_PACKAGE_MISMATCH,
            ProtocolStage::ResultAssembly,
        ),
        BundleRefusal::TemporalSubjectPackageMismatch => (
            &BUNDLE_TEMPORAL_SUBJECT_PACKAGE_MISMATCH,
            ProtocolStage::ResultAssembly,
        ),
        BundleRefusal::CheckedPredicateDeclarationMismatch => (
            &BUNDLE_CHECKED_PREDICATE_DECLARATION_MISMATCH,
            ProtocolStage::ResultAssembly,
        ),
        BundleRefusal::ResultRequestDeclarationMismatch => (
            &BUNDLE_RESULT_REQUEST_DECLARATION_MISMATCH,
            ProtocolStage::ResultAssembly,
        ),
        BundleRefusal::ResultRequestSubjectMismatch => (
            &BUNDLE_RESULT_REQUEST_SUBJECT_MISMATCH,
            ProtocolStage::ResultAssembly,
        ),
        BundleRefusal::ResultTemporalBindingMissing => (
            &BUNDLE_RESULT_TEMPORAL_BINDING_MISSING,
            ProtocolStage::ResultAssembly,
        ),
        BundleRefusal::ResultPredicateUnreachable => (
            &BUNDLE_RESULT_PREDICATE_UNREACHABLE,
            ProtocolStage::ResultAssembly,
        ),
        BundleRefusal::AmbiguousResultModel => (
            &BUNDLE_AMBIGUOUS_RESULT_MODEL,
            ProtocolStage::ResultAssembly,
        ),
        BundleRefusal::MissingActivationSelection => (
            &BUNDLE_MISSING_ACTIVATION_SELECTION,
            ProtocolStage::Conformance,
        ),
        BundleRefusal::ActivationRuntimeBindingMismatch => (
            &BUNDLE_ACTIVATION_RUNTIME_BINDING,
            ProtocolStage::Conformance,
        ),
        BundleRefusal::ActivationFactMismatch => {
            (&BUNDLE_ACTIVATION_FACT, ProtocolStage::Conformance)
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

fn row_assembly_refusal(error: RowAssemblyRefusal<'_>) -> ProtocolRefusal {
    let (entry, affected, cause) = match error {
        RowAssemblyRefusal::UnknownClaimStrength(value) => (
            &ROW_UNKNOWN_CLAIM_STRENGTH,
            value,
            ProtocolCause::UnsupportedCapability {
                capability: RetainedBytes::new(value),
            },
        ),
        RowAssemblyRefusal::MissingAssumptionState(identity) => (
            &ROW_MISSING_ASSUMPTION_STATE,
            identity,
            ProtocolCause::IncompleteEvidence {
                evidence: RetainedBytes::new(identity),
            },
        ),
        RowAssemblyRefusal::NonTextRequestIdentity(value) => (
            &ROW_NON_TEXT_REQUEST_IDENTITY,
            value,
            ProtocolCause::IncompleteEvidence {
                evidence: RetainedBytes::new(value),
            },
        ),
        RowAssemblyRefusal::NonTextAdmittedFragmentIdentity(value) => (
            &ROW_NON_TEXT_ADMITTED_FRAGMENT_IDENTITY,
            value,
            ProtocolCause::IncompleteEvidence {
                evidence: RetainedBytes::new(value),
            },
        ),
        RowAssemblyRefusal::NonTextBackendIdentity(value) => (
            &ROW_NON_TEXT_BACKEND_IDENTITY,
            value,
            ProtocolCause::IncompleteEvidence {
                evidence: RetainedBytes::new(value),
            },
        ),
        RowAssemblyRefusal::NonTextAssumptionIdentity(value) => (
            &ROW_NON_TEXT_ASSUMPTION_IDENTITY,
            value,
            ProtocolCause::IncompleteEvidence {
                evidence: RetainedBytes::new(value),
            },
        ),
        RowAssemblyRefusal::ForeignModelProvenance => (
            &ROW_FOREIGN_MODEL_PROVENANCE,
            b"model-provenance".as_slice(),
            ProtocolCause::IdentityMismatch {
                expected: RetainedBytes::new(b"engine-selected-model"),
                observed: RetainedBytes::new(b"foreign-model-provenance"),
            },
        ),
        RowAssemblyRefusal::ForeignRuntimeInput => (
            &ROW_FOREIGN_RUNTIME_INPUT,
            b"runtime-input-provenance".as_slice(),
            ProtocolCause::IdentityMismatch {
                expected: RetainedBytes::new(b"engine-bound-runtime-input"),
                observed: RetainedBytes::new(b"foreign-runtime-input"),
            },
        ),
        RowAssemblyRefusal::UnrepresentableRuntimeInput => (
            &ROW_UNREPRESENTABLE_RUNTIME_INPUT,
            b"runtime-input-identity".as_slice(),
            ProtocolCause::IncompleteEvidence {
                evidence: RetainedBytes::new(b"runtime-input-identity"),
            },
        ),
    };
    let code = RefusalCode::of(entry);
    ProtocolRefusal::builder(
        code,
        CatalogSelection::local(),
        ProtocolStage::ResultAssembly,
        RefusalAxis::from_registered_code(code),
        cause.into(),
        ResourceUsage::protocol("quire.protocol/assessment-usage/1", &[]),
    )
    .expect("row-assembly codes belong to the result stage")
    .identity(AffectedIdentity::retained(affected))
    .prerequisite(Prerequisite::new(
        ProtocolStage::ClaimDispatch,
        PrerequisiteState::Established,
    ))
    .unavailable_location(Unavailability::NotApplicableToStage)
    .emitting_nothing()
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
        | BundleRefusal::DispatchReportMismatch
        | BundleRefusal::MissingResultSelection
        | BundleRefusal::CheckedPredicatePackageMismatch
        | BundleRefusal::TemporalSubjectPackageMismatch
        | BundleRefusal::CheckedPredicateDeclarationMismatch
        | BundleRefusal::ResultRequestDeclarationMismatch
        | BundleRefusal::ResultRequestSubjectMismatch
        | BundleRefusal::ResultTemporalBindingMissing
        | BundleRefusal::ResultPredicateUnreachable
        | BundleRefusal::AmbiguousResultModel
        | BundleRefusal::MissingActivationSelection
        | BundleRefusal::ActivationRuntimeBindingMismatch
        | BundleRefusal::ActivationFactMismatch => ProtocolCause::IdentityMismatch {
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
    activation: Option<SelectedActivation<'a>>,
    result_selection: Option<ResultSelection<'a>>,
    requested: Vec<Claim>,
}

/// An assessment engine whose complete producer inventory resolved first.
///
/// Construction is private to the atomic `start_*_after_resolving` boundaries.
/// Consequently the public producer-dependent assessment path cannot consume
/// QObs/QSL values with an absent, unresolved, unpinned, repointed, or
/// differently request-bound manifest.
#[derive(Debug)]
pub struct ResolvedAssessmentEngine<'a> {
    engine: Engine<'a>,
    resolution: ProducerResolution,
}

impl ResolvedAssessmentEngine<'_> {
    /// Immutable evidence that authorized this producer-dependent assessment.
    #[must_use]
    pub const fn producer_resolution(&self) -> &ProducerResolution {
        &self.resolution
    }
}

impl<'a> Engine<'a> {
    /// Start one assessment using the shared public operation outcome algebra.
    pub(crate) fn start(
        context: LinkedAssessment<'a>,
        requested: Vec<Claim>,
    ) -> crate::outcome::OperationOutcome<Self> {
        match Self::new(context, requested) {
            Ok(engine) => crate::outcome::OperationOutcome::Completed(engine),
            Err(error) => crate::outcome::OperationOutcome::Refused(outcome_refusal(error)),
        }
    }

    /// Start one activated assessment through the shared public operation
    /// outcome algebra.
    fn start_activated(
        context: LinkedAssessment<'a>,
        requested: Vec<Claim>,
        mappings: &v3::AdmittedPackage,
        subject: &'a QualifiedSubject,
        temporal_subject: &'a ValidatedTemporalSubject,
        request: &'a temporal_v2::ValidatedRequest,
        result: &'a temporal_v2::ValidatedResult,
    ) -> crate::outcome::OperationOutcome<Self> {
        match Self::new_activated(
            context,
            requested,
            mappings,
            subject,
            temporal_subject,
            request,
            result,
        ) {
            Ok(engine) => crate::outcome::OperationOutcome::Completed(engine),
            Err(StartupRefusal::Bundle(error)) => {
                crate::outcome::OperationOutcome::Refused(outcome_refusal(error))
            }
            Err(StartupRefusal::Kernel(error)) => {
                crate::outcome::OperationOutcome::Refused(kernel_refusal(error))
            }
        }
    }

    /// Start canonical result orchestration bound to exact strict QSL owner
    /// views for this activated request.
    #[allow(clippy::too_many_arguments)]
    fn start_result_assessment(
        context: LinkedAssessment<'a>,
        requested: Vec<Claim>,
        mappings: &v3::AdmittedPackage,
        subject: &'a QualifiedSubject,
        request: &'a temporal_v2::ValidatedRequest,
        result: &'a temporal_v2::ValidatedResult,
        checked_predicate: &'a ValidatedCheckedPredicate,
        temporal_subject: &'a ValidatedTemporalSubject,
    ) -> crate::outcome::OperationOutcome<Self> {
        match Self::new_result_assessment(
            context,
            requested,
            mappings,
            subject,
            request,
            result,
            checked_predicate,
            temporal_subject,
        ) {
            Ok(engine) => crate::outcome::OperationOutcome::Completed(engine),
            Err(StartupRefusal::Bundle(error)) => {
                crate::outcome::OperationOutcome::Refused(outcome_refusal(error))
            }
            Err(StartupRefusal::Kernel(error)) => {
                crate::outcome::OperationOutcome::Refused(kernel_refusal(error))
            }
        }
    }

    /// Start one assessment over an exact linked context and claim population.
    pub(crate) fn new(
        context: LinkedAssessment<'a>,
        requested: Vec<Claim>,
    ) -> Result<Self, BundleRefusal> {
        Self::new_bound(context, requested, None)
    }

    /// Start one assessment with an exact QSL-v2/QObs activation binding.
    fn new_activated(
        context: LinkedAssessment<'a>,
        requested: Vec<Claim>,
        mappings: &v3::AdmittedPackage,
        subject: &'a QualifiedSubject,
        temporal_subject: &'a ValidatedTemporalSubject,
        request: &'a temporal_v2::ValidatedRequest,
        result: &'a temporal_v2::ValidatedResult,
    ) -> Result<Self, StartupRefusal> {
        let mut engine = Self::new_bound(context, requested, None)?;
        if !engine.context.runtime().bindings().iter().any(|binding| {
            binding
                .subject_value()
                .is_some_and(|runtime_subject| runtime_subject == subject)
        }) {
            return Err(BundleRefusal::ActivationRuntimeBindingMismatch.into());
        }
        let owner = engine.conformance.bind_activation(
            &engine.linked,
            mappings,
            subject,
            temporal_subject,
            request,
            result,
        )?;
        engine.activation = Some(SelectedActivation { owner, subject });
        Ok(engine)
    }

    #[allow(clippy::too_many_arguments)]
    fn new_result_assessment(
        context: LinkedAssessment<'a>,
        requested: Vec<Claim>,
        mappings: &v3::AdmittedPackage,
        subject: &'a QualifiedSubject,
        request: &'a temporal_v2::ValidatedRequest,
        result: &'a temporal_v2::ValidatedResult,
        checked_predicate: &'a ValidatedCheckedPredicate,
        temporal_subject: &'a ValidatedTemporalSubject,
    ) -> Result<Self, StartupRefusal> {
        let mut engine = Self::new_activated(
            context,
            requested,
            mappings,
            subject,
            temporal_subject,
            request,
            result,
        )?;
        validate_result_selection(
            engine.context.package().digest(),
            &engine.identity,
            request,
            checked_predicate,
            temporal_subject,
        )?;
        engine.result_selection = Some(ResultSelection {
            checked_predicate,
            temporal_subject,
        });
        Ok(engine)
    }

    fn new_bound(
        context: LinkedAssessment<'a>,
        requested: Vec<Claim>,
        result_selection: Option<ResultSelection<'a>>,
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
            activation: None,
            result_selection,
            requested,
        })
    }

    /// Returns every exact QSL-compensation/QObs-effect registration input
    /// captured when this non-retargetable request was created.
    #[must_use]
    pub(crate) fn compensation_inputs(&self) -> &[CompensationRegistrationInput<'a>] {
        &self.compensation_inputs
    }

    /// Returns the exact QSL graph and static compensation definitions linked
    /// from this request's immutable context.
    #[must_use]
    pub(crate) const fn linked(&self) -> &LinkedProtocol<'a> {
        &self.linked
    }

    /// Returns the one linked-graph/compensation kernel for both conformance
    /// frontends of this assessment.
    #[must_use]
    pub(crate) const fn conformance(&self) -> &ConformanceKernel {
        &self.conformance
    }

    /// Returns the replay-idempotent registrations derived for this request.
    #[must_use]
    pub(crate) const fn compensation(&self) -> &CompensationSnapshot {
        &self.compensation
    }

    /// Returns the owned graph-derived activation, attempt, effect, commit,
    /// ordering and recovery state committed by this non-retargetable request.
    #[must_use]
    pub(crate) const fn compensation_state(&self) -> &CompensationState {
        &self.compensation_state
    }

    /// Returns the immutable activation pair where this assessment is event-triggered.
    #[must_use]
    pub(crate) fn activation(&self) -> Option<&ConformanceActivationBinding<'a>> {
        self.activation.as_ref().map(|selected| &selected.owner)
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
    pub(crate) fn dispatch_activated(
        &self,
        requests: &[ClaimRequest<'_>],
        facts: &[ActivationFact<'a>],
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
        if let Err(error) = self.validate_activation_facts(facts) {
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

    fn validate_activation_facts(&self, facts: &[ActivationFact<'_>]) -> Result<(), BundleRefusal> {
        let Some(selected) = &self.activation else {
            return Err(BundleRefusal::MissingActivationSelection);
        };
        for fact in facts {
            let offered = fact.occurrence().activation();
            if !selected.owner.same_binding(offered) {
                return Err(BundleRefusal::ActivationFactMismatch);
            }
        }
        Ok(())
    }

    /// Evaluate the exact admitted guard and transactionally create every
    /// registration paired with the selected established forward effect.
    ///
    /// False, incomplete, refusing, exhausted, and non-Boolean evaluations
    /// add no registration. The complete QSL report is returned unchanged.
    pub(crate) fn activate_compensation(
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
    pub(crate) fn admit_compensation_attempt(
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
    pub(crate) fn record_compensation_effect(
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
    pub(crate) fn record_compensation_commit(
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
    pub(crate) fn evaluate_compensation_recovery(
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
    pub(crate) fn assess_compensation_ordering(
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
    /// public path is [`ResolvedAssessmentEngine::assess_activated`], which
    /// owns production and strict reading from the Engine's exact linked
    /// assessment after producer resolution.
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
        facts: &[ActivationFact<'a>],
        controls: Vec<Option<CanonicalRowControls>>,
        resources: &mut ResourceAccountant<'_>,
    ) -> crate::outcome::OperationOutcome<AssessmentBundle<'a, result::ValidatedResult>> {
        if let Err(error) = validate_dispatch_rows(&self.requested, dispatch, &controls) {
            return crate::outcome::OperationOutcome::Refused(outcome_refusal(error));
        }
        let Some(selection) = self.result_selection else {
            return crate::outcome::OperationOutcome::Refused(outcome_refusal(
                BundleRefusal::MissingResultSelection,
            ));
        };
        let model_artifact = match selected_model_artifact(&self.context, &self.identity) {
            Ok(model) => model,
            Err(error) => {
                return crate::outcome::OperationOutcome::Refused(outcome_refusal(error));
            }
        };
        let Some(activation) = &self.activation else {
            return crate::outcome::OperationOutcome::Refused(outcome_refusal(
                BundleRefusal::MissingResultSelection,
            ));
        };
        let runtime_input = activation.subject;
        let mut rows = Vec::with_capacity(controls.len());
        for (control, disposition) in controls.into_iter().zip(dispatch.dispositions()) {
            let outcome = if let Some(failure) = &disposition.failure {
                ClaimOutcome::Refused(failure.envelope().clone())
            } else {
                let Some(control) = control else {
                    return crate::outcome::OperationOutcome::Refused(outcome_refusal(
                        BundleRefusal::MissingClaimRow,
                    ));
                };
                let selected = derive_canonical_row_facts(
                    selection,
                    disposition,
                    conformance,
                    facts,
                    axes,
                    completeness,
                    &self.context,
                    model_artifact,
                    runtime_input,
                    control,
                );
                match selected {
                    Err(refusal) => ClaimOutcome::Refused(refusal.envelope().clone()),
                    Ok(selected) => match result_input_from_dispatch(
                        disposition,
                        &self.identity,
                        &self.context,
                        axes,
                        completeness,
                        global_conformance,
                        selected,
                    ) {
                        Err(refusal) => ClaimOutcome::Refused(row_assembly_refusal(refusal)),
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
                    },
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
            Ok(bundle) => crate::outcome::OperationOutcome::Completed(bundle),
            Err(error) => crate::outcome::OperationOutcome::Refused(outcome_refusal(error)),
        }
    }

    /// Run conformance, claim dispatch, canonical result production, and strict
    /// reading as one non-retargetable assessment operation.
    ///
    /// Callers provide `Some` local controls only for dispositions that can
    /// reach row construction and `None` for claim-local dispatch refusals. The
    /// Engine derives every semantic result value and provenance selection from
    /// exact authenticated stage outputs. One resource accountant spans every
    /// stage.
    ///
    /// ```compile_fail,E0599
    /// use quire_protocol::assessment::Engine;
    /// use quire_protocol::claims::dispatch::Report;
    ///
    /// fn retarget<'a>(
    ///     engine: Engine<'a>,
    ///     foreign_report: &Report,
    /// ) {
    ///     // Result construction accepts no report produced by another Engine.
    ///     let _ = engine.construct_canonical_outcome(foreign_report);
    /// }
    /// ```
    #[allow(clippy::too_many_arguments)]
    fn assess_activated_after_resolution(
        self,
        requests: &[ClaimRequest<'_>],
        facts: &[ActivationFact<'a>],
        axes: AxisInputs<'a>,
        completeness: &'a completeness::View,
        global_conformance: &GlobalConformanceInput,
        execution: crate::closure::AssessmentExecution,
        conformance_limits: ConformanceLimits,
        dispatch_limits: DispatchLimits,
        controls: Vec<Option<CanonicalRowControls>>,
        budget: Budget,
    ) -> crate::outcome::OperationOutcome<AssessmentBundle<'a, result::ValidatedResult>> {
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
        self.construct_canonical_bounded(
            &dispatch,
            &conformance,
            axes,
            completeness,
            global_conformance,
            facts,
            controls,
            &mut resources,
        )
    }
}

/// Producer-backed inputs loaded only after the complete manifest resolves.
pub struct AssessmentInputs<'a> {
    /// Exact linked package, static definition and runtime bindings.
    pub context: LinkedAssessment<'a>,
    /// Closed claim population selected for this assessment.
    pub requested: Vec<Claim>,
}

/// Event-activated inputs loaded only after the complete manifest resolves.
pub struct ActivatedAssessmentInputs<'a> {
    /// Exact linked package, static definition and runtime bindings.
    pub context: LinkedAssessment<'a>,
    /// Closed claim population selected for this assessment.
    pub requested: Vec<Claim>,
    /// Strict QSL v3 activation mapping package.
    pub mappings: &'a v3::AdmittedPackage,
    /// QObs authority-qualified activation subject.
    pub subject: &'a QualifiedSubject,
    /// Strict QSL temporal subject.
    pub temporal_subject: &'a ValidatedTemporalSubject,
    /// Strict QSL native-temporal request.
    pub request: &'a temporal_v2::ValidatedRequest,
    /// Strict QSL native-temporal result bound to `request`.
    pub result: &'a temporal_v2::ValidatedResult,
}

/// Canonical-result inputs loaded only after the complete manifest resolves.
pub struct ResultAssessmentInputs<'a> {
    /// Exact linked package, static definition and runtime bindings.
    pub context: LinkedAssessment<'a>,
    /// Closed claim population selected for this assessment.
    pub requested: Vec<Claim>,
    /// Strict QSL v3 activation mapping package.
    pub mappings: &'a v3::AdmittedPackage,
    /// QObs authority-qualified activation subject.
    pub subject: &'a QualifiedSubject,
    /// Strict QSL native-temporal request.
    pub request: &'a temporal_v2::ValidatedRequest,
    /// Strict QSL native-temporal result bound to `request`.
    pub result: &'a temporal_v2::ValidatedResult,
    /// Strict QSL checked predicate selected for result construction.
    pub checked_predicate: &'a ValidatedCheckedPredicate,
    /// Strict QSL temporal subject selected for result construction.
    pub temporal_subject: &'a ValidatedTemporalSubject,
}

/// Resolve every producer before invoking the loader for a non-temporal
/// assessment. The loader is not called on refusal or resource exhaustion.
#[must_use]
pub fn start_assessment_after_resolving<'a, C: Clock + 'static>(
    request: &ProducerResolutionRequest,
    manifest: &VersionLockManifest,
    budget: ResolutionBudget,
    clock: C,
    load: impl FnOnce() -> AssessmentInputs<'a>,
) -> ProducerReadiness<crate::outcome::OperationOutcome<ResolvedAssessmentEngine<'a>>> {
    resolve_before_producer_read(request, manifest, budget, clock, |resolution| {
        resolution.start(load())
    })
}

/// Resolve every producer before invoking the loader for an event-activated
/// assessment. The loader is not called on refusal or resource exhaustion.
#[must_use]
pub fn start_activated_after_resolving<'a, C: Clock + 'static>(
    request: &ProducerResolutionRequest,
    manifest: &VersionLockManifest,
    budget: ResolutionBudget,
    clock: C,
    load: impl FnOnce() -> ActivatedAssessmentInputs<'a>,
) -> ProducerReadiness<crate::outcome::OperationOutcome<ResolvedAssessmentEngine<'a>>> {
    resolve_before_producer_read(request, manifest, budget, clock, |resolution| {
        resolution.start_activated(load())
    })
}

/// Resolve every producer before invoking the loader for canonical result
/// assessment. The loader is not called on refusal or resource exhaustion.
#[must_use]
pub fn start_result_assessment_after_resolving<'a, C: Clock + 'static>(
    request: &ProducerResolutionRequest,
    manifest: &VersionLockManifest,
    budget: ResolutionBudget,
    clock: C,
    load: impl FnOnce() -> ResultAssessmentInputs<'a>,
) -> ProducerReadiness<crate::outcome::OperationOutcome<ResolvedAssessmentEngine<'a>>> {
    resolve_before_producer_read(request, manifest, budget, clock, |resolution| {
        resolution.start_result_assessment(load())
    })
}

impl ProducerResolution {
    fn start<'a>(
        self,
        inputs: AssessmentInputs<'a>,
    ) -> crate::outcome::OperationOutcome<ResolvedAssessmentEngine<'a>> {
        let actual = ProducerResolutionRequest::for_assessment(
            inputs.context.package().digest(),
            inputs.context.definition().selection().digest,
        );
        match self.authorize(&actual) {
            crate::outcome::OperationOutcome::Completed(()) => {}
            crate::outcome::OperationOutcome::Refused(refusal) => {
                return crate::outcome::OperationOutcome::Refused(refusal)
            }
            crate::outcome::OperationOutcome::ResourceIncomplete(incomplete) => {
                return crate::outcome::OperationOutcome::ResourceIncomplete(incomplete)
            }
        }
        Engine::start(inputs.context, inputs.requested).map(|engine| ResolvedAssessmentEngine {
            engine,
            resolution: self,
        })
    }

    /// Start one activated assessment only after this complete producer
    /// resolution exists.
    fn start_activated<'a>(
        self,
        inputs: ActivatedAssessmentInputs<'a>,
    ) -> crate::outcome::OperationOutcome<ResolvedAssessmentEngine<'a>> {
        let actual = ProducerResolutionRequest::for_temporal_assessment(
            inputs.context.package().digest(),
            inputs.context.definition().selection().digest,
            inputs.request.document().digest(),
        );
        match self.authorize(&actual) {
            crate::outcome::OperationOutcome::Completed(()) => {}
            crate::outcome::OperationOutcome::Refused(refusal) => {
                return crate::outcome::OperationOutcome::Refused(refusal)
            }
            crate::outcome::OperationOutcome::ResourceIncomplete(incomplete) => {
                return crate::outcome::OperationOutcome::ResourceIncomplete(incomplete)
            }
        }
        Engine::start_activated(
            inputs.context,
            inputs.requested,
            inputs.mappings,
            inputs.subject,
            inputs.temporal_subject,
            inputs.request,
            inputs.result,
        )
        .map(|engine| ResolvedAssessmentEngine {
            engine,
            resolution: self,
        })
    }

    /// Start canonical result orchestration only after this complete producer
    /// resolution exists.
    fn start_result_assessment<'a>(
        self,
        inputs: ResultAssessmentInputs<'a>,
    ) -> crate::outcome::OperationOutcome<ResolvedAssessmentEngine<'a>> {
        let actual = ProducerResolutionRequest::for_temporal_assessment(
            inputs.context.package().digest(),
            inputs.context.definition().selection().digest,
            inputs.request.document().digest(),
        );
        match self.authorize(&actual) {
            crate::outcome::OperationOutcome::Completed(()) => {}
            crate::outcome::OperationOutcome::Refused(refusal) => {
                return crate::outcome::OperationOutcome::Refused(refusal)
            }
            crate::outcome::OperationOutcome::ResourceIncomplete(incomplete) => {
                return crate::outcome::OperationOutcome::ResourceIncomplete(incomplete)
            }
        }
        Engine::start_result_assessment(
            inputs.context,
            inputs.requested,
            inputs.mappings,
            inputs.subject,
            inputs.request,
            inputs.result,
            inputs.checked_predicate,
            inputs.temporal_subject,
        )
        .map(|engine| ResolvedAssessmentEngine {
            engine,
            resolution: self,
        })
    }
}

impl<'a> ResolvedAssessmentEngine<'a> {
    /// Producer-backed compensation registrations retained by this request.
    #[must_use]
    pub fn compensation_inputs(&self) -> &[CompensationRegistrationInput<'a>] {
        self.engine.compensation_inputs()
    }

    /// Exact linked QSL graph selected for this request.
    #[must_use]
    pub const fn linked(&self) -> &LinkedProtocol<'a> {
        self.engine.linked()
    }

    /// Conformance kernel bound to this exact linked request.
    #[must_use]
    pub const fn conformance(&self) -> &ConformanceKernel {
        self.engine.conformance()
    }

    /// Replay-idempotent compensation registrations.
    #[must_use]
    pub const fn compensation(&self) -> &CompensationSnapshot {
        self.engine.compensation()
    }

    /// Owned compensation transition state.
    #[must_use]
    pub const fn compensation_state(&self) -> &CompensationState {
        self.engine.compensation_state()
    }

    /// Immutable event activation retained by an activated assessment.
    #[must_use]
    pub fn activation(&self) -> Option<&ConformanceActivationBinding<'a>> {
        self.engine.activation()
    }

    /// Dispatch claims only through this resolved assessment request.
    #[allow(
        clippy::too_many_arguments,
        reason = "the wrapper preserves the engine's independently owned conformance and dispatch inputs"
    )]
    pub fn dispatch_activated(
        &self,
        requests: &[ClaimRequest<'_>],
        facts: &[ActivationFact<'a>],
        axes: AxisInputs<'_>,
        completeness: &completeness::View,
        global_conformance: &GlobalConformanceInput,
        execution: crate::closure::AssessmentExecution,
        conformance_limits: ConformanceLimits,
        dispatch_limits: DispatchLimits,
        budget: Budget,
    ) -> crate::outcome::OperationOutcome<DispatchReport> {
        self.engine.dispatch_activated(
            requests,
            facts,
            axes,
            completeness,
            global_conformance,
            execution,
            conformance_limits,
            dispatch_limits,
            budget,
        )
    }

    /// Activate one graph-selected compensation under resolved producers.
    pub fn activate_compensation(
        &mut self,
        compensation_handle: u32,
        view: &StateView,
        limits: state_evaluation::Limits,
    ) -> CompensationOutcome {
        self.engine
            .activate_compensation(compensation_handle, view, limits)
    }

    /// Admit one compensation attempt under resolved producers.
    #[allow(
        clippy::too_many_arguments,
        reason = "the wrapper preserves the engine's independently owned compensation inputs"
    )]
    pub fn admit_compensation_attempt(
        &mut self,
        compensation_handle: u32,
        binding: &RuntimeBinding,
        established_effect: Option<&RuntimeBinding>,
        outcome: Option<AttemptOutcome>,
        retry_view: Option<&StateView>,
        limits: state_evaluation::Limits,
    ) -> CompensationOutcome {
        self.engine.admit_compensation_attempt(
            compensation_handle,
            binding,
            established_effect,
            outcome,
            retry_view,
            limits,
        )
    }

    /// Record one exact compensation effect under resolved producers.
    pub fn record_compensation_effect(
        &mut self,
        compensation_handle: u32,
        binding: &RuntimeBinding,
        disposition: Option<EffectDisposition>,
    ) -> CompensationOutcome {
        self.engine
            .record_compensation_effect(compensation_handle, binding, disposition)
    }

    /// Record one exact compensation commit under resolved producers.
    pub fn record_compensation_commit(
        &mut self,
        compensation_handle: u32,
        binding: &RuntimeBinding,
    ) -> CompensationOutcome {
        self.engine
            .record_compensation_commit(compensation_handle, binding)
    }

    /// Evaluate compensation recovery under resolved producers.
    pub fn evaluate_compensation_recovery(
        &mut self,
        compensation_handle: u32,
        view: &StateView,
        limits: state_evaluation::Limits,
    ) -> CompensationOutcome {
        self.engine
            .evaluate_compensation_recovery(compensation_handle, view, limits)
    }

    /// Assess compensation ordering through QObs views under resolved producers.
    pub fn assess_compensation_ordering(
        &mut self,
        compensation_handle: u32,
        forward: &observation_record::View,
        effect: &observation_record::View,
        commit: Option<&observation_record::View>,
        positions: &position::View,
    ) -> CompensationOutcome {
        self.engine.assess_compensation_ordering(
            compensation_handle,
            forward,
            effect,
            commit,
            positions,
        )
    }

    /// Run conformance, claim dispatch, canonical result production, and
    /// strict reading after exact producer resolution.
    #[allow(clippy::too_many_arguments)]
    pub fn assess_activated(
        self,
        requests: &[ClaimRequest<'_>],
        facts: &[ActivationFact<'a>],
        axes: AxisInputs<'a>,
        completeness: &'a completeness::View,
        global_conformance: &GlobalConformanceInput,
        execution: crate::closure::AssessmentExecution,
        conformance_limits: ConformanceLimits,
        dispatch_limits: DispatchLimits,
        controls: Vec<Option<CanonicalRowControls>>,
        budget: Budget,
    ) -> crate::outcome::OperationOutcome<AssessmentBundle<'a, result::ValidatedResult>> {
        self.engine.assess_activated_after_resolution(
            requests,
            facts,
            axes,
            completeness,
            global_conformance,
            execution,
            conformance_limits,
            dispatch_limits,
            controls,
            budget,
        )
    }
}

fn validate_dispatch_request(
    requested: &[Claim],
    requests: &[ClaimRequest<'_>],
) -> Result<(), BundleRefusal> {
    if requests.len() != requested.len()
        || requests
            .iter()
            .map(|request| request.claim)
            .ne(requested.iter().copied())
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

fn selected_model_artifact<'a>(
    context: &LinkedAssessment<'a>,
    definition: &DefinitionIdentity,
) -> Result<&'a quire_spec_language::protocol_artifact::wire::ArtifactRef, BundleRefusal> {
    let [model] = definition.closure().models() else {
        return Err(BundleRefusal::AmbiguousResultModel);
    };
    context
        .package()
        .inherited()
        .dependencies
        .get(usize::try_from(model.value().artifact).unwrap_or(usize::MAX))
        .map(|dependency| &dependency.artifact)
        .ok_or(BundleRefusal::AmbiguousResultModel)
}

#[allow(clippy::too_many_arguments)]
fn derive_canonical_row_facts<'a>(
    selection: ResultSelection<'a>,
    disposition: &dispatch::Disposition,
    conformance: &crate::conformance::Report,
    facts: &[ActivationFact<'a>],
    axes: AxisInputs<'a>,
    completeness: &completeness::View,
    context: &LinkedAssessment<'_>,
    model_artifact: &'a quire_spec_language::protocol_artifact::wire::ArtifactRef,
    runtime_input: &'a QualifiedSubject,
    controls: CanonicalRowControls,
) -> result::Result<CanonicalRowFacts<'a>> {
    let mut truth = truth_from_dispatch(disposition, conformance);
    let mut decision_support = decision_support_from_report(truth, conformance);
    if matches!(truth, result::Truth::Satisfied | result::Truth::Violated)
        && decision_support.is_empty()
    {
        truth = result::Truth::Unavailable;
    }
    if matches!(truth, result::Truth::Pending | result::Truth::Unavailable) {
        decision_support.clear();
    }
    let settlement = settlement_from_report(truth, axes, &decision_support);
    Ok(CanonicalRowFacts {
        checked_predicate: selection.checked_predicate,
        temporal_subject: selection.temporal_subject,
        truth,
        settlement,
        activation: activation_from_facts(facts),
        participation: participation_from_facts(facts, conformance, context)?,
        observation_adequacy: observation_adequacy_from_report(conformance, completeness),
        protocol_adequacy: protocol_adequacy_from_report(disposition, conformance, completeness),
        decision_support,
        limitations: limitations_from_report(conformance),
        model_artifact,
        runtime_input,
        reproduction: controls.reproduction,
        limits: controls.limits,
    })
}

fn decision_support_from_report(
    _truth: result::Truth,
    _conformance: &crate::conformance::Report,
) -> Vec<result::DecisionSupport> {
    // The conformance report retains exact obligation and observation
    // identities, but no owner-authenticated oracle identity or dependence
    // capability. FR-248 forbids substituting the obligation for that oracle.
    // Keep support absent; the caller below consequently reports truth as
    // unavailable instead of fabricating a settled row.
    Vec::new()
}

fn settlement_from_report(
    truth: result::Truth,
    axes: AxisInputs<'_>,
    support: &[result::DecisionSupport],
) -> result::Settlement {
    use quire_observation::authority::OpenClosed;
    let closed = axes.decision_scope_progress.payload().state() == OpenClosed::Closed
        && axes.decision_scope_closure.payload().state() == OpenClosed::Closed;
    let basis = match truth {
        result::Truth::Satisfied if closed => result::SettlementBasis::ClosedScope,
        result::Truth::Satisfied => result::SettlementBasis::DecisiveWitness,
        result::Truth::Violated if closed => result::SettlementBasis::ClosedScope,
        result::Truth::Violated => result::SettlementBasis::DecisiveCounterexample,
        result::Truth::Pending => result::SettlementBasis::Unsettled,
        result::Truth::Unavailable => result::SettlementBasis::Unavailable,
    };
    result::Settlement {
        basis,
        supporting_fact_identities: support
            .iter()
            .map(|record| record.observation_identity.clone())
            .collect(),
        supporting_progress_identities: vec![axes
            .decision_scope_progress
            .identity()
            .as_str()
            .to_owned()],
    }
}

fn activation_from_facts(facts: &[ActivationFact<'_>]) -> result::Activation {
    use quire_spec_language::protocol_artifact::native_temporal::result::ActivationState;
    let mut states = facts
        .iter()
        .map(|fact| fact.occurrence().activation().activation());
    let Some(first) = states.next() else {
        return result::Activation::ActivationUnknown;
    };
    if states.any(|state| state != first) {
        return result::Activation::ActivationUnknown;
    }
    match first {
        ActivationState::Active => result::Activation::Triggered,
        ActivationState::Inactive => result::Activation::Inactive,
        ActivationState::Unknown | ActivationState::Unavailable => {
            result::Activation::ActivationUnknown
        }
    }
}

fn participation_from_facts(
    facts: &[ActivationFact<'_>],
    conformance: &crate::conformance::Report,
    context: &LinkedAssessment<'_>,
) -> result::Result<result::Participation> {
    use quire_spec_language::protocol_artifact::NodeRole;
    let mut required = std::collections::BTreeSet::new();
    let mut participating = std::collections::BTreeSet::new();
    for fact in facts {
        let occurrence = fact.occurrence().occurrence();
        let NodeRole::Role(role) = occurrence.role() else {
            continue;
        };
        let subject = context.role_instance_subject(role.index).ok_or_else(|| {
            result_owner_evidence_refusal(
                "an activated role has no exact FR-287 role-instance binding",
                "participation-role-instance",
            )
        })?;
        required.insert(subject);
        if fact.observation().is_some() {
            participating.insert(subject);
        }
    }
    let encode = |subject: &&QualifiedSubject| {
        result::runtime_reference::encode_subject(subject).map_err(|_| {
            result_owner_evidence_refusal(
                "an exact role-instance reference cannot be represented",
                "participation-role-instance",
            )
        })
    };
    let required_refs = required
        .iter()
        .map(encode)
        .collect::<result::Result<Vec<_>>>()?;
    let participating_refs = participating
        .iter()
        .map(encode)
        .collect::<result::Result<Vec<_>>>()?;
    let mut missing_dependency_refs = required
        .difference(&participating)
        .map(encode)
        .collect::<result::Result<Vec<_>>>()?;
    missing_dependency_refs.extend(conformance.incomplete.iter().map(|cause| match cause {
        crate::conformance::IncompleteCause::MissingValuation { obligation, .. } => {
            obligation.as_str().to_owned()
        }
    }));
    missing_dependency_refs.sort();
    missing_dependency_refs.dedup();
    if missing_dependency_refs.is_empty() {
        Ok(result::Participation::Known {
            required_role_instance_refs: required_refs,
            participating_role_instance_refs: participating_refs,
        })
    } else {
        Ok(result::Participation::Unknown {
            participating_role_instance_refs: participating_refs,
            missing_dependency_refs,
            reasons: vec!["required-observation-missing".to_owned()],
        })
    }
}

fn observation_adequacy_from_report(
    conformance: &crate::conformance::Report,
    completeness: &completeness::View,
) -> result::Adequacy {
    use quire_observation::authority::completeness::State;
    match completeness.payload().state() {
        State::Complete if conformance.incomplete.is_empty() => result::Adequacy::Adequate,
        State::Incomplete => result::Adequacy::Inadequate,
        State::Contradicted => result::Adequacy::Unknown,
        State::Complete => result::Adequacy::Inadequate,
    }
}

fn protocol_adequacy_from_report(
    disposition: &dispatch::Disposition,
    conformance: &crate::conformance::Report,
    completeness: &completeness::View,
) -> result::ProtocolAdequacy {
    use crate::conformance::GlobalDisposition;
    use quire_observation::authority::completeness::State;
    if disposition.claim != Claim::Conformance {
        return result::ProtocolAdequacy::Inapplicable;
    }
    match (conformance.global, completeness.payload().state()) {
        (GlobalDisposition::Conforms, State::Complete) => result::ProtocolAdequacy::Demonstrated,
        (GlobalDisposition::Violates, _) => result::ProtocolAdequacy::NotDemonstrated,
        _ => result::ProtocolAdequacy::Unknown,
    }
}

fn limitations_from_report(conformance: &crate::conformance::Report) -> Vec<result::Limitation> {
    let incomplete = conformance
        .incomplete
        .iter()
        .enumerate()
        .map(|(index, cause)| {
            let affected_identity = match cause {
                crate::conformance::IncompleteCause::MissingValuation { obligation, .. } => {
                    obligation.as_str().to_owned()
                }
            };
            result::Limitation {
                code: format!("conformance-incomplete:{index}"),
                affected_identity,
            }
        });
    let premises = conformance
        .global_premise_defects
        .iter()
        .enumerate()
        .map(|(index, defect)| result::Limitation {
            code: format!("global-premise-defect:{index}"),
            affected_identity: defect.premise_identity.to_string(),
        });
    incomplete.chain(premises).collect()
}

#[allow(
    clippy::too_many_arguments,
    reason = "the private transition receives exact owner outputs separately so no caller-built result aggregate can substitute for them"
)]
fn result_input_from_dispatch<'a, 'd>(
    disposition: &'d dispatch::Disposition,
    definition: &DefinitionIdentity,
    context: &LinkedAssessment<'_>,
    axes: AxisInputs<'a>,
    completeness: &'a completeness::View,
    global_conformance: &GlobalConformanceInput,
    selected: CanonicalRowFacts<'a>,
) -> Result<(result::Input<'a>, result::Limits), RowAssemblyRefusal<'d>> {
    let strength = match disposition.strength.as_ref() {
        b"proven" => result::ClaimStrength::Proven,
        b"bounded-checked" => result::ClaimStrength::BoundedChecked,
        b"tested" => result::ClaimStrength::Tested,
        b"observed" => result::ClaimStrength::Observed,
        _ => {
            return Err(RowAssemblyRefusal::UnknownClaimStrength(
                &disposition.strength,
            ))
        }
    };
    let admitted_fragment_identity = retained_text(
        disposition.premises.admitted_fragment.identity.as_ref(),
        RowAssemblyRefusal::NonTextAdmittedFragmentIdentity,
    )?;
    let backend_identity = retained_text(
        disposition.premises.backend.identity.as_ref(),
        RowAssemblyRefusal::NonTextBackendIdentity,
    )?;
    let assumptions = disposition
        .premises
        .assumptions
        .iter()
        .map(|assumption| {
            Ok(result::Assumption {
                identity: retained_text(
                    assumption.component.identity.as_ref(),
                    RowAssemblyRefusal::NonTextAssumptionIdentity,
                )?,
                state: match assumption
                    .state
                    .ok_or(RowAssemblyRefusal::MissingAssumptionState(
                        &assumption.component.identity,
                    ))? {
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
        .collect::<Result<Vec<_>, RowAssemblyRefusal<'d>>>()?;
    let provenance = provenance_from_dispatch(disposition, definition, context, &selected)?;
    let limits = selected.limits;
    Ok((
        result::Input {
            checked_predicate: selected.checked_predicate,
            temporal_subject: selected.temporal_subject,
            axes,
            completeness,
            execution: disposition.execution,
            truth: selected.truth,
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
            correction: None,
        },
        limits,
    ))
}

fn provenance_from_dispatch<'d>(
    disposition: &'d dispatch::Disposition,
    definition: &DefinitionIdentity,
    context: &LinkedAssessment<'_>,
    selected: &CanonicalRowFacts<'_>,
) -> Result<result::ProvenanceRecord, RowAssemblyRefusal<'d>> {
    let model_selected = definition.closure().models().iter().any(|model| {
        context
            .package()
            .inherited()
            .dependencies
            .get(usize::try_from(model.value().artifact).unwrap_or(usize::MAX))
            .is_some_and(|dependency| dependency.artifact == *selected.model_artifact)
    });
    if !model_selected {
        return Err(RowAssemblyRefusal::ForeignModelProvenance);
    }
    let runtime_selected = context.runtime().bindings().iter().any(|binding| {
        binding
            .subject_value()
            .is_some_and(|subject| subject == selected.runtime_input)
    });
    if !runtime_selected {
        return Err(RowAssemblyRefusal::ForeignRuntimeInput);
    }
    let input_identity = result::runtime_reference::encode_subject(selected.runtime_input)
        .map_err(|_| RowAssemblyRefusal::UnrepresentableRuntimeInput)?;
    Ok(result::ProvenanceRecord {
        source_identity: definition.source_document().identity.clone(),
        profile_identity: definition.closure().profile().value().identity.clone(),
        model_identity: selected.model_artifact.identity.clone(),
        protocol_identity: format!(
            "{}:{}",
            definition.source_document().identity,
            definition.declaration_handle()
        ),
        request_identity: retained_text(
            disposition.request_identity.as_ref(),
            RowAssemblyRefusal::NonTextRequestIdentity,
        )?,
        binding_identity: definition.static_binding().identity.to_string(),
        input_identity,
        backend_identity: retained_text(
            disposition.premises.backend.identity.as_ref(),
            RowAssemblyRefusal::NonTextBackendIdentity,
        )?,
        producer_identity: context.definition().producer().bundle_identity.clone(),
        reproduction: selected.reproduction.clone(),
    })
}

fn retained_text<'a>(
    bytes: &'a [u8],
    refusal: fn(&'a [u8]) -> RowAssemblyRefusal<'a>,
) -> Result<String, RowAssemblyRefusal<'a>> {
    String::from_utf8(bytes.to_vec()).map_err(|_| refusal(bytes))
}

fn result_owner_evidence_refusal(detail: &'static str, identity: &'static str) -> result::Refusal {
    result::Refusal::assembly(
        result::ErrorCode::OwnerEvidenceUnavailable,
        detail,
        result::Usage::default(),
        Some(identity),
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

fn validate_result_selection(
    package: crate::ByteDigest,
    definition: &DefinitionIdentity,
    request: &temporal_v2::ValidatedRequest,
    checked_predicate: &ValidatedCheckedPredicate,
    temporal_subject: &ValidatedTemporalSubject,
) -> Result<(), BundleRefusal> {
    if checked_predicate.package_digest() != package {
        return Err(BundleRefusal::CheckedPredicatePackageMismatch);
    }
    if temporal_subject.package_digest() != package {
        return Err(BundleRefusal::TemporalSubjectPackageMismatch);
    }
    if checked_predicate.declaration() != temporal_subject.declaration() {
        return Err(BundleRefusal::CheckedPredicateDeclarationMismatch);
    }
    if temporal_subject.declaration() != request.declaration() {
        return Err(BundleRefusal::ResultRequestDeclarationMismatch);
    }
    if request.subject_identity() != temporal_subject.document().identity() {
        return Err(BundleRefusal::ResultRequestSubjectMismatch);
    }
    if !definition
        .temporal_bindings()
        .iter()
        .any(|binding| binding.declaration == temporal_subject.declaration())
    {
        return Err(BundleRefusal::ResultTemporalBindingMissing);
    }
    if !predicate_selection_matches(
        checked_predicate.declaration(),
        checked_predicate.leaf(),
        temporal_subject.declaration(),
        temporal_subject.predicate_leaves(),
    ) {
        return Err(BundleRefusal::ResultPredicateUnreachable);
    }
    Ok(())
}

fn predicate_selection_matches(
    checked_declaration: u32,
    checked_leaf: &quire_spec_language::protocol_artifact::wire::Handle,
    temporal_declaration: u32,
    reachable_leaves: Option<&[quire_spec_language::protocol_artifact::wire::Handle]>,
) -> bool {
    checked_declaration == temporal_declaration
        && reachable_leaves.is_some_and(|leaves| leaves.contains(checked_leaf))
}

/// Require the preceding closed dispatch stage to answer this Engine's exact
/// claim population in the same declared order as canonical-row construction.
fn validate_dispatch_rows(
    requested: &[Claim],
    dispatch: &DispatchReport,
    inputs: &[Option<CanonicalRowControls>],
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
        outcome_refusal, predicate_selection_matches, row_assembly_refusal,
        validate_dispatch_population, validate_dispatch_request, validate_requested, validate_rows,
        AccountingStage, BundleRefusal, ClaimRow, RowAssemblyRefusal,
    };
    use crate::bounds::inventory::{
        production_inventory, AccountingFault, Budget, BudgetDimension, ResourceAccountant,
    };
    use crate::bounds::{Phase, SiteId, UnitLabel};
    use crate::claims::{
        dispatch::{ClaimRequest, Report as DispatchReport},
        premises::{BackendSelection, Component, PremiseSet},
        Claim,
    };
    use crate::conformance::KernelRefusal;
    use crate::outcome::ClaimOutcome;
    use crate::refusal::{AffectedIdentity, PartialOutput, ProtocolStage, RefusalDomain};
    use crate::result::ClaimKind;
    use ix_trace_rs::trace;
    use std::collections::{BTreeSet, HashSet};
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

    fn dispatch_request(claim: Claim) -> ClaimRequest<'static> {
        let component = |identity: &[u8]| {
            Component::retained(
                identity.to_vec().into_boxed_slice(),
                identity.to_vec().into_boxed_slice(),
            )
        };
        ClaimRequest {
            identity: b"request".to_vec().into_boxed_slice(),
            claim,
            strength: b"tested".to_vec().into_boxed_slice(),
            premises: PremiseSet {
                admitted_fragment: component(b"fragment"),
                backend: BackendSelection::retained(
                    b"backend".to_vec().into_boxed_slice(),
                    b"1".to_vec().into_boxed_slice(),
                ),
                assumptions: Vec::new(),
                communication: Vec::new(),
                visibility: Vec::new(),
                environment: Vec::new(),
                sources: Vec::new(),
                derived_artifact: None,
            },
            correspondence: None,
        }
    }

    #[test]
    #[trace("TC-006", "FR-006-AC-1", "FR-006-AC-9")]
    fn dispatch_request_order_must_equal_the_engines_order() {
        let requested = [Claim::Conformance, Claim::Monitorability];
        let authored = [
            dispatch_request(Claim::Conformance),
            dispatch_request(Claim::Monitorability),
        ];
        assert_eq!(validate_dispatch_request(&requested, &authored), Ok(()));

        let reversed = [
            dispatch_request(Claim::Monitorability),
            dispatch_request(Claim::Conformance),
        ];
        assert_eq!(
            validate_dispatch_request(&requested, &reversed),
            Err(BundleRefusal::UnexpectedClaim)
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
                BundleRefusal::DispatchReportMismatch,
                RefusalDomain::Result,
                ProtocolStage::ResultAssembly,
            ),
            (
                BundleRefusal::MissingResultSelection,
                RefusalDomain::Result,
                ProtocolStage::ResultAssembly,
            ),
            (
                BundleRefusal::CheckedPredicatePackageMismatch,
                RefusalDomain::Result,
                ProtocolStage::ResultAssembly,
            ),
            (
                BundleRefusal::TemporalSubjectPackageMismatch,
                RefusalDomain::Result,
                ProtocolStage::ResultAssembly,
            ),
            (
                BundleRefusal::CheckedPredicateDeclarationMismatch,
                RefusalDomain::Result,
                ProtocolStage::ResultAssembly,
            ),
            (
                BundleRefusal::ResultRequestDeclarationMismatch,
                RefusalDomain::Result,
                ProtocolStage::ResultAssembly,
            ),
            (
                BundleRefusal::ResultRequestSubjectMismatch,
                RefusalDomain::Result,
                ProtocolStage::ResultAssembly,
            ),
            (
                BundleRefusal::ResultTemporalBindingMissing,
                RefusalDomain::Result,
                ProtocolStage::ResultAssembly,
            ),
            (
                BundleRefusal::ResultPredicateUnreachable,
                RefusalDomain::Result,
                ProtocolStage::ResultAssembly,
            ),
            (
                BundleRefusal::AmbiguousResultModel,
                RefusalDomain::Result,
                ProtocolStage::ResultAssembly,
            ),
            (
                BundleRefusal::MissingActivationSelection,
                RefusalDomain::Assessment,
                ProtocolStage::Conformance,
            ),
            (
                BundleRefusal::ActivationRuntimeBindingMismatch,
                RefusalDomain::Assessment,
                ProtocolStage::Conformance,
            ),
            (
                BundleRefusal::ActivationFactMismatch,
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
    #[trace("TC-006", "FR-006-AC-9", "FR-008-AC-1", "FR-014-AC-1")]
    fn row_assembly_failures_have_injective_typed_identities() {
        let failures = [
            RowAssemblyRefusal::UnknownClaimStrength(b"unknown"),
            RowAssemblyRefusal::MissingAssumptionState(b"assumption"),
            RowAssemblyRefusal::NonTextRequestIdentity(b"\xffrequest"),
            RowAssemblyRefusal::NonTextAdmittedFragmentIdentity(b"\xfffragment"),
            RowAssemblyRefusal::NonTextBackendIdentity(b"\xffbackend"),
            RowAssemblyRefusal::NonTextAssumptionIdentity(b"\xffassumption"),
            RowAssemblyRefusal::ForeignModelProvenance,
            RowAssemblyRefusal::ForeignRuntimeInput,
            RowAssemblyRefusal::UnrepresentableRuntimeInput,
        ];
        let mut codes = BTreeSet::new();
        let mut interiors = HashSet::new();
        for failure in failures {
            let refusal = row_assembly_refusal(failure);
            assert_eq!(refusal.domain(), RefusalDomain::Result);
            assert_eq!(refusal.stage(), ProtocolStage::ResultAssembly);
            assert!(codes.insert(refusal.code().as_str()));
            assert!(interiors.insert((
                refusal.axis().as_bytes().to_vec(),
                refusal.causes().declared_order()[0].clone(),
            )));
        }
    }

    #[test]
    #[trace("TC-006", "FR-006-AC-1", "FR-006-AC-4", "FR-031-AC-5")]
    fn result_selection_accepts_each_reachable_leaf_without_requiring_a_singleton() {
        use quire_spec_language::protocol_artifact::wire::Handle;

        let first = Handle {
            declaration: 7,
            index: 2,
        };
        let second = Handle {
            declaration: 7,
            index: 5,
        };
        let unreachable = Handle {
            declaration: 7,
            index: 9,
        };
        let leaves = [first.clone(), second.clone()];
        assert!(predicate_selection_matches(7, &first, 7, Some(&leaves)));
        assert!(predicate_selection_matches(7, &second, 7, Some(&leaves)));
        assert!(!predicate_selection_matches(8, &second, 7, Some(&leaves)));
        assert!(!predicate_selection_matches(
            7,
            &unreachable,
            7,
            Some(&leaves)
        ));
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
