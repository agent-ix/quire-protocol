// SPDX-License-Identifier: AGPL-3.0-or-later
//! TC-166: `realizability` claims dispatched to QSL's real `protocol-analysis`
//! engine, with no stand-in (FR-150, TL-387 AC1).

mod support {
    pub mod dynamic;
    pub mod handoff;
    pub mod projection;
    pub mod realizability;
}

use std::convert::Infallible;

use ix_trace_rs::trace;
use qsl_analyze::protocol_analysis::manifest::BACKEND_ID;
use qsl_analyze::protocol_analysis::ProtocolAnalysisArtifact;
use qsl_replay::protocol_analysis::{AnalysisOutcome, RealizabilityCounterexample};
use qsl_replay::{SettlementBasis, Truth};
use qsl_route::ProviderOrigin;
use quire_protocol::dispatch::{
    dispatch, Answer, BackendResult, Binding, ProtocolClaimKind, ProtocolOutcome, ProviderBoundary,
};
use quire_protocol::disposition::ClaimSettlement;
use quire_protocol::engine::{engine_descriptor, EngineRequest, EngineResult, QslEngine};
use support::projection::{claim_of, Asked, CANCELLED, PICK};
use support::realizability::Realizability;

fn settle(fixture: &Realizability) -> EngineResult {
    let mut boundary: ProviderBoundary<'_, Asked, Answer<EngineResult, Infallible>> =
        ProviderBoundary::new();
    let engine = QslEngine::new(fixture.package(), |_: &Binding<(), ()>| {
        EngineRequest::Realizability(Box::new(fixture.request()))
    })
    .expect("QSL's linked engines register");
    boundary
        .register(
            engine_descriptor(ProviderOrigin::Linked).expect("QSL's own descriptor"),
            engine,
        )
        .expect("the registration is accepted");
    let asked = claim_of(
        ProtocolClaimKind::Realizability,
        fixture.linked(),
        fixture.clause(),
        |form, name| fixture.named(form, name),
    );
    match dispatch(&mut boundary, fixture.linked().declarations(), &asked) {
        ProtocolOutcome::Settled(ClaimSettlement::Result { result, premises }) => {
            assert_eq!(
                premises.selected.as_ref().map(|id| id.as_str()),
                Some(BACKEND_ID)
            );
            result
        }
        other => panic!("the engine decides the claim: {other:?}"),
    }
}

#[test]
#[trace("TC-166", "FR-150-AC-16")]
fn a_realizable_protocol_is_proved_by_the_real_engine_and_carried_satisfied() {
    let result = settle(&Realizability::worked(PICK, CANCELLED));
    assert_eq!(result.truth(), Truth::Satisfied);
    assert_eq!(result.basis(), SettlementBasis::ClosedScope);
    assert!(result.witness().is_none());
    let Some(ProtocolAnalysisArtifact::Realizability(found)) = result.artifact() else {
        panic!("a realizability result");
    };
    assert!(matches!(found.outcome, AnalysisOutcome::Proved(_)));
}

#[test]
#[trace("TC-166", "FR-150-AC-17")]
fn an_unrealizable_protocol_carries_the_engines_record_and_witness_unchanged() {
    let result = settle(&Realizability::worked(PICK, ""));
    assert_eq!(result.truth(), Truth::Violated);
    assert_eq!(result.basis(), SettlementBasis::DecisiveCounterexample);
    let Some(ProtocolAnalysisArtifact::Realizability(found)) = result.artifact() else {
        panic!("a realizability result");
    };
    let AnalysisOutcome::Refuted(RealizabilityCounterexample::IndistinguishableHistories {
        separating_witness,
        ..
    }) = &found.outcome
    else {
        panic!("indistinguishable histories");
    };
    assert_eq!(result.witness(), Some(separating_witness));
}
