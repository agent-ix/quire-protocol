// SPDX-License-Identifier: AGPL-3.0-or-later
//! One immutable conformance input assembled from linked static and compensation facts.

use crate::compensation::registration::{Registration, Snapshot as CompensationSnapshot};
use crate::model::v2::{
    CompensationAttemptInput, CompensationEffectInput, DefinitionIdentity,
    LinkedCompensationDefinition, LinkedProtocol,
};
use quire_observation::QualifiedSubject;
use quire_spec_language::protocol_artifact::{
    native_temporal::{request::CaptureView, result::ActivationState, v2 as temporal_v2},
    v3,
    wire::{Body, ControlOperation, Handle},
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
        self.request.document().identity() == other.request.document().identity()
            && self.evaluation_anchor() == other.evaluation_anchor()
            && self.captures().eq(other.captures())
            && self.result.document().identity() == other.result.document().identity()
            && self.activation() == other.activation()
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

impl<'a> ActivationRegistry<'a> {
    /// Validates and retains one exact activation, idempotently on replay.
    pub fn admit(
        &mut self,
        kernel: &Kernel,
        linked: &LinkedProtocol<'_>,
        mappings: &v3::AdmittedPackage,
        subject: &QualifiedSubject,
        request: &'a temporal_v2::ValidatedRequest,
        result: &'a temporal_v2::ValidatedResult,
    ) -> Result<&ActivationBinding<'a>, Refusal> {
        let candidate = kernel.bind_activation(linked, mappings, subject, request, result)?;
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
        request: &'a temporal_v2::ValidatedRequest,
        result: &'a temporal_v2::ValidatedResult,
    ) -> Result<ActivationBinding<'a>, Refusal> {
        if self.definition != *linked.identity()
            || !same_activation_graph(
                &mappings.package().inherited,
                linked.context().package().inherited(),
            )
        {
            return Err(Refusal::ForeignActivationPackage);
        }
        if request.semantic_trigger() != subject.identity().as_bytes() {
            return Err(Refusal::ActivationSubjectMismatch);
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
            request,
            result,
        })
    }
}

/// Confirms that a `/3` activation table extends this exact admitted `/2`
/// graph.  Version headers intentionally differ; every graph-bearing field is
/// compared in QSL's typed vocabulary rather than normalizing either package
/// into a Protocol-owned surrogate.
fn same_activation_graph(
    mapped: &quire_spec_language::protocol_artifact::wire::Package,
    linked: &quire_spec_language::protocol_artifact::wire::Package,
) -> bool {
    mapped.package_type == linked.package_type
        && mapped.encoding == linked.encoding
        && mapped.numeric == linked.numeric
        && mapped.contract == linked.contract
        && mapped.producer == linked.producer
        && mapped.baseline == linked.baseline
        && mapped.language == linked.language
        && mapped.package_definition == linked.package_definition
        && mapped.features == linked.features
        && mapped.sources == linked.sources
        && mapped.dependencies == linked.dependencies
        && mapped.definitions == linked.definitions
        && mapped.models == linked.models
        && mapped.types == linked.types
        && mapped.declarations == linked.declarations
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
