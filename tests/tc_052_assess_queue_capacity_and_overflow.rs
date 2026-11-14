// SPDX-License-Identifier: AGPL-3.0-or-later
//! TC-052: an observed history against queue capacity, overflow outcome,
//! delivery order and delivery premise (FR-054).

mod support {
    pub mod dynamic;
    pub mod handoff;
}

use ix_trace_rs::trace;
use qsl_eval::simulation::protocol::Unsupported;
use qsl_package::protocol_view::Delivery;
use qsl_semantics::check::{ChannelOverflow, LossSelector};
use quire_protocol::disposition::{AssessmentUnsupported, UnsupportedCause};
use quire_protocol::dynamics::{
    assess_channels, ChannelAssessment, ChannelEvent, ChannelFact, ChannelOutcome, OverflowOutcome,
    PremiseBreach, QueueViolation,
};
use support::dynamic::{at, message, Dynamic};

fn enqueue(name: &str, key: &str) -> ChannelEvent {
    ChannelEvent::Enqueue {
        message: message(name),
        key: key.as_bytes().into(),
    }
}

fn deliver(name: &str) -> ChannelEvent {
    ChannelEvent::Deliver {
        message: message(name),
    }
}

fn overflow(name: &str, outcome: OverflowOutcome) -> ChannelEvent {
    ChannelEvent::Overflow {
        message: message(name),
        outcome,
    }
}

fn lost(selector: LossSelector, dropped: &str) -> OverflowOutcome {
    OverflowOutcome::Lost {
        selector,
        dropped: message(dropped),
    }
}

/// The history `events` on the channel `name`, one observation per event.
fn history(dynamic: &Dynamic, name: &str, events: Vec<ChannelEvent>) -> Vec<ChannelFact> {
    events
        .into_iter()
        .enumerate()
        .map(|(index, event)| ChannelFact {
            at: at(u64::try_from(index).unwrap() + 1),
            channel: dynamic.named("channel", name),
            event,
        })
        .collect()
}

fn outcome(dynamic: &Dynamic, name: &str, facts: &[ChannelFact]) -> ChannelOutcome {
    let id = dynamic.named("channel", name);
    assess_channels(dynamic.linked().declarations(), facts, |node| {
        dynamic.construct(node)
    })
    .expect("declared channels")
    .into_iter()
    .find(|outcome| match outcome {
        ChannelOutcome::Assessed(assessment) => assessment.channel == id,
        ChannelOutcome::Unsupported { channel, .. } => *channel == id,
    })
    .expect("the channel is assessed")
}

fn assessed(dynamic: &Dynamic, name: &str, events: Vec<ChannelEvent>) -> ChannelAssessment {
    match outcome(dynamic, name, &history(dynamic, name, events)) {
        ChannelOutcome::Assessed(assessment) => assessment,
        other @ ChannelOutcome::Unsupported { .. } => panic!("{name}: {other:?}"),
    }
}

#[test]
#[trace("TC-052", "FR-054-AC-1")]
fn a_history_within_capacity_reports_no_violation_and_the_peak() {
    let dynamic = Dynamic::with("workflow", 3);
    let assessment = assessed(
        &dynamic,
        "Head",
        vec![
            enqueue("m1", "k"),
            enqueue("m2", "k"),
            deliver("m1"),
            deliver("m2"),
        ],
    );
    assert!(assessment.within_capacity);
    assert_eq!((assessment.max_occupancy, assessment.capacity), (2, 4));
    assert!(assessment.violations.is_empty());
}

#[test]
#[trace("TC-052", "FR-054-AC-2")]
fn excess_occupancy_is_a_capacity_violation_only_without_an_overflow_event() {
    let dynamic = Dynamic::with("workflow", 3);
    let five = |extra: Vec<ChannelEvent>| {
        let mut events: Vec<_> = (1..=5).map(|n| enqueue(&format!("m{n}"), "k")).collect();
        events.extend(extra);
        events
    };
    let without = assessed(&dynamic, "Head", five(vec![]));
    assert_eq!(
        without.violations,
        [QueueViolation::Capacity {
            channel: dynamic.named("channel", "Head"),
            capacity: 4,
            occupancy: 5,
        }]
    );
    assert!(!without.within_capacity);
    let with = assessed(
        &dynamic,
        "Head",
        five(vec![overflow("m5", lost(LossSelector::Head, "m1"))]),
    );
    assert!(with.violations.is_empty());
    assert!(with.within_capacity);
}

/// A full queue of `capacity` messages `m1..`, then `events` for the arriving
/// message `x`.
fn full_queue(capacity: usize, events: Vec<ChannelEvent>) -> Vec<ChannelEvent> {
    let mut history: Vec<_> = (1..=capacity)
        .map(|n| enqueue(&format!("m{n}"), "k"))
        .collect();
    history.extend(events);
    history
}

#[test]
#[trace("TC-052", "FR-054-AC-3")]
fn each_authored_overflow_policy_matches_its_outcome_and_a_mismatch_names_both() {
    let dynamic = Dynamic::with("workflow", 3);
    // (channel, capacity, authored policy, the outcome the full queue yields)
    let policies = [
        ("Block", 3, ChannelOverflow::Block, OverflowOutcome::Blocked),
        (
            "Reject",
            2,
            ChannelOverflow::Reject,
            OverflowOutcome::Rejected,
        ),
        (
            "Head",
            4,
            ChannelOverflow::Loss(LossSelector::Head),
            lost(LossSelector::Head, "m1"),
        ),
        (
            "Tail",
            2,
            ChannelOverflow::Loss(LossSelector::Tail),
            lost(LossSelector::Tail, "m2"),
        ),
        (
            "New",
            2,
            ChannelOverflow::Loss(LossSelector::New),
            lost(LossSelector::New, "x"),
        ),
        (
            "BagNew",
            2,
            ChannelOverflow::Loss(LossSelector::New),
            lost(LossSelector::New, "x"),
        ),
    ];
    for (name, capacity, authored, matching) in policies {
        let clean = assessed(
            &dynamic,
            name,
            full_queue(capacity, vec![overflow("x", matching.clone())]),
        );
        assert!(clean.violations.is_empty(), "{name}");
        // An outcome of a different kind from the authored policy.
        let different = if authored == ChannelOverflow::Block {
            OverflowOutcome::Rejected
        } else {
            OverflowOutcome::Blocked
        };
        let wrong = assessed(
            &dynamic,
            name,
            full_queue(capacity, vec![overflow("x", different.clone())]),
        );
        assert_eq!(
            wrong.violations,
            [QueueViolation::Overflow {
                channel: dynamic.named("channel", name),
                authored,
                observed: different,
            }],
            "{name}"
        );
    }
}

#[test]
#[trace("TC-052", "FR-054-AC-8")]
fn a_loss_is_judged_against_the_queue_not_against_its_label() {
    let dynamic = Dynamic::with("workflow", 3);
    // Authored `loss head` on a queue [m1..m4]: the oldest is m1. A loss
    // labelled head that drops another message is a mismatch...
    let wrong_message = assessed(
        &dynamic,
        "Head",
        full_queue(4, vec![overflow("x", lost(LossSelector::Head, "m3"))]),
    );
    assert_eq!(wrong_message.violations.len(), 1);
    // ...and so is a head loss of the arriving message on an empty queue.
    let empty = assessed(
        &dynamic,
        "Head",
        vec![overflow("x", lost(LossSelector::Head, "x"))],
    );
    assert_eq!(empty.violations.len(), 1);
    // A loss of another selector than the authored one is a mismatch too.
    let other_selector = assessed(
        &dynamic,
        "Head",
        full_queue(4, vec![overflow("x", lost(LossSelector::Tail, "m4"))]),
    );
    assert_eq!(other_selector.violations.len(), 1);
}

#[test]
#[trace("TC-052", "FR-054-AC-4")]
fn delivery_out_of_send_order_within_a_key_is_an_order_violation() {
    let dynamic = Dynamic::with("workflow", 3);
    let out_of_order = assessed(
        &dynamic,
        "Head",
        vec![
            enqueue("m1", "k1"),
            enqueue("m2", "k1"),
            deliver("m2"),
            deliver("m1"),
        ],
    );
    assert_eq!(
        out_of_order.violations,
        [QueueViolation::Order {
            channel: dynamic.named("channel", "Head"),
            key: b"k1".as_slice().into(),
            expected: message("m1"),
            delivered: message("m2"),
        }]
    );
    let interleaved = assessed(
        &dynamic,
        "Head",
        vec![
            enqueue("m1", "k1"),
            enqueue("m2", "k2"),
            deliver("m2"),
            deliver("m1"),
        ],
    );
    assert!(interleaved.violations.is_empty());
}

#[test]
#[trace("TC-052", "FR-054-AC-5")]
fn a_symbolic_capacity_returns_the_unsupported_disposition_and_judges_nothing() {
    let dynamic = Dynamic::with("workflow", 3);
    let facts = history(
        &dynamic,
        "Symbolic",
        (1..=9).map(|n| enqueue(&format!("m{n}"), "k")).collect(),
    );
    let ChannelOutcome::Unsupported {
        cause, construct, ..
    } = outcome(&dynamic, "Symbolic", &facts)
    else {
        panic!("a symbolic capacity is unsupported");
    };
    assert_eq!(
        cause,
        UnsupportedCause::Construct(Unsupported::SymbolicCapacity)
    );
    assert_eq!(construct.node, dynamic.named("channel", "Symbolic"));
}

#[test]
#[trace("TC-052", "FR-054-AC-6")]
fn a_loss_the_premise_forbids_is_a_premise_violation_naming_channel_and_premise() {
    let dynamic = Dynamic::with("workflow", 3);
    let lose = || {
        vec![
            enqueue("m1", "k"),
            ChannelEvent::Lose {
                message: message("m1"),
            },
        ]
    };
    let forbidden = assessed(&dynamic, "Reject", lose());
    assert_eq!(
        forbidden.violations,
        [QueueViolation::Premise {
            channel: dynamic.named("channel", "Reject"),
            premise: Delivery::ExactlyOncePremise,
            observed: PremiseBreach::Loss,
        }]
    );
    let permitted = assessed(&dynamic, "Tail", lose());
    assert!(permitted.violations.is_empty());
    // Duplication is judged by the same premise.
    let duplicate = vec![ChannelEvent::Duplicate {
        message: message("m1"),
    }];
    assert_eq!(
        assessed(&dynamic, "Tail", duplicate.clone())
            .violations
            .len(),
        1
    );
    assert!(assessed(&dynamic, "Head", duplicate).violations.is_empty());
}

#[test]
#[trace("TC-052", "FR-054-AC-7")]
fn an_unordered_channel_has_no_order_violation_and_a_head_or_tail_loss_is_unsupported() {
    let dynamic = Dynamic::with("workflow", 3);
    let any_order = assessed(
        &dynamic,
        "Block",
        vec![
            enqueue("m1", "k"),
            enqueue("m2", "k"),
            deliver("m2"),
            deliver("m1"),
        ],
    );
    assert!(any_order.violations.is_empty());
    for dynamic in [
        Dynamic::with("workflow", 3),
        Dynamic::with_unordered_tail_loss(),
    ] {
        let facts = history(&dynamic, "BagHead", vec![enqueue("m1", "k")]);
        let ChannelOutcome::Unsupported { cause, .. } = outcome(&dynamic, "BagHead", &facts) else {
            panic!("an unordered head or tail loss is unsupported");
        };
        assert_eq!(
            cause,
            UnsupportedCause::Assessment(AssessmentUnsupported::UnorderedLossSelector)
        );
    }
}

#[test]
#[trace("TC-052", "FR-054-AC-9")]
fn a_fact_naming_no_declared_channel_refuses_the_history() {
    use quire_protocol::dynamics::HistoryRefusal;
    let dynamic = Dynamic::with("workflow", 3);
    let role = dynamic.named("role", "S");
    let fact = ChannelFact {
        at: at(1),
        channel: role.clone(),
        event: enqueue("m1", "k"),
    };
    assert_eq!(
        assess_channels(dynamic.linked().declarations(), &[fact], |node| {
            dynamic.construct(node)
        }),
        Err(HistoryRefusal::UndeclaredSubject { node: role })
    );
}
