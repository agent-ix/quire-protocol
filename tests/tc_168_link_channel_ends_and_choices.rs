// SPDX-License-Identifier: AGPL-3.0-or-later
//! TC-168: a linked channel carries its sender, receiver and message type, and
//! a linked choice its owner and visible set, as QSL's read view publishes
//! them; the dispatcher forwards the channel ends it linked (FR-051-AC-4,
//! FR-052-AC-4, FR-151-AC-7).

mod support {
    pub mod dynamic;
    pub mod handoff;
    pub mod projection;
    pub mod standin;
}

use ix_trace_rs::trace;
use qsl_package::protocol_view::{channel_endpoints, choice_visibility};
use qsl_replay::{SettlementBasis, Truth};
use qsl_semantics::check::Capability;
use quire_protocol::dispatch::{dispatch, ProtocolClaimKind};
use support::dynamic::Dynamic;
use support::projection::{worked, CANCELLED, PICK};
use support::standin::{always, request, serve, Boundary, Sentinel};

#[test]
#[trace("TC-168", "FR-051-AC-4")]
fn a_linked_channel_carries_its_ends_and_message_type_as_qsl_reads_them() {
    let dynamic = worked(PICK, CANCELLED);
    let declarations = dynamic.linked().declarations();
    for (channel, sender, receiver) in [
        ("Order", "Buyer", "Seller"),
        ("Pick", "Seller", "Warehouse"),
        ("Cancelled", "Seller", "Warehouse"),
    ] {
        let node = dynamic.named("channel", channel);
        let declared = declarations.channel(&node).expect("a declared channel");
        assert_eq!(declared.sender, dynamic.named("role", sender));
        assert_eq!(declared.receiver, dynamic.named("role", receiver));
        let graph_node = dynamic
            .package()
            .graph()
            .nodes
            .iter()
            .find(|candidate| candidate.node_id == node)
            .expect("the channel node");
        let ends = channel_endpoints(graph_node).expect("QSL reads the ends");
        assert_eq!(&declared.message_type, ends.message_type);
    }
    let order = declarations
        .channel(&dynamic.named("channel", "Order"))
        .expect("Order");
    let pick = declarations
        .channel(&dynamic.named("channel", "Pick"))
        .expect("Pick");
    assert_ne!(order.message_type, pick.message_type);
}

#[test]
#[trace("TC-168", "FR-052-AC-4")]
fn a_linked_choice_carries_its_owner_and_visible_set() {
    let dynamic = worked(PICK, CANCELLED);
    let declarations = dynamic.linked().declarations();
    let [choice] = declarations.choices() else {
        panic!("one choice: {:?}", declarations.choices());
    };
    assert_eq!(choice.owner, dynamic.named("role", "Seller"));
    let graph_node = dynamic
        .package()
        .graph()
        .nodes
        .iter()
        .find(|candidate| candidate.node_id == choice.control)
        .expect("the choice node");
    let evidence = choice_visibility(graph_node).expect("QSL reads the choice");
    assert_eq!(&choice.control, evidence.choice);
    assert_eq!(&choice.owner, evidence.owner);
    assert_eq!(choice.visible.len(), 1);
    assert_eq!(
        choice.visible.iter().collect::<Vec<_>>(),
        evidence.visible,
        "the visible nodes, in QSL's order"
    );
    assert_eq!(choice.key, dynamic.key(&choice.control));
}

#[test]
#[trace("TC-168", "FR-151-AC-7")]
fn the_backend_receives_each_channels_ends_and_message_type_as_linked() {
    let dynamic = Dynamic::with("workflow", 3);
    let mut boundary = Boundary::new();
    let calls = serve(
        &mut boundary,
        "backend",
        &[Capability::Realizability],
        always(Sentinel::new(
            "ok",
            Truth::Satisfied,
            SettlementBasis::ClosedScope,
        )),
    );
    let asked = request(
        &dynamic,
        ProtocolClaimKind::Realizability,
        &["Head", "Reject"],
    );
    dispatch(&mut boundary, dynamic.linked().declarations(), &asked);
    let received = calls.all();
    let [call] = received.as_slice() else {
        panic!("one call");
    };
    let declarations = dynamic.linked().declarations();
    for (index, node) in asked.fragment.channels.iter().enumerate() {
        let linked = declarations.channel(node).expect("linked");
        assert_eq!(call.channels[index].sender, linked.sender);
        assert_eq!(call.channels[index].receiver, linked.receiver);
        assert_eq!(call.channels[index].message_type, linked.message_type);
        let graph_node = dynamic
            .package()
            .graph()
            .nodes
            .iter()
            .find(|candidate| &candidate.node_id == node)
            .expect("the channel node");
        let ends = channel_endpoints(graph_node).expect("QSL reads the ends");
        assert_eq!(&call.channels[index].message_type, ends.message_type);
    }
    assert_eq!(
        call.channels[0].sender,
        dynamic.named("role", "S"),
        "Head is S to S"
    );
    assert_eq!(call.channels[1].receiver, dynamic.named("role", "R"));
}
