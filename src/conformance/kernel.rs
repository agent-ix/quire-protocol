// SPDX-License-Identifier: AGPL-3.0-or-later
//! One immutable conformance input assembled from linked static and compensation facts.

use crate::assessment::ActivationBinding;
use crate::compensation::registration::{Registration, Snapshot as CompensationSnapshot};
use crate::model::v2::{
    CompensationAttemptInput, CompensationEffectInput, DefinitionIdentity,
    LinkedCompensationDefinition, LinkedProtocol,
};
use quire_observation::QualifiedSubject;
use quire_spec_language::protocol_artifact::wire::{Body, ControlOperation};

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
    /// An activation row does not select this linked protocol declaration.
    ForeignActivationControl,
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
        activation: Option<&ActivationBinding<'_>>,
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
        let obligations = activation
            .map(|activation| {
                if activation.control().declaration != linked.identity().declaration_handle() {
                    return Err(Refusal::ForeignActivationControl);
                }
                let index = usize::try_from(activation.control().index)
                    .map_err(|_| Refusal::UnrepresentableControl)?;
                let control = controls
                    .get(index)
                    .ok_or(Refusal::ForeignActivationControl)?;
                Ok(vec![LinkedObligation {
                    handle: activation.control().index,
                    operation: control.operation.clone(),
                }])
            })
            .transpose()?
            .unwrap_or_default();
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

    /// Returns only `/3`-admitted event-triggered obligations in exact mapping order.
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
            .iter()
            .find(|obligation| obligation.handle() == handle)
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
