// SPDX-License-Identifier: AGPL-3.0-or-later
//! TC-162: a backend's witness, counterexample or projected artifact carried
//! unchanged beside the source identities the disposition retains (FR-152).

mod support {
    pub mod dynamic;
    pub mod handoff;
    pub mod standin;
}

use ix_trace_rs::trace;
use qsl_foundation::witness::ValuePathStep;
use qsl_replay::{SettlementBasis, Truth};
use qsl_semantics::check::Capability;
use quire_protocol::dispatch::{
    dispatch, dispatch_claims, ClaimDisposition, ClaimRequest, DispositionRefusal,
    ProtocolClaimKind, ProtocolOutcome,
};
use quire_protocol::disposition::ClaimSettlement;
use support::dynamic::Dynamic;
use support::standin::{always, request, serve, witness, Boundary, Sentinel};

const KIND: ProtocolClaimKind = ProtocolClaimKind::Realizability;

fn decisive(dynamic: &Dynamic, truth: Truth, basis: SettlementBasis) -> Sentinel {
    let mut result = Sentinel::new("decisive", truth, basis);
    result.witness = Some(witness(dynamic, vec![ValuePathStep::Index(2)]));
    result
}

fn carried(dynamic: &Dynamic, result: Sentinel) {
    let mut boundary = Boundary::new();
    serve(
        &mut boundary,
        "backend",
        &[Capability::Realizability],
        always(result.clone()),
    );
    let asked = request(dynamic, KIND, &["Head"]);
    let outcome = dispatch(&mut boundary, dynamic.linked().declarations(), &asked);
    let ProtocolOutcome::Settled(ClaimSettlement::Result {
        result: carried,
        premises,
    }) = outcome
    else {
        panic!("{outcome:?}");
    };
    assert_eq!(carried, result);
    assert_eq!(carried.witness, result.witness);
    assert!(!premises.request.sources.is_empty());
    assert_eq!(premises.request.sources, asked.sources);
}

#[test]
#[trace("TC-162", "FR-152-AC-1")]
fn a_decisive_witness_is_carried_with_the_retained_source_identities() {
    let dynamic = Dynamic::with("workflow", 3);
    carried(
        &dynamic,
        decisive(&dynamic, Truth::Satisfied, SettlementBasis::DecisiveWitness),
    );
}

#[test]
#[trace("TC-162", "FR-152-AC-2")]
fn a_decisive_counterexample_is_carried_with_the_retained_source_identities() {
    let dynamic = Dynamic::with("workflow", 3);
    carried(
        &dynamic,
        decisive(
            &dynamic,
            Truth::Violated,
            SettlementBasis::DecisiveCounterexample,
        ),
    );
}

#[test]
#[trace("TC-162", "FR-152-AC-3")]
fn a_non_minimal_witness_is_carried_with_every_step_in_order() {
    let dynamic = Dynamic::with("workflow", 3);
    let steps = vec![
        ValuePathStep::Field("a".to_owned()),
        ValuePathStep::Index(2),
        ValuePathStep::Index(0),
        ValuePathStep::Field("a".to_owned()),
        ValuePathStep::Index(2),
    ];
    let mut result = Sentinel::new(
        "long",
        Truth::Violated,
        SettlementBasis::DecisiveCounterexample,
    );
    result.witness = Some(witness(&dynamic, steps.clone()));
    let mut boundary = Boundary::new();
    serve(
        &mut boundary,
        "backend",
        &[Capability::Realizability],
        always(result),
    );
    let outcome = dispatch(
        &mut boundary,
        dynamic.linked().declarations(),
        &request(&dynamic, KIND, &["Head"]),
    );
    let ProtocolOutcome::Settled(ClaimSettlement::Result { result, .. }) = outcome else {
        panic!("{outcome:?}");
    };
    let record = result.witness.expect("the witness");
    assert_eq!(record.value_path.steps, steps);
}

#[test]
#[trace("TC-162", "FR-152-AC-4")]
fn a_decisive_result_with_no_source_identity_is_refused_alone() {
    let dynamic = Dynamic::with("workflow", 3);
    let mut boundary = Boundary::new();
    serve(
        &mut boundary,
        "backend",
        &[Capability::Realizability],
        always(decisive(
            &dynamic,
            Truth::Satisfied,
            SettlementBasis::DecisiveWitness,
        )),
    );
    let declarations = dynamic.linked().declarations();
    let mut unbound = request(&dynamic, KIND, &["Head"]);
    unbound.sources.clear();
    let bound = request(&dynamic, KIND, &["Head"]);
    let together = dispatch_claims(
        &mut boundary,
        declarations,
        &[
            ClaimRequest::Protocol(unbound),
            ClaimRequest::Protocol(bound.clone()),
        ],
    );
    let alone: Vec<ClaimDisposition<&str, _, _, _, _>> = dispatch_claims(
        &mut boundary,
        declarations,
        &[ClaimRequest::Protocol(bound)],
    );
    assert_eq!(
        together[0],
        ClaimDisposition::Protocol(ProtocolOutcome::Refused(
            DispositionRefusal::NoSourceIdentity
        ))
    );
    assert_eq!(together[1], alone[0]);
}

#[test]
#[trace("TC-162", "FR-152-AC-5")]
fn a_projected_local_artifact_is_carried_unchanged() {
    let dynamic = Dynamic::with("workflow", 3);
    let mut result = Sentinel::new("projected", Truth::Satisfied, SettlementBasis::ClosedScope);
    result.artifact = Some("local-artifact-bytes");
    let mut boundary = Boundary::new();
    serve(
        &mut boundary,
        "backend",
        &[Capability::LocalProjection],
        always(result.clone()),
    );
    let outcome = dispatch(
        &mut boundary,
        dynamic.linked().declarations(),
        &request(&dynamic, ProtocolClaimKind::LocalProjection, &["Head"]),
    );
    let ProtocolOutcome::Settled(ClaimSettlement::Result {
        result: carried, ..
    }) = outcome
    else {
        panic!("{outcome:?}");
    };
    assert_eq!(carried, result);
}

#[test]
#[trace("TC-162", "FR-152-AC-7")]
fn a_decisive_realizability_result_needs_a_positioned_witness_and_a_projection_does_not() {
    let dynamic = Dynamic::with("workflow", 3);
    let declarations = dynamic.linked().declarations();
    let bare = Sentinel::new("bare", Truth::Satisfied, SettlementBasis::DecisiveWitness);
    let mut unpositioned = bare.clone();
    let mut record = witness(&dynamic, Vec::new());
    record.trace_position = None;
    unpositioned.witness = Some(record);
    for result in [bare.clone(), unpositioned] {
        let mut boundary = Boundary::new();
        serve(
            &mut boundary,
            "backend",
            &[Capability::Realizability],
            always(result),
        );
        let outcome = dispatch(
            &mut boundary,
            declarations,
            &request(&dynamic, KIND, &["Head"]),
        );
        assert_eq!(
            outcome,
            ProtocolOutcome::Refused(DispositionRefusal::DecisiveWithoutWitness)
        );
    }
    let mut boundary = Boundary::new();
    serve(
        &mut boundary,
        "backend",
        &[Capability::LocalProjection],
        always(bare.clone()),
    );
    let outcome = dispatch(
        &mut boundary,
        declarations,
        &request(&dynamic, ProtocolClaimKind::LocalProjection, &["Head"]),
    );
    let ProtocolOutcome::Settled(ClaimSettlement::Result { result, .. }) = outcome else {
        panic!("{outcome:?}");
    };
    assert_eq!(result, bare);
}
