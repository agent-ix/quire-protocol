// SPDX-License-Identifier: AGPL-3.0-or-later
//! TC-054: an observed history against a loop's iteration maximum (FR-056),
//! and the loop's invariant and variant carried as undischarged (FR-059).

mod support {
    pub mod dynamic;
    pub mod handoff;
}

use ix_trace_rs::trace;
use quire_protocol::closure::{
    ClosureState, DecisionScopeClosure, DecisionScopeProgress, GlobalConformanceClosure, ScopeAxis,
    ScopeState, SurroundingExecutionClosure, SurroundingExecutionProgress,
};
use quire_protocol::disposition::{
    request_discharge, DischargeStatus, LoopProof, UnsupportedCause, REQUESTED_CAPABILITY,
};
use quire_protocol::dynamics::{
    assess_loops, LoopAssessment, LoopBoundViolation, LoopEvent, LoopFact,
};
use support::dynamic::{at, workflow, Dynamic};

fn run(dynamic: &Dynamic, control: &str, iterations: usize, exit: bool) -> LoopAssessment {
    let mut events = vec![LoopEvent::Iteration; iterations];
    if exit {
        events.push(LoopEvent::Exit);
    }
    let facts: Vec<_> = events
        .into_iter()
        .enumerate()
        .map(|(index, event)| LoopFact {
            at: at(u64::try_from(index).unwrap() + 1),
            control: dynamic.named("control", control),
            workflow: workflow("w1"),
            event,
        })
        .collect();
    let mut assessed = assess_loops(dynamic.linked().declarations(), &facts).expect("declared");
    assert_eq!(assessed.len(), 1);
    assessed.remove(0)
}

#[test]
#[trace("TC-054", "FR-056-AC-1")]
fn only_an_iteration_count_above_the_maximum_is_a_violation() {
    let dynamic = Dynamic::with("workflow", 3);
    for (count, violated) in [(2, false), (3, false), (4, true)] {
        let assessment = run(&dynamic, "Bounded", count, true);
        assert_eq!(assessment.within_maximum, Some(!violated), "{count}");
        assert_eq!(
            assessment.violation,
            violated.then(|| LoopBoundViolation {
                control: dynamic.named("control", "Bounded"),
                maximum: 3,
                observed: 4,
            }),
            "{count}"
        );
    }
}

#[test]
#[trace("TC-054", "FR-056-AC-2")]
fn a_history_ending_inside_the_loop_is_not_exited_and_not_a_violation() {
    let dynamic = Dynamic::with("workflow", 3);
    let inside = run(&dynamic, "Bounded", 2, false);
    assert!(!inside.exited);
    assert_eq!(inside.violation, None);
    assert!(run(&dynamic, "Bounded", 2, true).exited);
}

fn closure(open: bool) -> ClosureState {
    let state = if open {
        ScopeState::Open
    } else {
        ScopeState::Closed
    };
    ClosureState::new(
        Some(DecisionScopeProgress::of(state)),
        Some(DecisionScopeClosure::of(state)),
        Some(SurroundingExecutionProgress::of(state)),
        Some(SurroundingExecutionClosure::of(state)),
        Some(if open {
            GlobalConformanceClosure::Open
        } else {
            GlobalConformanceClosure::Closed
        }),
    )
    .expect("five axes")
}

#[test]
#[trace("TC-054", "FR-056-AC-3")]
fn the_loop_assessment_is_unchanged_by_the_closure_axis_value() {
    let dynamic = Dynamic::with("workflow", 3);
    // The assessment takes no closure input, so any closure state assessed for
    // the same history leaves it as it was.
    let before = run(&dynamic, "Bounded", 4, false);
    let states = [closure(true), closure(false)];
    assert_ne!(states[0], states[1]);
    assert_eq!(run(&dynamic, "Bounded", 4, false), before);
}

#[test]
#[trace("TC-054", "FR-056-AC-4")]
fn a_loop_with_no_maximum_reports_the_count_and_an_absent_maximum() {
    let dynamic = Dynamic::with("workflow", 3);
    for count in [1, 2, 50] {
        let assessment = run(&dynamic, "Free", count, false);
        assert_eq!(assessment.iterations, u64::try_from(count).unwrap());
        assert_eq!(assessment.maximum, None);
        assert_eq!(assessment.within_maximum, None);
        assert_eq!(assessment.violation, None);
    }
}

#[test]
#[trace("TC-054", "FR-059-AC-1")]
fn the_invariant_and_variant_are_carried_by_node_as_undischarged() {
    let dynamic = Dynamic::with("workflow", 3);
    let declaration = dynamic
        .linked()
        .declarations()
        .loop_at(&dynamic.named("control", "Bounded"))
        .expect("declared");
    let proof = LoopProof::of(declaration);
    assert_eq!(proof.control, declaration.control);
    assert_eq!(proof.invariant, declaration.invariant);
    assert_eq!(proof.variant, declaration.variant);
    assert!(proof.invariant.is_some() && proof.variant.is_some());
    assert_eq!(proof.status, DischargeStatus::Undischarged);
}

#[test]
#[trace("TC-054", "FR-059-AC-2")]
fn a_discharge_request_is_unsupported_naming_the_capability_with_the_requests_premises() {
    let dynamic = Dynamic::with("workflow", 3);
    let declaration = dynamic
        .linked()
        .declarations()
        .loop_at(&dynamic.named("control", "Bounded"))
        .expect("declared");
    let construct = dynamic.construct(&declaration.control);
    let premises = vec!["fragment-x".to_owned(), "backend-y".to_owned()];
    let disposition = request_discharge(declaration, construct.clone(), premises.clone());
    assert_eq!(disposition.cause, UnsupportedCause::RequestedCapability);
    assert_eq!(disposition.cause.name(), REQUESTED_CAPABILITY.as_bytes());
    assert_eq!(disposition.construct, construct);
    assert_eq!(disposition.premises, premises);
    assert_eq!(
        disposition.execution().label(),
        "unsupported",
        "never a proved status"
    );
}

#[test]
#[trace("TC-054", "FR-059-AC-3")]
fn a_loop_that_exits_within_its_maximum_is_still_undischarged() {
    let dynamic = Dynamic::with("workflow", 3);
    let assessment = run(&dynamic, "Bounded", 2, true);
    assert!(assessment.exited && assessment.violation.is_none());
    let declaration = dynamic
        .linked()
        .declarations()
        .loop_at(&assessment.control)
        .expect("declared");
    assert_eq!(
        LoopProof::of(declaration).status,
        DischargeStatus::Undischarged
    );
}

#[test]
#[trace("TC-054", "FR-059-AC-4")]
fn undischarged_is_the_only_discharge_status_and_no_proof_field_exists() {
    let dynamic = Dynamic::with("workflow", 3);
    let declaration = dynamic
        .linked()
        .declarations()
        .loops()
        .first()
        .expect("a loop");
    // Destructured with no rest pattern: a checked invariant, a variant rank
    // or a termination verdict would have to be a new field here.
    let LoopProof {
        control: _,
        invariant: _,
        variant: _,
        status,
    } = LoopProof::of(declaration);
    // An irrefutable pattern compiles only while `Undischarged` is the single
    // member of the status type.
    let DischargeStatus::Undischarged = status;
}
