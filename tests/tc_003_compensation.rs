// SPDX-License-Identifier: AGPL-3.0-or-later
//! TC-003: pure activation, registration, ordering, retry, and recovery.

use std::cell::Cell;
use std::num::NonZeroU32;

use ix_trace_rs::trace;
use quire_protocol::bounds::inventory::{
    production_inventory, Budget, BudgetDimension, OperationOutcome, ResourceAccountant,
};
use quire_protocol::compensation::activate::{
    assess as assess_activation, Activation, Binding, Guard, IdentityAxis, TriggerEvidence,
};
use quire_protocol::compensation::algebra::{
    assess_ordering, AttemptAdmission, AttemptFact, AttemptLedger, AttemptOutcome, AuthoredOrder,
    CausalEdge, CompensationEffectFact, Definition, EffectDisposition, ForwardFact,
    Limits as AlgebraLimits, Milestone, MissingOrderingInput, OrderingViolation,
    RecoveryRelationOutcome, Registry, Resource as AlgebraResource,
};
use quire_protocol::compensation::purity::{Capability, CapabilityInventory, PureAssessment};
use quire_protocol::compensation::recovery::{
    Decision as RecoveryDecision, Fact as RecoveryFact, Limits as RecoveryLimits, MissingInput,
    State,
};
use quire_protocol::compensation::{
    compose, compose_bounded, Availability, Limits as CompositionLimits, Parts,
    Resource as CompositionResource, UnavailableCause,
};
use quire_protocol::ids::{
    AttemptId, CommitId, DefinitionId, DeliveryId, EffectId, ReceiveId, RecoveryId, RegistrationId,
    RoleInstanceId, SendId, WorkflowInstanceId,
};

#[path = "support/observation_owner.rs"]
#[allow(dead_code)]
mod observation_owner;

use observation_owner::{scope_views, ScopeViews};

#[derive(Default)]
struct Surface {
    capabilities: Vec<Capability>,
    business_calls: Cell<usize>,
    compensation_calls: Cell<usize>,
}

impl CapabilityInventory for Surface {
    fn capabilities(&self) -> &[Capability] {
        &self.capabilities
    }
}

impl Surface {
    fn business_operation(&self) {
        self.business_calls.set(self.business_calls.get() + 1);
    }

    fn compensation_operation(&self) {
        self.compensation_calls
            .set(self.compensation_calls.get() + 1);
    }
}

fn pure(surface: &Surface) -> PureAssessment {
    PureAssessment::checked(surface).expect("empty capability surface")
}

fn effect(value: &str) -> EffectId {
    EffectId::new(value.as_bytes(), value)
}

fn binding(views: &ScopeViews) -> Binding {
    Binding {
        workflow: WorkflowInstanceId::new(b"workflow:one", "workflow one"),
        role: RoleInstanceId::new(b"role:one", "role one"),
        effect: effect("effect:forward"),
        predicate: b"predicate:checked".to_vec().into_boxed_slice(),
        capture: b"capture:immutable".to_vec().into_boxed_slice(),
        anchor: b"anchor:exact".to_vec().into_boxed_slice(),
        trigger_observation: views
            .observation
            .payload()
            .observation_identity()
            .as_bytes()
            .to_vec()
            .into_boxed_slice(),
        scope_identity: views.closure_closed.payload().scope_identity().into(),
        population_identity: views.population.payload().population_identity().into(),
    }
}

fn registered(
    surface: &Surface,
    bound: Option<NonZeroU32>,
) -> quire_protocol::compensation::algebra::Registration {
    let forward = effect("effect:forward");
    let definition = Definition {
        identity: DefinitionId::new(b"definition:compensation", "compensation definition"),
        paired_forward_effect: forward.clone(),
        retry_bound: bound,
    };
    let mut registry = Registry::default();
    registry
        .observe(
            pure(surface),
            &[definition],
            &ForwardFact::EstablishedEffect(forward),
        )
        .expect("bounded registration")[0]
        .registration
        .clone()
}

/// Established, complete-absent, and unavailable trigger states remain exact;
/// false guards create no obligation instance and absence never defaults.
#[trace("TC-003", "FR-003-AC-1", "FR-019-AC-1", "FR-019-AC-3", "FR-019-AC-4")]
#[test]
fn tc_003_activation_is_independent_from_truth_and_absence() {
    let surface = Surface::default();
    let views = scope_views("activation");
    let selected = binding(&views);

    let triggered = assess_activation(
        pure(&surface),
        &selected,
        &selected,
        Guard::True,
        TriggerEvidence::Available(&views.observation),
        &views.closure_closed,
        &views.completeness_complete,
    )
    .expect("exact binding")
    .expect("active instance");
    assert_eq!(triggered.activation, Activation::Triggered);
    assert_eq!(
        triggered.trigger_bytes.as_deref(),
        Some(views.observation.bytes())
    );

    let inactive = assess_activation(
        pure(&surface),
        &selected,
        &selected,
        Guard::True,
        TriggerEvidence::Absent,
        &views.closure_closed,
        &views.completeness_complete,
    )
    .expect("exact binding")
    .expect("reported inactive instance");
    assert_eq!(inactive.activation, Activation::Inactive);
    assert!(inactive.trigger_bytes.is_none());

    for (closure, completeness, trigger) in [
        (
            &views.closure_open,
            &views.completeness_complete,
            TriggerEvidence::Absent,
        ),
        (
            &views.closure_closed,
            &views.completeness_incomplete,
            TriggerEvidence::Absent,
        ),
        (
            &views.closure_closed,
            &views.completeness_complete,
            TriggerEvidence::Refused,
        ),
    ] {
        let unknown = assess_activation(
            pure(&surface),
            &selected,
            &selected,
            Guard::True,
            trigger,
            closure,
            completeness,
        )
        .expect("exact binding")
        .expect("reported unknown instance");
        assert_eq!(unknown.activation, Activation::ActivationUnknown);
    }
    assert!(assess_activation(
        pure(&surface),
        &selected,
        &selected,
        Guard::False,
        TriggerEvidence::Absent,
        &views.closure_closed,
        &views.completeness_complete,
    )
    .expect("false guard is not a refusal")
    .is_none());
}

/// Every selected binding axis is independently authoritative.
#[trace("TC-003", "FR-019-AC-2", "FR-019-AC-5")]
#[test]
fn tc_003_activation_refuses_each_identity_substitution() {
    let surface = Surface::default();
    let views = scope_views("activation-axis");
    let selected = binding(&views);
    let mut mutations = Vec::new();
    for axis in [
        IdentityAxis::Workflow,
        IdentityAxis::Role,
        IdentityAxis::Effect,
        IdentityAxis::Predicate,
        IdentityAxis::Capture,
        IdentityAxis::Anchor,
        IdentityAxis::Trigger,
        IdentityAxis::ClosureScope,
        IdentityAxis::CompletenessPopulation,
    ] {
        let mut offered = selected.clone();
        match axis {
            IdentityAxis::Workflow => {
                offered.workflow = WorkflowInstanceId::new(b"workflow:other", "other")
            }
            IdentityAxis::Role => offered.role = RoleInstanceId::new(b"role:other", "other"),
            IdentityAxis::Effect => offered.effect = effect("effect:other"),
            IdentityAxis::Predicate => offered.predicate = Box::from(&b"predicate:other"[..]),
            IdentityAxis::Capture => offered.capture = Box::from(&b"capture:other"[..]),
            IdentityAxis::Anchor => offered.anchor = Box::from(&b"anchor:other"[..]),
            IdentityAxis::Trigger => {
                offered.trigger_observation = Box::from(&b"observation:other"[..])
            }
            IdentityAxis::ClosureScope => offered.scope_identity = "scope:other".into(),
            IdentityAxis::CompletenessPopulation => {
                offered.population_identity = "population:other".into()
            }
        }
        let refusal = assess_activation(
            pure(&surface),
            &selected,
            &offered,
            Guard::True,
            TriggerEvidence::Available(&views.observation),
            &views.closure_closed,
            &views.completeness_complete,
        )
        .expect_err("substituted binding must refuse");
        assert_eq!(refusal.axes(), &[axis]);
        mutations.push(axis);
    }
    assert_eq!(mutations.len(), 9);
    let other = scope_views("activation-cross-wire");
    let closure_refusal = assess_activation(
        pure(&surface),
        &selected,
        &selected,
        Guard::True,
        TriggerEvidence::Available(&views.observation),
        &other.closure_closed,
        &views.completeness_complete,
    )
    .expect_err("foreign closure scope must refuse");
    assert_eq!(closure_refusal.axes(), &[IdentityAxis::ClosureScope]);
    let completeness_refusal = assess_activation(
        pure(&surface),
        &selected,
        &selected,
        Guard::True,
        TriggerEvidence::Available(&views.observation),
        &views.closure_closed,
        &other.completeness_complete,
    )
    .expect_err("foreign completeness population must refuse");
    assert_eq!(
        completeness_refusal.axes(),
        &[IdentityAxis::CompletenessPopulation]
    );
    assert_eq!(surface.business_calls.get(), 0);
    assert_eq!(surface.compensation_calls.get(), 0);
}

/// Only an independently established effect registers, replay is idempotent,
/// exact keys are retained, and bounds are checked transactionally.
#[trace(
    "TC-003",
    "FR-003-AC-2",
    "FR-020-AC-1",
    "FR-020-AC-2",
    "FR-020-AC-3",
    "FR-020-AC-4",
    "FR-020-AC-5"
)]
#[test]
fn tc_003_registration_requires_effect_and_is_replay_idempotent() {
    let surface = Surface::default();
    let forward = effect("effect:forward");
    let definition = Definition {
        identity: DefinitionId::new(b"definition:one", "definition one"),
        paired_forward_effect: forward.clone(),
        retry_bound: NonZeroU32::new(2),
    };
    let mut registry = Registry::default();
    for fact in [
        ForwardFact::Send(SendId::new(b"send:one", "send")),
        ForwardFact::Receive(ReceiveId::new(b"receive:one", "receive")),
        ForwardFact::Delivery(DeliveryId::new(b"delivery:duplicate", "duplicate")),
        ForwardFact::Attempt(AttemptId::new(b"attempt:failed", "failed")),
    ] {
        assert!(registry
            .observe(pure(&surface), std::slice::from_ref(&definition), &fact)
            .expect("bounded non-effect")
            .is_empty());
    }

    let created = registry
        .observe(
            pure(&surface),
            std::slice::from_ref(&definition),
            &ForwardFact::EstablishedEffect(forward.clone()),
        )
        .expect("first effect");
    assert_eq!(created.len(), 1);
    assert!(created[0].created);
    assert_eq!(created[0].registration.definition, definition.identity);
    assert_eq!(created[0].registration.forward_effect, forward);

    let replayed = registry
        .observe(
            pure(&surface),
            std::slice::from_ref(&definition),
            &ForwardFact::EstablishedEffect(effect("effect:forward")),
        )
        .expect("replayed effect");
    assert_eq!(replayed.len(), 1);
    assert!(!replayed[0].created);
    assert_eq!(replayed[0].registration, created[0].registration);
    assert_eq!(registry.registrations().len(), 1);

    let exact = AlgebraLimits {
        max_definitions: 1,
        max_registrations: 1,
        ..AlgebraLimits::owner_max()
    };
    let mut bounded = Registry::new(exact);
    assert_eq!(
        bounded
            .observe(
                pure(&surface),
                std::slice::from_ref(&definition),
                &ForwardFact::EstablishedEffect(effect("effect:forward")),
            )
            .expect("exact limit")
            .len(),
        1
    );
    let refusal = bounded
        .observe(
            pure(&surface),
            &[definition.clone(), definition],
            &ForwardFact::EstablishedEffect(effect("effect:forward")),
        )
        .expect_err("one-over definition population");
    assert_eq!(refusal.resource(), AlgebraResource::Definitions);
    assert_eq!(bounded.registrations().len(), 1);
    assert_eq!(surface.compensation_calls.get(), 0);
}

fn order(nodes: Vec<Milestone>, pairs: &[(usize, usize)]) -> AuthoredOrder {
    let edges: Vec<_> = pairs
        .iter()
        .map(|(before, after)| CausalEdge {
            before: nodes[*before].clone(),
            after: nodes[*after].clone(),
        })
        .collect();
    AuthoredOrder::new(&nodes, &edges, AlgebraLimits::owner_max()).expect("valid authored DAG")
}

/// The three commit laws are distinct graph properties; observation/source/key
/// order is absent from the API, and a later effect cannot repair an early one.
#[trace(
    "TC-003",
    "FR-003-AC-3",
    "FR-021-AC-1",
    "FR-021-AC-2",
    "FR-021-AC-3",
    "FR-021-AC-4",
    "TC-008",
    "NFR-001-AC-3"
)]
#[test]
fn tc_003_commit_violations_follow_only_authored_causal_order() {
    let surface = Surface::default();
    let registration = registered(&surface, NonZeroU32::new(2));
    let compensation = effect("effect:compensation");
    let commit = CommitId::new(b"commit:one", "commit one");
    let nodes = vec![
        Milestone::ForwardEffect(registration.forward_effect.clone()),
        Milestone::Registration(registration.identity.clone()),
        Milestone::CompensationEffect(compensation.clone()),
        Milestone::Commit(commit.clone()),
    ];

    let after_commit = order(nodes.clone(), &[(0, 3), (3, 1), (0, 2)]);
    assert_eq!(
        assess_ordering(
            pure(&surface),
            Some(&registration),
            Some(&compensation),
            Some(&commit),
            &after_commit,
        )
        .violations(),
        &[OrderingViolation::RegistrationAfterCommit]
    );

    let early = order(nodes.clone(), &[(2, 0), (0, 1), (0, 3)]);
    let early_result = assess_ordering(
        pure(&surface),
        Some(&registration),
        Some(&compensation),
        Some(&commit),
        &early,
    );
    assert_eq!(
        early_result.violations(),
        &[OrderingViolation::CompensationBeforeForwardEffect]
    );
    assert!(early.precedes(&nodes[2], &nodes[0]));

    let late = order(nodes.clone(), &[(0, 1), (1, 3), (3, 2)]);
    assert_eq!(
        assess_ordering(
            pure(&surface),
            Some(&registration),
            Some(&compensation),
            Some(&commit),
            &late,
        )
        .violations(),
        &[OrderingViolation::CompensationAfterCommit]
    );

    let reversed_nodes: Vec<_> = nodes.iter().rev().cloned().collect();
    let reordered = AuthoredOrder::new(
        &reversed_nodes,
        &[
            CausalEdge {
                before: nodes[0].clone(),
                after: nodes[1].clone(),
            },
            CausalEdge {
                before: nodes[1].clone(),
                after: nodes[3].clone(),
            },
            CausalEdge {
                before: nodes[3].clone(),
                after: nodes[2].clone(),
            },
        ],
        AlgebraLimits::owner_max(),
    )
    .expect("same graph in different presentation order");
    assert_eq!(
        assess_ordering(
            pure(&surface),
            Some(&registration),
            Some(&compensation),
            Some(&commit),
            &reordered,
        ),
        assess_ordering(
            pure(&surface),
            Some(&registration),
            Some(&compensation),
            Some(&commit),
            &late,
        )
    );

    let missing_commit = assess_ordering(
        pure(&surface),
        Some(&registration),
        Some(&compensation),
        None,
        &late,
    );
    assert!(!missing_commit.complete());
    assert_eq!(missing_commit.missing(), &[MissingOrderingInput::Commit]);
}

/// The initial attempt counts, exact-bound admission is inclusive, one-over is
/// transactional, outcomes remain exact, and absence receives no default.
#[trace(
    "TC-003",
    "FR-003-AC-3",
    "FR-003-AC-4",
    "FR-022-AC-1",
    "FR-022-AC-2",
    "FR-022-AC-3",
    "FR-022-AC-4",
    "FR-022-AC-5",
    "FR-023-AC-1",
    "FR-023-AC-2",
    "FR-023-AC-3",
    "FR-023-AC-4",
    "FR-023-AC-5"
)]
#[test]
fn tc_003_attempts_effects_outcomes_and_retry_bound_stay_distinct() {
    let surface = Surface::default();
    let registration = registered(&surface, NonZeroU32::new(2));
    let effect_id = effect("effect:refund");
    let mut ledger =
        AttemptLedger::new(registration.clone(), AlgebraLimits::owner_max()).expect("ledger");
    let failed = AttemptFact {
        attempt: AttemptId::new(b"attempt:failed", "failed attempt"),
        registration: registration.identity.clone(),
        established_effect: None,
        outcome: Some(AttemptOutcome::Failed),
    };
    let successful = AttemptFact {
        attempt: AttemptId::new(b"attempt:successful", "successful attempt"),
        registration: registration.identity.clone(),
        established_effect: Some(effect_id.clone()),
        outcome: Some(AttemptOutcome::Successful),
    };
    assert!(matches!(
        ledger.admit(pure(&surface), failed.clone()),
        AttemptAdmission::Admitted { position: 1, .. }
    ));
    assert!(matches!(
        ledger.admit(pure(&surface), successful.clone()),
        AttemptAdmission::Admitted { position: 2, .. }
    ));
    assert_eq!(ledger.attempts(), &[failed, successful]);
    assert_ne!(
        ledger.attempts()[1].attempt.identity_bytes(),
        effect_id.identity_bytes()
    );

    let over = AttemptFact {
        attempt: AttemptId::new(b"attempt:over", "over-bound attempt"),
        registration: registration.identity.clone(),
        established_effect: Some(effect("effect:over")),
        outcome: Some(AttemptOutcome::Successful),
    };
    let before = ledger.attempts().to_vec();
    assert!(matches!(
        ledger.admit(pure(&surface), over),
        AttemptAdmission::RetryBoundExceeded { bound, .. } if bound.get() == 2
    ));
    assert_eq!(ledger.attempts(), before);

    let attempt_labels: Vec<_> = [
        AttemptOutcome::Failed,
        AttemptOutcome::TimedOut,
        AttemptOutcome::Pending,
        AttemptOutcome::Successful,
    ]
    .into_iter()
    .map(AttemptOutcome::label)
    .collect();
    assert_eq!(
        attempt_labels,
        ["failed", "timed-out", "pending", "successful"]
    );
    assert_eq!(EffectDisposition::Duplicate.label(), "duplicate");
    let recovery_labels: Vec<_> = [
        RecoveryRelationOutcome::Failed,
        RecoveryRelationOutcome::TimedOut,
        RecoveryRelationOutcome::Pending,
        RecoveryRelationOutcome::Successful,
    ]
    .into_iter()
    .map(RecoveryRelationOutcome::label)
    .collect();
    assert_eq!(
        recovery_labels,
        ["failed", "timed-out", "pending", "successful"]
    );
    let absent = AttemptFact {
        attempt: AttemptId::new(b"attempt:unknown", "unknown outcome"),
        registration: registration.identity.clone(),
        established_effect: None,
        outcome: None,
    };
    assert!(absent.outcome.is_none());

    let absent_registration = registered(&surface, None);
    let mut absent_ledger =
        AttemptLedger::new(absent_registration.clone(), AlgebraLimits::owner_max())
            .expect("ledger");
    assert!(matches!(
        absent_ledger.admit(
            pure(&surface),
            AttemptFact {
                attempt: AttemptId::new(b"attempt:no-bound", "no bound"),
                registration: absent_registration.identity,
                established_effect: None,
                outcome: None,
            }
        ),
        AttemptAdmission::RetryBoundAbsent { .. }
    ));
    assert!(absent_ledger.attempts().is_empty());
}

fn state(identity: &str, value: &str) -> State {
    State::new(
        identity.as_bytes().to_vec().into_boxed_slice(),
        value.as_bytes().to_vec().into_boxed_slice(),
        RecoveryLimits::owner_max(),
    )
    .expect("bounded state")
}

fn recovery_fact(
    recovery: &RecoveryId,
    registration: &quire_protocol::compensation::algebra::Registration,
    commit: &CommitId,
    compensation_effect: &EffectId,
    target: &State,
    achieved: &State,
    relation_type: &[u8],
    relation_bytes: &[u8],
) -> RecoveryFact {
    RecoveryFact {
        recovery: recovery.clone(),
        registration: registration.identity.clone(),
        commit: commit.clone(),
        compensation_effect: compensation_effect.clone(),
        target: target.clone(),
        achieved: achieved.clone(),
        relation_type: relation_type.to_vec().into_boxed_slice(),
        relation_bytes: relation_bytes.to_vec().into_boxed_slice(),
        population_bytes: None,
        completeness_bytes: None,
    }
}

/// The facade retains atomic facts side by side and never upgrades a missing,
/// incomplete, or refused child into aggregate availability.
#[trace(
    "TC-003",
    "TC-009",
    "FR-003-AC-1",
    "FR-003-AC-2",
    "FR-003-AC-3",
    "FR-003-AC-4",
    "FR-003-AC-5",
    "FR-003-AC-6",
    "FR-003-AC-7",
    "FR-016-AC-2",
    "FR-016-AC-3",
    "FR-016-AC-4",
    "NFR-002-AC-1",
    "NFR-002-AC-4"
)]
#[test]
fn tc_003_aggregate_composes_every_atomic_fact_without_inference() {
    let surface = Surface::default();
    let views = scope_views("aggregate");
    let selected = binding(&views);
    let activation = assess_activation(
        pure(&surface),
        &selected,
        &selected,
        Guard::True,
        TriggerEvidence::Available(&views.observation),
        &views.closure_closed,
        &views.completeness_complete,
    )
    .expect("exact binding")
    .expect("triggered activation");
    let registration = registered(&surface, NonZeroU32::new(2));
    let compensation = effect("effect:aggregate-compensation");
    let commit = CommitId::new(b"commit:aggregate", "aggregate commit");
    let nodes = vec![
        Milestone::ForwardEffect(registration.forward_effect.clone()),
        Milestone::Registration(registration.identity.clone()),
        Milestone::CompensationEffect(compensation.clone()),
        Milestone::Commit(commit.clone()),
    ];
    let authored = order(nodes, &[(0, 1), (1, 2), (2, 3)]);
    let ordering = assess_ordering(
        pure(&surface),
        Some(&registration),
        Some(&compensation),
        Some(&commit),
        &authored,
    );
    assert!(ordering.complete());
    assert!(ordering.violations().is_empty());

    let attempt = AttemptFact {
        attempt: AttemptId::new(b"attempt:aggregate", "aggregate attempt"),
        registration: registration.identity.clone(),
        established_effect: Some(compensation.clone()),
        outcome: Some(AttemptOutcome::Successful),
    };
    let mut ledger =
        AttemptLedger::new(registration.clone(), AlgebraLimits::owner_max()).expect("ledger");
    let retry = ledger.admit(pure(&surface), attempt);
    let target = state("state:aggregate-target", "100");
    let achieved = state("state:aggregate-achieved", "40");
    let recovery = RecoveryId::new(b"recovery:aggregate-pair", "aggregate recovery pair");
    let recoveries = vec![
        RecoveryDecision::NotSatisfied(recovery_fact(
            &recovery,
            &registration,
            &commit,
            &compensation,
            &target,
            &achieved,
            b"relation:full",
            b"target == achieved",
        )),
        RecoveryDecision::Satisfied(recovery_fact(
            &recovery,
            &registration,
            &commit,
            &compensation,
            &target,
            &achieved,
            b"relation:partial",
            b"achieved == 40",
        )),
    ];
    let retry_outcomes = [retry];
    let effects = [CompensationEffectFact {
        effect: compensation.clone(),
        registration: registration.identity.clone(),
        disposition: None,
    }];
    let parts = Parts {
        activation: Some(&activation),
        registration: Some(&registration),
        ordering: Some(&ordering),
        attempts: ledger.attempts(),
        effects: &effects,
        retry_outcomes: &retry_outcomes,
        recoveries: &recoveries,
    };
    let snapshot =
        compose(pure(&surface), parts, CompositionLimits::owner_max()).expect("bounded aggregate");
    assert_eq!(snapshot.availability, Availability::Available);
    assert_eq!(snapshot.attempts, ledger.attempts());
    assert_eq!(snapshot.effects, effects);
    assert!(matches!(
        snapshot.recoveries.as_slice(),
        [
            RecoveryDecision::NotSatisfied(_),
            RecoveryDecision::Satisfied(_)
        ]
    ));

    let incomplete_recovery = [RecoveryDecision::Incomplete(vec![
        MissingInput::CompletenessAuthority,
    ])];
    let unavailable = compose(
        pure(&surface),
        Parts {
            recoveries: &incomplete_recovery,
            ..parts
        },
        CompositionLimits::owner_max(),
    )
    .expect("incomplete is retained, not a resource refusal");
    assert_eq!(
        unavailable.availability,
        Availability::Unavailable(vec![UnavailableCause::RecoveryIncomplete])
    );

    let missing_activation = compose(
        pure(&surface),
        Parts {
            activation: None,
            ..parts
        },
        CompositionLimits::owner_max(),
    )
    .expect("absence is retained");
    assert_eq!(
        missing_activation.availability,
        Availability::Unavailable(vec![UnavailableCause::ActivationAbsent])
    );

    let one_under = CompositionLimits {
        max_attempts: 0,
        ..CompositionLimits::owner_max()
    };
    let refusal = compose(pure(&surface), parts, one_under)
        .expect_err("attempt population must be prechecked before clone");
    assert_eq!(refusal.resource(), CompositionResource::Attempts);
    assert_eq!(refusal.observed(), 1);
    assert_eq!(refusal.limit(), 0);
    assert_eq!(surface.business_calls.get(), 0);
    assert_eq!(surface.compensation_calls.get(), 0);

    let inventory = production_inventory().expect("reconciled production inventory");
    let mut owner_resources = ResourceAccountant::new(Budget::nothing_granted(), &inventory);
    let owner_incomplete = compose_bounded(pure(&surface), parts, one_under, &mut owner_resources);
    assert!(matches!(
        owner_incomplete,
        OperationOutcome::ResourceIncomplete(value)
            if value.dimension() == BudgetDimension::ResultItem
                && value.granted() == 0
                && value.consumed() == 0
                && value.requested() == 1
    ));
    assert_eq!(
        owner_resources
            .usage()
            .consumed(BudgetDimension::ResultItem),
        0
    );

    let one_short = Budget::nothing_granted()
        .grant(BudgetDimension::ResultItem, 6)
        .expect("positive result-item grant");
    let mut one_short_resources = ResourceAccountant::new(one_short, &inventory);
    let incomplete = compose_bounded(
        pure(&surface),
        parts,
        CompositionLimits::owner_max(),
        &mut one_short_resources,
    );
    assert!(matches!(
        incomplete,
        OperationOutcome::ResourceIncomplete(value)
            if value.dimension() == BudgetDimension::ResultItem
                && value.consumed() == 6
                && value.requested() == 2
    ));
    assert_eq!(
        one_short_resources
            .usage()
            .consumed(BudgetDimension::ResultItem),
        0
    );

    let exact = Budget::nothing_granted()
        .grant(BudgetDimension::ResultItem, 8)
        .expect("positive result-item grant");
    let mut exact_resources = ResourceAccountant::new(exact, &inventory);
    let bounded = compose_bounded(
        pure(&surface),
        parts,
        CompositionLimits::owner_max(),
        &mut exact_resources,
    );
    assert!(matches!(bounded, OperationOutcome::Completed(_)));
    assert_eq!(
        exact_resources
            .usage()
            .consumed(BudgetDimension::ResultItem),
        8
    );

    let one_allocation = Budget::nothing_granted()
        .grant(BudgetDimension::AccountedAllocation, 1)
        .expect("one compensation snapshot allocation");
    let mut allocation_resources = ResourceAccountant::new(one_allocation, &inventory);
    assert!(matches!(
        compose_bounded(
            pure(&surface),
            parts,
            CompositionLimits::owner_max(),
            &mut allocation_resources,
        ),
        OperationOutcome::Completed(_)
    ));
    assert!(matches!(
        compose_bounded(
            pure(&surface),
            parts,
            CompositionLimits::owner_max(),
            &mut allocation_resources,
        ),
        OperationOutcome::ResourceIncomplete(value)
            if value.dimension() == BudgetDimension::AccountedAllocation
                && value.consumed() == 1
                && value.requested() == 1
    ));
    assert_eq!(surface.business_calls.get(), 0);
    assert_eq!(surface.compensation_calls.get(), 0);
}

/// The capability witness is constructed before assessment; every API and
/// dependency mutation is rejected by name and action-capable doubles stay idle.
#[trace("TC-003", "FR-003-AC-7", "FR-025-AC-1", "FR-025-AC-2", "FR-025-AC-3")]
#[test]
fn tc_003_purity_boundary_rejects_every_capability_before_assessment() {
    for capability in [
        Capability::Network,
        Capability::Storage,
        Capability::Telemetry,
        Capability::BusinessOperation,
        Capability::CompensationOperation,
    ] {
        let mutated_api = Surface {
            capabilities: vec![capability],
            ..Surface::default()
        };
        let refusal = PureAssessment::checked(&mutated_api)
            .expect_err("mutated API surface must refuse before assessment");
        assert_eq!(refusal.capability(), capability);
        assert_eq!(mutated_api.business_calls.get(), 0);
        assert_eq!(mutated_api.compensation_calls.get(), 0);

        let mutated_dependency = Surface {
            capabilities: vec![capability],
            ..Surface::default()
        };
        let refusal = PureAssessment::checked(&mutated_dependency)
            .expect_err("mutated dependency inventory must refuse before assessment");
        assert_eq!(refusal.capability(), capability);
        assert_eq!(mutated_dependency.business_calls.get(), 0);
        assert_eq!(mutated_dependency.compensation_calls.get(), 0);
    }

    let clean = Surface::default();
    let views = scope_views("purity");
    let selected = binding(&views);
    let _ = assess_activation(
        pure(&clean),
        &selected,
        &selected,
        Guard::True,
        TriggerEvidence::Available(&views.observation),
        &views.closure_closed,
        &views.completeness_complete,
    )
    .expect("pure activation");
    assert_eq!(clean.business_calls.get(), 0);
    assert_eq!(clean.compensation_calls.get(), 0);

    // The action methods exist and are callable by the harness, proving that a
    // non-zero assertion would detect accidental invocation.
    clean.business_operation();
    clean.compensation_operation();
    assert_eq!(clean.business_calls.get(), 1);
    assert_eq!(clean.compensation_calls.get(), 1);
}

/// Identity types remain distinct at the API boundary.
#[trace("TC-003", "FR-003-AC-4", "FR-023-AC-3")]
#[test]
fn tc_003_registration_identity_is_not_an_attempt_or_effect_identity() {
    let registration = RegistrationId::new(b"registration:one", "registration");
    let attempt = AttemptId::new(b"registration:one", "same display bytes");
    let effect = EffectId::new(b"registration:one", "same display bytes");
    assert_eq!(registration.identity_bytes(), attempt.identity_bytes());
    assert_eq!(attempt.identity_bytes(), effect.identity_bytes());
}
