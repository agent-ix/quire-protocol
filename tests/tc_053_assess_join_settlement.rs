// SPDX-License-Identifier: AGPL-3.0-or-later
//! TC-053: an observed history against join policy, threshold and
//! `outstanding` rule (FR-055). A `predicate` join is never evaluated.

mod support {
    pub mod dynamic;
    pub mod handoff;
}

use ix_trace_rs::trace;
use qsl_eval::simulation::protocol::Unsupported;
use qsl_package::OccurrenceKeyError;
use qsl_semantics::check::CheckedOutstanding;
use quire_exact::Integer;
use quire_protocol::declarations::JoinPolicy;
use quire_protocol::disposition::UnsupportedCause;
use quire_protocol::dynamics::{
    assess_joins, BranchOutcome, HistoryRefusal, JoinAssessment, JoinEvent, JoinFact, JoinOutcome,
    JoinViolation,
};
use support::dynamic::{at, workflow, Dynamic};

fn done(branch: &str) -> JoinEvent {
    JoinEvent::BranchCompleted {
        branch: branch.into(),
    }
}

fn left(branch: &str, outcome: BranchOutcome) -> JoinEvent {
    JoinEvent::Outstanding {
        branch: branch.into(),
        outcome,
    }
}

fn history(dynamic: &Dynamic, join: &str, who: &str, events: Vec<JoinEvent>) -> Vec<JoinFact> {
    events
        .into_iter()
        .enumerate()
        .map(|(index, event)| JoinFact {
            at: at(u64::try_from(index).unwrap() + 1),
            join: dynamic.named("control", join),
            workflow: workflow(who),
            repeat_ordinals: Vec::new(),
            event,
        })
        .collect()
}

fn outcomes(dynamic: &Dynamic, facts: &[JoinFact]) -> Result<Vec<JoinOutcome>, HistoryRefusal> {
    assess_joins(dynamic.linked().declarations(), facts, |node| {
        dynamic.construct(node)
    })
}

fn assessed(dynamic: &Dynamic, join: &str, events: Vec<JoinEvent>) -> JoinAssessment {
    let facts = history(dynamic, join, "w1", events);
    match outcomes(dynamic, &facts).expect("declared join").as_slice() {
        [JoinOutcome::Assessed(assessment)] => assessment.clone(),
        other => panic!("{join}: {other:?}"),
    }
}

fn settlement(
    dynamic: &Dynamic,
    join: &str,
    policy: JoinPolicy,
    threshold: u64,
    observed: u64,
) -> JoinViolation {
    JoinViolation::Settlement {
        join: dynamic.named("control", join),
        policy,
        threshold,
        observed,
    }
}

#[test]
#[trace("TC-053", "FR-055-AC-1")]
fn an_all_join_settles_only_with_every_branch_complete() {
    let dynamic = Dynamic::with("workflow", 3);
    let full = assessed(
        &dynamic,
        "JoinAll",
        vec![done("a"), done("b"), done("c"), JoinEvent::Settled],
    );
    assert!(full.settled);
    assert_eq!(full.permitted, Some(true));
    assert!(full.violations.is_empty());
    let short = assessed(
        &dynamic,
        "JoinAll",
        vec![done("a"), done("b"), JoinEvent::Settled],
    );
    assert_eq!(
        short.violations,
        [settlement(&dynamic, "JoinAll", JoinPolicy::All, 3, 2)]
    );
    assert_eq!(short.permitted, Some(false));
}

#[test]
#[trace("TC-053", "FR-055-AC-2")]
fn an_any_join_settles_after_the_first_completed_branch() {
    let dynamic = Dynamic::with("workflow", 3);
    let first = assessed(
        &dynamic,
        "AnyContinue",
        vec![
            done("b"),
            JoinEvent::Settled,
            left("a", BranchOutcome::Continued),
            left("c", BranchOutcome::Continued),
        ],
    );
    assert_eq!(first.permitted, Some(true));
    assert!(first.violations.is_empty());
    let none = assessed(&dynamic, "AnyContinue", vec![JoinEvent::Settled]);
    assert_eq!(
        none.violations,
        [settlement(&dynamic, "AnyContinue", JoinPolicy::Any, 1, 0)]
    );
}

#[test]
#[trace("TC-053", "FR-055-AC-3")]
fn a_quorum_join_needs_n_distinct_branches_and_counts_a_repeat_once() {
    let dynamic = Dynamic::with("workflow", 3);
    let run = |events: Vec<JoinEvent>| assessed(&dynamic, "QuorumCancel", events).violations;
    let violation = settlement(&dynamic, "QuorumCancel", JoinPolicy::Quorum(2), 2, 1);
    assert_eq!(
        run(vec![done("a"), JoinEvent::Settled]),
        std::slice::from_ref(&violation)
    );
    assert!(run(vec![done("a"), done("b"), JoinEvent::Settled]).is_empty());
    assert!(run(vec![done("a"), done("b"), done("c"), JoinEvent::Settled]).is_empty());
    // One branch observed twice is one branch.
    assert_eq!(
        run(vec![done("a"), done("a"), JoinEvent::Settled]),
        [violation]
    );
}

#[test]
#[trace("TC-053", "FR-055-AC-4")]
fn outstanding_branches_are_judged_against_the_authored_rule() {
    let dynamic = Dynamic::with("workflow", 3);
    let keep = [
        done("a"),
        JoinEvent::Settled,
        left("b", BranchOutcome::Continued),
        left("c", BranchOutcome::Continued),
    ];
    assert!(assessed(&dynamic, "AnyContinue", keep.to_vec())
        .violations
        .is_empty());
    let cancelled = [
        done("a"),
        JoinEvent::Settled,
        left("b", BranchOutcome::Cancelled),
        left("c", BranchOutcome::Continued),
    ];
    let wrong = assessed(&dynamic, "AnyContinue", cancelled.to_vec());
    assert_eq!(
        wrong.violations,
        [JoinViolation::Outstanding {
            join: dynamic.named("control", "AnyContinue"),
            branch: "b".into(),
            authored: CheckedOutstanding::Continue,
            observed: BranchOutcome::Cancelled,
        }]
    );
    assert_eq!(wrong.outstanding.len(), 2);

    let drop = [
        done("a"),
        done("b"),
        JoinEvent::Settled,
        left("c", BranchOutcome::Cancelled),
    ];
    assert!(assessed(&dynamic, "QuorumCancel", drop.to_vec())
        .violations
        .is_empty());
    let running = [
        done("a"),
        done("b"),
        JoinEvent::Settled,
        left("c", BranchOutcome::Continued),
    ];
    assert_eq!(
        assessed(&dynamic, "QuorumCancel", running.to_vec()).violations,
        [JoinViolation::Outstanding {
            join: dynamic.named("control", "QuorumCancel"),
            branch: "c".into(),
            authored: CheckedOutstanding::Cancel,
            observed: BranchOutcome::Continued,
        }]
    );
}

#[test]
#[trace("TC-053", "FR-055-AC-5")]
fn a_predicate_or_unruled_any_or_quorum_join_is_unsupported_and_neither_settled_nor_not() {
    let dynamic = Dynamic::with("workflow", 3);
    let cases = [
        ("Predicate", Unsupported::JoinPredicate),
        ("AnyUnruled", Unsupported::UnruledOutstanding),
        ("QuorumUnruled", Unsupported::UnruledOutstanding),
    ];
    for (join, expected) in cases {
        let facts = history(
            &dynamic,
            join,
            "w1",
            vec![done("a"), done("b"), JoinEvent::Settled],
        );
        let result = outcomes(&dynamic, &facts).expect("declared join");
        let [JoinOutcome::Unsupported {
            cause, construct, ..
        }] = result.as_slice()
        else {
            panic!("{join}: {result:?}");
        };
        assert_eq!(*cause, UnsupportedCause::Construct(expected), "{join}");
        assert_eq!(construct.node, dynamic.named("control", join));
    }
}

#[test]
#[trace("TC-053", "FR-055-AC-6")]
fn an_all_join_with_no_outstanding_rule_is_still_assessed() {
    let dynamic = Dynamic::with("workflow", 3);
    let full = assessed(
        &dynamic,
        "AllUnruled",
        vec![done("a"), done("b"), done("c"), JoinEvent::Settled],
    );
    assert!(full.violations.is_empty());
    assert!(full.outstanding.is_empty());
    let short = assessed(
        &dynamic,
        "AllUnruled",
        vec![done("a"), done("c"), JoinEvent::Settled],
    );
    assert_eq!(
        short.violations,
        [settlement(&dynamic, "AllUnruled", JoinPolicy::All, 3, 2)]
    );
}

#[test]
#[trace("TC-053", "FR-055-AC-7")]
fn a_branch_the_join_does_not_list_refuses_the_history() {
    let dynamic = Dynamic::with("workflow", 3);
    let facts = history(&dynamic, "JoinAll", "w1", vec![done("zzz")]);
    assert_eq!(
        outcomes(&dynamic, &facts),
        Err(HistoryRefusal::UndeclaredBranch {
            join: dynamic.named("control", "JoinAll"),
            branch: "zzz".into(),
        })
    );
}

fn nested(dynamic: &Dynamic, ordinals: &[u64], events: Vec<JoinEvent>) -> Vec<JoinFact> {
    events
        .into_iter()
        .enumerate()
        .map(|(index, event)| JoinFact {
            at: at(u64::try_from(index).unwrap() + 1),
            join: dynamic.named("control", "InnerJoin"),
            workflow: workflow("w1"),
            repeat_ordinals: ordinals.iter().map(|n| Integer::from(*n)).collect(),
            event,
        })
        .collect()
}

#[test]
#[trace("TC-053", "FR-055-AC-8")]
fn a_join_inside_a_repeat_is_one_occurrence_for_each_iteration() {
    let dynamic = Dynamic::with("workflow", 3);
    let settled = |ordinals: &[u64], complete: &[&str]| {
        let mut events: Vec<_> = complete.iter().map(|branch| done(branch)).collect();
        events.push(JoinEvent::Settled);
        nested(&dynamic, ordinals, events)
    };
    // Iteration 0 settles with every branch, iteration 1 with one: two
    // occurrences of one join in one workflow, never merged into one count.
    let mut facts = settled(&[0], &["a", "b", "c"]);
    facts.extend(settled(&[1], &["a"]));
    let result = outcomes(&dynamic, &facts).expect("declared join");
    let assessed: Vec<_> = result
        .iter()
        .map(|outcome| match outcome {
            JoinOutcome::Assessed(assessment) => assessment,
            other @ JoinOutcome::Unsupported { .. } => panic!("{other:?}"),
        })
        .collect();
    assert_eq!(assessed.len(), 2);
    assert_eq!(assessed[0].repeat_ordinals, [Integer::from(0u64)]);
    assert!(assessed[0].violations.is_empty());
    assert_eq!(assessed[1].repeat_ordinals, [Integer::from(1u64)]);
    assert_eq!(assessed[1].violations.len(), 1);
    // QSL's key refuses an ordinal outside the repeat's maximum and a missing one.
    assert!(matches!(
        outcomes(&dynamic, &settled(&[2], &["a"])),
        Err(HistoryRefusal::Occurrence(
            OccurrenceKeyError::RepeatOrdinal { .. }
        ))
    ));
    assert!(matches!(
        outcomes(&dynamic, &settled(&[], &["a"])),
        Err(HistoryRefusal::Occurrence(
            OccurrenceKeyError::RepeatCount { .. }
        ))
    ));
}

#[test]
#[trace("TC-053", "FR-055-AC-9")]
fn a_workflow_identity_qsl_cannot_key_refuses_the_history() {
    use quire_protocol::ids::WorkflowInstanceId;
    let dynamic = Dynamic::with("workflow", 3);
    let refused = |identity: &[u8]| {
        let mut facts = history(&dynamic, "JoinAll", "w1", vec![JoinEvent::Settled]);
        facts[0].workflow = WorkflowInstanceId::new(identity, "w");
        outcomes(&dynamic, &facts)
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
    assert!(matches!(
        refused(&vec![b'a'; 4097]),
        Err(HistoryRefusal::Occurrence(
            OccurrenceKeyError::WorkflowIdentityTooLong { .. }
        ))
    ));
    assert!(refused(&[b'a'; 4096]).is_ok());
}

#[test]
#[trace("TC-053", "FR-055-AC-10")]
fn a_fact_naming_no_declared_join_refuses_the_history() {
    let dynamic = Dynamic::with("workflow", 3);
    let node = dynamic.named("control", "Bounded");
    let mut facts = history(&dynamic, "JoinAll", "w1", vec![JoinEvent::Settled]);
    facts[0].join = node.clone();
    assert_eq!(
        outcomes(&dynamic, &facts),
        Err(HistoryRefusal::UndeclaredSubject { node })
    );
}
