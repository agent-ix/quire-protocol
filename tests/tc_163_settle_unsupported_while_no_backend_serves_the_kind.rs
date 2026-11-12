// SPDX-License-Identifier: AGPL-3.0-or-later
//! TC-163: projection and realizability claims settled `unsupported` while no
//! QSL backend serves the kind (FR-153).

mod support {
    pub mod dynamic;
    pub mod handoff;
    pub mod standin;
}

use ix_trace_rs::trace;
use qsl_replay::{SettlementBasis, Truth};
use qsl_semantics::check::Capability;
use quire_protocol::closure::AssessmentExecution;
use quire_protocol::dispatch::{
    dispatch, dispatch_claims, ClaimDisposition, ClaimRequest, ProtocolClaimKind, ProtocolOutcome,
};
use quire_protocol::disposition::{ClaimSettlement, Disposition, UnsupportedCause, Warning};
use support::dynamic::Dynamic;
use support::standin::{always, id, request, serve, Boundary, Sentinel};

const PROJECTION: ProtocolClaimKind = ProtocolClaimKind::LocalProjection;
const REALIZABILITY: ProtocolClaimKind = ProtocolClaimKind::Realizability;

fn unsupported(
    outcome: ProtocolOutcome<Sentinel, support::standin::Bind>,
) -> quire_protocol::disposition::UnsupportedDisposition<support::standin::Bind> {
    match outcome {
        ProtocolOutcome::Settled(ClaimSettlement::Disposition(Disposition::Unsupported(
            disposition,
        ))) => disposition,
        other => panic!("not unsupported: {other:?}"),
    }
}

fn protocol(
    disposition: ClaimDisposition<
        &'static str,
        support::standin::Assumptions,
        support::standin::Rest,
        &'static str,
        (),
        Sentinel,
    >,
) -> ProtocolOutcome<Sentinel, support::standin::Bind> {
    match disposition {
        ClaimDisposition::Protocol(outcome) => outcome,
        other => panic!("{other:?}"),
    }
}

#[test]
#[trace("TC-163", "FR-153-AC-1")]
fn an_empty_registry_settles_both_claims_unsupported_and_leaves_global_conformance() {
    let dynamic = Dynamic::with("workflow", 3);
    let mut boundary = Boundary::new();
    let declarations = dynamic.linked().declarations();
    let global = || ClaimRequest::GlobalConformance("conformant");
    let settled = dispatch_claims(
        &mut boundary,
        declarations,
        &[
            ClaimRequest::Protocol(request(&dynamic, PROJECTION, &["Head"])),
            ClaimRequest::Protocol(request(&dynamic, REALIZABILITY, &["Head"])),
            global(),
        ],
    );
    let alone = dispatch_claims(&mut boundary, declarations, &[global()]);
    for claim in settled.iter().take(2) {
        let disposition = unsupported(protocol(claim.clone()));
        assert_eq!(disposition.execution(), AssessmentExecution::Unsupported);
        assert_eq!(disposition.cause, UnsupportedCause::requested_capability());
    }
    assert_eq!(settled[2], alone[0]);
}

#[test]
#[trace("TC-163", "FR-153-AC-2")]
fn an_unsupported_disposition_holds_truth_and_basis_unavailable() {
    let dynamic = Dynamic::with("workflow", 3);
    let mut boundary = Boundary::new();
    for kind in [PROJECTION, REALIZABILITY] {
        let outcome = dispatch(
            &mut boundary,
            dynamic.linked().declarations(),
            &request(&dynamic, kind, &["Head"]),
        );
        assert_eq!(outcome.truth(), Truth::Unavailable);
        let disposition = unsupported(outcome);
        assert_eq!(disposition.truth(), Truth::Unavailable);
        assert_eq!(disposition.basis(), SettlementBasis::Unavailable);
    }
}

#[test]
#[trace("TC-163", "FR-153-AC-3")]
fn a_conformant_global_result_yields_no_projection_or_realizability_result() {
    let dynamic = Dynamic::with("workflow", 3);
    let mut boundary = Boundary::new();
    let settled = dispatch_claims(
        &mut boundary,
        dynamic.linked().declarations(),
        &[
            ClaimRequest::GlobalConformance("conformant"),
            ClaimRequest::Protocol(request(&dynamic, PROJECTION, &["Head"])),
            ClaimRequest::Protocol(request(&dynamic, REALIZABILITY, &["Head"])),
        ],
    );
    for disposition in &settled[1..] {
        let outcome = protocol(disposition.clone());
        assert!(!matches!(
            outcome,
            ProtocolOutcome::Settled(ClaimSettlement::Result { .. })
        ));
        assert!(!matches!(
            outcome.truth(),
            Truth::Satisfied | Truth::Violated
        ));
    }
}

#[test]
#[trace("TC-163", "FR-153-AC-4")]
fn the_dispositions_are_results_whose_warnings_name_the_kind_and_the_named_backend() {
    let dynamic = Dynamic::with("workflow", 3);
    let mut boundary = Boundary::new();
    serve(
        &mut boundary,
        "refiner",
        &[Capability::Refinement],
        always(Sentinel::new(
            "x",
            Truth::Satisfied,
            SettlementBasis::ClosedScope,
        )),
    );
    let declarations = dynamic.linked().declarations();
    for (kind, capability) in [
        (PROJECTION, Capability::LocalProjection),
        (REALIZABILITY, Capability::Realizability),
    ] {
        let plain = unsupported(dispatch(
            &mut boundary,
            declarations,
            &request(&dynamic, kind, &["Head"]),
        ));
        assert_eq!(
            plain.warnings,
            [Warning::UnservedCapability {
                kind: capability,
                backend: None
            }]
        );
        let mut named = request(&dynamic, kind, &["Head"]);
        named.backend = Some(id("refiner"));
        let named = unsupported(dispatch(&mut boundary, declarations, &named));
        assert_eq!(
            named.warnings,
            [Warning::UnservedCapability {
                kind: capability,
                backend: Some(id("refiner"))
            }]
        );
    }
}

#[test]
#[trace("TC-163", "FR-153-AC-5")]
fn a_backend_serving_only_refinement_is_sent_neither_claim() {
    let dynamic = Dynamic::with("workflow", 3);
    let mut boundary = Boundary::new();
    let calls = serve(
        &mut boundary,
        "refiner",
        &[Capability::Refinement],
        always(Sentinel::new(
            "x",
            Truth::Satisfied,
            SettlementBasis::ClosedScope,
        )),
    );
    for kind in [PROJECTION, REALIZABILITY] {
        unsupported(dispatch(
            &mut boundary,
            dynamic.linked().declarations(),
            &request(&dynamic, kind, &["Head"]),
        ));
    }
    assert_eq!(calls.len(), 0);
}

#[test]
#[trace("TC-163", "FR-153-AC-6")]
fn a_later_registered_backend_takes_over_the_identical_request() {
    let dynamic = Dynamic::with("workflow", 3);
    let mut boundary = Boundary::new();
    let declarations = dynamic.linked().declarations();
    let requests = [
        request(&dynamic, PROJECTION, &["Head"]),
        request(&dynamic, REALIZABILITY, &["Head"]),
    ];
    let before = requests
        .clone()
        .map(|r| dispatch(&mut boundary, declarations, &r));
    let result = Sentinel::new("projected", Truth::Satisfied, SettlementBasis::ClosedScope);
    serve(
        &mut boundary,
        "projector",
        &[Capability::LocalProjection],
        always(result.clone()),
    );
    let after = requests
        .clone()
        .map(|r| dispatch(&mut boundary, declarations, &r));
    assert!(matches!(
        before[0],
        ProtocolOutcome::Settled(ClaimSettlement::Disposition(Disposition::Unsupported(_)))
    ));
    let ProtocolOutcome::Settled(ClaimSettlement::Result {
        result: carried, ..
    }) = &after[0]
    else {
        panic!("{:?}", after[0]);
    };
    assert_eq!(*carried, result);
    assert_eq!(after[1], before[1]);
}

#[test]
#[trace("TC-163", "FR-153-AC-7")]
fn an_unsupported_disposition_retains_the_requests_exact_components() {
    let dynamic = Dynamic::with("workflow", 3);
    let mut boundary = Boundary::new();
    serve(
        &mut boundary,
        "refiner",
        &[Capability::Refinement],
        always(Sentinel::new(
            "x",
            Truth::Satisfied,
            SettlementBasis::ClosedScope,
        )),
    );
    let mut asked = request(&dynamic, REALIZABILITY, &["Head", "Reject"]);
    asked.backend = Some(id("refiner"));
    let disposition = unsupported(dispatch(
        &mut boundary,
        dynamic.linked().declarations(),
        &asked,
    ));
    let retained = disposition.premises.request;
    assert_eq!(retained.fragment, asked.fragment);
    assert_eq!(retained.backend, asked.backend);
    assert_eq!(retained.assumptions, asked.assumptions);
    assert_eq!(retained.premises, asked.premises);
    assert_eq!(retained.sources, asked.sources);
    assert_eq!(disposition.construct, None);
}
