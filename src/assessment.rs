// SPDX-License-Identifier: AGPL-3.0-or-later
//! Shared, producer-owned facts consumed by assessment and result construction.
//!
//! This module is deliberately neutral: conformance assesses these inputs and
//! the result owner serializes them, but neither subsystem owns or re-derives
//! the QObs facts or the declared global-closure premise set.

use quire_observation::authority::{
    closure as observation_closure, observation as observation_record, position, progress,
};
use quire_observation::QualifiedSubject;
use quire_spec_language::protocol_artifact::native_temporal::v2 as temporal_v2;
use quire_spec_language::protocol_artifact::wire::{BindingKind, ControlOperation};
use quire_spec_language::protocol_artifact::ProtocolNumber;
use quire_spec_language::state::{
    self as state_evaluation, EvaluationOutcome, EvaluationReport, StateView,
    ValueKind as StateValueKind,
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
use crate::conformance::Kernel as ConformanceKernel;
use crate::model::v2::{
    link as link_v2, CompensationRegistrationInput, DefinitionIdentity, DefinitionRegistry,
    LinkedAssessment, LinkedProtocol, RuntimeBinding, SlotCompatibility, SlotHandle,
};
use crate::outcome::ClaimOutcome;
use crate::refusal::catalog::{CatalogSelection, CodeEntry, CodeModule, Provenance, RefusalCode};
use crate::refusal::{
    ProtocolRefusal, ProtocolStage, RefusalAxis, RefusalDomain, RetryDecision, Unavailability,
};
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

/// One result-owner input selected for an exact requested claim.
///
/// The Engine owns construction and strict rereading; callers supply only the
/// independently typed subsystem facts required by the result owner.
#[derive(Clone, Debug)]
pub struct CanonicalRowInput<'a> {
    claim: Claim,
    input: result::Input<'a>,
    limits: result::Limits,
}

impl<'a> CanonicalRowInput<'a> {
    /// Retain one typed result-owner input for its exact requested claim.
    #[must_use]
    pub const fn new(claim: Claim, input: result::Input<'a>, limits: result::Limits) -> Self {
        Self {
            claim,
            input,
            limits,
        }
    }
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

/// Result-assembly assessment codes.
pub const RESULT_CODES: CodeModule = CodeModule {
    root: "assessment-result",
    owner: "Task-006",
    module_path: "quire_protocol::assessment",
    entries: &[
        BUNDLE_CLAIM_MISMATCH,
        BUNDLE_PACKAGE_MISMATCH,
        BUNDLE_DISPATCH_MISMATCH,
    ],
};

/// Compensation-stage activation-binding codes.
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
            (&BUNDLE_ACTIVATION_SUBJECT, ProtocolStage::Compensation)
        }
        BundleRefusal::ActivationDeclarationMismatch => {
            (&BUNDLE_ACTIVATION_DECLARATION, ProtocolStage::Compensation)
        }
    };
    let code = RefusalCode::of(entry);
    ProtocolRefusal::legacy_builder(
        code,
        CatalogSelection::local(),
        stage,
        RefusalAxis::from_registered_code(code),
    )
    .expect("assessment bundle code belongs to claim dispatch")
    .unavailable_location(Unavailability::NotApplicableToStage)
    .emitting_nothing()
}

/// One refusal from the Engine's closed-dispatch boundary.
///
/// This wrapper deliberately preserves the inner refusal envelope. A
/// whole-request dispatch limit is owned by dispatch, while an attempt to
/// retarget the Engine's fixed claim population is owned by the bundle; neither
/// is collapsed into a synthetic generic error.
#[derive(Clone, Debug)]
pub enum DispatchRefusal {
    /// The request does not name the Engine's exact selected claim population.
    Bundle(BundleRefusal),
    /// The closed dispatch owner refused with its original typed envelope.
    Dispatch(dispatch::Refusal),
}

impl DispatchRefusal {
    /// The complete source-bound envelope for this refusal.
    #[must_use]
    pub fn envelope(&self) -> ProtocolRefusal {
        match self {
            Self::Bundle(error) => outcome_refusal(*error),
            Self::Dispatch(error) => error.envelope().clone(),
        }
    }
}

impl core::fmt::Display for DispatchRefusal {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Bundle(error) => write!(formatter, "assessment dispatch rejected: {error:?}"),
            Self::Dispatch(error) => error.fmt(formatter),
        }
    }
}

impl std::error::Error for DispatchRefusal {}

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

    /// Dispatch this assessment's exact claim population through the closed
    /// local conformance capability.
    ///
    /// The supplied report is produced by QProtocol's conformance owner; this
    /// method neither accepts a backend callback nor permits a dispatch request
    /// to add, omit, or retarget one of this engine's selected claims.
    pub fn dispatch_conformance(
        &self,
        requests: &[ClaimRequest<'_>],
        report: &crate::conformance::Report,
        limits: DispatchLimits,
    ) -> Result<DispatchReport, DispatchRefusal> {
        if requests.len() != self.requested.len()
            || requests
                .iter()
                .any(|request| !self.requested.contains(&request.claim))
            || self.requested.iter().any(|claim| {
                requests
                    .iter()
                    .filter(|request| request.claim == *claim)
                    .count()
                    != 1
            })
        {
            return Err(DispatchRefusal::Bundle(BundleRefusal::UnexpectedClaim));
        }
        dispatch::dispatch_from_conformance(requests, report, limits)
            .map_err(DispatchRefusal::Dispatch)
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
                self.compensation = registration_candidate;
                self.compensation_state = state_candidate;
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
    /// public path is [`Self::construct_canonical`], which owns production and
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

    /// Construct and strict-read each canonical result row from typed owner
    /// inputs. A result-owner refusal remains local to its selected claim.
    pub fn construct_canonical(
        self,
        dispatch: &DispatchReport,
        inputs: Vec<CanonicalRowInput<'a>>,
    ) -> Result<AssessmentBundle<'a, result::ValidatedResult>, BundleRefusal> {
        validate_input_claims(&self.requested, &inputs)?;
        validate_dispatch_rows(&self.requested, dispatch, &inputs)?;
        let mut rows = Vec::with_capacity(inputs.len());
        for (selected, disposition) in inputs.into_iter().zip(&dispatch.dispositions) {
            validate_input_package(
                self.context.package().digest(),
                selected.input.checked_predicate.package_digest(),
                selected.input.temporal_subject.package_digest(),
            )?;
            if claim_from_result_kind(selected.input.claim.kind) != selected.claim {
                return Err(BundleRefusal::CanonicalClaimMismatch);
            }
            let outcome = if let Some(failure) = &disposition.failure {
                ClaimOutcome::Refused(failure.envelope().clone())
            } else {
                match result::produce(selected.input.clone(), selected.limits) {
                    Ok(document) => match result::read(
                        document.bytes(),
                        result::Expected {
                            input: selected.input,
                            enclosing_digest: document.digest(),
                        },
                        selected.limits,
                    ) {
                        Ok(validated) => ClaimOutcome::Canonical(validated),
                        Err(refusal) => ClaimOutcome::Refused(refusal.envelope().clone()),
                    },
                    Err(refusal) => ClaimOutcome::Refused(refusal.envelope().clone()),
                }
            };
            rows.push(ClaimRow {
                claim: selected.claim,
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
        self.assemble_canonical(rows)
    }

    /// Construct canonical rows through the complete three-member operation
    /// algebra. Whole-assessment cardinality failures cannot retain a bundle;
    /// individual result-owner failures remain claim-local inside a completed
    /// bundle.
    pub fn construct_canonical_outcome(
        self,
        dispatch: &DispatchReport,
        inputs: Vec<CanonicalRowInput<'a>>,
    ) -> crate::outcome::OperationOutcome<AssessmentBundle<'a, result::ValidatedResult>> {
        match self.construct_canonical(dispatch, inputs) {
            Ok(bundle) => crate::outcome::OperationOutcome::Completed(bundle),
            Err(error) => crate::outcome::OperationOutcome::Refused(outcome_refusal(error)),
        }
    }
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

fn validate_input_claims(
    requested: &[Claim],
    inputs: &[CanonicalRowInput<'_>],
) -> Result<(), BundleRefusal> {
    for (index, input) in inputs.iter().enumerate() {
        if !requested.contains(&input.claim) {
            return Err(BundleRefusal::UnexpectedClaim);
        }
        if inputs[..index]
            .iter()
            .any(|previous| previous.claim == input.claim)
        {
            return Err(BundleRefusal::DuplicateClaimRow);
        }
    }
    if inputs.len() != requested.len() {
        return Err(BundleRefusal::MissingClaimRow);
    }
    Ok(())
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
    inputs: &[CanonicalRowInput<'_>],
) -> Result<(), BundleRefusal> {
    validate_dispatch_population(requested, dispatch)?;
    for (disposition, input) in dispatch.dispositions.iter().zip(inputs) {
        if input.claim != disposition.claim || input.input.execution != disposition.execution {
            return Err(BundleRefusal::DispatchReportMismatch);
        }
    }
    Ok(())
}

fn validate_dispatch_population(
    requested: &[Claim],
    dispatch: &DispatchReport,
) -> Result<(), BundleRefusal> {
    if dispatch.dispositions.len() != requested.len() {
        return Err(BundleRefusal::DispatchReportMismatch);
    }
    for (requested, disposition) in requested.iter().zip(&dispatch.dispositions) {
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
        claim_from_result_kind, outcome_refusal, validate_dispatch_population,
        validate_input_package, validate_requested, validate_rows, BundleRefusal, ClaimRow,
    };
    use crate::claims::{
        dispatch::Report as DispatchReport, dispatch::Usage as DispatchUsage, Claim,
    };
    use crate::outcome::ClaimOutcome;
    use crate::refusal::{PartialOutput, ProtocolStage, RefusalDomain};
    use crate::result::ClaimKind;
    use crate::ByteDigest;
    use ix_trace_rs::trace;
    use std::collections::BTreeSet;

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
    #[trace("TC-011", "FR-008-AC-1", "FR-008-AC-4", "FR-036-AC-1", "FR-040-AC-1")]
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
                ProtocolStage::Compensation,
            ),
            (
                BundleRefusal::ActivationDeclarationMismatch,
                RefusalDomain::Assessment,
                ProtocolStage::Compensation,
            ),
        ];
        let mut codes = BTreeSet::new();
        let mut axes = BTreeSet::new();
        for (reason, domain, stage) in reasons {
            let refusal = outcome_refusal(reason);
            assert_eq!(refusal.domain(), domain);
            assert_eq!(refusal.stage(), stage);
            assert_eq!(refusal.partial_output(), PartialOutput::NoneEmitted);
            assert!(codes.insert(refusal.code().as_str()));
            assert!(axes.insert(refusal.axis().as_bytes().to_vec()));
        }
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
        let missing = DispatchReport {
            dispositions: Vec::new(),
            usage: DispatchUsage::default(),
        };
        assert_eq!(
            validate_dispatch_population(&[Claim::Conformance], &missing),
            Err(BundleRefusal::DispatchReportMismatch)
        );
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
