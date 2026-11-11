// SPDX-License-Identifier: AGPL-3.0-or-later
//! TC-051: an observed history against role lifetime and population bound
//! (FR-053). The `until(p)` predicate is never evaluated: the history states
//! when the instance retired, and that is the only end read.
//!
//! A static role (FR-053-AC-4) has no replication, no spawn and no population
//! bound; its activity is judged against nothing.

mod support {
    pub mod dynamic;
    pub mod handoff;
}

use ix_trace_rs::trace;
use qsl_package::protocol_view::{CausalEdge, PortRef};
use quire_contract_model::CheckedNodeId;
use quire_protocol::dynamics::{
    assess_roles, HistoryRefusal, LifetimeViolation, PopulationViolation, RoleAssessment,
    RoleEvent, RoleFact,
};
use support::dynamic::{at, instance, workflow, Dynamic};

const UNTIL: &str = "until(input = input and R = R)";

fn fact(position: u64, who: &str, event: RoleEvent, dynamic: &Dynamic, role: &str) -> RoleFact {
    RoleFact {
        at: at(position),
        role: dynamic.named("role", role),
        workflow: workflow("w1"),
        instance: instance(who),
        event,
    }
}

fn assess(dynamic: &Dynamic, facts: &[RoleFact], role: &str) -> RoleAssessment {
    let id = dynamic.named("role", role);
    assess_roles(dynamic.linked().declarations(), facts)
        .expect("declared roles")
        .into_iter()
        .find(|assessment| assessment.role == id)
        .expect("the role is assessed")
}

fn r(position: u64, who: &str, event: RoleEvent, dynamic: &Dynamic) -> RoleFact {
    fact(position, who, event, dynamic, "R")
}

#[test]
#[trace("TC-051", "FR-053-AC-1")]
fn activity_inside_the_lifetime_and_within_the_bound_reports_nothing() {
    let dynamic = Dynamic::with("workflow", 3);
    let facts = [
        r(1, "i1", RoleEvent::Spawn, &dynamic),
        r(2, "i1", RoleEvent::Act, &dynamic),
        r(3, "i2", RoleEvent::Spawn, &dynamic),
        r(4, "i2", RoleEvent::Act, &dynamic),
    ];
    let assessment = assess(&dynamic, &facts, "R");
    assert!(assessment.lifetime_violations.is_empty());
    assert_eq!(assessment.population_violation, None);
    assert_eq!(assessment.max_live, 2);
    assert_eq!(assessment.within_bound, Some(true));
    assert!(assessment.instances.values().all(|inside| *inside));
    assert_eq!(assessment.instances.len(), 2);
}

#[test]
#[trace("TC-051", "FR-053-AC-2")]
fn activity_before_spawn_after_retire_and_a_retire_before_spawn_are_violations() {
    let before = Dynamic::with("workflow", 3);
    let facts = [
        r(1, "i1", RoleEvent::Act, &before),
        r(2, "i1", RoleEvent::Spawn, &before),
    ];
    let assessment = assess(&before, &facts, "R");
    assert_eq!(
        assessment.lifetime_violations,
        [LifetimeViolation::BeforeSpawn {
            instance: instance("i1"),
            at: at(1)
        }]
    );
    assert_eq!(assessment.instances.get(&instance("i1")), Some(&false));

    let until = Dynamic::with(UNTIL, 1);
    let facts = [
        r(1, "i1", RoleEvent::Spawn, &until),
        r(2, "i1", RoleEvent::Retire, &until),
        r(3, "i1", RoleEvent::Act, &until),
    ];
    assert_eq!(
        assess(&until, &facts, "R").lifetime_violations,
        [LifetimeViolation::AfterEnd {
            instance: instance("i1"),
            at: at(3)
        }]
    );

    let facts = [
        r(1, "i1", RoleEvent::Retire, &until),
        r(2, "i1", RoleEvent::Spawn, &until),
    ];
    assert_eq!(
        assess(&until, &facts, "R").lifetime_violations,
        [LifetimeViolation::RetireBeforeSpawn {
            instance: instance("i1"),
            at: at(1)
        }]
    );
}

#[test]
#[trace("TC-051", "FR-053-AC-3")]
fn reaching_the_bound_is_clean_and_one_more_names_role_bound_and_count() {
    let dynamic = Dynamic::with("workflow", 2);
    let spawn = |position, who| r(position, who, RoleEvent::Spawn, &dynamic);
    let at_bound = assess(&dynamic, &[spawn(1, "i1"), spawn(2, "i2")], "R");
    assert_eq!(at_bound.population_violation, None);
    assert_eq!(at_bound.within_bound, Some(true));

    let over = assess(
        &dynamic,
        &[spawn(1, "i1"), spawn(2, "i2"), spawn(3, "i3")],
        "R",
    );
    assert_eq!(
        over.population_violation,
        Some(PopulationViolation {
            role: dynamic.named("role", "R"),
            workflow: workflow("w1"),
            bound: 2,
            observed: 3,
        })
    );
    assert_eq!(over.within_bound, Some(false));
}

#[test]
#[trace("TC-051", "FR-053-AC-4")]
fn a_static_role_has_no_bound_and_its_activity_is_never_a_lifetime_violation() {
    let dynamic = Dynamic::with("workflow", 3);
    let act = |position, who: &str| fact(position, who, RoleEvent::Act, &dynamic, "S");
    let assessment = assess(&dynamic, &[act(1, "s1"), act(2, "s1"), act(3, "s2")], "S");
    assert_eq!(assessment.bound, None);
    assert_eq!(assessment.within_bound, None);
    assert_eq!(assessment.population_violation, None);
    assert!(assessment.lifetime_violations.is_empty());
    assert!(assessment.instances.values().all(|inside| *inside));
}

#[test]
#[trace("TC-051", "FR-053-AC-10")]
fn a_spawn_retire_or_end_fact_for_a_static_role_refuses_the_history() {
    let dynamic = Dynamic::with("workflow", 3);
    // QSL never spawns a static role: a lifecycle fact for one is not a history.
    let spawn = fact(1, "s1", RoleEvent::Spawn, &dynamic, "S");
    assert_eq!(
        assess_roles(dynamic.linked().declarations(), &[spawn]),
        Err(HistoryRefusal::StaticRoleLifecycle {
            role: dynamic.named("role", "S")
        })
    );
}

/// Whether any port of `to` is reachable from any port of `from` over the
/// causal edges, following each edge from its source port to its target port.
fn reaches(edges: &[CausalEdge], from: &CheckedNodeId, to: &CheckedNodeId) -> bool {
    let mut frontier: Vec<&PortRef> = edges
        .iter()
        .map(|edge| &edge.from)
        .filter(|port| &port.node == from)
        .collect();
    let mut seen: Vec<&PortRef> = Vec::new();
    while let Some(port) = frontier.pop() {
        if seen.contains(&port) {
            continue;
        }
        seen.push(port);
        for edge in edges.iter().filter(|edge| &edge.from == port) {
            if &edge.to.node == to {
                return true;
            }
            frontier.push(&edge.to);
        }
    }
    false
}

#[test]
#[trace("TC-051", "FR-053-AC-5")]
fn a_permitted_reordering_of_concurrent_observations_changes_no_value() {
    let dynamic = Dynamic::with("workflow", 2);
    // The authored causal relation, read from the linked protocol: the two
    // branches of the `JoinAll` parallel are concurrent, neither reaches the
    // other, so observations made in them may be presented in either order.
    let edges = dynamic.linked().edges();
    let left = dynamic.named("control", "JoinAlla");
    let right = dynamic.named("control", "JoinAllb");
    assert!(!reaches(edges, &left, &right) && !reaches(edges, &right, &left));
    // The same relation does order a branch before the join that follows it.
    assert!(reaches(
        edges,
        &left,
        &dynamic.named("control", "AnyContinue")
    ));
    // Instance i1 acts in the left branch and i2 in the right: their
    // observations are the concurrent pair whose positions are exchanged.
    let at_positions = |order: [u64; 6]| {
        vec![
            r(order[0], "i1", RoleEvent::Spawn, &dynamic),
            r(order[1], "i2", RoleEvent::Spawn, &dynamic),
            r(order[2], "i3", RoleEvent::Spawn, &dynamic),
            r(order[3], "i1", RoleEvent::Act, &dynamic),
            r(order[4], "i2", RoleEvent::Act, &dynamic),
            r(order[5], "i3", RoleEvent::Act, &dynamic),
        ]
    };
    let reference = assess(&dynamic, &at_positions([1, 2, 3, 4, 5, 6]), "R");
    assert!(reference.population_violation.is_some());
    assert!(reference.lifetime_violations.is_empty());
    for order in [[2, 1, 3, 4, 5, 6], [1, 2, 3, 6, 5, 4], [1, 3, 2, 4, 6, 5]] {
        let assessment = assess(&dynamic, &at_positions(order), "R");
        assert_eq!(assessment.max_live, reference.max_live, "{order:?}");
        assert_eq!(
            assessment.population_violation,
            reference.population_violation
        );
        assert_eq!(
            assessment.lifetime_violations,
            reference.lifetime_violations
        );
        assert_eq!(assessment.instances, reference.instances);
    }
    // A reordering the relation does not permit does change the value: an
    // instance's act moved before its own spawn.
    let broken = assess(&dynamic, &at_positions([4, 2, 3, 1, 5, 6]), "R");
    assert_eq!(broken.lifetime_violations.len(), 1);
}

#[test]
#[trace("TC-051", "FR-053-AC-11")]
fn population_and_workflow_end_are_judged_per_workflow_instance() {
    let dynamic = Dynamic::with("workflow", 2);
    let in_workflow = |position, flow: &str, who: &str, event| RoleFact {
        at: at(position),
        role: dynamic.named("role", "R"),
        workflow: workflow(flow),
        instance: instance(who),
        event,
    };
    // Two workflow instances, two live instances each: under `max 2` in each,
    // four together.
    let facts = [
        in_workflow(1, "w1", "a1", RoleEvent::Spawn),
        in_workflow(2, "w1", "a2", RoleEvent::Spawn),
        in_workflow(3, "w2", "b1", RoleEvent::Spawn),
        in_workflow(4, "w2", "b2", RoleEvent::Spawn),
    ];
    let assessed = assess_roles(dynamic.linked().declarations(), &facts).expect("declared");
    let for_r: Vec<_> = assessed
        .iter()
        .filter(|a| a.role == dynamic.named("role", "R"))
        .collect();
    assert_eq!(for_r.len(), 2);
    assert!(for_r
        .iter()
        .all(|a| a.max_live == 2 && a.population_violation.is_none()));
    // A third instance in w1 alone is a violation, and names w1.
    let mut over = facts.to_vec();
    over.push(in_workflow(5, "w1", "a3", RoleEvent::Spawn));
    let violations: Vec<_> = assess_roles(dynamic.linked().declarations(), &over)
        .expect("declared")
        .into_iter()
        .filter_map(|a| a.population_violation)
        .collect();
    assert_eq!(
        violations,
        [PopulationViolation {
            role: dynamic.named("role", "R"),
            workflow: workflow("w1"),
            bound: 2,
            observed: 3,
        }]
    );
    // The end of w1's workflow ends w1's instances only: w2's instance acts on.
    let ended = [
        in_workflow(1, "w1", "x", RoleEvent::Spawn),
        in_workflow(2, "w2", "x", RoleEvent::Spawn),
        in_workflow(3, "w1", "x", RoleEvent::WorkflowEnd),
        in_workflow(4, "w2", "x", RoleEvent::Act),
        in_workflow(5, "w1", "x", RoleEvent::Act),
    ];
    let per_workflow: Vec<_> = assess_roles(dynamic.linked().declarations(), &ended)
        .expect("declared")
        .into_iter()
        .filter(|a| a.role == dynamic.named("role", "R"))
        .map(|a| (a.workflow, a.lifetime_violations.len()))
        .collect();
    assert_eq!(per_workflow, [(workflow("w1"), 1), (workflow("w2"), 0)]);
}

#[test]
#[trace("TC-051", "FR-053-AC-6")]
fn each_lifetime_kind_ends_at_its_own_observation() {
    let cases = [
        ("workflow", RoleEvent::WorkflowEnd),
        ("scope", RoleEvent::ScopeEnd),
        (UNTIL, RoleEvent::Retire),
    ];
    for (written, end) in cases {
        let dynamic = Dynamic::with(written, 3);
        let facts = [
            r(1, "i1", RoleEvent::Spawn, &dynamic),
            r(2, "i1", RoleEvent::Act, &dynamic),
            r(3, "i1", end.clone(), &dynamic),
            r(4, "i1", RoleEvent::Act, &dynamic),
        ];
        assert_eq!(
            assess(&dynamic, &facts, "R").lifetime_violations,
            [LifetimeViolation::AfterEnd {
                instance: instance("i1"),
                at: at(4)
            }],
            "{written}"
        );
    }
    // The end of another lifetime kind ends nothing: a workflow role is not
    // ended by a `retire`, and a scope role not by the end of the workflow.
    let dynamic = Dynamic::with("workflow", 3);
    let facts = [
        r(1, "i1", RoleEvent::Spawn, &dynamic),
        r(2, "i1", RoleEvent::Retire, &dynamic),
        r(3, "i1", RoleEvent::ScopeEnd, &dynamic),
        r(4, "i1", RoleEvent::Act, &dynamic),
    ];
    assert!(assess(&dynamic, &facts, "R").lifetime_violations.is_empty());
}

#[test]
#[trace("TC-051", "FR-053-AC-7")]
fn a_compensation_attempt_before_the_scope_ends_is_not_a_violation() {
    let dynamic = Dynamic::with("scope", 2);
    // Step 2 is the last branch step; the compensation attempt at 4 still
    // runs under the open registration; QSL's RR-4 step ends the scope at 5.
    let facts = [
        r(1, "i1", RoleEvent::Spawn, &dynamic),
        r(2, "i1", RoleEvent::Act, &dynamic),
        r(4, "i1", RoleEvent::Act, &dynamic),
        r(5, "i1", RoleEvent::ScopeEnd, &dynamic),
    ];
    assert!(assess(&dynamic, &facts, "R").lifetime_violations.is_empty());
    let mut later = facts.to_vec();
    later.push(r(6, "i1", RoleEvent::Act, &dynamic));
    assert_eq!(assess(&dynamic, &later, "R").lifetime_violations.len(), 1);
}

#[test]
#[trace("TC-051", "FR-053-AC-8")]
fn an_until_instance_acting_before_its_observed_retire_is_clean() {
    let dynamic = Dynamic::with(UNTIL, 1);
    // `p` holds from position 2 on in this history, but the evaluator reads
    // only the observed `retire` at 5.
    let facts = [
        r(1, "i1", RoleEvent::Spawn, &dynamic),
        r(3, "i1", RoleEvent::Act, &dynamic),
        r(4, "i1", RoleEvent::Act, &dynamic),
        r(5, "i1", RoleEvent::Retire, &dynamic),
    ];
    let assessment = assess(&dynamic, &facts, "R");
    assert!(assessment.lifetime_violations.is_empty());
    assert_eq!(assessment.max_live, 1);
}

#[test]
#[trace("TC-051", "FR-053-AC-9")]
fn a_fact_naming_no_declared_role_refuses_the_history() {
    let dynamic = Dynamic::with("workflow", 3);
    let channel = dynamic.named("channel", "Head");
    let fact = RoleFact {
        at: at(1),
        role: channel.clone(),
        workflow: workflow("w1"),
        instance: instance("i1"),
        event: RoleEvent::Act,
    };
    assert_eq!(
        assess_roles(dynamic.linked().declarations(), &[fact]),
        Err(HistoryRefusal::UndeclaredSubject { node: channel })
    );
}
