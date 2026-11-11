// SPDX-License-Identifier: AGPL-3.0-or-later
//! TC-054: an observed history against a loop's iteration maximum (FR-056),
//! and the loop's invariant and variant carried as undischarged (FR-059).

mod support {
    pub mod dynamic;
    pub mod handoff;
}

use ix_trace_rs::trace;
use quire_protocol::disposition::{
    request_discharge, DischargeStatus, LoopProof, UnsupportedCause,
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
            repeat_ordinals: Vec::new(),
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

#[test]
#[trace("TC-054", "FR-056-AC-3")]
fn the_loop_assessment_takes_and_holds_no_closure_axis() {
    let dynamic = Dynamic::with("workflow", 3);
    let assessment = run(&dynamic, "Bounded", 4, false);
    // Destructured with no rest pattern: a closure, progress or any other
    // axis would have to be a new field, and the assessor's signature has no
    // input to supply one from.
    let LoopAssessment {
        control: _,
        workflow: _,
        repeat_ordinals: _,
        iterations,
        maximum,
        within_maximum,
        exited,
        violation,
    } = assessment.clone();
    assert_eq!((iterations, maximum), (4, Some(3)));
    assert_eq!((within_maximum, exited), (Some(false), false));
    assert!(violation.is_some());
    // Assessing again, as another axis's value changes around it, is the same
    // value.
    assert_eq!(run(&dynamic, "Bounded", 4, false), assessment);
}

#[test]
#[trace("TC-054", "FR-056-AC-5")]
fn a_loop_inside_a_repeat_is_one_occurrence_for_each_iteration() {
    use qsl_package::OccurrenceKeyError;
    use quire_exact::Integer;
    use quire_protocol::dynamics::HistoryRefusal;
    let dynamic = Dynamic::with("workflow", 3);
    let facts = |ordinals: &[u64], iterations: usize| -> Vec<LoopFact> {
        (0..iterations)
            .map(|index| LoopFact {
                at: at(ordinals[0] * 10 + u64::try_from(index).unwrap() + 1),
                control: dynamic.named("control", "Inner"),
                workflow: workflow("w1"),
                repeat_ordinals: ordinals.iter().map(|n| Integer::from(*n)).collect(),
                event: LoopEvent::Iteration,
            })
            .collect()
    };
    let mut history = facts(&[0], 2);
    history.extend(facts(&[1], 3));
    let assessed = assess_loops(dynamic.linked().declarations(), &history).expect("declared");
    // Inner's maximum is 2: iteration 0 of Outer stays within it, iteration 1
    // exceeds it. Merged, one occurrence of five iterations would report one
    // violation that belongs to neither.
    let counts: Vec<_> = assessed
        .iter()
        .map(|a| (a.iterations, a.violation.is_some()))
        .collect();
    assert_eq!(counts, [(2, false), (3, true)]);
    assert!(matches!(
        assess_loops(dynamic.linked().declarations(), &facts(&[2], 1)),
        Err(HistoryRefusal::Occurrence(
            OccurrenceKeyError::RepeatOrdinal { .. }
        ))
    ));
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
    let disposition = request_discharge(dynamic.package(), declaration, premises.clone());
    assert_eq!(disposition.cause, UnsupportedCause::requested_capability());
    let UnsupportedCause::RequestedCapability(code) = &disposition.cause else {
        panic!("the requested-capability cause");
    };
    assert_eq!(code.cause(), "unsupported-requested-capability");
    assert_eq!(code.code(), "unsupported_projection");
    assert_eq!(disposition.construct, Some(construct));
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

#[test]
#[trace("TC-054", "FR-056-AC-6")]
fn a_fact_naming_no_declared_loop_refuses_the_history() {
    use quire_protocol::dynamics::HistoryRefusal;
    let dynamic = Dynamic::with("workflow", 3);
    let join = dynamic.named("control", "JoinAll");
    let fact = LoopFact {
        at: at(1),
        control: join.clone(),
        workflow: workflow("w1"),
        repeat_ordinals: Vec::new(),
        event: LoopEvent::Iteration,
    };
    assert_eq!(
        assess_loops(dynamic.linked().declarations(), &[fact]),
        Err(HistoryRefusal::UndeclaredSubject { node: join })
    );
}

#[test]
#[trace("TC-054", "FR-056-AC-7")]
fn a_workflow_identity_qsl_cannot_key_refuses_the_history() {
    use qsl_package::OccurrenceKeyError;
    use quire_protocol::dynamics::HistoryRefusal;
    use quire_protocol::ids::WorkflowInstanceId;
    let dynamic = Dynamic::with("workflow", 3);
    let refused = |identity: &[u8]| {
        let fact = LoopFact {
            at: at(1),
            control: dynamic.named("control", "Free"),
            workflow: WorkflowInstanceId::new(identity, "w"),
            repeat_ordinals: Vec::new(),
            event: LoopEvent::Iteration,
        };
        assess_loops(dynamic.linked().declarations(), &[fact])
    };
    assert_eq!(
        refused(b"\xff\xfe"),
        Err(HistoryRefusal::WorkflowNotText {
            workflow: b"\xff\xfe".as_slice().into()
        })
    );
    assert_eq!(
        refused(b""),
        Err(HistoryRefusal::Occurrence(
            OccurrenceKeyError::EmptyWorkflowIdentity
        ))
    );
    let long = vec![b'a'; 4097];
    assert!(matches!(
        refused(&long),
        Err(HistoryRefusal::Occurrence(
            OccurrenceKeyError::WorkflowIdentityTooLong { bytes: 4097 }
        ))
    ));
    assert!(refused(&long[..4096]).is_ok());
}
