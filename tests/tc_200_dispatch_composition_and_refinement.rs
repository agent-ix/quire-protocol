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
use qsl_foundation::BackendId;
use qsl_replay::{ItemLabel, SettlementBasis, Truth};
use qsl_semantics::check::Capability;
use quire_contract_model::CheckedNodeId;
use quire_protocol::assume_guarantee::{
    read_composition_label, read_refinement_label, AssumeGuaranteeKind, IdentityMismatch,
    PremiseList, PremiseLists,
};
use quire_protocol::closure::AssessmentExecution;
use quire_protocol::dispatch::{
    dispatch_claims, ClaimDisposition, ClaimRequest, DispositionRefusal, ProtocolOutcome,
};
use quire_protocol::disposition::{
    ClaimSettlement, Disposition, QslReport, UnsupportedCause, Warning,
};
use quire_protocol::refusal::ProtocolCauseKind;
use quire_semantic_value::catalog::CatalogCode;
use support::assume_guarantee::{
    always, foreign, honest, request, serve, Bind, Boundary, Calls, Claims, Out, Req, Settled,
};
use support::dynamic::Dynamic;

const COMPOSITION: AssumeGuaranteeKind = AssumeGuaranteeKind::Composition;
const REFINEMENT: AssumeGuaranteeKind = AssumeGuaranteeKind::Refinement;
const BOTH: [Capability; 2] = [Capability::Composition, Capability::Refinement];
const CONFORMANT: &str = "conformant";

type Outcome = ProtocolOutcome<Out, Bind>;

/// Settle `requests` through the one dispatch path, in order.
fn settle(boundary: &mut Boundary, dynamic: &Dynamic, requests: Vec<Claims>) -> Vec<Settled> {
    dispatch_claims(boundary, dynamic.linked().declarations(), &requests)
}

fn one(boundary: &mut Boundary, dynamic: &Dynamic, asked: &Req) -> Outcome {
    let settled = settle(
        boundary,
        dynamic,
        vec![Claims::AssumeGuarantee(asked.clone())],
    );
    guaranteed(settled.into_iter().next().expect("one disposition"))
}

fn guaranteed(disposition: Settled) -> Outcome {
    match disposition {
        ClaimDisposition::AssumeGuarantee(outcome) => outcome,
        other => panic!("not an assume-guarantee disposition: {other:?}"),
    }
}

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

fn unsupported_with(cause: UnsupportedCause) -> Boundary {
    let mut boundary = Boundary::new();
    serve(
        &mut boundary,
        "b",
        &BOTH,
        Box::new(move |_| QslReport::Unsupported {
            cause: cause.clone(),
            construct: None,
        }),
    );
    boundary
}

fn proved() -> Out {
    Out::new(ItemLabel::Proved)
}

/// A boundary whose one stand-in serves both kinds and answers `result`
/// shaped to each request.
fn serving(result: Out) -> (Boundary, Calls) {
    let mut boundary = Boundary::new();
    let calls = serve(
        &mut boundary,
        "composer",
        &BOTH,
        Box::new(move |bound| QslReport::Result(honest(&bound.request, result.clone()))),
    );
    (boundary, calls)
}

fn malformed() -> CatalogCode {
    CatalogCode::new("invalid_capability", "malformed-declaration")
}

// ---------------------------------------------------------------- TC-200

#[test]
#[trace("TC-200", "FR-200-AC-1")]
fn a_composition_claim_sends_one_item_naming_the_component_packages() {
    let dynamic = Dynamic::with("workflow", 3);
    let (mut boundary, calls) = serving(proved());
    let asked = request(&dynamic, COMPOSITION);
    let result = carried(one(&mut boundary, &dynamic, &asked));
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
    let Disposition::Unsupported(unsupported) = disposition(one(&mut boundary, &dynamic, &asked))
    else {
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
fn an_unsupported_composition_leaves_conformance_and_refinement_unchanged() {
    let dynamic = Dynamic::with("workflow", 3);
    let mut boundary = Boundary::new();
    serve(
        &mut boundary,
        "refiner",
        &[Capability::Refinement],
        always(proved()),
    );
    let refinement = request(&dynamic, REFINEMENT);
    let conformance = || Claims::GlobalConformance(CONFORMANT);
    let alone = settle(
        &mut boundary,
        &dynamic,
        vec![conformance(), Claims::AssumeGuarantee(refinement.clone())],
    );
    let together = settle(
        &mut boundary,
        &dynamic,
        vec![
            conformance(),
            Claims::AssumeGuarantee(request(&dynamic, COMPOSITION)),
            Claims::AssumeGuarantee(refinement),
        ],
    );
    assert!(matches!(
        guaranteed(together[1].clone()),
        ProtocolOutcome::Settled(ClaimSettlement::Disposition(Disposition::Unsupported(_)))
    ));
    assert_eq!(together[0], alone[0]);
    assert_eq!(together[2], alone[1]);
}

#[test]
#[trace("TC-200", "FR-200-AC-4")]
fn a_conformance_result_neither_decides_nor_is_decided_by_a_composition() {
    let dynamic = Dynamic::with("workflow", 3);
    let (mut boundary, calls) = serving(Out::new(ItemLabel::Inconclusive));
    let asked = request(&dynamic, COMPOSITION);
    let alone = one(&mut boundary, &dynamic, &asked);
    let beside = settle(
        &mut boundary,
        &dynamic,
        vec![
            Claims::GlobalConformance(CONFORMANT),
            Claims::AssumeGuarantee(asked),
        ],
    );
    assert_eq!(beside[0], ClaimDisposition::GlobalConformance(CONFORMANT));
    assert_eq!(guaranteed(beside[1].clone()), alone);
    assert_eq!(carried(alone).label, ItemLabel::Inconclusive);
    // The backend was asked the composition only; no conformance premise set
    // reached it.
    assert_eq!(calls.len(), 2);
}

/// One claim per component, each with that component removed or substituted.
fn broken_components(dynamic: &Dynamic, whole: &Req) -> Vec<(&'static str, Req, bool)> {
    let stranger = foreign(dynamic);
    let mut cases = Vec::new();
    let mut vary = |name, change: &dyn Fn(&mut Req)| {
        let mut broken = whole.clone();
        change(&mut broken);
        cases.push((name, broken, false));
    };
    vary("fragment removed", &|r| r.fragment = Default::default());
    vary("fragment substituted", &|r| {
        r.fragment.channels = vec![stranger.clone()];
    });
    vary("assumption substituted", &|r| {
        r.assumptions = vec![stranger.clone()];
    });
    vary("source substituted", &|r| {
        r.sources = vec![stranger.clone()]
    });
    vary("backend substituted", &|r| {
        r.backend = Some(BackendId::new("elsewhere"));
    });
    cases
}

#[test]
#[trace("TC-200", "FR-200-AC-5")]
fn removing_or_substituting_a_component_refuses_only_that_disposition() {
    let dynamic = Dynamic::with("workflow", 3);
    let (mut boundary, _) = serving(proved());
    let whole = request(&dynamic, COMPOSITION);
    let alone = one(&mut boundary, &dynamic, &whole);
    assert_eq!(
        carried(alone.clone()),
        honest(&whole, proved()),
        "the intact claim is carried"
    );
    for (name, broken, _) in broken_components(&dynamic, &whole) {
        let settled = settle(
            &mut boundary,
            &dynamic,
            vec![
                Claims::AssumeGuarantee(broken),
                Claims::AssumeGuarantee(whole.clone()),
            ],
        );
        let first = guaranteed(settled[0].clone());
        assert!(
            matches!(
                first,
                ProtocolOutcome::Refused(_) | ProtocolOutcome::InvalidRequest { .. }
            ),
            "{name}: {first:?}"
        );
        assert_eq!(guaranteed(settled[1].clone()), alone, "{name}");
    }
    // A proved result retaining no source identity is refused, and the
    // disposition retains the backend FR-290 selected.
    let mut unsourced = whole.clone();
    unsourced.sources.clear();
    assert_eq!(
        refused(one(&mut boundary, &dynamic, &unsourced)),
        DispositionRefusal::NoSourceIdentity
    );
    let ProtocolOutcome::Settled(ClaimSettlement::Result { premises, .. }) = alone else {
        panic!("not carried");
    };
    assert_eq!(premises.selected, Some(BackendId::new("composer")));
}

#[test]
#[trace("TC-200", "FR-200-AC-5")]
fn an_empty_subject_list_or_fragment_is_refused_by_name() {
    let dynamic = Dynamic::with("workflow", 3);
    let (mut boundary, _) = serving(proved());
    let whole = request(&dynamic, COMPOSITION);
    let mut no_subjects = whole.clone();
    no_subjects.subjects.clear();
    assert_eq!(
        refused(one(&mut boundary, &dynamic, &no_subjects)),
        DispositionRefusal::NoSubjects
    );
    let mut no_fragment = whole.clone();
    no_fragment.fragment = Default::default();
    assert_eq!(
        refused(one(&mut boundary, &dynamic, &no_fragment)),
        DispositionRefusal::NoFragment
    );
    let stranger = foreign(&dynamic);
    let mut substituted = whole;
    substituted.subjects = vec![stranger.clone()];
    assert_eq!(
        refused(one(&mut boundary, &dynamic, &substituted)),
        DispositionRefusal::UndeclaredSubject(stranger)
    );
}

#[test]
#[trace("TC-200", "FR-200-AC-6")]
fn the_disposition_equals_the_backends_result_and_nothing_when_it_returns_none() {
    let dynamic = Dynamic::with("workflow", 3);
    let (mut boundary, _) = serving(Out::new(ItemLabel::Inconclusive));
    let asked = request(&dynamic, COMPOSITION);
    assert_eq!(
        carried(one(&mut boundary, &dynamic, &asked)).label,
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
        disposition(one(&mut silent, &dynamic, &asked)),
        Disposition::Failed(_)
    ));
}

// ---------------------------------------------------------------- TC-201

#[test]
#[trace("TC-201", "FR-201-AC-1")]
fn the_four_premise_lists_reach_the_backend_as_four_separate_lists() {
    let dynamic = Dynamic::with("workflow", 3);
    let (mut boundary, calls) = serving(proved());
    one(&mut boundary, &dynamic, &request(&dynamic, COMPOSITION));
    let sent = &calls.all()[0].request.premises;
    assert_eq!(sent.environment, vec!["env-1"]);
    assert_eq!(sent.relies, vec!["rely-1", "rely-2"]);
    assert_eq!(sent.liveness_dependencies, vec!["live-1"]);
    assert_eq!(sent.fairness, vec!["fair-1"]);
}

#[test]
#[trace("TC-201", "FR-201-AC-2")]
fn a_result_retaining_dropped_merged_or_omitted_lists_is_refused_alone() {
    let dynamic = Dynamic::with("workflow", 3);
    let asked = request(&dynamic, COMPOSITION);
    let retaining = |lists: Option<PremiseLists<&'static str>>| {
        let mut boundary = Boundary::new();
        let mut out = honest(&asked, proved());
        out.premises = lists;
        serve(&mut boundary, "b", &BOTH, always(out));
        one(&mut boundary, &dynamic, &asked)
    };
    let mut dropped = asked.premises.clone();
    dropped.fairness.clear();
    assert_eq!(
        refused(retaining(Some(dropped))),
        DispositionRefusal::RetainedPremiseList(PremiseList::Fairness)
    );
    let mut merged = asked.premises.clone();
    merged.relies.append(&mut merged.environment);
    assert_eq!(
        refused(retaining(Some(merged))),
        DispositionRefusal::RetainedPremiseList(PremiseList::Environment)
    );
    assert_eq!(
        refused(retaining(None)),
        DispositionRefusal::IdentityMismatch(IdentityMismatch::AbsentPremiseLists)
    );
    assert!(matches!(
        retaining(Some(asked.premises.clone())),
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
    let result = carried(one(
        &mut boundary,
        &dynamic,
        &request(&dynamic, COMPOSITION),
    ));
    assert_eq!(result.discharge, discharge);
}

#[test]
#[trace("TC-201", "FR-201-AC-4")]
fn a_result_naming_foreign_subjects_is_refused_naming_each() {
    let dynamic = Dynamic::with("workflow", 3);
    let asked = request(&dynamic, COMPOSITION);
    let first = dynamic.named("channel", "Reject");
    let second = dynamic.named("channel", "Tail");
    let mut out = honest(&asked, proved());
    out.subjects.extend([first.clone(), second.clone()]);
    let mut boundary = Boundary::new();
    serve(&mut boundary, "b", &BOTH, always(out));
    let refusal = refused(one(&mut boundary, &dynamic, &asked));
    assert_eq!(
        refusal,
        DispositionRefusal::IdentityMismatch(IdentityMismatch::Subjects(vec![first, second]))
    );
    assert_eq!(refusal.cause(), Some(ProtocolCauseKind::IdentityMismatch));
}

#[test]
#[trace("TC-201", "FR-201-AC-5")]
fn a_slot_the_backend_returns_undischarged_stays_undischarged_and_is_no_success() {
    let dynamic = Dynamic::with("workflow", 3);
    let mut asked = request(&dynamic, COMPOSITION);
    // A local reading would discharge this rely slot from a supplied guarantee.
    asked.premises.relies = vec!["rely-1"];
    let (mut boundary, _) = serving(Out::new(ItemLabel::Inconclusive));
    let outcome = one(&mut boundary, &dynamic, &asked);
    let reading = outcome.label_reading().expect("a carried result");
    assert_eq!(reading.truth, Some(Truth::Pending));
    assert_eq!(reading.basis, Some(SettlementBasis::Unsettled));
    let ProtocolOutcome::Settled(ClaimSettlement::Result { result, premises }) = outcome else {
        panic!("not carried");
    };
    assert!(result.discharge.is_empty());
    assert_eq!(premises.request.premises.relies, vec!["rely-1"]);
}

#[test]
#[trace("TC-201", "FR-201-AC-6")]
fn an_empty_discharge_record_marks_no_slot_discharged() {
    let dynamic = Dynamic::with("workflow", 3);
    let (mut boundary, _) = serving(proved());
    let result = carried(one(
        &mut boundary,
        &dynamic,
        &request(&dynamic, COMPOSITION),
    ));
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
        carried(one(&mut boundary, &dynamic, &asked)),
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
    let result = carried(one(&mut boundary, &dynamic, &request(&dynamic, REFINEMENT)));
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
    let result = carried(one(&mut boundary, &dynamic, &request(&dynamic, REFINEMENT)));
    assert_eq!(result.implementation_half, Some("implementation-trace"));
    assert_eq!(result.specification_half, None);
}

#[test]
#[trace("TC-202", "FR-202-AC-4")]
fn causes_are_carried_unchanged_and_none_is_a_success() {
    let dynamic = Dynamic::with("workflow", 3);
    let asked = request(&dynamic, COMPOSITION);
    let named = |name: &'static str| {
        UnsupportedCause::RequestedCapability(CatalogCode::new("unsupported_projection", name))
    };
    let causes = [
        UnsupportedCause::requested_capability(),
        named("premise-unproved"),
        named("protocol-assumption-not-established"),
        UnsupportedCause::unrecognised(b"a-cause-the-library-does-not-recognise"),
    ];
    for cause in causes {
        let mut boundary = unsupported_with(cause.clone());
        let Disposition::Unsupported(unsupported) =
            disposition(one(&mut boundary, &dynamic, &asked))
        else {
            panic!("not unsupported");
        };
        assert_eq!(unsupported.cause, cause);
        assert!(
            std::mem::discriminant(&unsupported.cause) == std::mem::discriminant(&cause),
            "the cause was mapped to another kind"
        );
        assert_eq!(unsupported.truth(), Truth::Unavailable);
        assert_eq!(unsupported.basis(), SettlementBasis::Unavailable);
    }
}

#[test]
#[trace("TC-202", "FR-202-AC-5")]
fn labels_read_into_the_axes_only_through_the_owner_tables() {
    let dynamic = Dynamic::with("workflow", 3);
    let read = |out: Out| {
        let (mut boundary, _) = serving(out);
        one(&mut boundary, &dynamic, &request(&dynamic, COMPOSITION))
            .label_reading()
            .expect("a carried result")
    };
    let proved = read(proved());
    assert_eq!(
        (proved.execution, proved.truth, proved.basis),
        (
            Some(AssessmentExecution::Completed),
            Some(Truth::Satisfied),
            Some(SettlementBasis::DecisiveWitness)
        )
    );
    let inconclusive = read(Out::new(ItemLabel::Inconclusive));
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
    let unsupported = read(Out {
        certificate: Some("certificate-shaped"),
        ..Out::new(ItemLabel::Unsupported)
    });
    assert_eq!(
        (unsupported.execution, unsupported.truth, unsupported.basis),
        (
            Some(AssessmentExecution::Unsupported),
            Some(Truth::Unavailable),
            Some(SettlementBasis::Unavailable)
        )
    );
    // An incomplete result is `failed` with the stated `resource-incomplete`.
    for stated in [
        AssessmentExecution::ResourceIncomplete,
        AssessmentExecution::Failed,
    ] {
        let failed = read(Out {
            execution: Some(stated),
            ..Out::new(ItemLabel::Failed)
        });
        assert_eq!(
            (failed.execution, failed.truth, failed.basis),
            (
                Some(stated),
                Some(Truth::Unavailable),
                Some(SettlementBasis::Unavailable)
            )
        );
    }
    // `incomplete` is not a label of the table: carried, nothing derived.
    let incomplete = read(Out::new(ItemLabel::Incomplete));
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
#[trace("TC-202", "FR-202-AC-5")]
fn a_stated_execution_that_contradicts_the_label_is_refused() {
    let dynamic = Dynamic::with("workflow", 3);
    let asked = request(&dynamic, COMPOSITION);
    for (label, stated) in [
        (ItemLabel::Failed, AssessmentExecution::Completed),
        (ItemLabel::Proved, AssessmentExecution::Failed),
        (ItemLabel::Unsupported, AssessmentExecution::Refused),
    ] {
        let (mut boundary, _) = serving(Out {
            execution: Some(stated),
            ..Out::new(label)
        });
        assert_eq!(
            refused(one(&mut boundary, &dynamic, &asked)),
            DispositionRefusal::ContradictoryExecution(label)
        );
    }
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
        Box::new(|_| QslReport::Refusal(malformed())),
    );
    let Disposition::Refused(refusal) = disposition(one(
        &mut boundary,
        &dynamic,
        &request(&dynamic, COMPOSITION),
    )) else {
        panic!("not refused");
    };
    assert_eq!(refusal.code, malformed());
}

#[test]
#[trace("TC-202", "FR-202-AC-7")]
fn an_unsupported_behaviour_or_missing_correspondence_is_never_satisfied() {
    let dynamic = Dynamic::with("workflow", 3);
    let cause = UnsupportedCause::unrecognised(b"missing-correspondence");
    let mut boundary = unsupported_with(cause.clone());
    let Disposition::Unsupported(unsupported) =
        disposition(one(&mut boundary, &dynamic, &request(&dynamic, REFINEMENT)))
    else {
        panic!("not unsupported");
    };
    assert_eq!(unsupported.cause, cause);
    assert_ne!(unsupported.truth(), Truth::Satisfied);
}

// ---------------------------------------------------------------- TC-203

fn cyclic(dynamic: &Dynamic) -> Req {
    let mut asked = request(dynamic, COMPOSITION);
    asked.cycles = vec![vec![
        dynamic.named("channel", "Reject"),
        dynamic.named("channel", "Tail"),
    ]];
    asked.assumptions = vec![dynamic.named("channel", "Head")];
    asked
}

#[test]
#[trace("TC-203", "FR-203-AC-1")]
fn a_cycle_reported_unsupported_carries_the_cause_and_the_cycles_node_ids() {
    let dynamic = Dynamic::with("workflow", 3);
    let mut boundary = unsupported_with(UnsupportedCause::requested_capability());
    let asked = cyclic(&dynamic);
    let Disposition::Unsupported(unsupported) = disposition(one(&mut boundary, &dynamic, &asked))
    else {
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
    let (mut boundary, _) = serving(Out::new(ItemLabel::Inconclusive));
    assert_eq!(
        carried(one(&mut boundary, &dynamic, &asked)).label,
        ItemLabel::Inconclusive
    );
    let mut refusing = Boundary::new();
    serve(
        &mut refusing,
        "b",
        &BOTH,
        Box::new(|_| QslReport::Refusal(malformed())),
    );
    let Disposition::Refused(refusal) = disposition(one(&mut refusing, &dynamic, &asked)) else {
        panic!("not refused");
    };
    assert_eq!(refusal.code, malformed());
}

#[test]
#[trace("TC-203", "FR-203-AC-3")]
fn a_cycle_guarantee_is_never_an_assumption_and_the_backend_is_asked_once() {
    let dynamic = Dynamic::with("workflow", 3);
    let (mut boundary, calls) = serving(proved());
    let asked = cyclic(&dynamic);
    carried(one(&mut boundary, &dynamic, &asked));
    assert_eq!(calls.len(), 1);
    let received = &calls.all()[0].request;
    assert!(received
        .cycles
        .iter()
        .flatten()
        .all(|node| !received.assumptions.contains(node)));
    // Offering a cycle guarantee as an assumption, in the request or in the
    // result, is refused and no backend is asked for the request.
    let member = asked.cycles[0][0].clone();
    let mut offered = asked.clone();
    offered.assumptions.push(member.clone());
    assert_eq!(
        refused(one(&mut boundary, &dynamic, &offered)),
        DispositionRefusal::CycleGuaranteeAsAssumption(member.clone())
    );
    assert_eq!(calls.len(), 1);
    let mut leaking = honest(&asked, proved());
    leaking
        .assumptions
        .as_mut()
        .expect("retained")
        .push(member.clone());
    let mut leaky = Boundary::new();
    serve(&mut leaky, "b", &BOTH, always(leaking));
    assert_eq!(
        refused(one(&mut leaky, &dynamic, &asked)),
        DispositionRefusal::CycleGuaranteeAsAssumption(member)
    );
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
    let cycle = cyclic(&dynamic);
    let mut member = request(&dynamic, COMPOSITION);
    member.subjects = vec![cycle.cycles[0][0].clone()];
    let settled = settle(
        &mut boundary,
        &dynamic,
        vec![
            Claims::AssumeGuarantee(member),
            Claims::AssumeGuarantee(cycle),
        ],
    );
    let member = guaranteed(settled[0].clone());
    assert_eq!(
        member.label_reading().map(|r| r.truth),
        Some(Some(Truth::Satisfied))
    );
    let cycle = guaranteed(settled[1].clone());
    assert!(cycle.label_reading().is_none());
    assert!(matches!(disposition(cycle), Disposition::Unsupported(_)));
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
    let alone = one(&mut boundary, &dynamic, &acyclic);
    let beside = settle(
        &mut boundary,
        &dynamic,
        vec![
            Claims::AssumeGuarantee(cyclic(&dynamic)),
            Claims::AssumeGuarantee(acyclic),
        ],
    );
    assert_eq!(guaranteed(beside[1].clone()), alone);
}

#[test]
#[trace("TC-203", "FR-203-AC-6")]
fn the_cycle_disposition_introduces_no_cause_name() {
    let dynamic = Dynamic::with("workflow", 3);
    let cause = UnsupportedCause::requested_capability();
    let mut boundary = unsupported_with(cause.clone());
    let Disposition::Unsupported(unsupported) =
        disposition(one(&mut boundary, &dynamic, &cyclic(&dynamic)))
    else {
        panic!("not unsupported");
    };
    assert_eq!(unsupported.cause, cause);
}

// ---------------------------------------------------------------- TC-204

fn retaining(
    dynamic: &Dynamic,
    asked: &Req,
    assumptions: Option<Vec<CheckedNodeId>>,
    label: ItemLabel,
) -> Outcome {
    let mut out = honest(asked, Out::new(label));
    out.assumptions = assumptions;
    let mut boundary = Boundary::new();
    serve(&mut boundary, "b", &BOTH, always(out));
    one(&mut boundary, dynamic, asked)
}

fn stronger(dynamic: &Dynamic, asked: &Req) -> (Vec<CheckedNodeId>, CheckedNodeId) {
    let extra = dynamic.named("channel", "New");
    let mut assumptions = asked.assumptions.clone();
    assumptions.push(extra.clone());
    (assumptions, extra)
}

#[test]
#[trace("TC-204", "FR-204-AC-1")]
fn a_result_retaining_exactly_the_requests_assumptions_is_carried() {
    let dynamic = Dynamic::with("workflow", 3);
    let asked = request(&dynamic, REFINEMENT);
    let outcome = retaining(
        &dynamic,
        &asked,
        Some(asked.assumptions.clone()),
        ItemLabel::Proved,
    );
    assert_eq!(carried(outcome).label, ItemLabel::Proved);
}

#[test]
#[trace("TC-204", "FR-204-AC-2")]
fn a_proof_under_extra_assumptions_is_refused_naming_each() {
    let dynamic = Dynamic::with("workflow", 3);
    let asked = request(&dynamic, REFINEMENT);
    let (mut assumptions, extra) = stronger(&dynamic, &asked);
    let second = dynamic.named("channel", "Block");
    assumptions.push(second.clone());
    assert_eq!(
        refused(retaining(
            &dynamic,
            &asked,
            Some(assumptions),
            ItemLabel::Proved
        )),
        DispositionRefusal::IdentityMismatch(IdentityMismatch::Assumptions(vec![extra, second]))
    );
}

#[test]
#[trace("TC-204", "FR-204-AC-3")]
fn a_strict_subset_of_the_requests_assumptions_is_carried() {
    let dynamic = Dynamic::with("workflow", 3);
    let asked = request(&dynamic, REFINEMENT);
    let subset = vec![asked.assumptions[0].clone()];
    assert!(matches!(
        retaining(&dynamic, &asked, Some(subset), ItemLabel::Proved),
        ProtocolOutcome::Settled(ClaimSettlement::Result { .. })
    ));
}

#[test]
#[trace("TC-204", "FR-204-AC-4")]
fn a_violation_under_an_extra_assumption_is_refused_too() {
    let dynamic = Dynamic::with("workflow", 3);
    let asked = request(&dynamic, REFINEMENT);
    let (assumptions, extra) = stronger(&dynamic, &asked);
    let outcome = retaining(&dynamic, &asked, Some(assumptions), ItemLabel::Refuted);
    assert_eq!(
        refused(outcome),
        DispositionRefusal::IdentityMismatch(IdentityMismatch::Assumptions(vec![extra]))
    );
}

#[test]
#[trace("TC-204", "FR-204-AC-5")]
fn a_refused_refinement_leaves_other_claims_unchanged_and_calls_no_fallback() {
    let dynamic = Dynamic::with("workflow", 3);
    let mut boundary = Boundary::new();
    let composing = serve(
        &mut boundary,
        "composer",
        &[Capability::Composition],
        always(proved()),
    );
    let (_, extra) = stronger(&dynamic, &request(&dynamic, REFINEMENT));
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
    let conformance = || ClaimRequest::GlobalConformance(CONFORMANT);
    let alone = settle(
        &mut boundary,
        &dynamic,
        vec![conformance(), Claims::AssumeGuarantee(composition.clone())],
    );
    let beside = settle(
        &mut boundary,
        &dynamic,
        vec![
            conformance(),
            Claims::AssumeGuarantee(request(&dynamic, REFINEMENT)),
            Claims::AssumeGuarantee(composition),
        ],
    );
    assert!(matches!(
        guaranteed(beside[1].clone()),
        ProtocolOutcome::Refused(_)
    ));
    assert_eq!(beside[0], alone[0]);
    assert_eq!(beside[2], alone[1]);
    assert_eq!(refining.len(), 1);
    assert_eq!(composing.len(), 2);
}

#[test]
#[trace("TC-204", "FR-204-AC-6")]
fn the_backends_own_strengthened_assumption_finding_is_carried_not_refused() {
    let dynamic = Dynamic::with("workflow", 3);
    let cause = UnsupportedCause::RequestedCapability(CatalogCode::new(
        "unsupported_projection",
        "protocol-assumption-not-established",
    ));
    let mut boundary = unsupported_with(cause.clone());
    let Disposition::Unsupported(unsupported) =
        disposition(one(&mut boundary, &dynamic, &request(&dynamic, REFINEMENT)))
    else {
        panic!("not carried");
    };
    assert_eq!(unsupported.cause, cause);
}

#[test]
#[trace("TC-204", "FR-204-AC-7")]
fn an_omitted_field_is_refused_and_an_empty_set_is_a_strict_subset() {
    let dynamic = Dynamic::with("workflow", 3);
    let asked = request(&dynamic, REFINEMENT);
    assert_eq!(
        refused(retaining(&dynamic, &asked, None, ItemLabel::Proved)),
        DispositionRefusal::IdentityMismatch(IdentityMismatch::AbsentAssumptions)
    );
    assert!(matches!(
        retaining(&dynamic, &asked, Some(Vec::new()), ItemLabel::Proved),
        ProtocolOutcome::Settled(ClaimSettlement::Result { .. })
    ));
}
