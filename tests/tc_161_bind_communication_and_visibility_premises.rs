// SPDX-License-Identifier: AGPL-3.0-or-later
//! TC-161: communication and visibility premises bound to projection and
//! realizability dispositions (FR-151).

mod support {
    pub mod dynamic;
    pub mod handoff;
    pub mod standin;
}

use ix_trace_rs::trace;
use qsl_eval::simulation::RequiresBound;
use qsl_foundation::bound::{DomainKey, DomainKind};
use qsl_package::protocol_view::Delivery;
use qsl_replay::{InconclusiveCause, SettlementBasis, Truth};
use qsl_semantics::check::Capability;
use qsl_semantics::family::ClaimExtent;
use quire_protocol::declarations::Ordering;
use quire_protocol::dispatch::{
    dispatch, dispatch_claims, ChannelPremise, ClaimDisposition, ClaimRequest, DispositionRefusal,
    ProtocolClaimKind, ProtocolOutcome,
};
use quire_protocol::disposition::{ClaimSettlement, Disposition, QslReport};
use support::dynamic::Dynamic;
use support::standin::{always, request, rest, serve, Boundary, Rest, Sentinel};

type Change = Box<dyn Fn(ChannelPremise<Rest>) -> ChannelPremise<Rest>>;

const KIND: ProtocolClaimKind = ProtocolClaimKind::Realizability;

fn sentinel() -> Sentinel {
    Sentinel::new("ok", Truth::Satisfied, SettlementBasis::ClosedScope)
}

fn serving() -> (Boundary, support::standin::Calls) {
    let mut boundary = Boundary::new();
    let calls = serve(
        &mut boundary,
        "backend",
        &[Capability::Realizability],
        always(sentinel()),
    );
    (boundary, calls)
}

#[test]
#[trace("TC-161", "FR-151-AC-1")]
fn the_backend_receives_and_the_disposition_retains_the_premises_as_authored() {
    let dynamic = Dynamic::with("workflow", 3);
    let (mut boundary, calls) = serving();
    let asked = request(&dynamic, KIND, &["Head", "Reject"]);
    let outcome = dispatch(&mut boundary, dynamic.linked().declarations(), &asked);
    let received = calls.all();
    let [call] = received.as_slice() else {
        panic!("one call");
    };
    let declarations = dynamic.linked().declarations();
    for (index, node) in asked.fragment.channels.iter().enumerate() {
        assert_eq!(Some(&call.channels[index]), declarations.channel(node));
        assert_eq!(
            call.request.premises.channels[node],
            asked.premises.channels[node]
        );
    }
    for (index, node) in asked.fragment.roles.iter().enumerate() {
        assert_eq!(Some(&call.roles[index]), declarations.role(node));
        assert_eq!(
            call.request.premises.roles[node],
            asked.premises.roles[node]
        );
    }
    let ProtocolOutcome::Settled(ClaimSettlement::Result { premises, .. }) = outcome else {
        panic!("{outcome:?}");
    };
    assert_eq!(&premises, call);
}

#[test]
#[trace("TC-161", "FR-151-AC-2")]
fn requests_differing_in_one_premise_component_differ_in_identity_and_provenance() {
    let dynamic = Dynamic::with("workflow", 3);
    let base = request(&dynamic, KIND, &["Head"]);
    let head = dynamic.named("channel", "Head");
    let declared = dynamic
        .linked()
        .declarations()
        .channel(&head)
        .expect("Head")
        .clone();
    let other_delivery = if declared.delivery == Delivery::AtMostOnce {
        Delivery::AtLeastOnce
    } else {
        Delivery::AtMostOnce
    };
    let components: [Change; 5] = [
        Box::new(|c| ChannelPremise {
            unread: Rest {
                sender: "other",
                ..c.unread
            },
            ..c
        }),
        Box::new(|c| ChannelPremise {
            unread: Rest {
                receiver: "other",
                ..c.unread
            },
            ..c
        }),
        Box::new(|c| ChannelPremise {
            unread: Rest {
                message_type: "other",
                ..c.unread
            },
            ..c
        }),
        Box::new(|c| ChannelPremise {
            ordering: Ordering::Unordered,
            ..c
        }),
        Box::new(move |c| ChannelPremise {
            delivery: other_delivery,
            ..c
        }),
    ];
    let (mut boundary, _) = serving();
    let base_binding = match dispatch(&mut boundary, dynamic.linked().declarations(), &base) {
        ProtocolOutcome::Settled(ClaimSettlement::Result { premises, .. }) => premises,
        other => panic!("{other:?}"),
    };
    let mut retained = vec![base_binding];
    for (index, change) in components.iter().enumerate() {
        let mut other = base.clone();
        let premise = other.premises.channels.get_mut(&head).expect("premise");
        *premise = change(premise.clone());
        assert_ne!(base, other, "component {index}");
        // Only the components the read view does not publish can differ from
        // the package and still be bound (FR-151-AC-8 refuses the others).
        if index < 3 {
            let (mut boundary, _) = serving();
            let settled = dispatch(&mut boundary, dynamic.linked().declarations(), &other);
            let ProtocolOutcome::Settled(ClaimSettlement::Result { premises, .. }) = settled else {
                panic!("{settled:?}");
            };
            assert_eq!(premises.request, other);
            retained.push(premises);
        }
    }
    for (left, first) in retained.iter().enumerate() {
        for second in &retained[left + 1..] {
            assert_ne!(first, second);
        }
    }
}

#[test]
#[trace("TC-161", "FR-151-AC-3")]
fn an_omitted_premise_refuses_only_that_disposition_and_calls_no_backend() {
    let dynamic = Dynamic::with("workflow", 3);
    let head = dynamic.named("channel", "Head");
    let role = dynamic.named("role", "S");
    let complete = request(&dynamic, KIND, &["Head"]);
    let mut no_channel = complete.clone();
    no_channel.premises.channels.remove(&head);
    let mut no_role = complete.clone();
    no_role.premises.roles.remove(&role);
    let (mut boundary, calls) = serving();
    let declarations = dynamic.linked().declarations();
    let settled = dispatch_claims(
        &mut boundary,
        declarations,
        &[],
        &[
            ClaimRequest::GlobalConformance("conformant"),
            ClaimRequest::Protocol(no_channel),
            ClaimRequest::Protocol(no_role),
        ],
    );
    assert_eq!(
        settled[1],
        ClaimDisposition::Protocol(ProtocolOutcome::Refused(
            DispositionRefusal::AbsentChannelPremise(head)
        ))
    );
    assert_eq!(
        settled[2],
        ClaimDisposition::Protocol(ProtocolOutcome::Refused(
            DispositionRefusal::AbsentRoleInternality(role)
        ))
    );
    assert_eq!(
        settled[0],
        ClaimDisposition::GlobalConformance("conformant")
    );
    assert_eq!(calls.len(), 0);
}

#[test]
#[trace("TC-161", "FR-151-AC-4")]
fn an_external_role_reaches_the_backend_as_external_and_nothing_more() {
    let dynamic = Dynamic::with("workflow", 3);
    let mut asked = request(&dynamic, KIND, &["Head"]);
    let role = dynamic.named("role", "R");
    asked.premises.roles.insert(role.clone(), "external");
    let (mut boundary, calls) = serving();
    dispatch(&mut boundary, dynamic.linked().declarations(), &asked);
    let received = calls.all();
    let [call] = received.as_slice() else {
        panic!("one call");
    };
    assert_eq!(call.request.premises.roles[&role], "external");
    assert_eq!(call.request.premises.roles.len(), 2);
    assert_eq!(call.request.premises, asked.premises);
}

#[test]
#[trace("TC-161", "FR-151-AC-5")]
fn inconclusive_and_requires_bound_are_carried_and_hold_no_decided_truth() {
    let dynamic = Dynamic::with("workflow", 3);
    let asked = request(&dynamic, KIND, &["Head"]);
    let cause = InconclusiveCause::BoundReached { depth: 3 };
    // The backend's own `pending` is carried; a decided `satisfied` beside an
    // Inconclusive verdict is not.
    for (stated, expected) in [
        (Truth::Pending, Truth::Pending),
        (Truth::Satisfied, Truth::Unavailable),
    ] {
        let mut inconclusive = Sentinel::new("v6", stated, SettlementBasis::Unsettled);
        inconclusive.inconclusive = Some(cause.clone());
        let mut boundary = Boundary::new();
        serve(
            &mut boundary,
            "backend",
            &[Capability::Realizability],
            always(inconclusive.clone()),
        );
        let outcome = dispatch(&mut boundary, dynamic.linked().declarations(), &asked);
        assert_eq!(outcome.truth(), expected);
        let ProtocolOutcome::Settled(ClaimSettlement::Result { result, .. }) = outcome else {
            panic!("{outcome:?}");
        };
        assert_eq!(result, inconclusive);
        assert_eq!(result.inconclusive, Some(cause.clone()));
    }

    let node = dynamic.key(&dynamic.named("control", "Bounded"));
    let ClaimExtent::Unbounded(domains) = ClaimExtent::from_domains(
        [(DomainKey::Node { node, path: vec![] }, DomainKind::Loop)]
            .into_iter()
            .collect(),
    ) else {
        panic!("an unbounded domain");
    };
    let bound = RequiresBound { domains };
    let mut boundary = Boundary::new();
    let reported = bound.clone();
    serve(
        &mut boundary,
        "backend",
        &[Capability::Realizability],
        Box::new(move |_| QslReport::RequiresBound {
            outcome: reported.clone(),
            construct: None,
        }),
    );
    let outcome = dispatch(&mut boundary, dynamic.linked().declarations(), &asked);
    assert!(!matches!(
        outcome.truth(),
        Truth::Satisfied | Truth::Violated
    ));
    let ProtocolOutcome::Settled(ClaimSettlement::Disposition(Disposition::RequiresBound(carried))) =
        outcome
    else {
        panic!("{outcome:?}");
    };
    assert_eq!(carried.outcome, bound);
}

#[test]
#[trace("TC-161", "FR-151-AC-6")]
fn a_refused_disposition_holds_no_decided_truth_and_the_restored_premise_is_sent_exactly() {
    let dynamic = Dynamic::with("workflow", 3);
    let head = dynamic.named("channel", "Head");
    let complete = request(&dynamic, KIND, &["Head"]);
    let mut omitted = complete.clone();
    omitted.premises.channels.remove(&head);
    let (mut boundary, calls) = serving();
    let refused = dispatch(&mut boundary, dynamic.linked().declarations(), &omitted);
    assert!(matches!(refused, ProtocolOutcome::Refused(_)));
    assert!(!matches!(
        refused.truth(),
        Truth::Satisfied | Truth::Violated
    ));
    // The refusal names the node only; it holds no value for the premise.
    assert_eq!(
        refused,
        ProtocolOutcome::Refused(DispositionRefusal::AbsentChannelPremise(head.clone()))
    );
    assert_eq!(calls.len(), 0);

    let mut restored = omitted;
    let value = ChannelPremise {
        unread: Rest {
            sender: "restored-sender",
            ..rest()
        },
        ..complete.premises.channels[&head].clone()
    };
    restored
        .premises
        .channels
        .insert(head.clone(), value.clone());
    dispatch(&mut boundary, dynamic.linked().declarations(), &restored);
    let received = calls.all();
    let [call] = received.as_slice() else {
        panic!("one call");
    };
    assert_eq!(call.request.premises.channels[&head], value);
}

#[test]
#[trace("TC-161", "FR-151-AC-8")]
fn a_requester_premise_that_disagrees_with_the_package_is_refused_alone() {
    let dynamic = Dynamic::with("workflow", 3);
    let head = dynamic.named("channel", "Head");
    let complete = request(&dynamic, KIND, &["Head"]);
    let declared = dynamic
        .linked()
        .declarations()
        .channel(&head)
        .expect("Head")
        .clone();
    let mut wrong_ordering = complete.clone();
    wrong_ordering
        .premises
        .channels
        .get_mut(&head)
        .expect("premise")
        .ordering = Ordering::Unordered;
    let mut wrong_delivery = complete;
    wrong_delivery
        .premises
        .channels
        .get_mut(&head)
        .expect("premise")
        .delivery = if declared.delivery == Delivery::AtMostOnce {
        Delivery::AtLeastOnce
    } else {
        Delivery::AtMostOnce
    };
    let (mut boundary, calls) = serving();
    let settled = dispatch_claims(
        &mut boundary,
        dynamic.linked().declarations(),
        &[],
        &[
            ClaimRequest::Protocol(wrong_ordering),
            ClaimRequest::Protocol(wrong_delivery),
            ClaimRequest::GlobalConformance("conformant"),
        ],
    );
    for refused in &settled[..2] {
        assert_eq!(
            *refused,
            ClaimDisposition::Protocol(ProtocolOutcome::Refused(
                DispositionRefusal::SubstitutedChannelPremise(head.clone())
            ))
        );
    }
    assert_eq!(
        settled[2],
        ClaimDisposition::GlobalConformance("conformant")
    );
    assert_eq!(calls.len(), 0);
}
