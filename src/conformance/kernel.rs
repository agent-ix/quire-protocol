// SPDX-License-Identifier: AGPL-3.0-or-later
//! One immutable conformance input assembled from linked static and compensation facts.

use crate::assessment::{AxisInputs, GlobalConformanceInput};
use crate::bounds::inventory::{
    BudgetDimension, ChargeError, OperationOutcome, ResourceAccountant, SourceSite,
};
use crate::bounds::{SiteId, SiteKind};
use crate::closure::AssessmentExecution;
use crate::compensation::registration::{Registration, Snapshot as CompensationSnapshot};
use crate::conformance::batch::{
    assess as assess_batch, assess_bounded as assess_batch_bounded, BoundObservation,
    Limits as ConformanceLimits, Model, ObligationId, ObservationObligation, Report, Request,
};
use crate::conformance::incremental::Incremental;
use crate::model::v2::{
    CompensationAttemptInput, CompensationEffectInput, DefinitionIdentity,
    LinkedCompensationDefinition, LinkedProtocol,
};
use quire_observation::{
    authority::{
        closure as observation_closure, completeness, observation, progress, AuthoritySelection,
        BoundaryRef, OpenClosed,
    },
    QualifiedSubject,
};
use quire_spec_language::protocol_artifact::{
    native_temporal::{self, request::CaptureView, result::ActivationState, v2 as temporal_v2},
    temporal_subject::ValidatedTemporalSubject,
    v3,
    wire::{Body, ControlOperation, EdgeKind, Handle},
    OccurrenceKey,
};
use sha2::{Digest as _, Sha256};

/// Exact owner-document bytes traversed while correlating QSL evidence to QObs.
pub(crate) const TEMPORAL_EVIDENCE_SOURCE: SourceSite = SourceSite::new(
    SiteId::new("conformance/temporal-evidence/hash"),
    "conformance::kernel::validate_temporal_evidence_bounded",
    BudgetDimension::InputByte,
    SiteKind::Iteration,
);
/// Activation-derived model nodes retained before batch assessment.
pub(crate) const ACTIVATION_NODE_SOURCE: SourceSite = SourceSite::new(
    SiteId::new("conformance/activation-model/node"),
    "conformance::kernel::assess_activated_bounded",
    BudgetDimension::Node,
    SiteKind::Retention,
);
/// Caller/owner identity bytes traversed while deriving obligation identities.
pub(crate) const ACTIVATION_IDENTITY_SOURCE: SourceSite = SourceSite::new(
    SiteId::new("conformance/activation-model/identity"),
    "conformance::kernel::assess_activated_bounded",
    BudgetDimension::InputByte,
    SiteKind::Iteration,
);
/// Duplicate and predecessor comparisons required to derive the fixed model.
pub(crate) const ACTIVATION_OPERATION_SOURCE: SourceSite = SourceSite::new(
    SiteId::new("conformance/activation-model/operation"),
    "conformance::kernel::assess_activated_bounded",
    BudgetDimension::Operation,
    SiteKind::Traversal,
);
/// Stable activation-model and observation-vector allocation population.
pub(crate) const ACTIVATION_ALLOCATION_SOURCE: SourceSite = SourceSite::new(
    SiteId::new("conformance/activation-model/allocate"),
    "conformance::kernel::assess_activated_bounded",
    BudgetDimension::AccountedAllocation,
    SiteKind::Allocation,
);

/// Why linked static and derived compensation state cannot form one kernel.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Refusal {
    /// A registration names another complete linked definition.
    ForeignDefinition,
    /// A registration names no compensation definition in the linked graph.
    UnknownCompensation,
    /// An admitted attempt or effect names no derived registration.
    UnknownRegistration,
    /// A selected QSL control table is too large for its published `u32` handle domain.
    UnrepresentableControl,
    /// A supplied `/3` mapping package is not this linked `/2` graph.
    ForeignActivationPackage,
    /// No unique source-authorized event-control mapping selects the request.
    MissingActivationMapping,
    /// The selected mapping does not name an event control in this graph.
    InvalidActivationControl,
    /// QObs subject identity and QSL opaque semantic trigger disagree.
    ActivationSubjectMismatch,
    /// The checked temporal result is not the exact result of this request.
    ActivationResultMismatch,
    /// The request names no temporal declaration selected by this definition.
    InvalidActivationTemporal,
    /// A QSL-validated runtime occurrence names another control.
    ActivationOccurrenceMismatch,
    /// The same FR-300 key was replayed with different checked owner content.
    ConflictingActivation,
    /// An activation occurrence belongs to another linked definition.
    ForeignActivatedOccurrence,
    /// More than one activation occurrence derives one FR-230 obligation key.
    DuplicateActivatedObligation,
    /// The graph-derived activation population cannot form a finite model.
    InvalidDerivedModel,
    /// An incremental fact was not selected in this kernel's activation population.
    UnexpectedActivatedOccurrence,
    /// QSL's admitted temporal evidence does not name the supplied QObs owner views.
    TemporalEvidenceMismatch,
}

/// Constructor-private FR-300 join of QSL, linked-graph, and QObs facts.
///
/// It deliberately retains QSL's checked owner values rather than reproducing
/// the trigger, captures, anchor, or activation state in Protocol vocabulary.
#[derive(Clone)]
pub struct ActivationBinding<'a> {
    definition: DefinitionIdentity,
    control: Handle,
    subject: QualifiedSubject,
    temporal_subject: &'a ValidatedTemporalSubject,
    request: &'a temporal_v2::ValidatedRequest,
    result: &'a temporal_v2::ValidatedResult,
}

impl core::fmt::Debug for ActivationBinding<'_> {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter
            .debug_struct("ActivationBinding")
            .field("definition", &self.definition)
            .field("control", &self.control)
            .field("subject", &self.subject)
            .field("semantic_trigger", &self.semantic_trigger())
            .finish_non_exhaustive()
    }
}

impl ActivationBinding<'_> {
    /// Returns the fixed qualified definition component of the binding key.
    #[must_use]
    pub const fn definition(&self) -> &DefinitionIdentity {
        &self.definition
    }

    /// Returns the declaration-local QSL event-control component of the key.
    #[must_use]
    pub const fn control(&self) -> &Handle {
        &self.control
    }

    /// Returns QObs's exact qualified clause-subject component of the key.
    #[must_use]
    pub const fn subject(&self) -> &QualifiedSubject {
        &self.subject
    }

    /// Returns QSL's opaque semantic-trigger component without reinterpretation.
    #[must_use]
    pub fn semantic_trigger(&self) -> &[u8] {
        self.request.semantic_trigger()
    }

    /// Returns QSL's exact admitted evaluation anchor.
    #[must_use]
    pub fn evaluation_anchor(&self) -> &str {
        self.request.evaluation_anchor()
    }

    /// Returns QSL's immutable trigger captures in their admitted order.
    #[must_use]
    pub fn captures(&self) -> impl ExactSizeIterator<Item = CaptureView<'_>> {
        self.request.activation_captures()
    }

    /// Returns QSL's checked activation state from strict result admission.
    #[must_use]
    pub fn activation(&self) -> ActivationState {
        self.result.activation()
    }

    /// Verifies that one QSL-validated runtime occurrence is this activation's
    /// exact declaration-local event control.
    ///
    /// QSL owns validation of the workflow identity, role, and repeat
    /// dimensions. Protocol only compares the already-validated control
    /// coordinate with the control selected by the admitted `/3` mapping, so
    /// an independent caller association cannot retarget this activation.
    pub fn require_occurrence(&self, occurrence: &OccurrenceKey<'_>) -> Result<(), Refusal> {
        if occurrence.declaration() == self.control.declaration
            && occurrence.node() == &self.control
        {
            Ok(())
        } else {
            Err(Refusal::ActivationOccurrenceMismatch)
        }
    }

    fn same_key(&self, other: &Self) -> bool {
        self.definition == other.definition
            && self.control == other.control
            && self.subject == other.subject
            && self.semantic_trigger() == other.semantic_trigger()
    }

    fn same_checked_content(&self, other: &Self) -> bool {
        self.temporal_subject.package_digest() == other.temporal_subject.package_digest()
            && self.temporal_subject.document().identity()
                == other.temporal_subject.document().identity()
            && self.request.document().identity() == other.request.document().identity()
            && self.evaluation_anchor() == other.evaluation_anchor()
            && self.captures().eq(other.captures())
            && self.result.document().identity() == other.result.document().identity()
            && self.activation() == other.activation()
    }

    /// Whether another binding carries the same complete FR-300 key and the
    /// same strict QSL request/result content.
    pub(crate) fn same_binding(&self, other: &Self) -> bool {
        self.same_key(other) && self.same_checked_content(other)
    }
}

/// Replay-idempotent retained FR-300 bindings for one assessment input.
///
/// The registry never fabricates a key or owner fact: it only retains values
/// formed by [`Kernel::bind_activation`]. An exact replay returns the original
/// binding, while another checked request/result under the same four-part key
/// refuses instead of overwriting or coalescing its owner content.
#[derive(Default)]
pub struct ActivationRegistry<'a> {
    bindings: Vec<ActivationBinding<'a>>,
}

/// One FR-300 activation paired with its QSL-validated node occurrence.
///
/// The constructor is kernel-owned so a caller cannot associate an activation
/// with a different declaration-local control or an unchecked workflow/repeat
/// tuple.
#[derive(Clone, Debug)]
pub struct ActivatedOccurrence<'a> {
    activation: ActivationBinding<'a>,
    occurrence: OccurrenceKey<'a>,
}

impl ActivatedOccurrence<'_> {
    /// Returns the retained FR-300 activation binding.
    #[must_use]
    pub const fn activation(&self) -> &ActivationBinding<'_> {
        &self.activation
    }

    /// Returns QSL's already-validated runtime occurrence key.
    #[must_use]
    pub const fn occurrence(&self) -> &OccurrenceKey<'_> {
        &self.occurrence
    }
}

/// One optional admitted QObs fact for a constructor-validated activation
/// occurrence. `None` retains a known activated obligation with no admitted
/// observation, allowing the shared batch kernel to report pending or missing
/// state at the owner-provided boundary.
#[derive(Clone, Debug)]
pub struct ActivationFact<'a> {
    occurrence: ActivatedOccurrence<'a>,
    observation: Option<&'a observation::View>,
}

impl ActivationFact<'_> {
    /// Returns the exact activated occurrence that selects this fact.
    #[must_use]
    pub const fn occurrence(&self) -> &ActivatedOccurrence<'_> {
        &self.occurrence
    }

    /// Returns the unaltered admitted QObs observation, if present.
    #[must_use]
    pub const fn observation(&self) -> Option<&observation::View> {
        self.observation
    }
}

/// Distinguishes activation-input refusal from conformance assessment refusal.
#[derive(Debug)]
pub enum ActivatedAssessmentError {
    /// The graph-derived activation input is not admissible.
    Kernel(Refusal),
    /// The shared finite conformance kernel refused the admitted fact set.
    Conformance(crate::conformance::Refusal),
}

impl core::fmt::Display for ActivatedAssessmentError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Kernel(refusal) => write!(formatter, "activation input refused: {refusal:?}"),
            Self::Conformance(refusal) => refusal.fmt(formatter),
        }
    }
}

impl std::error::Error for ActivatedAssessmentError {}

/// Settled incremental conformance over one fixed activation-derived model.
///
/// The complete activation population is fixed before the first prefix, while
/// QObs facts arrive one at a time. This preserves the existing earliest-prefix
/// settlement behavior without reopening the caller-built `Model` boundary.
#[derive(Clone)]
pub struct ActivatedIncremental<'a> {
    model: Model,
    inner: Incremental<'a>,
    latest_request: Option<&'a temporal_v2::ValidatedRequest>,
}

impl core::fmt::Debug for ActivatedIncremental<'_> {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter
            .debug_struct("ActivatedIncremental")
            .field("model", &self.model)
            .field("inner", &self.inner)
            .field(
                "latest_request",
                &self
                    .latest_request
                    .map(|request| request.document().identity()),
            )
            .finish()
    }
}

impl<'a> ActivatedIncremental<'a> {
    /// Starts a settled incremental path over the complete admitted activation
    /// population.
    pub fn new(kernel: &Kernel, activations: &[ActivatedOccurrence<'a>]) -> Result<Self, Refusal> {
        let model = kernel.derive_model(activations)?;
        Ok(Self {
            inner: Incremental::new(model.clone()),
            model,
            latest_request: None,
        })
    }

    /// Assesses one admitted activation-bound fact as the next prefix member.
    pub fn push(
        &mut self,
        fact: &ActivationFact<'a>,
        axes: AxisInputs<'a>,
        completeness: &'a completeness::View,
        global_conformance: &'a GlobalConformanceInput,
        execution: AssessmentExecution,
        limits: ConformanceLimits,
    ) -> std::result::Result<&Report, ActivatedAssessmentError> {
        validate_temporal_evidence(
            fact.occurrence.activation.request,
            axes,
            completeness,
            execution,
        )
        .map_err(ActivatedAssessmentError::Kernel)?;
        let observation = fact.observation.ok_or(ActivatedAssessmentError::Kernel(
            Refusal::UnexpectedActivatedOccurrence,
        ))?;
        let identity = derived_obligation_id(&fact.occurrence);
        let obligation = self
            .model
            .obligations()
            .iter()
            .find(|candidate| candidate.identity == identity)
            .ok_or(ActivatedAssessmentError::Kernel(
                Refusal::UnexpectedActivatedOccurrence,
            ))?;
        let report = self
            .inner
            .push(
                BoundObservation {
                    obligation: obligation.identity.clone(),
                    observation,
                    workflow_identity: fact.occurrence.occurrence.workflow().as_str().into(),
                    effect_identity: obligation.effect_identity.clone(),
                },
                axes,
                completeness,
                global_conformance,
                execution,
                limits,
            )
            .map_err(ActivatedAssessmentError::Conformance)?;
        self.latest_request = Some(fact.occurrence.activation.request);
        Ok(report)
    }

    /// Finishes the retained prefix under final owner-provided axes.
    pub fn finish(
        &mut self,
        axes: AxisInputs<'a>,
        completeness: &'a completeness::View,
        global_conformance: &'a GlobalConformanceInput,
        execution: AssessmentExecution,
        limits: ConformanceLimits,
    ) -> std::result::Result<&Report, ActivatedAssessmentError> {
        let request = self.latest_request.ok_or(ActivatedAssessmentError::Kernel(
            Refusal::UnexpectedActivatedOccurrence,
        ))?;
        validate_temporal_evidence(request, axes, completeness, execution)
            .map_err(ActivatedAssessmentError::Kernel)?;
        self.inner
            .finish(axes, completeness, global_conformance, execution, limits)
            .map_err(ActivatedAssessmentError::Conformance)
    }

    /// Returns the latest settled report, if any prefix has been assessed.
    #[must_use]
    pub const fn report(&self) -> Option<&Report> {
        self.inner.report()
    }
}

impl<'a> ActivationRegistry<'a> {
    /// Retains one kernel-validated activation, idempotently on replay.
    pub fn admit(
        &mut self,
        candidate: ActivationBinding<'a>,
    ) -> Result<&ActivationBinding<'a>, Refusal> {
        if let Some(index) = self
            .bindings
            .iter()
            .position(|existing| existing.same_key(&candidate))
        {
            if self.bindings[index].same_checked_content(&candidate) {
                return Ok(&self.bindings[index]);
            }
            return Err(Refusal::ConflictingActivation);
        }
        self.bindings.push(candidate);
        match self.bindings.last() {
            Some(binding) => Ok(binding),
            None => unreachable!("a retained activation binding is immediately readable"),
        }
    }

    /// Retained exact bindings in first-admission order.
    #[must_use]
    pub fn bindings(&self) -> &[ActivationBinding<'a>] {
        &self.bindings
    }
}

/// One exact QSL control retained as a conformance obligation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LinkedObligation {
    handle: u32,
    operation: ControlOperation,
}

/// One admitted compensation attempt bound to its exact derived registration.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LinkedCompensationAttempt {
    registration: Registration,
    subject: QualifiedSubject,
}

impl LinkedCompensationAttempt {
    /// Returns the exact registration this attempt belongs to.
    #[must_use]
    pub const fn registration(&self) -> &Registration {
        &self.registration
    }

    /// Returns QObs's complete authority-qualified attempt subject.
    #[must_use]
    pub const fn subject(&self) -> &QualifiedSubject {
        &self.subject
    }
}

/// One admitted compensation effect bound to its exact derived registration.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LinkedCompensationEffect {
    registration: Registration,
    subject: QualifiedSubject,
}

impl LinkedCompensationEffect {
    /// Returns the exact registration this effect belongs to.
    #[must_use]
    pub const fn registration(&self) -> &Registration {
        &self.registration
    }

    /// Returns QObs's complete authority-qualified compensation-effect subject.
    #[must_use]
    pub const fn subject(&self) -> &QualifiedSubject {
        &self.subject
    }
}

impl LinkedObligation {
    /// Returns the exact declaration-local QSL control coordinate.
    #[must_use]
    pub const fn handle(&self) -> u32 {
        self.handle
    }

    /// Returns QSL's complete typed control operation.
    #[must_use]
    pub const fn operation(&self) -> &ControlOperation {
        &self.operation
    }
}

/// Non-retargetable conformance input for one assessment request.
///
/// This kernel retains the exact linked definition and derived registration
/// population. Batch and incremental frontends must consume this same value;
/// neither receives a separately assembled compensation model.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Kernel {
    definition: DefinitionIdentity,
    obligations: Vec<LinkedObligation>,
    causal_edges: Vec<quire_spec_language::protocol_artifact::wire::CausalEdge>,
    compensations: Vec<LinkedCompensationDefinition>,
    registrations: Vec<Registration>,
    attempts: Vec<LinkedCompensationAttempt>,
    effects: Vec<LinkedCompensationEffect>,
}

impl Kernel {
    /// Validates and retains one linked graph with its derived registrations.
    pub fn new(
        linked: &LinkedProtocol<'_>,
        compensation: &CompensationSnapshot,
        attempts: &[CompensationAttemptInput<'_>],
        effects: &[CompensationEffectInput<'_>],
    ) -> Result<Self, Refusal> {
        for registration in compensation.registrations() {
            if registration.key().definition() != linked.identity() {
                return Err(Refusal::ForeignDefinition);
            }
            if !linked
                .compensations()
                .iter()
                .any(|definition| definition.handle() == registration.key().compensation_handle())
            {
                return Err(Refusal::UnknownCompensation);
            }
        }
        let Body::Protocol { controls, .. } = &linked.declaration().body else {
            unreachable!("LinkedProtocol only retains a QSL protocol declaration");
        };
        let mut obligations = Vec::with_capacity(controls.len());
        for (index, control) in controls.iter().enumerate() {
            let handle = u32::try_from(index).map_err(|_| Refusal::UnrepresentableControl)?;
            obligations.push(LinkedObligation {
                handle,
                operation: control.operation.clone(),
            });
        }
        let attempts = attempts
            .iter()
            .map(|attempt| {
                let registration = registration_for(compensation, attempt.registration())?;
                Ok(LinkedCompensationAttempt {
                    registration,
                    subject: attempt.subject().clone(),
                })
            })
            .collect::<Result<Vec<_>, _>>()?;
        let effects = effects
            .iter()
            .map(|effect| {
                let registration = registration_for(compensation, effect.registration())?;
                Ok(LinkedCompensationEffect {
                    registration,
                    subject: effect.subject().clone(),
                })
            })
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Self {
            definition: linked.identity().clone(),
            obligations,
            causal_edges: linked.causal_edges().to_vec(),
            compensations: linked.compensations().to_vec(),
            registrations: compensation.registrations().to_vec(),
            attempts,
            effects,
        })
    }

    /// Replaces only the Engine-derived registration population while retaining
    /// this already-validated linked graph and its static conformance values.
    ///
    /// This is crate-private because an Engine derives the snapshot from its
    /// own immutable linked/runtime capability. Public callers must still use
    /// [`Self::new`], which validates every registration against the supplied
    /// graph before retaining it.
    #[must_use]
    pub(crate) fn with_engine_compensation(&self, compensation: &CompensationSnapshot) -> Self {
        Self {
            definition: self.definition.clone(),
            obligations: self.obligations.clone(),
            causal_edges: self.causal_edges.clone(),
            compensations: self.compensations.clone(),
            registrations: compensation.registrations().to_vec(),
            attempts: self.attempts.clone(),
            effects: self.effects.clone(),
        }
    }

    /// Returns the exact linked static identity governing this kernel.
    #[must_use]
    pub const fn definition(&self) -> &DefinitionIdentity {
        &self.definition
    }

    /// Returns every QSL control in exact declaration-table order.
    #[must_use]
    pub fn obligations(&self) -> &[LinkedObligation] {
        &self.obligations
    }

    /// Selects one exact QSL control obligation by its published handle.
    ///
    /// The lookup is declaration-local to this kernel's fixed FR-288
    /// definition and never treats a display name or caller ordering as a
    /// substitute coordinate.
    #[must_use]
    pub fn obligation(&self, handle: u32) -> Option<&LinkedObligation> {
        self.obligations
            .get(usize::try_from(handle).ok()?)
            .filter(|obligation| obligation.handle() == handle)
    }

    /// Returns QSL's exact authored causal graph edges in declaration order.
    ///
    /// These are retained alongside the control obligations so neither batch
    /// nor incremental conformance can infer precedence from arrival order,
    /// timestamps, or a caller-constructed substitute model.
    #[must_use]
    pub fn causal_edges(&self) -> &[quire_spec_language::protocol_artifact::wire::CausalEdge] {
        &self.causal_edges
    }

    /// Returns QSL's complete compensation definitions in declaration order.
    ///
    /// Each returned definition is bound to this kernel's fixed FR-288
    /// definition; callers cannot combine a registration from one graph with a
    /// recovery or retry expression from another.
    #[must_use]
    pub fn compensations(&self) -> &[LinkedCompensationDefinition] {
        &self.compensations
    }

    /// Selects one exact compensation definition by its QSL table coordinate.
    #[must_use]
    pub fn compensation(&self, handle: u32) -> Option<&LinkedCompensationDefinition> {
        self.compensations
            .get(usize::try_from(handle).ok()?)
            .filter(|definition| definition.handle() == handle)
    }

    /// Returns replay-idempotent registrations in QSL declaration order.
    #[must_use]
    pub fn registrations(&self) -> &[Registration] {
        &self.registrations
    }

    /// Returns admitted attempt subjects bound to registrations in this kernel.
    #[must_use]
    pub fn attempts(&self) -> &[LinkedCompensationAttempt] {
        &self.attempts
    }

    /// Returns admitted effect subjects bound to registrations in this kernel.
    #[must_use]
    pub fn effects(&self) -> &[LinkedCompensationEffect] {
        &self.effects
    }

    /// Construct the one owner-authorized FR-300 activation binding.
    ///
    /// Repeating this operation with the same strict inputs is idempotent:
    /// every retained component is copied from the same immutable owner values.
    /// Any package, mapping, subject, trigger, or result substitution refuses
    /// before a binding can be returned.
    pub fn bind_activation<'a>(
        &self,
        linked: &LinkedProtocol<'_>,
        mappings: &v3::AdmittedPackage,
        subject: &QualifiedSubject,
        temporal_subject: &'a ValidatedTemporalSubject,
        request: &'a temporal_v2::ValidatedRequest,
        result: &'a temporal_v2::ValidatedResult,
    ) -> Result<ActivationBinding<'a>, Refusal> {
        if self.definition != *linked.identity()
            || !same_activation_graph(mappings.package(), linked.context().package().package())
        {
            return Err(Refusal::ForeignActivationPackage);
        }
        if request.semantic_trigger() != subject.identity().as_bytes() {
            return Err(Refusal::ActivationSubjectMismatch);
        }
        if temporal_subject.package_digest() != linked.context().package().digest()
            || temporal_subject.declaration() != request.declaration()
            || temporal_subject.document().identity() != request.subject_identity()
        {
            return Err(Refusal::InvalidActivationTemporal);
        }
        if result.semantic_trigger() != request.semantic_trigger()
            || result.request_identity() != request.document().identity()
        {
            return Err(Refusal::ActivationResultMismatch);
        }
        if !self
            .definition
            .temporal_bindings()
            .iter()
            .any(|binding| binding.declaration == request.declaration())
        {
            return Err(Refusal::InvalidActivationTemporal);
        }
        let mut selected = mappings.activation_mappings().iter().filter(|mapping| {
            mapping.control.declaration == self.definition.declaration_handle()
                && mapping.temporal_declaration == request.declaration()
        });
        let Some(mapping) = selected.next() else {
            return Err(Refusal::MissingActivationMapping);
        };
        if selected.next().is_some() {
            return Err(Refusal::MissingActivationMapping);
        }
        let Some(obligation) = self.obligation(mapping.control.index) else {
            return Err(Refusal::InvalidActivationControl);
        };
        if mapping.control.declaration != self.definition.declaration_handle()
            || !matches!(obligation.operation(), ControlOperation::Event { .. })
        {
            return Err(Refusal::InvalidActivationControl);
        }
        Ok(ActivationBinding {
            definition: self.definition.clone(),
            control: mapping.control.clone(),
            subject: subject.clone(),
            temporal_subject,
            request,
            result,
        })
    }

    /// Pair one retained FR-300 binding with its validated QSL occurrence.
    pub fn activate_occurrence<'a>(
        &self,
        activation: &ActivationBinding<'a>,
        occurrence: OccurrenceKey<'a>,
    ) -> Result<ActivatedOccurrence<'a>, Refusal> {
        if activation.definition != self.definition {
            return Err(Refusal::ForeignActivatedOccurrence);
        }
        activation.require_occurrence(&occurrence)?;
        Ok(ActivatedOccurrence {
            activation: activation.clone(),
            occurrence,
        })
    }

    /// Derive the finite evaluator model from owner-authorized activations.
    ///
    /// This is the production replacement for a caller-authored obligation
    /// list: the FR-230 key comes from FR-300, workflow/repeat dimensions come
    /// from QSL's occurrence key, and predecessor membership comes only from
    /// authored non-repeat causal edges. A QObs observation is later matched by
    /// its complete selected subject, never by a caller-supplied association.
    pub(crate) fn derive_model(
        &self,
        activations: &[ActivatedOccurrence<'_>],
    ) -> Result<Model, Refusal> {
        let mut keyed = activations
            .iter()
            .map(|activation| (derived_obligation_id(activation), activation))
            .collect::<Vec<_>>();
        keyed.sort_by(|(left, _), (right, _)| left.cmp(right));
        if keyed.windows(2).any(|pair| pair[0].0 == pair[1].0) {
            return Err(Refusal::DuplicateActivatedObligation);
        }
        let mut obligations = Vec::with_capacity(activations.len());
        for (identity, activation) in &keyed {
            if activation.activation.definition != self.definition {
                return Err(Refusal::ForeignActivatedOccurrence);
            }
            let predecessors = keyed
                .iter()
                .filter(|(_, candidate)| {
                    same_occurrence_scope(candidate, activation)
                        && candidate.activation.control() != activation.activation.control()
                        && self.causal_edges.iter().any(|edge| {
                            edge.kind != EdgeKind::RepeatProgress
                                && edge.from.node == *candidate.activation.control()
                                && edge.to.node == *activation.activation.control()
                        })
                })
                .map(|(identity, _)| identity.clone())
                .collect();
            obligations.push(ObservationObligation {
                identity: identity.clone(),
                subject_kind: activation.activation.subject().kind().as_bytes().into(),
                subject_identity: activation.activation.subject().identity().as_bytes().into(),
                workflow_identity: activation.occurrence.workflow().as_str().into(),
                effect_identity: None,
                // A strict activation selects a concrete QObs subject. Its
                // value is therefore required to establish a decisive event
                // fact; QObs preserves an absent value distinctly.
                valuation_required: true,
                predecessors,
            });
        }
        let protocol_identity = format!(
            "{}:{}",
            self.definition.source_document().identity,
            self.definition.declaration_handle()
        );
        Model::new(
            protocol_identity,
            obligations,
            ConformanceLimits::owner_max(),
        )
        .map_err(|_| Refusal::InvalidDerivedModel)
    }

    /// Associates an optional admitted observation only with its already
    /// validated FR-300/QSL occurrence. Callers cannot provide an obligation
    /// identifier or a workflow/effect correlation separately.
    #[must_use]
    pub fn activation_fact<'a>(
        &self,
        occurrence: ActivatedOccurrence<'a>,
        observation: Option<&'a observation::View>,
    ) -> ActivationFact<'a> {
        ActivationFact {
            occurrence,
            observation,
        }
    }

    /// Assess activation-bound QObs facts through the shared deterministic
    /// batch kernel without admitting a caller-built model or obligation id.
    pub fn assess_activated(
        &self,
        facts: &[ActivationFact<'_>],
        axes: AxisInputs<'_>,
        completeness: &completeness::View,
        global_conformance: &GlobalConformanceInput,
        execution: AssessmentExecution,
        limits: ConformanceLimits,
    ) -> std::result::Result<Report, ActivatedAssessmentError> {
        for fact in facts {
            validate_temporal_evidence(
                fact.occurrence.activation.request,
                axes,
                completeness,
                execution,
            )
            .map_err(ActivatedAssessmentError::Kernel)?;
        }
        let activations = facts
            .iter()
            .map(|fact| fact.occurrence.clone())
            .collect::<Vec<_>>();
        let model = self
            .derive_model(&activations)
            .map_err(ActivatedAssessmentError::Kernel)?;
        let observations = facts
            .iter()
            .filter_map(|fact| {
                let identity = derived_obligation_id(&fact.occurrence);
                let obligation = model
                    .obligations()
                    .iter()
                    .find(|obligation| obligation.identity == identity)?;
                fact.observation.map(|observation| BoundObservation {
                    obligation: identity,
                    observation,
                    workflow_identity: fact.occurrence.occurrence.workflow().as_str().into(),
                    effect_identity: obligation.effect_identity.clone(),
                })
            })
            .collect::<Vec<_>>();
        assess_batch(
            Request {
                model: &model,
                observations: &observations,
                axes,
                completeness,
                global_conformance,
                execution,
            },
            limits,
        )
        .map_err(ActivatedAssessmentError::Conformance)
    }

    /// Assess activation-bound facts under the shared Protocol accountant.
    /// Resource exhaustion remains a distinct operation outcome.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn assess_activated_bounded(
        &self,
        facts: &[ActivationFact<'_>],
        axes: AxisInputs<'_>,
        completeness: &completeness::View,
        global_conformance: &GlobalConformanceInput,
        execution: AssessmentExecution,
        limits: ConformanceLimits,
        resources: &mut ResourceAccountant<'_>,
    ) -> OperationOutcome<Report, ActivatedAssessmentError> {
        if let Err(error) = precharge_activation_model(facts, limits, resources) {
            return OperationOutcome::from_charge(error);
        }
        let activations = facts
            .iter()
            .map(|fact| fact.occurrence.clone())
            .collect::<Vec<_>>();
        let model = match self.derive_model(&activations) {
            Ok(model) => model,
            Err(error) => {
                return OperationOutcome::DomainError(ActivatedAssessmentError::Kernel(error));
            }
        };
        let observations = facts
            .iter()
            .filter_map(|fact| {
                let identity = derived_obligation_id(&fact.occurrence);
                let obligation = model
                    .obligations()
                    .iter()
                    .find(|obligation| obligation.identity == identity)?;
                fact.observation.map(|observation| BoundObservation {
                    obligation: identity,
                    observation,
                    workflow_identity: fact.occurrence.occurrence.workflow().as_str().into(),
                    effect_identity: obligation.effect_identity.clone(),
                })
            })
            .collect::<Vec<_>>();
        match assess_batch_bounded(
            Request {
                model: &model,
                observations: &observations,
                axes,
                completeness,
                global_conformance,
                execution,
            },
            limits,
            resources,
        ) {
            OperationOutcome::Completed(report) => OperationOutcome::Completed(report),
            OperationOutcome::DomainError(error) => {
                OperationOutcome::DomainError(ActivatedAssessmentError::Conformance(error))
            }
            OperationOutcome::ResourceIncomplete(incomplete) => {
                OperationOutcome::ResourceIncomplete(incomplete)
            }
            OperationOutcome::AccountingFault(fault) => OperationOutcome::AccountingFault(fault),
        }
    }
}

fn derived_obligation_id(activation: &ActivatedOccurrence<'_>) -> ObligationId {
    let definition = activation.activation.definition();
    let digest = occurrence_identity_digest(
        definition.source_document().identity.as_bytes(),
        definition.declaration_handle(),
        activation.activation.control().index,
        activation.activation.subject().identity().as_bytes(),
        activation.activation.semantic_trigger(),
        activation.occurrence.workflow().as_str().as_bytes(),
        activation
            .occurrence
            .repeat_ordinals()
            .iter()
            .map(|ordinal| ordinal.value()),
    );
    ObligationId::retained(format!("fr230:sha256:{digest:x}"))
}

fn same_occurrence_scope(left: &ActivatedOccurrence<'_>, right: &ActivatedOccurrence<'_>) -> bool {
    left.occurrence.workflow().as_str() == right.occurrence.workflow().as_str()
        && left.occurrence.repeat_ordinals() == right.occurrence.repeat_ordinals()
}

fn occurrence_identity_digest(
    source_document: &[u8],
    declaration: u32,
    control: u32,
    subject: &[u8],
    trigger: &[u8],
    workflow: &[u8],
    repeats: impl IntoIterator<Item = i64>,
) -> sha2::digest::Output<Sha256> {
    let mut digest = Sha256::new();
    for component in [source_document, subject, trigger, workflow] {
        let mut length = [0_u8; 16];
        let native_length = component.len().to_be_bytes();
        let offset = length.len() - native_length.len();
        length[offset..].copy_from_slice(&native_length);
        digest.update(length);
        digest.update(component);
    }
    digest.update(declaration.to_be_bytes());
    digest.update(control.to_be_bytes());
    for ordinal in repeats {
        digest.update(ordinal.to_be_bytes());
    }
    digest.finalize()
}

fn precharge_activation_model(
    facts: &[ActivationFact<'_>],
    limits: ConformanceLimits,
    resources: &mut ResourceAccountant<'_>,
) -> Result<(), ChargeError> {
    const COMPONENT_COUNT: usize = 4;
    const COMPONENT_LENGTH_BYTES: usize = 16;
    const HANDLE_BYTES: usize = 2 * core::mem::size_of::<u32>();

    let population = facts.len();
    resources.charge(ACTIVATION_OPERATION_SOURCE.id(), population)?;
    let mut identity_bytes = 0usize;
    for fact in facts {
        let activation = &fact.occurrence.activation;
        let occurrence = &fact.occurrence.occurrence;
        let framing_bytes = COMPONENT_COUNT
            .checked_mul(COMPONENT_LENGTH_BYTES)
            .and_then(|bytes| bytes.checked_add(HANDLE_BYTES))
            .ok_or(ChargeError::CounterOverflow {
                site: ACTIVATION_IDENTITY_SOURCE.id(),
                consumed: identity_bytes,
                requested: COMPONENT_LENGTH_BYTES,
            })?;
        identity_bytes =
            identity_bytes
                .checked_add(framing_bytes)
                .ok_or(ChargeError::CounterOverflow {
                    site: ACTIVATION_IDENTITY_SOURCE.id(),
                    consumed: identity_bytes,
                    requested: framing_bytes,
                })?;
        for bytes in [
            activation
                .definition()
                .source_document()
                .identity
                .as_bytes(),
            activation.subject().identity().as_bytes(),
            activation.semantic_trigger(),
            occurrence.workflow().as_str().as_bytes(),
        ] {
            identity_bytes =
                identity_bytes
                    .checked_add(bytes.len())
                    .ok_or(ChargeError::CounterOverflow {
                        site: ACTIVATION_IDENTITY_SOURCE.id(),
                        consumed: identity_bytes,
                        requested: bytes.len(),
                    })?;
        }
        let repeat_bytes = occurrence.repeat_ordinals().len().checked_mul(8).ok_or(
            ChargeError::CounterOverflow {
                site: ACTIVATION_IDENTITY_SOURCE.id(),
                consumed: identity_bytes,
                requested: occurrence.repeat_ordinals().len(),
            },
        )?;
        identity_bytes =
            identity_bytes
                .checked_add(repeat_bytes)
                .ok_or(ChargeError::CounterOverflow {
                    site: ACTIVATION_IDENTITY_SOURCE.id(),
                    consumed: identity_bytes,
                    requested: repeat_bytes,
                })?;
    }
    let pair_operations =
        population
            .checked_mul(population)
            .ok_or(ChargeError::CounterOverflow {
                site: ACTIVATION_OPERATION_SOURCE.id(),
                consumed: population,
                requested: population,
            })?;
    let allocation_count = population
        .checked_mul(6)
        .and_then(|count| count.checked_add(4))
        .ok_or(ChargeError::CounterOverflow {
            site: ACTIVATION_ALLOCATION_SOURCE.id(),
            consumed: population,
            requested: 6,
        })?;
    resources.charge_all_mixed(&[
        (
            ACTIVATION_NODE_SOURCE.id(),
            population,
            Some(limits.effective().max_obligations),
        ),
        (ACTIVATION_IDENTITY_SOURCE.id(), identity_bytes, None),
        (ACTIVATION_OPERATION_SOURCE.id(), pair_operations, None),
        (ACTIVATION_ALLOCATION_SOURCE.id(), allocation_count, None),
    ])
}

pub(crate) fn validate_temporal_evidence_bounded(
    request: &temporal_v2::ValidatedRequest,
    axes: AxisInputs<'_>,
    completeness: &completeness::View,
    execution: AssessmentExecution,
    resources: &mut ResourceAccountant<'_>,
) -> OperationOutcome<(), Refusal> {
    let charges = [
        axes.decision_scope_progress.bytes().len(),
        axes.decision_scope_closure.bytes().len(),
        axes.surrounding_execution_progress.bytes().len(),
        axes.surrounding_execution_closure.bytes().len(),
        completeness.bytes().len(),
    ]
    .map(|bytes| (TEMPORAL_EVIDENCE_SOURCE.id(), bytes));
    if let Err(error) = resources.charge_all(&charges) {
        return OperationOutcome::from_charge(error);
    }
    OperationOutcome::from_domain(validate_temporal_evidence(
        request,
        axes,
        completeness,
        execution,
    ))
}

fn validate_temporal_evidence(
    request: &temporal_v2::ValidatedRequest,
    axes: AxisInputs<'_>,
    completeness: &completeness::View,
    execution: AssessmentExecution,
) -> Result<(), Refusal> {
    let matches = progress_evidence_matches(
        request.decision_progress(),
        axes.decision_scope_progress,
        "decision-progress",
    ) && closure_evidence_matches(
        request.decision_closure(),
        axes.decision_scope_closure,
        "decision-closure",
    ) && progress_evidence_matches(
        request.surrounding_progress(),
        axes.surrounding_execution_progress,
        "surrounding-progress",
    ) && closure_evidence_matches(
        request.surrounding_closure(),
        axes.surrounding_execution_closure,
        "surrounding-closure",
    ) && completeness_evidence_matches(request.completeness(), completeness)
        && execution_matches(request.execution(), execution);
    matches
        .then_some(())
        .ok_or(Refusal::TemporalEvidenceMismatch)
}

fn progress_evidence_matches(
    admitted: native_temporal::request::ProgressView<'_>,
    owner: &progress::View,
    scope: &str,
) -> bool {
    evidence_matches(
        admitted.reference(),
        progress::CONTRACT,
        progress::SCHEMA_SHA256,
        owner.identity().as_str(),
        owner.bytes(),
        owner.authority(),
        scope,
        owner.payload().required_sources().len(),
    ) && progress_watermark(owner).is_some_and(|watermark| admitted.watermark() == watermark)
}

fn closure_evidence_matches(
    admitted: native_temporal::request::ClosureView<'_>,
    owner: &observation_closure::View,
    scope: &str,
) -> bool {
    evidence_matches(
        admitted.reference(),
        observation_closure::CONTRACT,
        observation_closure::SCHEMA_SHA256,
        owner.identity().as_str(),
        owner.bytes(),
        owner.authority(),
        scope,
        owner.payload().required_sources().len(),
    ) && matches!(
        (owner.payload().state(), admitted.state()),
        (
            OpenClosed::Open,
            quire_spec_language::temporal::Closure::Open
        ) | (
            OpenClosed::Closed,
            quire_spec_language::temporal::Closure::Closed
        )
    )
}

fn completeness_evidence_matches(
    admitted: native_temporal::request::CompletenessView<'_>,
    owner: &completeness::View,
) -> bool {
    evidence_matches(
        admitted.reference(),
        completeness::CONTRACT,
        completeness::SCHEMA_SHA256,
        owner.identity().as_str(),
        owner.bytes(),
        owner.authority(),
        "completeness",
        owner.payload().fact_count(),
    ) && matches!(
        (owner.payload().state(), admitted.state()),
        (
            completeness::State::Complete,
            quire_spec_language::temporal::Completeness::Complete
        ) | (
            completeness::State::Incomplete,
            quire_spec_language::temporal::Completeness::Incomplete
        )
    )
}

#[allow(
    clippy::too_many_arguments,
    reason = "the comparison preserves every independently owned evidence coordinate"
)]
fn evidence_matches(
    admitted: &native_temporal::EvidenceRef,
    contract: &str,
    schema_sha256: &str,
    identity: &str,
    bytes: &[u8],
    authority: &AuthoritySelection,
    scope: &str,
    population: usize,
) -> bool {
    admitted.contract() == contract
        && admitted.schema_digest() == schema_sha256
        && admitted.identity() == identity
        && digest_matches(bytes, admitted.digest())
        && admitted.authority_identity() == authority.definition_identity.as_str()
        && admitted.authority_revision() == authority.definition_revision.as_str()
        && hex_matches(
            authority.definition_digest.as_bytes(),
            admitted.authority_digest(),
        )
        && admitted.scope() == scope
        && u64::try_from(population).is_ok_and(|population| admitted.population() == population)
}

fn progress_watermark(view: &progress::View) -> Option<i64> {
    let watermark = match view.payload().boundary() {
        BoundaryRef::EventPosition { watermark, .. }
        | BoundaryRef::FixedSample { watermark, .. } => watermark,
        BoundaryRef::TimestampedEvent {
            watermark_nanos, ..
        } => watermark_nanos,
    };
    watermark.parse().ok()
}

fn execution_matches(
    admitted: quire_spec_language::temporal::Execution,
    owner: AssessmentExecution,
) -> bool {
    matches!(
        (admitted, owner),
        (
            quire_spec_language::temporal::Execution::Completed,
            AssessmentExecution::Completed
        ) | (
            quire_spec_language::temporal::Execution::Failed,
            AssessmentExecution::Failed
        )
    )
}

fn digest_matches(bytes: &[u8], expected: &str) -> bool {
    hex_matches(&Sha256::digest(bytes), expected)
}

fn hex_matches(bytes: &[u8], expected: &str) -> bool {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    expected.len() == bytes.len() * 2
        && bytes.iter().enumerate().all(|(index, byte)| {
            expected.as_bytes()[index * 2] == HEX[usize::from(byte >> 4)]
                && expected.as_bytes()[index * 2 + 1] == HEX[usize::from(byte & 0x0f)]
        })
}

/// Confirms that a `/3` activation table extends this exact admitted `/2`
/// graph.  Version headers intentionally differ; every graph-bearing field is
/// compared in QSL's typed vocabulary rather than normalizing either package
/// into a Protocol-owned surrogate.
fn same_activation_graph(
    mapped: &v3::wire::Package,
    linked: &quire_spec_language::protocol_artifact::v2::wire::Package,
) -> bool {
    let mapped_inherited = &mapped.inherited;
    let linked_inherited = &linked.inherited;
    mapped.temporal_bindings == linked.temporal_bindings
        && mapped_inherited.package_type == linked_inherited.package_type
        && mapped_inherited.encoding == linked_inherited.encoding
        && mapped_inherited.numeric == linked_inherited.numeric
        && mapped_inherited.contract == linked_inherited.contract
        && mapped_inherited.producer == linked_inherited.producer
        && mapped_inherited.baseline == linked_inherited.baseline
        && mapped_inherited.language == linked_inherited.language
        && mapped_inherited.package_definition == linked_inherited.package_definition
        && mapped_inherited.features == linked_inherited.features
        && mapped_inherited.sources == linked_inherited.sources
        && mapped_inherited.dependencies == linked_inherited.dependencies
        && mapped_inherited.definitions == linked_inherited.definitions
        && mapped_inherited.models == linked_inherited.models
        && mapped_inherited.types == linked_inherited.types
        && mapped_inherited.declarations == linked_inherited.declarations
}

fn registration_for(
    compensation: &CompensationSnapshot,
    input: crate::model::v2::CompensationRegistrationInput<'_>,
) -> Result<Registration, Refusal> {
    compensation
        .registrations()
        .iter()
        .find(|registration| {
            registration.key().compensation_handle() == input.compensation_handle()
                && registration.key().forward_effect() == input.effect().subject()
        })
        .cloned()
        .ok_or(Refusal::UnknownRegistration)
}

#[cfg(test)]
mod tests {
    use super::{evidence_matches, occurrence_identity_digest};
    use ix_trace_rs::trace;
    use quire_observation::{authority::AuthoritySelection, Digest as ObservationDigest, Identity};
    use quire_spec_language::protocol_artifact::native_temporal::EvidenceRef;
    use sha2::{Digest as _, Sha256};

    #[test]
    #[trace("Task-021")]
    fn temporal_evidence_requires_every_owner_coordinate() {
        let bytes = b"owner document";
        let document_digest = format!("{:x}", Sha256::digest(bytes));
        let authority_digest_bytes = Sha256::digest(b"authority definition");
        let authority_digest = format!("{authority_digest_bytes:x}");
        let authority = AuthoritySelection {
            definition_identity: Identity::new("authority:owner"),
            definition_revision: Identity::new("7"),
            definition_digest: ObservationDigest::new(authority_digest_bytes.into()),
        };
        let make = |contract: &str,
                    schema: &str,
                    identity: &str,
                    digest: &str,
                    authority_identity: &str,
                    authority_revision: &str,
                    authority_digest: &str,
                    scope: &str,
                    population: u64| {
            EvidenceRef::new(
                contract,
                schema,
                identity,
                digest,
                authority_identity,
                authority_revision,
                authority_digest,
                scope,
                population,
            )
        };
        let matches = |evidence: &EvidenceRef| {
            evidence_matches(
                evidence,
                "owner.contract/1",
                "schema-digest",
                "document:owner",
                bytes,
                &authority,
                "decision-progress",
                7,
            )
        };

        assert!(matches(&make(
            "owner.contract/1",
            "schema-digest",
            "document:owner",
            &document_digest,
            "authority:owner",
            "7",
            &authority_digest,
            "decision-progress",
            7,
        )));

        let mutations = [
            (
                "contract",
                make(
                    "owner.contract/2",
                    "schema-digest",
                    "document:owner",
                    &document_digest,
                    "authority:owner",
                    "7",
                    &authority_digest,
                    "decision-progress",
                    7,
                ),
            ),
            (
                "schema digest",
                make(
                    "owner.contract/1",
                    "other-schema",
                    "document:owner",
                    &document_digest,
                    "authority:owner",
                    "7",
                    &authority_digest,
                    "decision-progress",
                    7,
                ),
            ),
            (
                "document identity",
                make(
                    "owner.contract/1",
                    "schema-digest",
                    "document:other",
                    &document_digest,
                    "authority:owner",
                    "7",
                    &authority_digest,
                    "decision-progress",
                    7,
                ),
            ),
            (
                "document digest",
                make(
                    "owner.contract/1",
                    "schema-digest",
                    "document:owner",
                    &"0".repeat(64),
                    "authority:owner",
                    "7",
                    &authority_digest,
                    "decision-progress",
                    7,
                ),
            ),
            (
                "authority identity",
                make(
                    "owner.contract/1",
                    "schema-digest",
                    "document:owner",
                    &document_digest,
                    "authority:other",
                    "7",
                    &authority_digest,
                    "decision-progress",
                    7,
                ),
            ),
            (
                "authority revision",
                make(
                    "owner.contract/1",
                    "schema-digest",
                    "document:owner",
                    &document_digest,
                    "authority:owner",
                    "8",
                    &authority_digest,
                    "decision-progress",
                    7,
                ),
            ),
            (
                "authority digest",
                make(
                    "owner.contract/1",
                    "schema-digest",
                    "document:owner",
                    &document_digest,
                    "authority:owner",
                    "7",
                    &"f".repeat(64),
                    "decision-progress",
                    7,
                ),
            ),
            (
                "scope",
                make(
                    "owner.contract/1",
                    "schema-digest",
                    "document:owner",
                    &document_digest,
                    "authority:owner",
                    "7",
                    &authority_digest,
                    "decision-closure",
                    7,
                ),
            ),
            (
                "population",
                make(
                    "owner.contract/1",
                    "schema-digest",
                    "document:owner",
                    &document_digest,
                    "authority:owner",
                    "7",
                    &authority_digest,
                    "decision-progress",
                    8,
                ),
            ),
        ];
        for (coordinate, mutation) in mutations {
            assert!(!matches(&mutation), "accepted mutated {coordinate}");
        }
    }

    #[test]
    #[trace("Task-021")]
    fn repeat_ordinals_are_part_of_the_derived_obligation_identity() {
        let derive = |repeats: &[i64]| {
            occurrence_identity_digest(
                b"source:document",
                3,
                5,
                b"subject:identity",
                b"semantic-trigger",
                b"workflow:identity",
                repeats.iter().copied(),
            )
        };

        assert_ne!(derive(&[0]), derive(&[1]));
        assert_ne!(derive(&[0, 1]), derive(&[0, 2]));
        assert_ne!(
            occurrence_identity_digest(b"a", 3, 5, b"bc", b"d", b"e", []),
            occurrence_identity_digest(b"ab", 3, 5, b"c", b"d", b"e", [])
        );
    }
}
