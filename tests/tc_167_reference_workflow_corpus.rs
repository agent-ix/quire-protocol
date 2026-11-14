// SPDX-License-Identifier: AGPL-3.0-or-later
//! TC-167: a finite observed history of one choreography (a replicated worker
//! role, a bounded queue, a fan-out and join, a bounded repeat) assessed on
//! every dynamic axis at once, twice, with the same result both times. A
//! conforming run holds on every axis; a faulty run is judged on each axis
//! separately (IT-005).

mod support {
    pub mod dynamic;
    pub mod handoff;
}

use ix_trace_rs::trace;
use quire_exact::Integer;
use quire_protocol::dynamics::{
    assess_channels, assess_joins, assess_loops, assess_roles, ChannelEvent, ChannelFact,
    ChannelOutcome, JoinEvent, JoinFact, JoinOutcome, LoopEvent, LoopFact, RoleEvent, RoleFact,
};
use support::dynamic::{at, instance, message, workflow, Dynamic};

/// One run's observed history.
struct Run {
    roles: Vec<RoleFact>,
    channels: Vec<ChannelFact>,
    joins: Vec<JoinFact>,
    loops: Vec<LoopFact>,
}

/// What the assessment says of a run, on each axis.
#[derive(Debug, Eq, PartialEq)]
struct Verdict {
    max_live: u64,
    within_bound: Option<bool>,
    channel_violations: usize,
    within_capacity: bool,
    join_settled: bool,
    join_permitted: Option<bool>,
    loop_iterations: u64,
    loop_within_maximum: Option<bool>,
}

/// `workers` spawn and act; `queued` messages enter the `Reject` queue
/// (capacity 2) with none delivered; `branches` of `JoinAll` complete and the
/// join settles; `iterations` of `Bounded` (maximum 3) run and the loop exits.
fn run(dynamic: &Dynamic, workers: u64, queued: u64, branches: &[&str], iterations: u64) -> Run {
    let mut position = 0;
    let mut next = || {
        position += 1;
        at(position)
    };
    let role = dynamic.named("role", "R");
    let mut roles = Vec::new();
    for worker in 0..workers {
        let name = format!("r{worker}");
        for event in [RoleEvent::Spawn, RoleEvent::Act] {
            roles.push(RoleFact {
                at: next(),
                role: role.clone(),
                workflow: workflow("w1"),
                instance: instance(&name),
                event,
            });
        }
    }
    let channel = dynamic.named("channel", "Reject");
    let channels = (0..queued)
        .map(|sent| ChannelFact {
            at: next(),
            channel: channel.clone(),
            event: ChannelEvent::Enqueue {
                message: message(&format!("m{sent}")),
                key: Box::from(&b"k"[..]),
            },
        })
        .collect();
    let join = dynamic.named("control", "JoinAll");
    let mut joins: Vec<JoinFact> = branches
        .iter()
        .map(|branch| JoinFact {
            at: next(),
            join: join.clone(),
            workflow: workflow("w1"),
            repeat_ordinals: Vec::new(),
            event: JoinEvent::BranchCompleted {
                branch: (*branch).into(),
            },
        })
        .collect();
    joins.push(JoinFact {
        at: next(),
        join: join.clone(),
        workflow: workflow("w1"),
        repeat_ordinals: Vec::new(),
        event: JoinEvent::Settled,
    });
    let control = dynamic.named("control", "Bounded");
    let ordinals: Vec<Integer> = Vec::new();
    let loops = (0..iterations)
        .map(|_| LoopEvent::Iteration)
        .chain([LoopEvent::Exit])
        .map(|event| LoopFact {
            at: next(),
            control: control.clone(),
            workflow: workflow("w1"),
            repeat_ordinals: ordinals.clone(),
            event,
        })
        .collect();
    Run {
        roles,
        channels,
        joins,
        loops,
    }
}

fn assess(dynamic: &Dynamic, history: &Run) -> Verdict {
    let declarations = dynamic.linked().declarations();
    let roles = assess_roles(declarations, &history.roles).expect("declared role");
    let [role] = roles.as_slice() else {
        panic!("one role assessment: {roles:?}");
    };
    let channels = assess_channels(declarations, &history.channels, |node| {
        dynamic.construct(node)
    })
    .expect("declared channel");
    let reject = dynamic.named("channel", "Reject");
    let channel = channels
        .iter()
        .find_map(|outcome| match outcome {
            ChannelOutcome::Assessed(found) if found.channel == reject => Some(found),
            _ => None,
        })
        .expect("the Reject channel is assessed");
    let joins = assess_joins(declarations, &history.joins, |node| dynamic.construct(node))
        .expect("declared join");
    let [JoinOutcome::Assessed(join)] = joins.as_slice() else {
        panic!("one assessed join: {joins:?}");
    };
    let loops = assess_loops(declarations, &history.loops).expect("declared loop");
    let [looped] = loops.as_slice() else {
        panic!("one loop assessment: {loops:?}");
    };
    Verdict {
        max_live: role.max_live,
        within_bound: role.within_bound,
        channel_violations: channel.violations.len(),
        within_capacity: channel.within_capacity,
        join_settled: join.settled,
        join_permitted: join.permitted,
        loop_iterations: looped.iterations,
        loop_within_maximum: looped.within_maximum,
    }
}

#[test]
#[trace("TC-167")]
fn a_conforming_run_holds_on_every_axis() {
    let dynamic = Dynamic::with("workflow", 3);
    let history = run(&dynamic, 2, 2, &["a", "b", "c"], 3);
    assert_eq!(
        assess(&dynamic, &history),
        Verdict {
            max_live: 2,
            within_bound: Some(true),
            channel_violations: 0,
            within_capacity: true,
            join_settled: true,
            join_permitted: Some(true),
            loop_iterations: 3,
            loop_within_maximum: Some(true),
        }
    );
}

#[test]
#[trace("TC-167")]
fn a_faulty_run_is_judged_on_each_axis_separately() {
    let dynamic = Dynamic::with("workflow", 3);
    let history = run(&dynamic, 4, 3, &["a", "b"], 4);
    assert_eq!(
        assess(&dynamic, &history),
        Verdict {
            max_live: 4,
            within_bound: Some(false),
            channel_violations: 1,
            within_capacity: false,
            join_settled: true,
            join_permitted: Some(false),
            loop_iterations: 4,
            loop_within_maximum: Some(false),
        }
    );
}

#[test]
#[trace("TC-167")]
fn replaying_the_same_finite_history_gives_the_same_result() {
    let dynamic = Dynamic::with("workflow", 3);
    for history in [
        run(&dynamic, 2, 2, &["a", "b", "c"], 3),
        run(&dynamic, 4, 3, &["a", "b"], 4),
    ] {
        assert_eq!(assess(&dynamic, &history), assess(&dynamic, &history));
    }
}
