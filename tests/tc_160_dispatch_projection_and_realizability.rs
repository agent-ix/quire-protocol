// SPDX-License-Identifier: AGPL-3.0-or-later
//! TC-160: `local-projection` and `realizability` claims dispatched to the
//! backend QSL's registry selects (FR-150).

mod support {
    pub mod dynamic;
    pub mod handoff;
    pub mod standin;
}

use ix_trace_rs::trace;
use qsl_eval::simulation::protocol::Unsupported;
use qsl_replay::{SettlementBasis, Truth};
use qsl_route::{BackendDescriptor, Mode, ProviderOrigin};
use qsl_semantics::check::Capability;
use quire_protocol::dispatch::RegistrationError;
use quire_protocol::dispatch::{
    dispatch, dispatch_claims, ClaimDisposition, ClaimRequest, ProtocolClaimKind, ProtocolOutcome,
};
use quire_protocol::disposition::{ClaimSettlement, Disposition, QslReport, UnsupportedCause};
use quire_semantic_value::catalog::CatalogCode;
use support::dynamic::Dynamic;
use support::standin::{
    always, candidate, id, request, serve, Boundary, Calls, Req, Sentinel, Standin,
};

const PROJECTION: ProtocolClaimKind = ProtocolClaimKind::LocalProjection;
const REALIZABILITY: ProtocolClaimKind = ProtocolClaimKind::Realizability;

fn sentinel(label: &'static str) -> Sentinel {
    Sentinel::new(label, Truth::Satisfied, SettlementBasis::ClosedScope)
}

fn run(
    boundary: &mut Boundary,
    dynamic: &Dynamic,
    request: &Req,
) -> ProtocolOutcome<Sentinel, support::standin::Bind> {
    dispatch(boundary, dynamic.linked().declarations(), request)
}

#[test]
#[trace("TC-160", "FR-150-AC-1")]
fn a_projection_claim_is_sent_once_and_its_result_carried_unchanged() {
    let dynamic = Dynamic::with("workflow", 3);
    let mut boundary = Boundary::new();
    let calls = serve(
        &mut boundary,
        "projector",
        &[Capability::LocalProjection],
        always(sentinel("projected")),
    );
    let asked = request(&dynamic, PROJECTION, &["Head", "Reject"]);
    let outcome = run(&mut boundary, &dynamic, &asked);
    let received = calls.all();
    assert_eq!(received.len(), 1);
    assert_eq!(received[0].request, asked);
    let ProtocolOutcome::Settled(ClaimSettlement::Result { result, .. }) = outcome else {
        panic!("{outcome:?}");
    };
    assert_eq!(result, sentinel("projected"));
}

#[test]
#[trace("TC-160", "FR-150-AC-2")]
fn a_realizability_claim_is_sent_once_and_its_result_carried_unchanged() {
    let dynamic = Dynamic::with("workflow", 3);
    let mut boundary = Boundary::new();
    let calls = serve(
        &mut boundary,
        "realizer",
        &[Capability::Realizability],
        always(sentinel("realized")),
    );
    let outcome = run(
        &mut boundary,
        &dynamic,
        &request(&dynamic, REALIZABILITY, &["Head"]),
    );
    assert_eq!(calls.len(), 1);
    let ProtocolOutcome::Settled(ClaimSettlement::Result { result, .. }) = outcome else {
        panic!("{outcome:?}");
    };
    assert_eq!(result, sentinel("realized"));
}

#[test]
#[trace("TC-160", "FR-150-AC-3")]
fn a_backend_is_sent_only_the_kind_it_serves() {
    let dynamic = Dynamic::with("workflow", 3);
    let mut boundary = Boundary::new();
    let realizer = serve(
        &mut boundary,
        "realizer",
        &[Capability::Realizability],
        always(sentinel("r")),
    );
    let projector = serve(
        &mut boundary,
        "projector",
        &[Capability::LocalProjection],
        always(sentinel("p")),
    );
    let projection = run(
        &mut boundary,
        &dynamic,
        &request(&dynamic, PROJECTION, &["Head"]),
    );
    assert_eq!((realizer.len(), projector.len()), (0, 1));
    assert!(matches!(
        projection,
        ProtocolOutcome::Settled(ClaimSettlement::Result { .. })
    ));
    let mut only_realizer = Boundary::new();
    let realizer = serve(
        &mut only_realizer,
        "realizer",
        &[Capability::Realizability],
        always(sentinel("r")),
    );
    let unserved = run(
        &mut only_realizer,
        &dynamic,
        &request(&dynamic, PROJECTION, &["Head"]),
    );
    assert_eq!(realizer.len(), 0);
    assert!(matches!(
        unserved,
        ProtocolOutcome::Settled(ClaimSettlement::Disposition(Disposition::Unsupported(_)))
    ));
}

#[test]
#[trace("TC-160", "FR-150-AC-4")]
fn the_dispatcher_derives_no_verdict_of_its_own() {
    let dynamic = Dynamic::with("workflow", 3);
    let mut boundary = Boundary::new();
    serve(
        &mut boundary,
        "fixed",
        &[Capability::Realizability],
        always(sentinel("fixed")),
    );
    // Two fragments with different channel declarations get the same result.
    for channels in [&["Head"][..], &["Reject", "Tail"][..]] {
        let outcome = run(
            &mut boundary,
            &dynamic,
            &request(&dynamic, REALIZABILITY, channels),
        );
        let ProtocolOutcome::Settled(ClaimSettlement::Result { result, .. }) = outcome else {
            panic!("{outcome:?}");
        };
        assert_eq!(result, sentinel("fixed"));
    }
    let mut silent = Boundary::new();
    serve(
        &mut silent,
        "silent",
        &[Capability::Realizability],
        always(Sentinel::new(
            "none",
            Truth::Unavailable,
            SettlementBasis::Unavailable,
        )),
    );
    let outcome = run(
        &mut silent,
        &dynamic,
        &request(&dynamic, REALIZABILITY, &["Head"]),
    );
    assert_eq!(outcome.truth(), Truth::Unavailable);
}

#[test]
#[trace("TC-160", "FR-150-AC-5")]
fn each_backend_cause_is_carried_in_the_backends_vocabulary_and_no_other_is_called() {
    let dynamic = Dynamic::with("workflow", 3);
    let refusal = CatalogCode::new("invalid_input", "distinct-cause");
    let tool_unavailable = CatalogCode::new("unsupported_projection", "tool-unavailable");
    let answers: Vec<(&str, QslReport<Sentinel>)> = vec![
        ("refusal", QslReport::Refusal(refusal)),
        (
            "unsupported",
            QslReport::Unsupported {
                cause: UnsupportedCause::Construct(Unsupported::SymbolicCapacity),
                construct: None,
            },
        ),
        (
            "tool",
            QslReport::Unsupported {
                cause: UnsupportedCause::RequestedCapability(tool_unavailable),
                construct: None,
            },
        ),
    ];
    for (name, report) in answers {
        let mut boundary = Boundary::new();
        let reported = report.clone();
        let calls = serve(
            &mut boundary,
            name,
            &[Capability::Realizability],
            Box::new(move |_| reported.clone()),
        );
        let bystander = serve(
            &mut boundary,
            "bystander",
            &[Capability::Realizability],
            always(sentinel("never")),
        );
        let mut asked = request(&dynamic, REALIZABILITY, &["Head"]);
        asked.backend = Some(id(name));
        let outcome = run(&mut boundary, &dynamic, &asked);
        assert_eq!((calls.len(), bystander.len()), (1, 0), "{name}");
        match (name, outcome) {
            (
                "refusal",
                ProtocolOutcome::Settled(ClaimSettlement::Disposition(Disposition::Refused(
                    carried,
                ))),
            ) => assert_eq!(carried.code, refusal),
            (
                "unsupported",
                ProtocolOutcome::Settled(ClaimSettlement::Disposition(Disposition::Unsupported(
                    carried,
                ))),
            ) => assert_eq!(
                carried.cause,
                UnsupportedCause::Construct(Unsupported::SymbolicCapacity)
            ),
            (
                "tool",
                ProtocolOutcome::Settled(ClaimSettlement::Disposition(Disposition::Unsupported(
                    carried,
                ))),
            ) => assert_eq!(
                carried.cause,
                UnsupportedCause::RequestedCapability(tool_unavailable)
            ),
            (name, other) => panic!("{name}: {other:?}"),
        }
    }
}

#[test]
#[trace("TC-160", "FR-150-AC-6")]
fn a_failed_backend_fails_exactly_its_claim_and_no_other_backend_is_called() {
    let dynamic = Dynamic::with("workflow", 3);
    let claims = |dynamic: &Dynamic| -> Vec<ClaimRequest<&'static str, _, _, _>> {
        [&["Head"][..], &["Reject"][..], &["Tail"][..]]
            .into_iter()
            .map(|channels| {
                let mut asked = request(dynamic, REALIZABILITY, channels);
                asked.backend = Some(id("flaky"));
                ClaimRequest::Protocol(asked)
            })
            .collect()
    };
    let settle = |fail: bool| {
        let mut boundary = Boundary::new();
        let reject = dynamic.named("channel", "Reject");
        serve(
            &mut boundary,
            "flaky",
            &[Capability::Realizability],
            Box::new(move |call| {
                if fail && call.request.fragment.channels == [reject.clone()] {
                    QslReport::Failed(None)
                } else {
                    QslReport::Result(sentinel("ok"))
                }
            }),
        );
        let spare = serve(
            &mut boundary,
            "spare",
            &[Capability::Realizability],
            always(sentinel("never")),
        );
        let settled = dispatch_claims(
            &mut boundary,
            dynamic.linked().declarations(),
            &claims(&dynamic),
        );
        assert_eq!(spare.len(), 0);
        settled
    };
    let failing = settle(true);
    let healthy = settle(false);
    assert!(matches!(
        &failing[1],
        ClaimDisposition::Protocol(ProtocolOutcome::Settled(ClaimSettlement::Disposition(
            Disposition::Failed(_)
        )))
    ));
    assert_eq!(failing[0], healthy[0]);
    assert_eq!(failing[2], healthy[2]);
}

#[test]
#[trace("TC-160", "FR-150-AC-7")]
fn global_conformance_is_the_same_whether_or_not_the_advanced_claims_are_requested() {
    let dynamic = Dynamic::with("workflow", 3);
    let mut boundary = Boundary::new();
    serve(
        &mut boundary,
        "both",
        &[Capability::LocalProjection, Capability::Realizability],
        always(sentinel("x")),
    );
    let declarations = dynamic.linked().declarations();
    let global = || ClaimRequest::GlobalConformance("conformant");
    let together = dispatch_claims(
        &mut boundary,
        declarations,
        &[
            ClaimRequest::Protocol(request(&dynamic, PROJECTION, &["Head"])),
            ClaimRequest::Protocol(request(&dynamic, REALIZABILITY, &["Head"])),
            global(),
        ],
    );
    let alone = dispatch_claims(&mut boundary, declarations, &[global()]);
    assert_eq!(together.len(), 3);
    assert_eq!(together[2], alone[0]);
    assert_eq!(alone[0], ClaimDisposition::GlobalConformance("conformant"));
}

#[test]
#[trace("TC-160", "FR-150-AC-8")]
fn several_candidates_and_an_unknown_backend_are_settled_without_a_call() {
    let dynamic = Dynamic::with("workflow", 3);
    let mut boundary = Boundary::new();
    let second = serve(
        &mut boundary,
        "b-backend",
        &[Capability::Realizability],
        always(sentinel("b")),
    );
    let first = serve(
        &mut boundary,
        "a-backend",
        &[Capability::Realizability],
        always(sentinel("a")),
    );
    let ambiguous = run(
        &mut boundary,
        &dynamic,
        &request(&dynamic, REALIZABILITY, &["Head"]),
    );
    let ProtocolOutcome::InvalidRequest {
        code, candidates, ..
    } = ambiguous
    else {
        panic!("{ambiguous:?}");
    };
    assert_eq!(
        code,
        CatalogCode::new("invalid_capability", "ambiguous-backend")
    );
    assert_eq!(candidates, [candidate("a-backend"), candidate("b-backend")]);
    let mut ghost = request(&dynamic, REALIZABILITY, &["Head"]);
    ghost.backend = Some(id("ghost"));
    let unknown = run(&mut boundary, &dynamic, &ghost);
    let ProtocolOutcome::InvalidRequest {
        code, candidates, ..
    } = unknown
    else {
        panic!("{unknown:?}");
    };
    assert_eq!(
        code,
        CatalogCode::new("invalid_capability", "unknown-backend")
    );
    assert!(candidates.is_empty());
    assert_eq!((first.len(), second.len()), (0, 0));
}

#[test]
#[trace("TC-160", "FR-150-AC-9")]
fn a_named_backend_alone_is_called_under_every_registration_order() {
    let dynamic = Dynamic::with("workflow", 3);
    let settle = |a_first: bool| {
        let mut boundary = Boundary::new();
        let mut calls = Vec::new();
        let order: [(&str, &'static str); 2] = if a_first {
            [("a-backend", "a"), ("b-backend", "b")]
        } else {
            [("b-backend", "b"), ("a-backend", "a")]
        };
        for (name, label) in order {
            calls.push((
                name,
                serve(
                    &mut boundary,
                    name,
                    &[Capability::Realizability],
                    always(sentinel(label)),
                ),
            ));
        }
        let mut asked = request(&dynamic, REALIZABILITY, &["Head"]);
        asked.backend = Some(id("b-backend"));
        let outcome = run(&mut boundary, &dynamic, &asked);
        let counts: Vec<_> = calls.iter().map(|(name, c)| (*name, c.len())).collect();
        (outcome, counts)
    };
    let (forward, forward_calls) = settle(true);
    let (reverse, reverse_calls) = settle(false);
    assert_eq!(forward, reverse);
    for counts in [forward_calls, reverse_calls] {
        assert_eq!(
            counts
                .iter()
                .find(|(name, _)| *name == "b-backend")
                .map(|c| c.1),
            Some(1)
        );
        assert_eq!(
            counts
                .iter()
                .find(|(name, _)| *name == "a-backend")
                .map(|c| c.1),
            Some(0)
        );
    }
    let ProtocolOutcome::Settled(ClaimSettlement::Result { result, .. }) = forward else {
        panic!("{forward:?}");
    };
    assert_eq!(result, sentinel("b"));
}

#[test]
#[trace("TC-160", "FR-150-AC-10")]
fn the_selected_backend_is_recorded_on_the_disposition() {
    let dynamic = Dynamic::with("workflow", 3);
    let mut boundary = Boundary::new();
    serve(
        &mut boundary,
        "only-backend",
        &[Capability::Realizability],
        always(sentinel("x")),
    );
    let asked = request(&dynamic, REALIZABILITY, &["Head"]);
    assert_eq!(asked.backend, None);
    let outcome = run(&mut boundary, &dynamic, &asked);
    let ProtocolOutcome::Settled(ClaimSettlement::Result { premises, .. }) = outcome else {
        panic!("{outcome:?}");
    };
    assert_eq!(premises.selected, Some(id("only-backend")));
    assert_eq!(premises.request.backend, None);

    let unserved = run(
        &mut boundary,
        &dynamic,
        &request(&dynamic, PROJECTION, &["Head"]),
    );
    let ProtocolOutcome::Settled(ClaimSettlement::Disposition(Disposition::Unsupported(
        unsupported,
    ))) = unserved
    else {
        panic!("{unserved:?}");
    };
    assert_eq!(unsupported.premises.selected, None);
}

#[test]
#[trace("TC-160", "FR-150-AC-11")]
fn a_second_provider_for_a_held_identity_is_refused_and_the_first_is_called() {
    let dynamic = Dynamic::with("workflow", 3);
    let mut boundary = Boundary::new();
    let first = serve(
        &mut boundary,
        "backend",
        &[Capability::Realizability],
        always(sentinel("first")),
    );
    let descriptor = BackendDescriptor::new(
        candidate("backend"),
        ProviderOrigin::Linked,
        [(Capability::Realizability, Mode::Unbounded)],
        None,
        None,
    )
    .expect("a descriptor");
    let refused = boundary.register(
        descriptor,
        Standin::new(always(sentinel("second")), Calls::default()),
    );
    assert_eq!(refused, Err(RegistrationError::ProviderHeld(id("backend"))));
    let outcome = run(
        &mut boundary,
        &dynamic,
        &request(&dynamic, REALIZABILITY, &["Head"]),
    );
    assert_eq!(first.len(), 1);
    let ProtocolOutcome::Settled(ClaimSettlement::Result { result, .. }) = outcome else {
        panic!("{outcome:?}");
    };
    assert_eq!(result, sentinel("first"));
}
