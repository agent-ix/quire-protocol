// SPDX-License-Identifier: AGPL-3.0-or-later
//! One immutable conformance input assembled from linked static and compensation facts.

use crate::assessment::{AxisInputs, GlobalConformanceInput};
use crate::bounds::inventory::{OperationOutcome, ResourceAccountant};
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
    authority::{completeness, observation},
    QualifiedSubject,
};
use quire_spec_language::protocol_artifact::{
    native_temporal::{request::CaptureView, result::ActivationState, v2 as temporal_v2},
    temporal_subject::ValidatedTemporalSubject,
    v3,
    wire::{Body, ControlOperation, EdgeKind, Handle},
    OccurrenceKey,
};

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
#[derive(Clone, Debug)]
pub struct ActivatedIncremental<'a> {
    model: Model,
    inner: Incremental<'a>,
}

impl<'a> ActivatedIncremental<'a> {
    /// Starts a settled incremental path over the complete admitted activation
    /// population.
    pub fn new(kernel: &Kernel, activations: &[ActivatedOccurrence<'a>]) -> Result<Self, Refusal> {
        let model = kernel.derive_model(activations)?;
        Ok(Self {
            inner: Incremental::new(model.clone()),
            model,
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
        self.inner
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
            .map_err(ActivatedAssessmentError::Conformance)
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
    /// Validates and retains one exact activation, idempotently on replay.
    pub fn admit(
        &mut self,
        kernel: &Kernel,
        linked: &LinkedProtocol<'_>,
        mappings: &v3::AdmittedPackage,
        subject: &QualifiedSubject,
        temporal_subject: &'a ValidatedTemporalSubject,
        request: &'a temporal_v2::ValidatedRequest,
        result: &'a temporal_v2::ValidatedResult,
    ) -> Result<&ActivationBinding<'a>, Refusal> {
        let candidate =
            kernel.bind_activation(linked, mappings, subject, temporal_subject, request, result)?;
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
        let mut obligations = Vec::with_capacity(activations.len());
        for activation in activations {
            if activation.activation.definition != self.definition {
                return Err(Refusal::ForeignActivatedOccurrence);
            }
            let identity = derived_obligation_id(activation);
            if obligations
                .iter()
                .any(|existing: &ObservationObligation| existing.identity == identity)
            {
                return Err(Refusal::DuplicateActivatedObligation);
            }
            let predecessors = activations
                .iter()
                .filter(|candidate| {
                    same_occurrence_scope(candidate, activation)
                        && candidate.activation.control() != activation.activation.control()
                        && self.causal_edges.iter().any(|edge| {
                            edge.kind != EdgeKind::RepeatProgress
                                && edge.from.node == *candidate.activation.control()
                                && edge.to.node == *activation.activation.control()
                        })
                })
                .map(derived_obligation_id)
                .collect();
            obligations.push(ObservationObligation {
                identity,
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
        let activations = facts
            .iter()
            .map(|fact| fact.occurrence.clone())
            .collect::<Vec<_>>();
        let model = self
            .derive_model(&activations)
            .map_err(ActivatedAssessmentError::Kernel)?;
        let observations = facts
            .iter()
            .zip(model.obligations())
            .filter_map(|(fact, obligation)| {
                fact.observation.map(|observation| BoundObservation {
                    obligation: obligation.identity.clone(),
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
            .zip(model.obligations())
            .filter_map(|(fact, obligation)| {
                fact.observation.map(|observation| BoundObservation {
                    obligation: obligation.identity.clone(),
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
    let mut identity = String::from("fr230:");
    identity.push_str(
        &activation
            .activation
            .definition()
            .source_document()
            .identity,
    );
    identity.push(':');
    identity.push_str(
        &activation
            .activation
            .definition()
            .declaration_handle()
            .to_string(),
    );
    identity.push(':');
    identity.push_str(&activation.activation.control().index.to_string());
    identity.push(':');
    append_hex(
        &mut identity,
        activation.activation.subject().identity().as_bytes(),
    );
    identity.push(':');
    append_hex(&mut identity, activation.activation.semantic_trigger());
    ObligationId::retained(identity)
}

fn same_occurrence_scope(left: &ActivatedOccurrence<'_>, right: &ActivatedOccurrence<'_>) -> bool {
    left.occurrence.workflow().as_str() == right.occurrence.workflow().as_str()
        && left.occurrence.repeat_ordinals() == right.occurrence.repeat_ordinals()
}

fn append_hex(target: &mut String, bytes: &[u8]) {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    for byte in bytes {
        target.push(char::from(HEX[usize::from(byte >> 4)]));
        target.push(char::from(HEX[usize::from(byte & 0x0f)]));
    }
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
