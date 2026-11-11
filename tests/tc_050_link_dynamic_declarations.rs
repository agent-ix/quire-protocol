// SPDX-License-Identifier: AGPL-3.0-or-later
//! TC-050: linking role, queue, join and loop declarations (FR-050 to FR-052).

mod support {
    pub mod dynamic;
    pub mod handoff;
}

use ix_trace_rs::trace;
use qsl_package::protocol_view::{delivery, queue_capacity, Delivery, QueueCapacity};
use qsl_package::{RoleLifetime, RoleReplication};
use qsl_semantics::check::{ChannelOverflow, CheckedOutstanding, LossSelector};
use quire_protocol::declarations::{Capacity, JoinPolicy, Ordering};
use quire_protocol::link::link;
use support::dynamic::Dynamic;

const UNTIL: &str = "until(input = input and R = R)";

#[test]
#[trace("TC-050", "FR-050-AC-1")]
fn each_lifetime_kind_is_linked_with_its_checked_node() {
    for (written, max, expected) in [
        ("workflow", 3, RoleLifetime::Workflow),
        ("scope", 2, RoleLifetime::Scope),
        (UNTIL, 1, RoleLifetime::Until),
    ] {
        let dynamic = Dynamic::with(written, max);
        let role = dynamic.named("role", "R");
        let declared = dynamic
            .linked()
            .declarations()
            .role(&role)
            .expect("linked role");
        assert_eq!(declared.node, role);
        let RoleReplication::Replicated {
            population,
            lifetime,
            ..
        } = &declared.replication
        else {
            panic!("a replicated role, got {:?}", declared.replication);
        };
        assert_eq!(*lifetime, expected);
        assert_eq!(population.as_str(), "workers");
        let static_role = dynamic.named("role", "S");
        assert_eq!(
            dynamic
                .linked()
                .declarations()
                .role(&static_role)
                .map(|role| role.replication.clone()),
            Some(RoleReplication::Static)
        );
    }
}

#[test]
#[trace("TC-050", "FR-050-AC-2")]
fn the_authored_bound_is_carried_exactly() {
    for max in [1, 3, 7] {
        let dynamic = Dynamic::with("workflow", max);
        let role = dynamic.named("role", "R");
        let declared = dynamic.linked().declarations().role(&role).expect("linked");
        assert_eq!(declared.max(), Some(max));
    }
    // A static role has no population and so no bound; none is supplied.
    let dynamic = Dynamic::with("workflow", 3);
    let declared = dynamic
        .linked()
        .declarations()
        .role(&dynamic.named("role", "S"))
        .expect("linked");
    assert_eq!(declared.max(), None);
}

#[test]
#[trace("TC-050", "FR-050-AC-3")]
fn linking_twice_is_equal_and_mints_no_instance() {
    let dynamic = Dynamic::with("workflow", 3);
    let again = link(dynamic.package(), dynamic.clause()).expect("links again");
    assert_eq!(dynamic.linked().declarations(), again.declarations());
    // A role declaration holds the checked node and the authored group only:
    // one entry per authored role, none per instance.
    assert_eq!(dynamic.linked().declarations().roles().len(), 2);
}

#[test]
#[trace("TC-050", "FR-050-AC-4")]
fn a_replicated_role_is_one_declaration() {
    let dynamic = Dynamic::with("workflow", 3);
    let declarations = dynamic.linked().declarations();
    let role = dynamic.named("role", "R");
    let copies = declarations
        .roles()
        .iter()
        .filter(|declared| declared.node == role)
        .count();
    assert_eq!(copies, 1);
    assert_eq!(declarations.roles().len(), dynamic.linked().roles().len());
}

struct Authored {
    name: &'static str,
    delivery: Delivery,
    fifo: bool,
    capacity: Capacity,
    overflow: ChannelOverflow,
}

fn authored() -> Vec<Authored> {
    let loss = |selector| ChannelOverflow::Loss(selector);
    vec![
        Authored {
            name: "Head",
            delivery: Delivery::AtLeastOnce,
            fifo: true,
            capacity: Capacity::Literal(4),
            overflow: loss(LossSelector::Head),
        },
        Authored {
            name: "Reject",
            delivery: Delivery::ExactlyOncePremise,
            fifo: true,
            capacity: Capacity::Literal(2),
            overflow: ChannelOverflow::Reject,
        },
        Authored {
            name: "Tail",
            delivery: Delivery::AtMostOnce,
            fifo: true,
            capacity: Capacity::Literal(2),
            overflow: loss(LossSelector::Tail),
        },
        Authored {
            name: "New",
            delivery: Delivery::Unknown,
            fifo: true,
            capacity: Capacity::Literal(2),
            overflow: loss(LossSelector::New),
        },
        Authored {
            name: "Block",
            delivery: Delivery::AtLeastOnce,
            fifo: false,
            capacity: Capacity::Literal(3),
            overflow: ChannelOverflow::Block,
        },
        Authored {
            name: "BagNew",
            delivery: Delivery::Unknown,
            fifo: false,
            capacity: Capacity::Literal(2),
            overflow: loss(LossSelector::New),
        },
        Authored {
            name: "BagHead",
            delivery: Delivery::Unknown,
            fifo: false,
            capacity: Capacity::Literal(2),
            overflow: loss(LossSelector::Head),
        },
        Authored {
            name: "Symbolic",
            delivery: Delivery::Unknown,
            fifo: false,
            capacity: Capacity::Symbolic("Bound".into()),
            overflow: ChannelOverflow::Reject,
        },
    ]
}

#[test]
#[trace("TC-050", "FR-051-AC-1")]
fn every_channel_field_equals_the_checked_node_and_the_owners_accessors() {
    let dynamic = Dynamic::with("workflow", 3);
    let declarations = dynamic.linked().declarations();
    assert_eq!(declarations.channels().len(), authored().len());
    for expected in authored() {
        let id = dynamic.named("channel", expected.name);
        let declared = declarations.channel(&id).expect("linked channel");
        let node = dynamic
            .package()
            .graph()
            .nodes
            .iter()
            .find(|n| n.node_id == id)
            .unwrap();
        // QSL's own accessors, read from the same checked nodes.
        assert_eq!(Ok(declared.delivery), delivery(node), "{}", expected.name);
        assert_eq!(declared.delivery, expected.delivery, "{}", expected.name);
        let queue = dynamic
            .package()
            .graph()
            .nodes
            .iter()
            .find(|n| n.node_id == declared.queue)
            .unwrap();
        match (queue_capacity(queue), &declared.capacity) {
            (Ok(QueueCapacity::Literal(owner)), Capacity::Literal(linked)) => {
                assert_eq!(owner, *linked, "{}", expected.name);
            }
            (Ok(QueueCapacity::Symbolic(owner)), Capacity::Symbolic(linked)) => {
                assert_eq!(owner, &**linked, "{}", expected.name);
            }
            other => panic!("{}: capacity {other:?}", expected.name),
        }
        assert_eq!(declared.capacity, expected.capacity, "{}", expected.name);
        assert_eq!(declared.overflow, expected.overflow, "{}", expected.name);
        assert_eq!(
            matches!(declared.ordering, Ordering::Fifo { .. }),
            expected.fifo,
            "{}",
            expected.name
        );
    }
}

#[test]
#[trace("TC-050", "FR-051-AC-2")]
fn a_symbolic_capacity_stays_its_authored_name() {
    let dynamic = Dynamic::with("workflow", 3);
    let id = dynamic.named("channel", "Symbolic");
    let declared = dynamic.linked().declarations().channel(&id).unwrap();
    assert_eq!(declared.capacity, Capacity::Symbolic("Bound".into()));
}

fn join_of(dynamic: &Dynamic, name: &str) -> quire_protocol::declarations::JoinDeclaration {
    let id = dynamic.named("control", name);
    dynamic
        .linked()
        .declarations()
        .join(&id)
        .cloned()
        .unwrap_or_else(|| panic!("join {name}"))
}

#[test]
#[trace("TC-050", "FR-051-AC-3")]
fn every_channel_links_with_the_overflow_policy_it_authors() {
    let dynamic = Dynamic::with("workflow", 3);
    let policies: Vec<ChannelOverflow> = dynamic
        .linked()
        .declarations()
        .channels()
        .iter()
        .map(|channel| channel.overflow)
        .collect();
    for expected in [
        ChannelOverflow::Block,
        ChannelOverflow::Reject,
        ChannelOverflow::Loss(LossSelector::Head),
        ChannelOverflow::Loss(LossSelector::Tail),
        ChannelOverflow::Loss(LossSelector::New),
    ] {
        assert!(policies.contains(&expected), "{expected:?}");
    }
    assert_eq!(policies.len(), authored().len());
}

#[test]
#[trace("TC-050", "FR-052-AC-1")]
fn each_join_policy_and_threshold_is_linked_with_its_control() {
    let dynamic = Dynamic::with("workflow", 3);
    let all = join_of(&dynamic, "JoinAll");
    assert_eq!(all.control, dynamic.named("control", "JoinAll"));
    assert_eq!(all.policy, JoinPolicy::All);
    assert_eq!(all.threshold(), Some(3));
    assert_eq!(all.outstanding, Some(CheckedOutstanding::Continue));
    let any = join_of(&dynamic, "AnyContinue");
    assert_eq!(
        (any.policy.clone(), any.threshold()),
        (JoinPolicy::Any, Some(1))
    );
    let quorum = join_of(&dynamic, "QuorumCancel");
    assert_eq!(quorum.policy, JoinPolicy::Quorum(2));
    assert_eq!(quorum.threshold(), Some(2));
    assert_eq!(quorum.outstanding, Some(CheckedOutstanding::Cancel));
    assert_eq!(
        quorum.joined.iter().map(|l| &**l).collect::<Vec<_>>(),
        ["a", "b", "c"]
    );
}

#[test]
#[trace("TC-050", "FR-052-AC-2")]
fn a_predicate_join_and_an_unruled_join_link_as_declared() {
    let dynamic = Dynamic::with("workflow", 3);
    let predicate = join_of(&dynamic, "Predicate");
    let JoinPolicy::Predicate(function) = &predicate.policy else {
        panic!("a predicate policy, got {:?}", predicate.policy);
    };
    assert!(
        dynamic.package().term(function).is_some(),
        "the function node, unevaluated"
    );
    assert_eq!(predicate.threshold(), None);
    assert_eq!(predicate.outstanding, None);
    for name in ["AnyUnruled", "QuorumUnruled", "AllUnruled"] {
        assert_eq!(join_of(&dynamic, name).outstanding, None, "{name}");
    }
}

#[test]
#[trace("TC-050", "FR-052-AC-3")]
fn a_loop_links_its_invariant_and_variant_by_node_without_checking_them() {
    let dynamic = Dynamic::with("workflow", 3);
    let bounded = dynamic
        .linked()
        .declarations()
        .loop_at(&dynamic.named("control", "Bounded"))
        .cloned()
        .expect("bounded loop");
    assert_eq!(bounded.maximum, Some(3));
    for node in [&bounded.invariant, &bounded.variant] {
        let node = node.as_ref().expect("declared");
        assert!(dynamic.package().term(node).is_some());
    }
    assert_ne!(bounded.invariant, bounded.variant);
    let free = dynamic
        .linked()
        .declarations()
        .loop_at(&dynamic.named("control", "Free"))
        .cloned()
        .expect("free loop");
    assert_eq!(
        (free.maximum, free.invariant, free.variant),
        (None, None, None)
    );
}
