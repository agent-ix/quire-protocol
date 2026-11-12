// SPDX-License-Identifier: AGPL-3.0-or-later
//! Exact trigger/capture/anchor activation over QObs owner facts.

use qsl_package::protocol_view::{choice_visibility, ChoiceVisibility, ViewRefusal};
use quire_contract_model::{CheckedNodeId, CheckedSemanticNodeV2};
use quire_observation::authority::{completeness, observation, OpenClosed};

use crate::ids::{EffectId, RoleInstanceId, WorkflowInstanceId};

use super::purity::PureAssessment;

/// Owned choice-visibility evidence read from QSL's checked accessor
/// (QSpec FR-172-AC-6): the `choice` control, the node that owns the decision
/// and the exact visible set in carried order. Only the accessor's result
/// constructs it.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ChoiceVisibilityEvidence {
    choice: CheckedNodeId,
    owner: CheckedNodeId,
    visible: Box<[CheckedNodeId]>,
}

impl ChoiceVisibilityEvidence {
    /// Read the evidence of one checked `choice` control through
    /// `qsl_package::protocol_view::choice_visibility`.
    ///
    /// # Errors
    ///
    /// QSL's refusal when `node` is no well-formed `choice` control.
    pub fn read(node: &CheckedSemanticNodeV2) -> Result<Self, ViewRefusal> {
        choice_visibility(node).map(|checked| Self::from_checked(&checked))
    }

    /// Retain the identities of one checked accessor result unchanged.
    #[must_use]
    pub fn from_checked(checked: &ChoiceVisibility<'_>) -> Self {
        Self {
            choice: checked.choice.clone(),
            owner: checked.owner.clone(),
            visible: checked.visible.iter().map(|node| (*node).clone()).collect(),
        }
    }

    /// The `choice` control the evidence belongs to.
    #[must_use]
    pub const fn choice(&self) -> &CheckedNodeId {
        &self.choice
    }

    /// The `protocol` or `role` node that owns the decision.
    #[must_use]
    pub const fn owner(&self) -> &CheckedNodeId {
        &self.owner
    }

    /// The `expression` nodes the owner sees, in carried order.
    #[must_use]
    pub fn visible(&self) -> &[CheckedNodeId] {
        &self.visible
    }
}

/// Immutable activation-binding dependencies.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Binding {
    /// Exact workflow instance.
    pub workflow: WorkflowInstanceId,
    /// Exact role instance.
    pub role: RoleInstanceId,
    /// Exact paired effect.
    pub effect: EffectId,
    /// Exact checked predicate identity bytes.
    pub predicate: Box<[u8]>,
    /// Exact immutable capture identity bytes.
    pub capture: Box<[u8]>,
    /// Exact evaluation-anchor identity bytes.
    pub anchor: Box<[u8]>,
    /// Exact choice-visibility evidence (FR-479 activation binding).
    pub choice_visibility: ChoiceVisibilityEvidence,
    /// Exact expected QObs observation identity bytes.
    pub trigger_observation: Box<[u8]>,
    /// Exact trigger scope selected for closure authority.
    pub scope_identity: Box<str>,
    /// Exact trigger population selected for completeness authority.
    pub population_identity: Box<str>,
}

/// Identity axis whose offered binding differs from the exact selection.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub enum IdentityAxis {
    /// The workflow instance identity.
    Workflow,
    /// The role instance identity.
    Role,
    /// The paired effect identity.
    Effect,
    /// The checked predicate identity.
    Predicate,
    /// The immutable capture identity.
    Capture,
    /// The evaluation-anchor identity.
    Anchor,
    /// The choice-visibility evidence identity (choice, owner, visible set).
    ChoiceVisibility,
    /// The expected trigger observation identity.
    Trigger,
    /// The trigger scope selected for closure authority.
    ClosureScope,
    /// The trigger population selected for completeness authority.
    CompletenessPopulation,
}

/// Complete identity-substitution refusal.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Refusal {
    axes: Vec<IdentityAxis>,
}

impl Refusal {
    /// Every substituted or absent axis in declared order.
    #[must_use]
    pub fn axes(&self) -> &[IdentityAxis] {
        &self.axes
    }
}

impl core::fmt::Display for Refusal {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            formatter,
            "activation binding differs on {} axes",
            self.axes.len()
        )
    }
}

impl std::error::Error for Refusal {}

/// Admitted activation guard result.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum Guard {
    /// Guard admitted and true; assess a trigger instance.
    True,
    /// Guard admitted and false; create no instance and no refusal.
    False,
    /// Guard could not be established; activation is unknown.
    Unavailable,
}

/// Exact trigger evidence state.
#[derive(Clone, Copy, Debug)]
pub enum TriggerEvidence<'a> {
    /// Strict QObs trigger view is available.
    Available(&'a observation::View),
    /// No trigger observation was admitted.
    Absent,
    /// Required trigger evidence refused at its owner boundary.
    Refused,
}

/// Adopted activation vocabulary.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum Activation {
    /// The trigger fired; a trigger instance was assessed.
    Triggered,
    /// The trigger's guard was admitted and false; no instance was created.
    Inactive,
    /// The trigger's guard could not be established.
    ActivationUnknown,
}

/// Exact retained activation output.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Record {
    /// Selected binding, retained unchanged.
    pub binding: Binding,
    /// Independent activation value.
    pub activation: Activation,
    /// Exact trigger owner bytes, only when available.
    pub trigger_bytes: Option<Vec<u8>>,
}

/// Assess one activation without performing I/O or deriving truth.
pub fn assess(
    _pure: PureAssessment,
    selected: &Binding,
    offered: &Binding,
    guard: Guard,
    trigger: TriggerEvidence<'_>,
    closure: &quire_observation::authority::closure::View,
    completeness: &completeness::View,
) -> Result<Option<Record>, Refusal> {
    if guard == Guard::False {
        return Ok(None);
    }
    let axes = mismatches(selected, offered, trigger, closure, completeness);
    if !axes.is_empty() {
        return Err(Refusal { axes });
    }
    let activation = match (guard, trigger) {
        (Guard::True, TriggerEvidence::Available(_)) => Activation::Triggered,
        (Guard::True, TriggerEvidence::Absent)
            if closure.payload().state() == OpenClosed::Closed
                && completeness.payload().state() == completeness::State::Complete =>
        {
            Activation::Inactive
        }
        (Guard::True, TriggerEvidence::Absent | TriggerEvidence::Refused)
        | (Guard::Unavailable, TriggerEvidence::Available(_))
        | (Guard::Unavailable, TriggerEvidence::Absent | TriggerEvidence::Refused) => {
            Activation::ActivationUnknown
        }
        (Guard::False, _) => return Ok(None),
    };
    Ok(Some(Record {
        binding: selected.clone(),
        activation,
        trigger_bytes: match trigger {
            TriggerEvidence::Available(view) => Some(view.bytes().to_vec()),
            TriggerEvidence::Absent | TriggerEvidence::Refused => None,
        },
    }))
}

fn mismatches(
    selected: &Binding,
    offered: &Binding,
    trigger: TriggerEvidence<'_>,
    closure: &quire_observation::authority::closure::View,
    completeness: &completeness::View,
) -> Vec<IdentityAxis> {
    let mut axes = Vec::new();
    if selected.workflow.identity_bytes().is_empty() || selected.workflow != offered.workflow {
        axes.push(IdentityAxis::Workflow);
    }
    if selected.role.identity_bytes().is_empty() || selected.role != offered.role {
        axes.push(IdentityAxis::Role);
    }
    if selected.effect.identity_bytes().is_empty() || selected.effect != offered.effect {
        axes.push(IdentityAxis::Effect);
    }
    compare_bytes(
        &mut axes,
        IdentityAxis::Predicate,
        &selected.predicate,
        &offered.predicate,
    );
    compare_bytes(
        &mut axes,
        IdentityAxis::Capture,
        &selected.capture,
        &offered.capture,
    );
    compare_bytes(
        &mut axes,
        IdentityAxis::Anchor,
        &selected.anchor,
        &offered.anchor,
    );
    if selected.choice_visibility != offered.choice_visibility {
        axes.push(IdentityAxis::ChoiceVisibility);
    }
    let offered_trigger = match trigger {
        TriggerEvidence::Available(view) => Some(view.payload().observation_identity().as_bytes()),
        TriggerEvidence::Absent | TriggerEvidence::Refused => None,
    };
    if offered.trigger_observation != selected.trigger_observation
        || offered_trigger.is_some_and(|identity| identity != selected.trigger_observation.as_ref())
        || selected.trigger_observation.is_empty()
    {
        axes.push(IdentityAxis::Trigger);
    }
    if selected.scope_identity.is_empty()
        || selected.scope_identity != offered.scope_identity
        || closure.payload().scope_identity() != selected.scope_identity.as_ref()
    {
        axes.push(IdentityAxis::ClosureScope);
    }
    if selected.population_identity.is_empty()
        || selected.population_identity != offered.population_identity
        || completeness.payload().population_identity() != selected.population_identity.as_ref()
    {
        axes.push(IdentityAxis::CompletenessPopulation);
    }
    axes
}

fn compare_bytes(
    axes: &mut Vec<IdentityAxis>,
    axis: IdentityAxis,
    selected: &[u8],
    offered: &[u8],
) {
    if selected.is_empty() || selected != offered {
        axes.push(axis);
    }
}
