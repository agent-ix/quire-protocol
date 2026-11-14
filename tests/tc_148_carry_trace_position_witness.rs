// SPDX-License-Identifier: AGPL-3.0-or-later
//! TC-148: a decisive `realizability` result carries the trace-position witness
//! its `native-run-result/2` protocol-analysis record holds, byte for byte;
//! any other basis carries none; a backend that writes no record settles
//! `unsupported` (FR-302).

mod support {
    pub mod dynamic;
    pub mod handoff;
    pub mod projection;
    pub mod realizability;
    pub mod standin;
}

use ix_trace_rs::trace;
use qsl_replay::{DecidingElement, SettlementBasis, Truth};
use qsl_semantics::check::Capability;
use quire_exact::{Integer, Value};
use quire_protocol::dispatch::{
    dispatch, result_contract, BackendResult, DispositionRefusal, ProtocolClaimKind,
    ProtocolOutcome, WITNESS_RESULT_CONTRACT,
};
use quire_protocol::disposition::{ClaimSettlement, Disposition, Warning};
use support::dynamic::Dynamic;
use support::projection::{CANCELLED, PICK};
use support::realizability::{refuted_record, settle, Realizability};
use support::standin::{always, request, serve, Boundary, Sentinel};

const KIND: ProtocolClaimKind = ProtocolClaimKind::Realizability;

fn dispatched(result: Sentinel) -> ProtocolOutcome<Sentinel, support::standin::Bind> {
    let dynamic = Dynamic::with("workflow", 3);
    let mut boundary = Boundary::new();
    serve(
        &mut boundary,
        "backend",
        &[Capability::Realizability],
        always(result),
    );
    dispatch(
        &mut boundary,
        dynamic.linked().declarations(),
        &request(&dynamic, KIND, &["Head"]),
    )
}

/// A decisive stand-in result with QSL's record and `witness` in its place.
fn decisive(
    basis: SettlementBasis,
    witness: Option<qsl_replay::SeparatingWitnessRecord>,
) -> Sentinel {
    let (record, _) = refuted_record();
    let mut result = Sentinel::new("decisive", Truth::Violated, basis);
    result.record = Some(record);
    result.witness = witness;
    result
}

#[test]
#[trace("TC-148", "FR-302-AC-1")]
fn a_decisive_result_carries_exactly_the_witness_its_record_holds() {
    let fixture = Realizability::worked(PICK, "");
    let result = settle(&fixture);
    let record = result.run_record().expect("QSL writes the record");
    let witness = result
        .witness()
        .expect("a decisive result carries a witness");
    assert!(record.witness_is(witness));
    assert_eq!(
        record.trace_position().map(|position| position.to_string()),
        witness
            .trace_position
            .as_ref()
            .map(|held| held.as_str().to_owned())
    );
    // The protocol family has no quantifier and assigns no index.
    assert!(witness.quantifier.is_none() && witness.index.is_none());
    assert!(record.deciding_node().is_some());
    // A stand-in decisive under either decisive basis carries the same.
    for basis in [
        SettlementBasis::DecisiveWitness,
        SettlementBasis::DecisiveCounterexample,
    ] {
        let (_, held) = refuted_record();
        let outcome = dispatched(decisive(basis, Some(held.clone())));
        let ProtocolOutcome::Settled(ClaimSettlement::Result { result, .. }) = outcome else {
            panic!("{outcome:?}");
        };
        assert_eq!(result.witness, Some(held));
    }
}

#[test]
#[trace("TC-148", "FR-302-AC-2")]
fn a_basis_that_is_not_decisive_carries_no_witness() {
    let fixture = Realizability::worked(PICK, CANCELLED);
    let result = settle(&fixture);
    assert_eq!(result.basis(), SettlementBasis::ClosedScope);
    assert!(result.witness().is_none());
    assert!(!result.run_record().expect("a record").has_witness());
    for basis in [
        SettlementBasis::ClosedScope,
        SettlementBasis::Unsettled,
        SettlementBasis::Unavailable,
    ] {
        let (_, held) = refuted_record();
        let mut stray = decisive(basis, Some(held));
        stray.truth = Truth::Satisfied;
        assert_eq!(
            dispatched(stray),
            ProtocolOutcome::Refused(DispositionRefusal::WitnessWithoutDecisiveBasis),
            "{basis:?}"
        );
    }
    // A decisive basis without its witness refuses the result.
    assert_eq!(
        dispatched(decisive(SettlementBasis::DecisiveCounterexample, None)),
        ProtocolOutcome::Refused(DispositionRefusal::DecisiveWithoutWitness)
    );
}

#[test]
#[trace("TC-148", "FR-302-AC-3")]
fn a_witness_that_is_not_the_records_is_refused_not_repaired() {
    let (_, held) = refuted_record();
    let mut changed = Vec::new();
    let mut position = held.clone();
    position.trace_position = Some(qsl_replay::TracePosition::new("99".to_owned()));
    changed.push(position);
    let mut element = held.clone();
    element.deciding_element = DecidingElement::Value(Value::Integer(Integer::from(1_i64)));
    changed.push(element);
    let mut display = held.clone();
    display.index = Some(0);
    changed.push(display);
    let mut quantified = held.clone();
    quantified.quantifier = Some(
        qsl_foundation::source::provenance::OccurrenceKey::generated(
            Dynamic::with("workflow", 3).key(Dynamic::with("workflow", 3).clause()),
        ),
    );
    changed.push(quantified);
    for witness in changed {
        assert_eq!(
            dispatched(decisive(
                SettlementBasis::DecisiveCounterexample,
                Some(witness)
            )),
            ProtocolOutcome::Refused(DispositionRefusal::WitnessNotTheRecords)
        );
    }
}

#[test]
#[trace("TC-148", "FR-302-AC-4")]
fn a_backend_that_writes_no_record_settles_unsupported_with_a_warning() {
    let (_, held) = refuted_record();
    let mut bare = decisive(SettlementBasis::DecisiveCounterexample, Some(held));
    bare.record = None;
    let ProtocolOutcome::Settled(ClaimSettlement::Disposition(Disposition::Unsupported(found))) =
        dispatched(bare)
    else {
        panic!("settles unsupported");
    };
    assert!(
        matches!(
            found.warnings.as_slice(),
            [Warning::NoRunRecord {
                kind: Capability::Realizability,
                ..
            }]
        ),
        "{:?}",
        found.warnings
    );
    // QSL's own engine writes the record, so its refutation is carried
    // (FR-150-AC-17).
    let fixture = Realizability::worked(PICK, "");
    assert!(settle(&fixture).witness().is_some());
}

#[test]
#[trace("TC-148", "FR-302-AC-5")]
fn a_witness_bearing_result_is_produced_under_the_draft_two_contract() {
    assert_eq!(WITNESS_RESULT_CONTRACT, "quire.protocol.result/v1-draft.2");
    let fixture = Realizability::worked(PICK, "");
    let result = settle(&fixture);
    assert_eq!(result_contract(&result), Some(WITNESS_RESULT_CONTRACT));
    let mut recordless = Sentinel::new("none", Truth::Satisfied, SettlementBasis::ClosedScope);
    recordless.record = None;
    assert_eq!(result_contract(&recordless), None);
}
