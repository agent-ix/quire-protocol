// SPDX-License-Identifier: AGPL-3.0-or-later
//! TC-200 to TC-204: `composition` and `refinement` claims dispatched to the
//! backend QSL's registry selects, their results carried unchanged and bound
//! to the request they were established for (FR-200 to FR-204).

mod support {
    pub mod assume_guarantee;
    pub mod dynamic;
    pub mod handoff;
}

use ix_trace_rs::trace;
use qsl_replay::{ItemLabel, SettlementBasis, Truth};
use qsl_semantics::check::Capability;
use quire_protocol::assume_guarantee::{
    dispatch, dispatch_claims, read_composition_label, read_refinement_label, AssumeGuaranteeKind,
    IdentityMismatch, PremiseList, PremiseLists,
};
use quire_protocol::closure::AssessmentExecution;
use quire_protocol::dispatch::{DispositionRefusal, ProtocolOutcome};
use quire_protocol::disposition::{
    ClaimSettlement, Disposition, QslReport, UnsupportedCause, Warning,
};
use quire_semantic_value::catalog::CatalogCode;
use support::assume_guarantee::{always, honest, request, serve, Bind, Boundary, Out};
use support::dynamic::Dynamic;

const COMPOSITION: AssumeGuaranteeKind = AssumeGuaranteeKind::Composition;
const REFINEMENT: AssumeGuaranteeKind = AssumeGuaranteeKind::Refinement;
const BOTH: [Capability; 2] = [Capability::Composition, Capability::Refinement];

type Outcome = ProtocolOutcome<Out, Bind>;

fn carried(outcome: Outcome) -> Out {
    match outcome {
        ProtocolOutcome::Settled(ClaimSettlement::Result { result, .. }) => result,
        other => panic!("not a carried result: {other:?}"),
    }
}

fn refused(outcome: Outcome) -> DispositionRefusal {
    match outcome {
        ProtocolOutcome::Refused(refusal) => refusal,
        other => panic!("not refused: {other:?}"),
    }
}

fn disposition(outcome: Outcome) -> Disposition<Bind> {
    match outcome {
        ProtocolOutcome::Settled(ClaimSettlement::Disposition(disposition)) => disposition,
        other => panic!("not a non-success disposition: {other:?}"),
    }
}

fn proved() -> Out {
    Out::new(ItemLabel::Proved)
}

/// A boundary whose one stand-in serves both kinds and answers `result`
/// shaped to each request.
fn serving(result: Out) -> (Boundary, support::assume_guarantee::Calls) {
    let mut boundary = Boundary::new();
    let calls = serve(
        &mut boundary,
        "composer",
        &BOTH,
        Box::new(move |bound| QslReport::Result(honest(&bound.request, result.clone()))),
    );
    (boundary, calls)
}

// ---------------------------------------------------------------- TC-200

#[test]
#[trace("TC-200", "FR-200-AC-1")]
fn a_composition_claim_sends_one_item_naming_the_component_packages() {
    let dynamic = Dynamic::with("workflow", 3);
    let (mut boundary, calls) = serving(proved());
    let asked = request(&dynamic, COMPOSITION);
    let result = carried(dispatch(&mut boundary, &asked));
    let received = calls.all();
    assert_eq!(received.len(), 1);
    assert_eq!(received[0].request.subjects, asked.subjects);
    assert_eq!(result, honest(&asked, proved()));
}

#[test]
#[trace("TC-200", "FR-200-AC-2")]
fn a_composition_claim_without_a_backend_settles_unsupported_with_a_warning() {
    let dynamic = Dynamic::with("workflow", 3);
    let mut boundary = Boundary::new();
    let mut asked = request(&dynamic, COMPOSITION);
    asked.premises.relies = vec!["rely-supplied-by-a-matching-guarantee"];
    let Disposition::Unsupported(unsupported) = disposition(dispatch(&mut boundary, &asked)) else {
        panic!("not unsupported");
    };
    assert_eq!(unsupported.cause, UnsupportedCause::requested_capability());
    assert_eq!(
        unsupported.warnings,
        vec![Warning::UnservedCapability {
            kind: Capability::Composition,
            backend: None
        }]
    );
    assert_eq!(unsupported.execution(), AssessmentExecution::Unsupported);
}

#[test]
#[trace("TC-200", "FR-200-AC-3")]
fn an_unsupported_composition_leaves_every_other_claim_unchanged() {
    let dynamic = Dynamic::with("workflow", 3);
    let mut refining = Boundary::new();
    serve(
        &mut refining,
        "refiner",
        &[Capability::Refinement],
        always(proved()),
    );
    let refinement = request(&dynamic, REFINEMENT);
    let alone = dispatch_claims(&mut refining, std::slice::from_ref(&refinement));
    let together = dispatch_claims(
        &mut refining,
        &[request(&dynamic, COMPOSITION), refinement.clone()],
    );
    assert!(matches!(
        together[0],
        ProtocolOutcome::Settled(ClaimSettlement::Disposition(Disposition::Unsupported(_)))
    ));
    assert_eq!(together[1], alone[0]);
}

#[test]
#[trace("TC-200", "FR-200-AC-4")]
fn a_composition_disposition_is_independent_of_any_other_claims_success() {
    let dynamic = Dynamic::with("workflow", 3);
    let (mut boundary, _) = serving(Out::new(ItemLabel::Inconclusive));
    let asked = request(&dynamic, COMPOSITION);
    let first = dispatch(&mut boundary, &asked);
    let _satisfied_beside = dispatch(&mut boundary, &request(&dynamic, REFINEMENT));
    let second = dispatch(&mut boundary, &asked);
    assert_eq!(first, second);
    assert_eq!(carried(second).label, ItemLabel::Inconclusive);
}

#[test]
#[trace("TC-200", "FR-200-AC-5")]
fn removing_a_component_of_the_binding_refuses_only_that_disposition() {
    let dynamic = Dynamic::with("workflow", 3);
    let (mut boundary, _) = serving(Out {
        decisive: true,
        ..proved()
    });
    let whole = request(&dynamic, COMPOSITION);
    let mut no_subjects = whole.clone();
    no_subjects.subjects.clear();
    let mut no_sources = whole.clone();
    no_sources.sources.clear();
    let mut unknown_backend = whole.clone();
    unknown_backend.backend = Some(qsl_foundation::BackendId::new("elsewhere"));
    let settled = dispatch_claims(
        &mut boundary,
        &[no_subjects, whole.clone(), no_sources, unknown_backend],
    );
    assert_eq!(refused(settled[0].clone()), DispositionRefusal::NoSubjects);
    assert_eq!(settled[1], dispatch(&mut boundary, &whole));
    assert_eq!(
        refused(settled[2].clone()),
        DispositionRefusal::NoSourceIdentity
    );
    assert!(matches!(settled[3], ProtocolOutcome::InvalidRequest { .. }));
}

#[test]
#[trace("TC-200", "FR-200-AC-6")]
fn the_disposition_equals_the_backends_result_and_nothing_when_it_returns_none() {
    let dynamic = Dynamic::with("workflow", 3);
    let mut boundary = Boundary::new();
    // The request's premises would all be met by a local reading; the backend
    // says otherwise.
    serve(
        &mut boundary,
        "contrary",
        &BOTH,
        Box::new(|bound| {
            QslReport::Result(honest(&bound.request, Out::new(ItemLabel::Inconclusive)))
        }),
    );
    let asked = request(&dynamic, COMPOSITION);
    assert_eq!(
        carried(dispatch(&mut boundary, &asked)).label,
        ItemLabel::Inconclusive
    );
    let mut silent = Boundary::new();
    serve(
        &mut silent,
        "silent",
        &BOTH,
        Box::new(|_| QslReport::Failed(None)),
    );
    assert!(matches!(
        disposition(dispatch(&mut silent, &asked)),
        Disposition::Failed(_)
    ));
}

// ---------------------------------------------------------------- TC-201

#[test]
#[trace("TC-201", "FR-201-AC-1")]
fn the_four_premise_lists_reach_the_backend_as_four_separate_lists() {
    let dynamic = Dynamic::with("workflow", 3);
    let (mut boundary, calls) = serving(proved());
    let asked = request(&dynamic, COMPOSITION);
    dispatch(&mut boundary, &asked);
    let sent = &calls.all()[0].request.premises;
    assert_eq!(sent.environment, vec!["env-1"]);
    assert_eq!(sent.relies, vec!["rely-1", "rely-2"]);
    assert_eq!(sent.liveness_dependencies, vec!["live-1"]);
    assert_eq!(sent.fairness, vec!["fair-1"]);
}

#[test]
#[trace("TC-201", "FR-201-AC-2")]
fn a_result_retaining_dropped_or_merged_lists_is_refused_alone() {
    let dynamic = Dynamic::with("workflow", 3);
    let asked = request(&dynamic, COMPOSITION);
    let retaining = |lists: PremiseLists<&'static str>| {
        let mut boundary = Boundary::new();
        let mut out = honest(&asked, proved());
        out.premises = Some(lists);
        serve(&mut boundary, "b", &BOTH, always(out));
        dispatch(&mut boundary, &asked)
    };
    let mut dropped = asked.premises.clone();
    dropped.fairness.clear();
    assert_eq!(
        refused(retaining(dropped)),
        DispositionRefusal::RetainedPremiseList(PremiseList::Fairness)
    );
    let mut merged = asked.premises.clone();
    merged.relies.append(&mut merged.environment);
    assert_eq!(
        refused(retaining(merged)),
        DispositionRefusal::RetainedPremiseList(PremiseList::Environment)
    );
    assert!(matches!(
        retaining(asked.premises.clone()),
        ProtocolOutcome::Settled(ClaimSettlement::Result { .. })
    ));
}

#[test]
#[trace("TC-201", "FR-201-AC-3")]
fn a_discharge_record_is_carried_entry_for_entry() {
    let dynamic = Dynamic::with("workflow", 3);
    let discharge = vec![
        ("component-guarantee", "slot-0"),
        ("fixed-point", "slot-1"),
        ("a-role-the-library-does-not-know", "slot-2"),
    ];
    let (mut boundary, _) = serving(Out {
        discharge: discharge.clone(),
        ..proved()
    });
    let result = carried(dispatch(&mut boundary, &request(&dynamic, COMPOSITION)));
    assert_eq!(result.discharge, discharge);
}

#[test]
#[trace("TC-201", "FR-201-AC-4")]
fn a_result_naming_a_foreign_subject_is_refused_with_identity_mismatch() {
    let dynamic = Dynamic::with("workflow", 3);
    let asked = request(&dynamic, COMPOSITION);
    let foreign = dynamic.named("channel", "Reject");
    let mut out = honest(&asked, proved());
    out.subjects.push(foreign.clone());
    let mut boundary = Boundary::new();
    serve(&mut boundary, "b", &BOTH, always(out));
    assert_eq!(
        refused(dispatch(&mut boundary, &asked)),
        DispositionRefusal::IdentityMismatch(IdentityMismatch::Subject(foreign))
    );
}

#[test]
#[trace("TC-201", "FR-201-AC-5")]
fn a_slot_the_backend_returns_undischarged_stays_undischarged() {
    let dynamic = Dynamic::with("workflow", 3);
    let (mut boundary, _) = serving(Out::new(ItemLabel::Inconclusive));
    let result = carried(dispatch(&mut boundary, &request(&dynamic, COMPOSITION)));
    let reading = read_composition_label(result.label, result.execution);
    assert!(result.discharge.is_empty());
    assert_eq!(reading.truth, Some(Truth::Pending));
}

#[test]
#[trace("TC-201", "FR-201-AC-6")]
fn an_empty_discharge_record_marks_no_slot_discharged() {
    let dynamic = Dynamic::with("workflow", 3);
    let (mut boundary, _) = serving(proved());
    let result = carried(dispatch(&mut boundary, &request(&dynamic, COMPOSITION)));
    assert!(result.discharge.is_empty());
}

// ---------------------------------------------------------------- TC-202

#[test]
#[trace("TC-202", "FR-202-AC-1")]
fn composition_artifacts_appear_byte_equal() {
    let dynamic = Dynamic::with("workflow", 3);
    let out = Out {
        certificate: Some("incompatibility-witness-bytes"),
        discharge: vec![("component-guarantee", "slot-0")],
        ..proved()
    };
    let (mut boundary, _) = serving(out.clone());
    let asked = request(&dynamic, COMPOSITION);
    assert_eq!(
        carried(dispatch(&mut boundary, &asked)),
        honest(&asked, out)
    );
}

#[test]
#[trace("TC-202", "FR-202-AC-2")]
fn both_counterexample_halves_and_the_certificate_are_carried() {
    let dynamic = Dynamic::with("workflow", 3);
    let out = Out {
        certificate: Some("simulation-certificate"),
        implementation_half: Some("implementation-trace"),
        specification_half: Some("specification-trace"),
        ..Out::new(ItemLabel::Refuted)
    };
    let (mut boundary, _) = serving(out);
    let result = carried(dispatch(&mut boundary, &request(&dynamic, REFINEMENT)));
    assert_eq!(result.certificate, Some("simulation-certificate"));
    assert_eq!(result.implementation_half, Some("implementation-trace"));
    assert_eq!(result.specification_half, Some("specification-trace"));
}

#[test]
#[trace("TC-202", "FR-202-AC-3")]
fn a_one_halved_counterexample_gains_no_second_half() {
    let dynamic = Dynamic::with("workflow", 3);
    let out = Out {
        implementation_half: Some("implementation-trace"),
        ..Out::new(ItemLabel::Refuted)
    };
    let (mut boundary, _) = serving(out);
    let result = carried(dispatch(&mut boundary, &request(&dynamic, REFINEMENT)));
    assert_eq!(result.implementation_half, Some("implementation-trace"));
    assert_eq!(result.specification_half, None);
}

#[test]
#[trace("TC-202", "FR-202-AC-4")]
fn causes_are_carried_unchanged_and_none_is_a_success() {
    let dynamic = Dynamic::with("workflow", 3);
    let asked = request(&dynamic, COMPOSITION);
    let causes = [
        UnsupportedCause::requested_capability(),
        UnsupportedCause::unrecognised(b"premise-unproved"),
        UnsupportedCause::unrecognised(b"protocol-assumption-not-established"),
        UnsupportedCause::unrecognised(b"a-cause-the-library-does-not-recognise"),
    ];
    for cause in causes {
        let mut boundary = Boundary::new();
        let reported = cause.clone();
        serve(
            &mut boundary,
            "b",
            &BOTH,
            Box::new(move |_| QslReport::Unsupported {
                cause: reported.clone(),
                construct: None,
            }),
        );
        let Disposition::Unsupported(unsupported) = disposition(dispatch(&mut boundary, &asked))
        else {
            panic!("not unsupported");
        };
        assert_eq!(unsupported.cause, cause);
        assert_eq!(unsupported.truth(), Truth::Unavailable);
    }
}

#[test]
#[trace("TC-202", "FR-202-AC-5")]
fn labels_read_into_the_axes_only_through_the_owner_tables() {
    let reading = |label| read_composition_label(label, None);
    let proved = reading(ItemLabel::Proved);
    assert_eq!(
        (proved.execution, proved.truth, proved.basis),
        (
            Some(AssessmentExecution::Completed),
            Some(Truth::Satisfied),
            Some(SettlementBasis::DecisiveWitness)
        )
    );
    let inconclusive = reading(ItemLabel::Inconclusive);
    assert_eq!(
        (
            inconclusive.execution,
            inconclusive.truth,
            inconclusive.basis
        ),
        (
            Some(AssessmentExecution::Completed),
            Some(Truth::Pending),
            Some(SettlementBasis::Unsettled)
        )
    );
    // Certificate-shaped bytes beside an `unsupported` label derive nothing.
    let dynamic = Dynamic::with("workflow", 3);
    let (mut boundary, _) = serving(Out {
        certificate: Some("certificate-shaped"),
        ..Out::new(ItemLabel::Unsupported)
    });
    let result = carried(dispatch(&mut boundary, &request(&dynamic, COMPOSITION)));
    let unsupported = read_composition_label(result.label, result.execution);
    assert_eq!(
        (unsupported.execution, unsupported.truth, unsupported.basis),
        (
            Some(AssessmentExecution::Unsupported),
            Some(Truth::Unavailable),
            Some(SettlementBasis::Unavailable)
        )
    );
    // `failed` takes the execution the backend states: an incomplete result
    // is `failed` with `resource-incomplete`.
    let exhausted = read_composition_label(
        ItemLabel::Failed,
        Some(AssessmentExecution::ResourceIncomplete),
    );
    assert_eq!(
        exhausted.execution,
        Some(AssessmentExecution::ResourceIncomplete)
    );
    assert_eq!(exhausted.truth, Some(Truth::Unavailable));
    assert_eq!(exhausted.basis, Some(SettlementBasis::Unavailable));
    let misstated = read_composition_label(ItemLabel::Failed, Some(AssessmentExecution::Completed));
    assert_eq!(misstated.execution, None);
    // `incomplete` is not a label of the table: carried, nothing derived.
    let incomplete = reading(ItemLabel::Incomplete);
    assert_eq!(
        (
            incomplete.label,
            incomplete.execution,
            incomplete.truth,
            incomplete.basis
        ),
        (ItemLabel::Incomplete, None, None, None)
    );
}

#[test]
#[trace("TC-202", "FR-202-AC-6")]
fn refused_is_never_derived_and_a_refinement_label_reads_into_nothing() {
    for label in [
        ItemLabel::Proved,
        ItemLabel::Refuted,
        ItemLabel::Inconclusive,
        ItemLabel::Unsupported,
        ItemLabel::Failed,
        ItemLabel::Incomplete,
        ItemLabel::Declined,
    ] {
        assert_ne!(
            read_composition_label(label, None).execution,
            Some(AssessmentExecution::Refused)
        );
        let refinement = read_refinement_label(label);
        assert_eq!(
            (refinement.execution, refinement.truth, refinement.basis),
            (None, None, None)
        );
    }
    let dynamic = Dynamic::with("workflow", 3);
    let mut boundary = Boundary::new();
    serve(
        &mut boundary,
        "b",
        &BOTH,
        Box::new(|_| {
            QslReport::Refusal(CatalogCode::new(
                "invalid_capability",
                "malformed-declaration",
            ))
        }),
    );
    let Disposition::Refused(refusal) =
        disposition(dispatch(&mut boundary, &request(&dynamic, COMPOSITION)))
    else {
        panic!("not refused");
    };
    assert_eq!(
        refusal.code,
        CatalogCode::new("invalid_capability", "malformed-declaration")
    );
}

#[test]
#[trace("TC-202", "FR-202-AC-7")]
fn an_unsupported_behaviour_or_missing_correspondence_is_never_satisfied() {
    let dynamic = Dynamic::with("workflow", 3);
    let mut boundary = Boundary::new();
    serve(
        &mut boundary,
        "b",
        &BOTH,
        Box::new(|_| QslReport::Unsupported {
            cause: UnsupportedCause::unrecognised(b"missing-correspondence"),
            construct: None,
        }),
    );
    let Disposition::Unsupported(unsupported) =
        disposition(dispatch(&mut boundary, &request(&dynamic, REFINEMENT)))
    else {
        panic!("not unsupported");
    };
    assert_eq!(
        unsupported.cause,
        UnsupportedCause::unrecognised(b"missing-correspondence")
    );
    assert_ne!(unsupported.truth(), Truth::Satisfied);
}

// ---------------------------------------------------------------- TC-203

fn cyclic(dynamic: &Dynamic) -> support::assume_guarantee::Req {
    let mut asked = request(dynamic, COMPOSITION);
    asked.cycles = vec![vec![
        dynamic.named("channel", "Head"),
        dynamic.named("channel", "Tail"),
    ]];
    asked.assumptions = vec![dynamic.named("channel", "New")];
    asked
}

#[test]
#[trace("TC-203", "FR-203-AC-1")]
fn a_cycle_reported_unsupported_carries_the_cause_and_the_cycles_node_ids() {
    let dynamic = Dynamic::with("workflow", 3);
    let mut boundary = Boundary::new();
    serve(
        &mut boundary,
        "b",
        &BOTH,
        Box::new(|_| QslReport::Unsupported {
            cause: UnsupportedCause::requested_capability(),
            construct: None,
        }),
    );
    let asked = cyclic(&dynamic);
    let Disposition::Unsupported(unsupported) = disposition(dispatch(&mut boundary, &asked)) else {
        panic!("not unsupported");
    };
    assert_eq!(unsupported.cause, UnsupportedCause::requested_capability());
    assert_eq!(unsupported.premises.request.cycles, asked.cycles);
}

#[test]
#[trace("TC-203", "FR-203-AC-2")]
fn other_cycle_outcomes_are_not_remapped_to_unsupported() {
    let dynamic = Dynamic::with("workflow", 3);
    let asked = cyclic(&dynamic);
    let mut boundary = Boundary::new();
    serve(
        &mut boundary,
        "b",
        &BOTH,
        always(Out::new(ItemLabel::Inconclusive)),
    );
    assert_eq!(
        carried(dispatch(&mut boundary, &asked)).label,
        ItemLabel::Inconclusive
    );
    let mut refusing = Boundary::new();
    serve(
        &mut refusing,
        "b",
        &BOTH,
        Box::new(|_| {
            QslReport::Refusal(CatalogCode::new(
                "invalid_capability",
                "malformed-declaration",
            ))
        }),
    );
    assert!(matches!(
        disposition(dispatch(&mut refusing, &asked)),
        Disposition::Refused(_)
    ));
}

#[test]
#[trace("TC-203", "FR-203-AC-3")]
fn the_cycle_request_makes_one_call_and_adds_no_cycle_guarantee_as_assumption() {
    let dynamic = Dynamic::with("workflow", 3);
    let (mut boundary, calls) = serving(proved());
    let asked = cyclic(&dynamic);
    let outcome = dispatch(&mut boundary, &asked);
    assert_eq!(calls.len(), 1);
    let received = &calls.all()[0].request;
    assert_eq!(received.assumptions, asked.assumptions);
    assert!(received
        .cycles
        .iter()
        .flatten()
        .all(|node| !received.assumptions.contains(node)));
    let ProtocolOutcome::Settled(ClaimSettlement::Result { premises, .. }) = outcome else {
        panic!("not carried");
    };
    assert_eq!(premises.request.assumptions, asked.assumptions);
}

#[test]
#[trace("TC-203", "FR-203-AC-4")]
fn a_proved_member_does_not_discharge_the_cycle_or_make_it_success() {
    let dynamic = Dynamic::with("workflow", 3);
    let mut boundary = Boundary::new();
    serve(
        &mut boundary,
        "b",
        &BOTH,
        Box::new(|_| QslReport::Unsupported {
            cause: UnsupportedCause::requested_capability(),
            construct: None,
        }),
    );
    let outcome = dispatch(&mut boundary, &cyclic(&dynamic));
    assert!(matches!(disposition(outcome), Disposition::Unsupported(_)));
}

#[test]
#[trace("TC-203", "FR-203-AC-5")]
fn an_unsupported_cycle_leaves_an_acyclic_claim_unchanged() {
    let dynamic = Dynamic::with("workflow", 3);
    let mut boundary = Boundary::new();
    serve(
        &mut boundary,
        "b",
        &BOTH,
        Box::new(|bound| {
            if bound.request.cycles.is_empty() {
                QslReport::Result(honest(&bound.request, proved()))
            } else {
                QslReport::Unsupported {
                    cause: UnsupportedCause::requested_capability(),
                    construct: None,
                }
            }
        }),
    );
    let acyclic = request(&dynamic, COMPOSITION);
    let alone = dispatch_claims(&mut boundary, std::slice::from_ref(&acyclic));
    let beside = dispatch_claims(&mut boundary, &[cyclic(&dynamic), acyclic]);
    assert_eq!(beside[1], alone[0]);
}

#[test]
#[trace("TC-203", "FR-203-AC-6")]
fn the_cycle_disposition_introduces_no_cause_name() {
    let dynamic = Dynamic::with("workflow", 3);
    let mut boundary = Boundary::new();
    let cause = UnsupportedCause::requested_capability();
    let reported = cause.clone();
    serve(
        &mut boundary,
        "b",
        &BOTH,
        Box::new(move |_| QslReport::Unsupported {
            cause: reported.clone(),
            construct: None,
        }),
    );
    let Disposition::Unsupported(unsupported) =
        disposition(dispatch(&mut boundary, &cyclic(&dynamic)))
    else {
        panic!("not unsupported");
    };
    assert_eq!(unsupported.cause, cause);
}

// ---------------------------------------------------------------- TC-204

fn retaining(
    asked: &support::assume_guarantee::Req,
    assumptions: Option<Vec<quire_contract_model::CheckedNodeId>>,
    label: ItemLabel,
) -> Outcome {
    let mut out = honest(asked, Out::new(label));
    out.assumptions = assumptions;
    let mut boundary = Boundary::new();
    serve(&mut boundary, "b", &BOTH, always(out));
    dispatch(&mut boundary, asked)
}

#[test]
#[trace("TC-204", "FR-204-AC-1")]
fn a_result_retaining_exactly_the_requests_assumptions_is_carried() {
    let dynamic = Dynamic::with("workflow", 3);
    let asked = request(&dynamic, REFINEMENT);
    let outcome = retaining(&asked, Some(asked.assumptions.clone()), ItemLabel::Proved);
    assert_eq!(carried(outcome).label, ItemLabel::Proved);
}

#[test]
#[trace("TC-204", "FR-204-AC-2")]
fn a_proof_under_an_extra_assumption_is_refused_naming_it() {
    let dynamic = Dynamic::with("workflow", 3);
    let asked = request(&dynamic, REFINEMENT);
    let extra = dynamic.named("channel", "New");
    let mut stronger = asked.assumptions.clone();
    stronger.push(extra.clone());
    assert_eq!(
        refused(retaining(&asked, Some(stronger), ItemLabel::Proved)),
        DispositionRefusal::IdentityMismatch(IdentityMismatch::Assumption(extra))
    );
}

#[test]
#[trace("TC-204", "FR-204-AC-3")]
fn a_strict_subset_of_the_requests_assumptions_is_carried() {
    let dynamic = Dynamic::with("workflow", 3);
    let asked = request(&dynamic, REFINEMENT);
    let subset = vec![asked.assumptions[0].clone()];
    assert!(matches!(
        retaining(&asked, Some(subset), ItemLabel::Proved),
        ProtocolOutcome::Settled(ClaimSettlement::Result { .. })
    ));
}

#[test]
#[trace("TC-204", "FR-204-AC-4")]
fn a_violation_under_an_extra_assumption_is_refused_too() {
    let dynamic = Dynamic::with("workflow", 3);
    let asked = request(&dynamic, REFINEMENT);
    let extra = dynamic.named("channel", "New");
    let mut stronger = asked.assumptions.clone();
    stronger.push(extra.clone());
    let outcome = retaining(&asked, Some(stronger), ItemLabel::Refuted);
    assert_eq!(
        refused(outcome),
        DispositionRefusal::IdentityMismatch(IdentityMismatch::Assumption(extra))
    );
}

#[test]
#[trace("TC-204", "FR-204-AC-5")]
fn a_refused_refinement_leaves_other_claims_unchanged_and_calls_no_fallback() {
    let dynamic = Dynamic::with("workflow", 3);
    let mut boundary = Boundary::new();
    let fallback = serve(
        &mut boundary,
        "fallback",
        &[Capability::Composition],
        always(proved()),
    );
    let extra = dynamic.named("channel", "New");
    let refining = serve(
        &mut boundary,
        "refiner",
        &[Capability::Refinement],
        Box::new(move |bound| {
            let mut out = honest(&bound.request, proved());
            out.assumptions
                .as_mut()
                .expect("retained")
                .push(extra.clone());
            QslReport::Result(out)
        }),
    );
    let composition = request(&dynamic, COMPOSITION);
    let alone = dispatch_claims(&mut boundary, std::slice::from_ref(&composition));
    let beside = dispatch_claims(&mut boundary, &[request(&dynamic, REFINEMENT), composition]);
    assert!(matches!(beside[0], ProtocolOutcome::Refused(_)));
    assert_eq!(beside[1], alone[0]);
    assert_eq!(refining.len(), 1);
    assert_eq!(fallback.len(), 2);
}

#[test]
#[trace("TC-204", "FR-204-AC-6")]
fn the_backends_own_strengthened_assumption_finding_is_carried_not_refused() {
    let dynamic = Dynamic::with("workflow", 3);
    let mut boundary = Boundary::new();
    serve(
        &mut boundary,
        "b",
        &BOTH,
        Box::new(|_| QslReport::Unsupported {
            cause: UnsupportedCause::unrecognised(b"protocol-assumption-not-established"),
            construct: None,
        }),
    );
    let Disposition::Unsupported(unsupported) =
        disposition(dispatch(&mut boundary, &request(&dynamic, REFINEMENT)))
    else {
        panic!("not carried");
    };
    assert_eq!(
        unsupported.cause,
        UnsupportedCause::unrecognised(b"protocol-assumption-not-established")
    );
}

#[test]
#[trace("TC-204", "FR-204-AC-7")]
fn an_omitted_field_is_refused_and_an_empty_set_is_a_strict_subset() {
    let dynamic = Dynamic::with("workflow", 3);
    let asked = request(&dynamic, REFINEMENT);
    assert_eq!(
        refused(retaining(&asked, None, ItemLabel::Proved)),
        DispositionRefusal::IdentityMismatch(IdentityMismatch::AbsentAssumptions)
    );
    assert!(matches!(
        retaining(&asked, Some(Vec::new()), ItemLabel::Proved),
        ProtocolOutcome::Settled(ClaimSettlement::Result { .. })
    ));
}
