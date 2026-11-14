// SPDX-License-Identifier: AGPL-3.0-or-later
//! TC-164: `local-projection` claims dispatched to QSL's real
//! `protocol-analysis` engine, with no stand-in (FR-150, FR-152, FR-153,
//! TL-387 AC1, AC2).

mod support {
    pub mod dynamic;
    pub mod handoff;
    pub mod projection;
}

use ix_trace_rs::trace;
use qsl_analyze::protocol_analysis::manifest::BACKEND_ID;
use qsl_analyze::protocol_analysis::ProtocolAnalysisArtifact;
use qsl_replay::protocol_analysis::AnalysisOutcome;
use qsl_replay::{SettlementBasis, Truth};
use quire_protocol::dispatch::{BackendResult, ProtocolOutcome};
use quire_protocol::disposition::{ClaimSettlement, Disposition};
use support::projection::{settle, worked, CANCELLED, PICK};

#[test]
#[trace("TC-164", "FR-150-AC-12")]
fn a_sound_projection_is_decided_by_the_real_engine_and_carried_satisfied() {
    let dynamic = worked(PICK, CANCELLED);
    let ProtocolOutcome::Settled(ClaimSettlement::Result { result, premises }) =
        settle(&dynamic, "Warehouse", "bounded-static")
    else {
        panic!("the engine decides the claim");
    };
    assert_eq!(
        premises.selected.as_ref().map(|id| id.as_str()),
        Some(BACKEND_ID)
    );
    assert_eq!(result.truth(), Truth::Satisfied);
    assert_eq!(result.basis(), SettlementBasis::ClosedScope);
    assert!(result.witness().is_none());
    let Some(ProtocolAnalysisArtifact::Projection(found)) = result.artifact() else {
        panic!("a projection result");
    };
    assert!(matches!(found.outcome, AnalysisOutcome::Proved(_)));
}

#[test]
#[trace("TC-164", "FR-150-AC-13")]
fn a_refuted_projection_carries_the_engines_witness_unchanged() {
    let dynamic = worked(PICK, "");
    let ProtocolOutcome::Settled(ClaimSettlement::Result { result, .. }) =
        settle(&dynamic, "Warehouse", "bounded-static")
    else {
        panic!("the engine decides the claim");
    };
    assert_eq!(result.truth(), Truth::Violated);
    assert_eq!(result.basis(), SettlementBasis::DecisiveCounterexample);
    let Some(ProtocolAnalysisArtifact::Projection(found)) = result.artifact() else {
        panic!("a projection result");
    };
    let AnalysisOutcome::Refuted(counterexample) = &found.outcome else {
        panic!("a counterexample");
    };
    assert_eq!(result.witness(), Some(&counterexample.separating_witness));
}

#[test]
#[trace("TC-164", "FR-153-AC-8")]
fn a_request_the_engine_refuses_is_not_a_result_and_settles_no_truth() {
    let dynamic = worked(PICK, CANCELLED);
    let outcome = settle(&dynamic, "Warehouse", "unbounded");
    assert!(
        !matches!(
            outcome,
            ProtocolOutcome::Settled(ClaimSettlement::Result { .. })
        ),
        "{outcome:?}"
    );
    assert!(
        matches!(
            outcome,
            ProtocolOutcome::Settled(ClaimSettlement::Disposition(Disposition::Refused(_)))
        ),
        "{outcome:?}"
    );
}
