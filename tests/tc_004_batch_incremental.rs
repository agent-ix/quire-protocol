// SPDX-License-Identifier: AGPL-3.0-or-later
//! TC-004: bounded batch and genuinely incremental finite conformance.

use ix_trace_rs::trace;
use quire_protocol::assessment::{AxisInputs, GlobalConformanceInput, GlobalPremise, PremiseKind};
use quire_protocol::bounds::inventory::{
    production_inventory, Budget, BudgetDimension, OperationOutcome, ResourceAccountant,
};
use quire_protocol::closure::{AssessmentExecution, GlobalConformanceClosure};
use quire_protocol::conformance::{
    assess, assess_bounded, compare, AdverseKind, BoundObservation, ErrorCode, GlobalDisposition,
    GlobalPremiseDefectKind, IncompleteCause, Incremental, IncrementalContext, Limits, Model,
    ObligationDisposition, ObligationId, ObservationObligation, Request,
};
use quire_protocol::refusal::ProtocolStage;

#[path = "support/observation_owner.rs"]
#[allow(dead_code)]
mod observation_owner;

use observation_owner::{scope_views, scope_views_with_value, ScopeViews};

fn oid(value: &str) -> ObligationId {
    ObligationId::retained(value)
}

fn obligation(
    identity: &str,
    expected: &quire_observation::authority::observation::View,
    effect: &str,
    predecessors: Vec<ObligationId>,
) -> ObservationObligation {
    ObservationObligation {
        identity: oid(identity),
        expected_observation_identity: expected.payload().observation_identity().into(),
        subject_kind: expected.payload().subject().kind.into(),
        subject_identity: expected.payload().subject().identity.into(),
        workflow_identity: "workflow:one".into(),
        effect_identity: Some(effect.into()),
        predecessors,
        valuation_required: true,
    }
}

fn binding<'a>(
    obligation: &str,
    observation: &'a quire_observation::authority::observation::View,
    workflow: &str,
    effect: &str,
) -> BoundObservation<'a> {
    BoundObservation {
        obligation: oid(obligation),
        observation,
        workflow_identity: workflow.into(),
        effect_identity: Some(effect.into()),
    }
}

fn premises(state: GlobalConformanceClosure) -> GlobalConformanceInput {
    let premises = vec![
        GlobalPremise {
            kind: PremiseKind::Execution,
            identity: "premise:execution".into(),
            scope_identity: "scope:execution".into(),
            authority_identity: "authority:execution".into(),
        },
        GlobalPremise {
            kind: PremiseKind::Branch,
            identity: "premise:branch".into(),
            scope_identity: "scope:branch".into(),
            authority_identity: "authority:branch".into(),
        },
        GlobalPremise {
            kind: PremiseKind::Workflow,
            identity: "premise:workflow".into(),
            scope_identity: "scope:workflow".into(),
            authority_identity: "authority:workflow".into(),
        },
    ];
    GlobalConformanceInput {
        state,
        required_premises: premises.clone(),
        premises,
    }
}

fn axes<'a>(decision: &'a ScopeViews, surrounding: &'a ScopeViews, closed: bool) -> AxisInputs<'a> {
    if closed {
        AxisInputs {
            decision_scope_progress: &decision.progress_closed,
            decision_scope_closure: &decision.closure_closed,
            surrounding_execution_progress: &surrounding.progress_closed,
            surrounding_execution_closure: &surrounding.closure_closed,
        }
    } else {
        AxisInputs {
            decision_scope_progress: &decision.progress_open,
            decision_scope_closure: &decision.closure_open,
            surrounding_execution_progress: &surrounding.progress_open,
            surrounding_execution_closure: &surrounding.closure_open,
        }
    }
}

fn request<'a>(
    model: &'a Model,
    observations: &'a [BoundObservation<'a>],
    decision: &'a ScopeViews,
    surrounding: &'a ScopeViews,
    global: &'a GlobalConformanceInput,
    closed: bool,
) -> Request<'a> {
    Request {
        model,
        observations,
        axes: axes(decision, surrounding, closed),
        completeness: if closed {
            &decision.completeness_complete
        } else {
            &decision.completeness_incomplete
        },
        global_conformance: global,
        execution: AssessmentExecution::Completed,
    }
}

/// The first obligation settles at its earliest deciding prefix, and the final
/// incremental report agrees exactly with batch assessment.
#[trace("TC-004", "FR-004-AC-4", "FR-041-AC-1", "FR-041-AC-2")]
#[test]
fn tc_004_incremental_settles_before_final_fact_and_agrees_with_batch() {
    let decision = scope_views("decision-incremental");
    let surrounding = scope_views("surrounding-incremental");
    let first = obligation(
        "obligation:first",
        &decision.observation,
        "effect:first",
        vec![],
    );
    let second = obligation(
        "obligation:second",
        &decision.observation,
        "effect:second",
        vec![first.identity.clone()],
    );
    let model = Model::new(
        "protocol:incremental",
        vec![first, second],
        Limits::default(),
    )
    .expect("valid model");
    let first_binding = binding(
        "obligation:first",
        &decision.observation,
        "workflow:one",
        "effect:first",
    );
    let second_binding = binding(
        "obligation:second",
        &decision.observation,
        "workflow:one",
        "effect:second",
    );
    let open = premises(GlobalConformanceClosure::Open);
    let closed = premises(GlobalConformanceClosure::Closed);
    let mut incremental = Incremental::new(model.clone());
    let prefix = incremental
        .push(
            first_binding.clone(),
            axes(&decision, &surrounding, false),
            &decision.completeness_incomplete,
            &open,
            AssessmentExecution::Completed,
            Limits::default(),
        )
        .expect("first prefix");
    assert_eq!(
        prefix.obligations[0].disposition,
        ObligationDisposition::Conforms
    );
    assert_eq!(
        prefix.obligations[1].disposition,
        ObligationDisposition::Pending
    );

    let final_report = incremental
        .push(
            second_binding.clone(),
            axes(&decision, &surrounding, true),
            &decision.completeness_complete,
            &closed,
            AssessmentExecution::Completed,
            Limits::default(),
        )
        .expect("final prefix")
        .clone();
    let bindings = [first_binding, second_binding];
    let batch = assess(
        request(&model, &bindings, &decision, &surrounding, &closed, true),
        Limits::default(),
    )
    .expect("batch report");
    assert_eq!(final_report.global, GlobalDisposition::Conforms);
    compare(&batch, &final_report).expect("batch/incremental agreement");
}

/// Independent obligations admit every presentation permutation; the exact
/// retained observation order changes, while semantic values do not.
#[trace(
    "TC-004",
    "TC-008",
    "FR-004-AC-1",
    "FR-026-AC-1",
    "FR-026-AC-3",
    "NFR-001-AC-1"
)]
#[test]
fn tc_004_permitted_presentation_orders_are_semantically_invariant() {
    let decision = scope_views("decision-order");
    let surrounding = scope_views("surrounding-order");
    let model = Model::new(
        "protocol:order",
        vec![
            obligation("obligation:a", &decision.observation, "effect:a", vec![]),
            obligation("obligation:b", &decision.observation, "effect:b", vec![]),
        ],
        Limits::default(),
    )
    .expect("valid model");
    let a = binding(
        "obligation:a",
        &decision.observation,
        "workflow:one",
        "effect:a",
    );
    let b = binding(
        "obligation:b",
        &decision.observation,
        "workflow:one",
        "effect:b",
    );
    let global = premises(GlobalConformanceClosure::Closed);
    let left_bindings = [a.clone(), b.clone()];
    let right_bindings = [b, a];
    let left = assess(
        request(
            &model,
            &left_bindings,
            &decision,
            &surrounding,
            &global,
            true,
        ),
        Limits::default(),
    )
    .expect("left order");
    let right = assess(
        request(
            &model,
            &right_bindings,
            &decision,
            &surrounding,
            &global,
            true,
        ),
        Limits::default(),
    )
    .expect("right order");
    assert_ne!(left.observations, right.observations);
    compare(&left, &right).expect("same semantic values");
}

/// Every adverse class is retained distinctly and the admitted owner bytes are
/// neither removed nor rewritten.
#[trace("TC-004", "FR-004-AC-2", "FR-027-AC-1", "FR-027-AC-2", "FR-027-AC-5")]
#[test]
fn tc_004_reports_all_adverse_kinds_without_repair() {
    let decision = scope_views("decision-adverse");
    let surrounding = scope_views("surrounding-adverse");
    let first = obligation(
        "obligation:first",
        &decision.observation,
        "effect:shared",
        vec![],
    );
    let second = obligation(
        "obligation:second",
        &surrounding.observation,
        "effect:shared",
        vec![first.identity.clone()],
    );
    let missing = obligation(
        "obligation:missing",
        &decision.observation,
        "effect:missing",
        vec![],
    );
    let model = Model::new(
        "protocol:adverse",
        vec![first, second, missing],
        Limits::default(),
    )
    .expect("valid model");
    let bindings = [
        binding(
            "obligation:second",
            &surrounding.observation,
            "workflow:one",
            "effect:shared",
        ),
        binding(
            "obligation:first",
            &surrounding.observation,
            "workflow:other",
            "effect:shared",
        ),
    ];
    let global = premises(GlobalConformanceClosure::Closed);
    let report = assess(
        request(&model, &bindings, &decision, &surrounding, &global, true),
        Limits::default(),
    )
    .expect("adverse report");
    let kinds: std::collections::BTreeSet<_> =
        report.adverse.iter().map(|adverse| adverse.kind).collect();
    assert_eq!(
        kinds,
        [
            AdverseKind::Unexpected,
            AdverseKind::Missing,
            AdverseKind::CausallyMisordered,
            AdverseKind::DuplicateEffect,
            AdverseKind::CrossWorkflow,
        ]
        .into_iter()
        .collect()
    );
    assert_eq!(report.observations.len(), bindings.len());
    assert_eq!(
        report.observations[0].document.bytes,
        bindings[0].observation.bytes()
    );
    assert_eq!(
        report.observations[1].document.bytes,
        bindings[1].observation.bytes()
    );
}

/// Missing valuation is an explicit incomplete outcome, never a false or
/// violation result.
#[trace("TC-004", "FR-027-AC-4")]
#[test]
fn tc_004_missing_valuation_is_incomplete_not_false() {
    let decision = scope_views_with_value("decision-missing-value", false);
    let surrounding = scope_views("surrounding-missing-value");
    let model = Model::new(
        "protocol:missing-value",
        vec![obligation(
            "obligation:value",
            &decision.observation,
            "effect:value",
            vec![],
        )],
        Limits::default(),
    )
    .expect("valid model");
    let bindings = [binding(
        "obligation:value",
        &decision.observation,
        "workflow:one",
        "effect:value",
    )];
    let global = premises(GlobalConformanceClosure::Closed);
    let report = assess(
        request(&model, &bindings, &decision, &surrounding, &global, true),
        Limits::default(),
    )
    .expect("incomplete report");
    assert_eq!(
        report.obligations[0].disposition,
        ObligationDisposition::Incomplete
    );
    assert!(matches!(
        report.incomplete.as_slice(),
        [IncompleteCause::MissingValuation { .. }]
    ));
    assert_eq!(report.global, GlobalDisposition::Unavailable);
}

/// Resource ceilings discriminate exact/one-over requests, and path divergence
/// has a stable refusal axis and its own identity.
#[trace("TC-004", "FR-041-AC-3", "FR-041-AC-4", "NFR-002-AC-2")]
#[test]
fn tc_004_bounds_and_path_divergence_are_typed() {
    let decision = scope_views("decision-refusal");
    let surrounding = scope_views("surrounding-refusal");
    let model = Model::new(
        "protocol:refusal",
        vec![obligation(
            "obligation:one",
            &decision.observation,
            "effect:one",
            vec![],
        )],
        Limits::default(),
    )
    .expect("valid model");
    let bindings = [binding(
        "obligation:one",
        &decision.observation,
        "workflow:one",
        "effect:one",
    )];
    let global = premises(GlobalConformanceClosure::Closed);
    let request = || request(&model, &bindings, &decision, &surrounding, &global, true);
    let report = assess(request(), Limits::default()).expect("at owner maximum");
    let too_small = Limits {
        max_bindings: 0,
        ..Limits::default()
    };
    let resource = assess(request(), too_small).expect_err("one binding over caller limit");
    assert_eq!(resource.code(), ErrorCode::ResourceIncomplete);

    let mut divergent = report.clone();
    divergent.global = GlobalDisposition::Pending;
    let divergence = compare(&report, &divergent).expect_err("different value");
    assert_eq!(divergence.code(), ErrorCode::PathDivergence);
    assert_ne!(divergence.outcome_identity(), resource.outcome_identity());
}

#[trace("TC-004", "TC-009", "NFR-002-AC-1", "FR-016-AC-2", "FR-016-AC-4")]
#[test]
fn tc_009_batch_charges_observations_before_assessment() {
    let decision = scope_views("decision-caller-budget");
    let surrounding = scope_views("surrounding-caller-budget");
    let model = Model::new(
        "protocol:caller-budget",
        vec![
            obligation(
                "obligation:one",
                &decision.observation,
                "effect:one",
                vec![],
            ),
            obligation(
                "obligation:two",
                &decision.observation,
                "effect:two",
                vec![oid("obligation:one")],
            ),
        ],
        Limits::default(),
    )
    .expect("valid model");
    let bindings = [
        binding(
            "obligation:one",
            &decision.observation,
            "workflow:one",
            "effect:one",
        ),
        binding(
            "obligation:one",
            &decision.observation,
            "workflow:one",
            "effect:one",
        ),
    ];
    let global = premises(GlobalConformanceClosure::Closed);
    let request = || request(&model, &bindings, &decision, &surrounding, &global, true);
    let inventory = production_inventory().expect("production inventory");

    let one_short = Budget::nothing_granted()
        .grant(BudgetDimension::Observation, 1)
        .expect("positive one-short grant");
    let mut one_short_resources = ResourceAccountant::new(one_short, &inventory);
    let incomplete = assess_bounded(request(), Limits::default(), &mut one_short_resources);
    assert!(matches!(
        incomplete,
        OperationOutcome::ResourceIncomplete(value)
            if value.dimension() == BudgetDimension::Observation
                && value.consumed() == 0
                && value.requested() == 2
    ));
    assert_eq!(
        one_short_resources
            .usage()
            .consumed(BudgetDimension::Observation),
        0
    );

    let owner_short = Limits {
        max_bindings: 1,
        ..Limits::default()
    };
    let mut owner_resources = ResourceAccountant::new(Budget::nothing_granted(), &inventory);
    assert!(matches!(
        assess_bounded(request(), owner_short, &mut owner_resources),
        OperationOutcome::ResourceIncomplete(value)
            if value.dimension() == BudgetDimension::Observation
                && value.granted() == 1
                && value.requested() == 2
    ));
    assert_eq!(
        owner_resources
            .usage()
            .consumed(BudgetDimension::Observation),
        0
    );

    let exact = Budget::nothing_granted()
        .grant(BudgetDimension::Observation, 2)
        .expect("positive exact grant");
    let mut exact_resources = ResourceAccountant::new(exact, &inventory);
    assert!(matches!(
        assess_bounded(request(), Limits::default(), &mut exact_resources),
        OperationOutcome::Completed(_)
    ));
    assert_eq!(
        exact_resources
            .usage()
            .consumed(BudgetDimension::Observation),
        2
    );

    let one_short = Budget::nothing_granted()
        .grant(BudgetDimension::Operation, 1)
        .expect("positive one-short grant");
    let mut one_short_resources = ResourceAccountant::new(one_short, &inventory);
    assert!(matches!(
        assess_bounded(
            request(),
            Limits::default(),
            &mut one_short_resources
        ),
        OperationOutcome::ResourceIncomplete(value)
            if value.dimension() == BudgetDimension::Operation
                && value.requested() == 2
    ));
    assert_eq!(
        one_short_resources
            .usage()
            .consumed(BudgetDimension::Observation),
        0
    );
    assert_eq!(
        one_short_resources
            .usage()
            .consumed(BudgetDimension::Operation),
        0
    );

    let predecessor_short = Budget::nothing_granted()
        .grant(BudgetDimension::Operation, 2)
        .expect("positive predecessor one-short grant");
    let mut predecessor_resources = ResourceAccountant::new(predecessor_short, &inventory);
    assert!(matches!(
        assess_bounded(request(), Limits::default(), &mut predecessor_resources),
        OperationOutcome::ResourceIncomplete(value)
            if value.dimension() == BudgetDimension::Operation
                && value.consumed() == 2
                && value.requested() == 1
    ));

    let exact = Budget::nothing_granted()
        .grant(BudgetDimension::Operation, 3)
        .expect("positive exact grant");
    let mut exact_resources = ResourceAccountant::new(exact, &inventory);
    assert!(matches!(
        assess_bounded(request(), Limits::default(), &mut exact_resources),
        OperationOutcome::Completed(_)
    ));
    assert_eq!(
        exact_resources.usage().consumed(BudgetDimension::Operation),
        3
    );

    let retained_bytes = bindings
        .iter()
        .map(|binding| binding.observation.bytes().len())
        .chain([
            decision.progress_closed.bytes().len(),
            decision.closure_closed.bytes().len(),
            surrounding.progress_closed.bytes().len(),
            surrounding.closure_closed.bytes().len(),
            decision.completeness_complete.bytes().len(),
        ])
        .sum::<usize>();
    let retained_short = Budget::nothing_granted()
        .grant(BudgetDimension::InputByte, retained_bytes - 1)
        .expect("positive retained-byte one-short grant");
    let mut retained_resources = ResourceAccountant::new(retained_short, &inventory);
    assert!(matches!(
        assess_bounded(request(), Limits::default(), &mut retained_resources),
        OperationOutcome::ResourceIncomplete(value)
            if value.dimension() == BudgetDimension::InputByte
                && value.requested() == retained_bytes
    ));

    let retained_exact = Budget::nothing_granted()
        .grant(BudgetDimension::InputByte, retained_bytes)
        .expect("positive retained-byte exact grant");
    let mut retained_resources = ResourceAccountant::new(retained_exact, &inventory);
    assert!(matches!(
        assess_bounded(request(), Limits::default(), &mut retained_resources),
        OperationOutcome::Completed(_)
    ));
    assert_eq!(
        retained_resources
            .usage()
            .consumed(BudgetDimension::InputByte),
        retained_bytes
    );

    let adverse_short = Budget::nothing_granted()
        .grant(BudgetDimension::ResultItem, 2)
        .expect("positive one-short adverse grant");
    let mut adverse_resources = ResourceAccountant::new(adverse_short, &inventory);
    assert!(matches!(
        assess_bounded(request(), Limits::default(), &mut adverse_resources),
        OperationOutcome::ResourceIncomplete(value)
            if value.dimension() == BudgetDimension::ResultItem
                && value.consumed() == 2
                && value.requested() == 1
    ));

    let adverse_exact = Budget::nothing_granted()
        .grant(BudgetDimension::ResultItem, 3)
        .expect("positive exact adverse grant");
    let mut adverse_resources = ResourceAccountant::new(adverse_exact, &inventory);
    let OperationOutcome::Completed(adverse_report) =
        assess_bounded(request(), Limits::default(), &mut adverse_resources)
    else {
        panic!("exact adverse budget must complete");
    };
    assert_eq!(adverse_report.adverse.len(), 3);

    let adverse_owner_short = Limits {
        max_adverse: 2,
        ..Limits::default()
    };
    let mut owner_resources = ResourceAccountant::new(Budget::nothing_granted(), &inventory);
    assert!(matches!(
        assess_bounded(request(), adverse_owner_short, &mut owner_resources),
        OperationOutcome::ResourceIncomplete(value)
            if value.dimension() == BudgetDimension::ResultItem
                && value.granted() == 2
                && value.consumed() == 2
                && value.requested() == 1
    ));

    let predecessor_owner_short = Limits {
        max_predecessors: 0,
        ..Limits::default()
    };
    let mut owner_resources = ResourceAccountant::new(Budget::nothing_granted(), &inventory);
    assert!(matches!(
        assess_bounded(request(), predecessor_owner_short, &mut owner_resources),
        OperationOutcome::ResourceIncomplete(value)
            if value.dimension() == BudgetDimension::Operation
                && value.granted() == 0
                && value.requested() == 1
    ));

    let one_allocation = Budget::nothing_granted()
        .grant(BudgetDimension::AccountedAllocation, 1)
        .expect("one conformance report allocation");
    let mut allocation_resources = ResourceAccountant::new(one_allocation, &inventory);
    assert!(matches!(
        assess_bounded(request(), Limits::default(), &mut allocation_resources),
        OperationOutcome::Completed(_)
    ));
    assert!(matches!(
        assess_bounded(request(), Limits::default(), &mut allocation_resources),
        OperationOutcome::ResourceIncomplete(value)
            if value.dimension() == BudgetDimension::AccountedAllocation
                && value.consumed() == 1
                && value.requested() == 1
    ));

    let mut incremental = Incremental::new(model.clone());
    incremental
        .push(
            bindings[0].clone(),
            axes(&decision, &surrounding, true),
            &decision.completeness_complete,
            &global,
            AssessmentExecution::Completed,
            Limits::default(),
        )
        .expect("first retained prefix");
    let one_short = Budget::nothing_granted()
        .grant(BudgetDimension::Observation, 1)
        .expect("positive one-short grant");
    let mut one_short_resources = ResourceAccountant::new(one_short, &inventory);
    let bounded_context = IncrementalContext {
        axes: axes(&decision, &surrounding, true),
        completeness: &decision.completeness_complete,
        global_conformance: &global,
        execution: AssessmentExecution::Completed,
        limits: Limits::default(),
    };
    let mut allocation_incremental = Incremental::new(model.clone());
    let one_allocation = Budget::nothing_granted()
        .grant(BudgetDimension::AccountedAllocation, 1)
        .expect("one-short incremental allocation grant");
    let mut allocation_resources = ResourceAccountant::new(one_allocation, &inventory);
    assert!(matches!(
        allocation_incremental.push_bounded(
            bindings[0].clone(),
            bounded_context,
            &mut allocation_resources,
        ),
        OperationOutcome::ResourceIncomplete(value)
            if value.dimension() == BudgetDimension::AccountedAllocation
                && value.consumed() == 1
                && value.requested() == 1
    ));
    assert!(allocation_incremental.observations().is_empty());

    let two_allocations = Budget::nothing_granted()
        .grant(BudgetDimension::AccountedAllocation, 2)
        .expect("prefix and report allocation grant");
    let mut allocation_resources = ResourceAccountant::new(two_allocations, &inventory);
    assert!(matches!(
        allocation_incremental.push_bounded(
            bindings[0].clone(),
            bounded_context,
            &mut allocation_resources,
        ),
        OperationOutcome::Completed(_)
    ));
    assert_eq!(allocation_incremental.observations().len(), 1);

    let incremental_incomplete = incremental.push_bounded(
        bindings[1].clone(),
        bounded_context,
        &mut one_short_resources,
    );
    assert!(matches!(
        incremental_incomplete,
        OperationOutcome::ResourceIncomplete(value)
            if value.dimension() == BudgetDimension::Observation
                && value.requested() == 2
    ));
    assert_eq!(incremental.observations().len(), 1);

    let exact = Budget::nothing_granted()
        .grant(BudgetDimension::Observation, 2)
        .expect("positive exact grant");
    let mut exact_resources = ResourceAccountant::new(exact, &inventory);
    let reached_assessment =
        incremental.push_bounded(bindings[1].clone(), bounded_context, &mut exact_resources);
    assert!(matches!(
        reached_assessment,
        OperationOutcome::DomainError(_)
    ));
    assert_eq!(incremental.observations().len(), 1);

    let open = premises(GlobalConformanceClosure::Open);
    let second_binding = binding(
        "obligation:two",
        &decision.observation,
        "workflow:one",
        "effect:two",
    );
    let mut finishing = Incremental::new(model);
    finishing
        .push(
            bindings[0].clone(),
            axes(&decision, &surrounding, false),
            &decision.completeness_incomplete,
            &open,
            AssessmentExecution::Completed,
            Limits::default(),
        )
        .expect("open first prefix");
    finishing
        .push(
            second_binding,
            axes(&decision, &surrounding, true),
            &decision.completeness_complete,
            &global,
            AssessmentExecution::Completed,
            Limits::default(),
        )
        .expect("settled second prefix");

    let one_short = Budget::nothing_granted()
        .grant(BudgetDimension::Observation, 1)
        .expect("positive one-short grant");
    let mut one_short_resources = ResourceAccountant::new(one_short, &inventory);
    assert!(matches!(
        finishing.finish_bounded(
            bounded_context,
            &mut one_short_resources,
        ),
        OperationOutcome::ResourceIncomplete(value)
            if value.dimension() == BudgetDimension::Observation
                && value.requested() == 2
    ));
    assert_eq!(finishing.observations().len(), 2);

    let exact = Budget::nothing_granted()
        .grant(BudgetDimension::Observation, 2)
        .expect("positive exact grant");
    let mut exact_resources = ResourceAccountant::new(exact, &inventory);
    assert!(matches!(
        finishing.finish_bounded(bounded_context, &mut exact_resources,),
        OperationOutcome::Completed(_)
    ));

    let one_allocation = Budget::nothing_granted()
        .grant(BudgetDimension::AccountedAllocation, 1)
        .expect("one final report allocation");
    let mut allocation_resources = ResourceAccountant::new(one_allocation, &inventory);
    assert!(matches!(
        finishing.finish_bounded(bounded_context, &mut allocation_resources),
        OperationOutcome::Completed(_)
    ));
}

/// The conformance layer retains all four owner views and never promotes one
/// axis, completeness, global closure, or execution into another.
#[trace("TC-004", "FR-004-AC-3", "FR-028-AC-1", "FR-028-AC-2", "FR-028-AC-3")]
#[test]
fn tc_004_all_axis_execution_and_completeness_states_remain_independent() {
    let decision = scope_views("decision-product");
    let surrounding = scope_views("surrounding-product");
    let model = Model::new(
        "protocol:product",
        vec![obligation(
            "obligation:one",
            &decision.observation,
            "effect:one",
            vec![],
        )],
        Limits::default(),
    )
    .expect("valid model");
    let bindings = [binding(
        "obligation:one",
        &decision.observation,
        "workflow:one",
        "effect:one",
    )];
    let progress = [&decision.progress_open, &decision.progress_closed];
    let closure = [&decision.closure_open, &decision.closure_closed];
    let surrounding_progress = [&surrounding.progress_open, &surrounding.progress_closed];
    let surrounding_closure = [&surrounding.closure_open, &surrounding.closure_closed];
    let completeness = [
        &decision.completeness_complete,
        &decision.completeness_incomplete,
        &decision.completeness_contradicted,
    ];
    let global_states = [
        GlobalConformanceClosure::Closed,
        GlobalConformanceClosure::Open,
        GlobalConformanceClosure::Incomplete,
        GlobalConformanceClosure::Contradicted,
    ];
    for decision_progress in progress {
        for decision_closure in closure {
            for execution_progress in surrounding_progress {
                for execution_closure in surrounding_closure {
                    for completeness in completeness {
                        for global_state in global_states {
                            for execution in AssessmentExecution::ALL {
                                let global = premises(global_state);
                                let report = assess(
                                    Request {
                                        model: &model,
                                        observations: &bindings,
                                        axes: AxisInputs {
                                            decision_scope_progress: decision_progress,
                                            decision_scope_closure: decision_closure,
                                            surrounding_execution_progress: execution_progress,
                                            surrounding_execution_closure: execution_closure,
                                        },
                                        completeness,
                                        global_conformance: &global,
                                        execution,
                                    },
                                    Limits::default(),
                                )
                                .expect("independent product row");
                                let expected = if execution != AssessmentExecution::Completed
                                    || completeness.payload().state()
                                        != quire_observation::authority::completeness::State::Complete
                                {
                                    GlobalDisposition::Unavailable
                                } else if global_state == GlobalConformanceClosure::Contradicted {
                                    GlobalDisposition::Violates
                                } else if decision_progress.payload().state()
                                    == quire_observation::authority::OpenClosed::Closed
                                    && decision_closure.payload().state()
                                        == quire_observation::authority::OpenClosed::Closed
                                    && execution_progress.payload().state()
                                        == quire_observation::authority::OpenClosed::Closed
                                    && execution_closure.payload().state()
                                        == quire_observation::authority::OpenClosed::Closed
                                    && global_state == GlobalConformanceClosure::Closed
                                {
                                    GlobalDisposition::Conforms
                                } else {
                                    GlobalDisposition::Pending
                                };
                                assert_eq!(report.global, expected);
                                assert_eq!(
                                    report.obligations[0].disposition,
                                    if execution == AssessmentExecution::Completed {
                                        ObligationDisposition::Conforms
                                    } else {
                                        ObligationDisposition::Unavailable
                                    }
                                );
                                assert_eq!(
                                    report.premises.decision_scope_progress.bytes,
                                    decision_progress.bytes()
                                );
                                assert_eq!(
                                    report.premises.decision_scope_closure.bytes,
                                    decision_closure.bytes()
                                );
                                assert_eq!(
                                    report.premises.surrounding_execution_progress.bytes,
                                    execution_progress.bytes()
                                );
                                assert_eq!(
                                    report.premises.surrounding_execution_closure.bytes,
                                    execution_closure.bytes()
                                );
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Missing, duplicated, unexpected, and cross-wired global premises withhold
/// only global success; an independently settled obligation stays settled.
#[trace("TC-004", "FR-028-AC-4", "FR-028-AC-5")]
#[test]
fn tc_004_global_premise_defects_do_not_rewrite_obligation_truth() {
    let decision = scope_views("decision-premise-defect");
    let surrounding = scope_views("surrounding-premise-defect");
    let model = Model::new(
        "protocol:premise-defect",
        vec![obligation(
            "obligation:one",
            &decision.observation,
            "effect:one",
            vec![],
        )],
        Limits::default(),
    )
    .expect("valid model");
    let bindings = [binding(
        "obligation:one",
        &decision.observation,
        "workflow:one",
        "effect:one",
    )];
    let mut global = premises(GlobalConformanceClosure::Closed);
    let missing = global.premises.remove(0);
    global.premises.push(global.premises[0].clone());
    global.premises[1].scope_identity = "scope:cross-wired".into();
    global.premises.push(GlobalPremise {
        kind: missing.kind,
        identity: "premise:unexpected".into(),
        scope_identity: missing.scope_identity,
        authority_identity: missing.authority_identity,
    });
    let report = assess(
        request(&model, &bindings, &decision, &surrounding, &global, true),
        Limits::default(),
    )
    .expect("global defects are reportable");
    assert_eq!(
        report.obligations[0].disposition,
        ObligationDisposition::Conforms
    );
    assert_eq!(report.global, GlobalDisposition::Unavailable);
    let kinds: std::collections::BTreeSet<_> = report
        .global_premise_defects
        .iter()
        .map(|defect| defect.kind)
        .collect();
    assert_eq!(
        kinds,
        [
            GlobalPremiseDefectKind::Missing,
            GlobalPremiseDefectKind::Duplicate,
            GlobalPremiseDefectKind::Unexpected,
            GlobalPremiseDefectKind::CrossWired,
        ]
        .into_iter()
        .collect()
    );

    let not_required = premises(GlobalConformanceClosure::NotRequired);
    let refusal = assess(
        request(
            &model,
            &bindings,
            &decision,
            &surrounding,
            &not_required,
            true,
        ),
        Limits::default(),
    )
    .expect_err("global conformance cannot be not-required");
    assert_eq!(refusal.code(), ErrorCode::InvalidPremises);
    assert_eq!(refusal.envelope().stage(), ProtocolStage::Conformance);
}
