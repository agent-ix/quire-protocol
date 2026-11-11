// SPDX-License-Identifier: AGPL-3.0-or-later
//! TC-154: inclusive deadline branches settle only on complete progress
//! (FR-100) and obligation captures bind as ordered, atomic occurrences
//! (FR-101).

#[path = "support/handoff.rs"]
#[allow(dead_code)]
mod handoff;
#[path = "support/limits.rs"]
#[allow(dead_code)]
mod limits;
#[path = "support/observation_owner.rs"]
#[allow(dead_code)]
mod observation_owner;

use ix_trace_rs::trace;
use qsl_eval::value::{ProtocolSlot, ProtocolSuppliedValue, ProtocolSupply, SuppliedValue};
use qsl_package::{BarePackage, BareSupply, ProtocolSlotDescriptor};
use qsl_semantics::check::wire_id::WireGraph;
use quire_contract_model::{CheckedNodeId, CheckedNodePathSegment, CheckedPackageReadLimits};
use quire_exact::{FieldValue, Integer, Meter, NodeKey, Value, ValueType};
use quire_observation::authority::TemporalBoundary;
use quire_protocol::captures::{bind_captures, CaptureFailure, CaptureSupply};
use quire_protocol::checked_package::{admit, AdmittedHandoff, HandoffFiles};
use quire_protocol::closure::{AssessmentExecution, TruthDisposition};
use quire_protocol::compensation::activate::Binding;
use quire_protocol::compensation::purity::{Capability, CapabilityInventory, PureAssessment};
use quire_protocol::deadline::{
    assess, Branch, Constraint, Deadline, Wait, WaitAssessment, WaitFact, WaitKey, WaitRefusal,
};
use quire_protocol::dynamics::ObservationRef;
use quire_protocol::ids::{EffectId, RoleInstanceId, WorkflowInstanceId};
use quire_protocol::slot::{evaluate, SlotOutcome};
use quire_spec_language::checked_handoff::Corpus;

use handoff::{join_budget, written};
use limits::UNLIMITED;
use observation_owner::{scope_views, ScopeViews};

struct Surface(Vec<Capability>);

impl CapabilityInventory for Surface {
    fn capabilities(&self) -> &[Capability] {
        &self.0
    }
}

fn pure() -> PureAssessment {
    PureAssessment::checked(&Surface(Vec::new())).expect("no capability is admitted")
}

fn node(name: &str) -> CheckedNodeId {
    CheckedNodeId {
        package: "ix://p".into(),
        path: vec![CheckedNodePathSegment::Name(name.into())],
        ordinal: 0,
    }
}

fn at(position: u64) -> ObservationRef {
    ObservationRef {
        position,
        identity: format!("o{position}").into_bytes().into(),
    }
}

fn key(who: &str) -> WaitKey {
    WaitKey::new(
        node("Wait"),
        WorkflowInstanceId::new(who.as_bytes(), who),
        Vec::new(),
    )
    .expect("a text workflow")
}

/// The deadline the owner fixtures' progress views name: upper bound 29.
fn deadline() -> Deadline {
    Deadline::new(TemporalBoundary::TimestampedEvent {
        lower_nanos: 0,
        upper_inclusive_nanos: 29,
        carrier_end_exclusive_nanos: 30,
        watermark_nanos: 30,
    })
}

#[derive(Clone, Copy, Debug)]
enum Progress {
    Open,
    Closed,
}

#[derive(Clone, Copy, Debug)]
enum Completeness {
    Complete,
    Incomplete,
    Contradicted,
}

fn authority(
    views: &ScopeViews,
    position: u64,
    progress: Progress,
    completeness: Completeness,
) -> WaitFact {
    let progress = match progress {
        Progress::Open => &views.progress_open,
        Progress::Closed => &views.progress_closed,
    };
    let completeness = match completeness {
        Completeness::Complete => &views.completeness_complete,
        Completeness::Incomplete => &views.completeness_incomplete,
        Completeness::Contradicted => &views.completeness_contradicted,
    };
    WaitFact::authority(at(position), deadline(), progress, completeness).expect("the deadline")
}

fn response(position: u64, observed: i128) -> WaitFact {
    WaitFact::observation(at(position), observed, Constraint::Holds)
}

fn settle(who: &str, facts: &[WaitFact]) -> WaitAssessment {
    assess(pure(), key(who), deadline(), facts).expect("settles")
}

fn current(assessment: &WaitAssessment) -> &quire_protocol::deadline::Settlement {
    assessment.results.last().expect("a result").settlement()
}

// ---------------------------------------------------------------------------
// FR-100
// ---------------------------------------------------------------------------

/// A response at the deadline instant selects the response branch; the same
/// response strictly after it, with closed and complete progress, selects
/// timeout.
#[trace("TC-154", "FR-100-AC-1")]
#[test]
fn a_response_at_the_deadline_is_inside_the_wait() {
    let views = scope_views("ac1");
    let closed = authority(&views, 2, Progress::Closed, Completeness::Complete);
    let at_deadline = settle("w", &[response(1, 29), closed.clone()]);
    assert_eq!(
        current(&at_deadline).selected,
        Some(Branch::Response {
            observation: at(1)
        })
    );
    assert_eq!(current(&at_deadline).execution, AssessmentExecution::Completed);
    let after = settle("w", &[response(1, 30), closed]);
    assert_eq!(current(&after).selected, Some(Branch::Timeout));
    assert!(after.supersessions.is_empty());
}

/// No response: timeout needs progress `closed` and completeness `complete`;
/// open progress is pending; incomplete or contradicted completeness is
/// unavailable, also with open progress; none of those selects timeout.
#[trace("TC-154", "FR-100-AC-2")]
#[test]
fn timeout_needs_closed_progress_and_complete_input() {
    let views = scope_views("ac2");
    let expected = [
        (Progress::Closed, Completeness::Complete, None),
        (
            Progress::Open,
            Completeness::Complete,
            Some(TruthDisposition::Pending),
        ),
        (
            Progress::Closed,
            Completeness::Incomplete,
            Some(TruthDisposition::Unavailable),
        ),
        (
            Progress::Closed,
            Completeness::Contradicted,
            Some(TruthDisposition::Unavailable),
        ),
        (
            Progress::Open,
            Completeness::Incomplete,
            Some(TruthDisposition::Unavailable),
        ),
        (
            Progress::Open,
            Completeness::Contradicted,
            Some(TruthDisposition::Unavailable),
        ),
    ];
    for (progress, completeness, truth) in expected {
        let assessment = settle("w", &[authority(&views, 1, progress, completeness)]);
        let settlement = current(&assessment);
        let label = format!("{progress:?} {completeness:?}");
        assert_eq!(settlement.truth, truth, "{label}");
        assert_eq!(settlement.execution, AssessmentExecution::Completed, "{label}");
        let timeout = matches!(
            (progress, completeness),
            (Progress::Closed, Completeness::Complete)
        );
        assert_eq!(settlement.selected, timeout.then_some(Branch::Timeout), "{label}");
        // The completeness axis carries the owner's value and progress is unchanged.
        let owner_progress = match progress {
            Progress::Open => &views.progress_open,
            Progress::Closed => &views.progress_closed,
        };
        assert_eq!(settlement.progress, Some(owner_progress.payload().state()), "{label}");
    }
    let incomplete = settle(
        "w",
        &[authority(&views, 1, Progress::Closed, Completeness::Incomplete)],
    );
    assert_eq!(
        current(&incomplete).completeness,
        Some(views.completeness_incomplete.payload().state())
    );
    let contradicted = settle(
        "w",
        &[authority(&views, 1, Progress::Closed, Completeness::Contradicted)],
    );
    assert_eq!(
        current(&contradicted).completeness,
        Some(views.completeness_contradicted.payload().state())
    );
    assert_ne!(
        current(&incomplete).completeness,
        current(&contradicted).completeness
    );
}

/// A false bound constraint selects no response and does not by itself
/// select timeout; an unestablished one is read as no Boolean either.
#[trace("TC-154", "FR-100-AC-3")]
#[test]
fn a_false_constraint_selects_neither_branch() {
    let views = scope_views("ac3");
    assert_eq!(Constraint::of(&SlotOutcome::Holds), Constraint::Holds);
    assert_eq!(Constraint::of(&SlotOutcome::DoesNotHold), Constraint::DoesNotHold);
    assert_eq!(
        Constraint::of(&SlotOutcome::NotBoolean(Value::Integer(Integer::from(1)))),
        Constraint::Unestablished
    );
    for constraint in [Constraint::DoesNotHold, Constraint::Unestablished] {
        let facts = [
            WaitFact::observation(at(1), 10, constraint),
            authority(&views, 2, Progress::Closed, Completeness::Complete),
        ];
        let assessment = settle("w", &facts);
        let settlement = current(&assessment);
        assert_eq!(settlement.selected, None, "{constraint:?}");
        assert_eq!(settlement.truth, Some(TruthDisposition::Unavailable));
        let open = settle(
            "w",
            &[
                WaitFact::observation(at(1), 10, constraint),
                authority(&views, 2, Progress::Open, Completeness::Complete),
            ],
        );
        assert_eq!(current(&open).selected, None);
        assert_eq!(current(&open).truth, Some(TruthDisposition::Pending));
    }
    // The constraint is read per observation: a later true one still selects.
    let later = settle(
        "w",
        &[
            WaitFact::observation(at(1), 10, Constraint::DoesNotHold),
            response(2, 11),
        ],
    );
    assert_eq!(
        current(&later).selected,
        Some(Branch::Response {
            observation: at(2)
        })
    );
}

/// A matching response admitted after timeout but observed at or before the
/// deadline supersedes it under FR-033 and leaves the earlier result as it
/// was; one observed after the deadline does not; batch and incremental runs
/// agree on both.
#[trace("TC-154", "FR-100-AC-4")]
#[test]
fn a_late_admitted_response_supersedes_without_altering_the_timeout() {
    let views = scope_views("ac4");
    let closed = authority(&views, 1, Progress::Closed, Completeness::Complete);

    let mut wait = Wait::new(pure(), key("w"), deadline());
    assert!(wait.admit(closed.clone()).expect("admits").is_none());
    let timeout = wait.current().expect("timeout").clone();
    assert_eq!(timeout.settlement().selected, Some(Branch::Timeout));
    let supersession = wait
        .admit(response(2, 29))
        .expect("admits")
        .expect("superseded")
        .clone();
    assert_eq!(supersession.earlier, *timeout.id());
    assert_eq!(supersession.contradicted_premise, at(1));
    assert_eq!(supersession.corrected_input, at(2));
    assert_eq!(
        supersession.superseding.settlement().selected,
        Some(Branch::Response {
            observation: at(2)
        })
    );
    let incremental = wait.finish();
    assert_eq!(incremental.results[0], timeout, "the earlier result is unaltered");
    assert_eq!(incremental.results.len(), 2);

    // Batch, with the facts presented out of order, agrees.
    let batch = settle("w", &[response(2, 29), closed.clone()]);
    assert_eq!(batch, incremental);

    // A response observed after the deadline contradicts nothing.
    let mut wait = Wait::new(pure(), key("w"), deadline());
    wait.admit(closed.clone()).expect("admits");
    assert!(wait.admit(response(2, 30)).expect("admits").is_none());
    let incremental = wait.finish();
    assert!(incremental.supersessions.is_empty());
    assert_eq!(incremental.results.len(), 1);
    assert_eq!(settle("w", &[response(2, 30), closed.clone()]), incremental);

    // The history's order is not negotiable, and the settled premise holds.
    let mut wait = Wait::new(pure(), key("w"), deadline());
    wait.admit(closed).expect("admits");
    assert!(matches!(
        wait.admit(response(1, 5)),
        Err(WaitRefusal::OutOfOrder { .. })
    ));
    assert!(matches!(
        wait.admit(authority(&views, 3, Progress::Open, Completeness::Complete)),
        Err(WaitRefusal::PremiseRegressed { .. })
    ));
}

/// Two waiting instances of one static branch settle independently; the key
/// is the instance, refused where QSL cannot key it.
#[trace("TC-154", "FR-100-AC-5")]
#[test]
fn waiting_instances_of_one_branch_settle_independently() {
    let views = scope_views("ac5");
    let closed = authority(&views, 2, Progress::Closed, Completeness::Complete);
    let answered = settle("w1", &[response(1, 29), closed.clone()]);
    let unanswered = settle("w2", &[closed]);
    assert_eq!(answered.key.branch(), unanswered.key.branch());
    assert_ne!(answered.key, unanswered.key);
    assert!(matches!(
        current(&answered).selected,
        Some(Branch::Response { .. })
    ));
    assert_eq!(current(&unanswered).selected, Some(Branch::Timeout));
    assert_ne!(
        answered.results[0].id().wait,
        unanswered.results[0].id().wait
    );

    // QSL keys a workflow by non-empty UTF-8 of at most 4096 bytes.
    for bytes in [Vec::new(), vec![0xff, 0xfe], vec![b'a'; 4097]] {
        let refused = WaitKey::new(
            node("Wait"),
            WorkflowInstanceId::new(&bytes, "w"),
            Vec::new(),
        );
        assert!(matches!(refused, Err(WaitRefusal::Key(_))), "{}", bytes.len());
    }
}

/// No capability is admitted to any settlement path: a surface that admits a
/// business operation yields no witness to settle under, and the settlement
/// that runs under the witness invokes none.
#[trace("TC-154", "FR-100-AC-6")]
#[test]
fn settlement_invokes_no_business_operation() {
    for capability in [Capability::BusinessOperation, Capability::CompensationOperation] {
        let refused = PureAssessment::checked(&Surface(vec![capability]));
        assert_eq!(refused.expect_err("refused").capability(), capability);
    }
    let views = scope_views("ac6");
    let assessment = settle(
        "w",
        &[authority(&views, 1, Progress::Closed, Completeness::Complete)],
    );
    assert_eq!(current(&assessment).selected, Some(Branch::Timeout));
}

/// The progress view must be the one through this deadline.
#[trace("TC-154", "FR-100-AC-2")]
#[test]
fn progress_through_another_boundary_is_refused() {
    let views = scope_views("other");
    let other = Deadline::new(TemporalBoundary::TimestampedEvent {
        lower_nanos: 0,
        upper_inclusive_nanos: 28,
        carrier_end_exclusive_nanos: 29,
        watermark_nanos: 29,
    });
    let refused = WaitFact::authority(
        at(1),
        other,
        &views.progress_closed,
        &views.completeness_complete,
    );
    assert_eq!(refused.err(), Some(WaitRefusal::ProgressNotThroughDeadline));
}

// ---------------------------------------------------------------------------
// FR-101
// ---------------------------------------------------------------------------

fn boundary(upper: i128) -> TemporalBoundary {
    TemporalBoundary::TimestampedEvent {
        lower_nanos: 0,
        upper_inclusive_nanos: upper,
        carrier_end_exclusive_nanos: upper + 1,
        watermark_nanos: upper + 1,
    }
}

fn activation(trigger: &str) -> Binding {
    let id = |name: &str| name.as_bytes().to_vec().into_boxed_slice();
    Binding {
        workflow: WorkflowInstanceId::new(b"w", "w"),
        role: RoleInstanceId::new(b"r", "r"),
        effect: EffectId::new(b"e", "e"),
        predicate: id("p"),
        capture: id("c"),
        anchor: id("a"),
        trigger_observation: id(trigger),
        scope_identity: "scope".into(),
        population_identity: "population".into(),
    }
}

fn integer(value: i64) -> SlotOutcome {
    SlotOutcome::NotBoolean(Value::Integer(Integer::from(value)))
}

fn supply(capture: &CheckedNodeId, at: TemporalBoundary, evaluation: SlotOutcome) -> CaptureSupply {
    CaptureSupply {
        capture: capture.clone(),
        boundary: at,
        evaluation,
    }
}

fn is_integer(value: &Value, expected: i64) -> bool {
    matches!(value, Value::Integer(found) if *found == Integer::from(expected))
}

fn is_boolean(value: &Value, expected: bool) -> bool {
    matches!(value, Value::Boolean(found) if *found == expected)
}

/// Values bind in authored order, and a later change of a source leaves every
/// bound value as it was.
#[trace("TC-154", "FR-101-AC-1")]
#[test]
fn captures_bind_in_authored_order_and_stay_bound() {
    let authored = [node("c1"), node("c2"), node("c3")];
    let at_activation = boundary(10);
    let supplies = |values: [i64; 3]| {
        authored
            .iter()
            .zip(values)
            .map(|(capture, value)| supply(capture, at_activation, integer(value)))
            .collect::<Vec<_>>()
    };
    let first = bind_captures(
        pure(),
        &activation("t1"),
        at_activation,
        &authored,
        supplies([3, 1, 2]),
    )
    .expect("binds");
    let bound = first
        .occurrences()
        .iter()
        .map(|occurrence| occurrence.capture().clone())
        .collect::<Vec<_>>();
    assert_eq!(bound, authored);
    for (occurrence, expected) in first.occurrences().iter().zip([3, 1, 2]) {
        assert!(is_integer(occurrence.value(), expected));
    }
    // The source changes and the capture is evaluated again: a new activation
    // result; the first is the same as it was.
    let changed = bind_captures(
        pure(),
        &activation("t1"),
        at_activation,
        &authored,
        supplies([30, 10, 20]),
    )
    .expect("binds");
    assert!(is_integer(changed.occurrences()[0].value(), 30));
    for (occurrence, expected) in first.occurrences().iter().zip([3, 1, 2]) {
        assert!(is_integer(occurrence.value(), expected));
    }
}

/// Two identical captures are two occurrences with distinct identities.
#[trace("TC-154", "FR-101-AC-2")]
#[test]
fn identical_captures_are_distinct_occurrences() {
    let at_activation = boundary(10);
    for authored in [
        [node("c1"), node("c2")],
        [node("c1"), node("c1")],
    ] {
        let supplies = authored
            .iter()
            .map(|capture| supply(capture, at_activation, integer(7)))
            .collect();
        let bound = bind_captures(
            pure(),
            &activation("t"),
            at_activation,
            &authored,
            supplies,
        )
        .expect("binds");
        let [one, two] = bound.occurrences() else {
            panic!("two occurrences");
        };
        assert!(is_integer(one.value(), 7) && is_integer(two.value(), 7));
        assert_ne!(one.id(), two.id());
    }
}

fn guard_outcome(ready: Option<bool>) -> SlotOutcome {
    let admitted = admitted();
    let bare = BarePackage::new(admitted.package())
        .with_supply(
            BareSupply {
                identities: admitted.identities(),
                equality: Some(admitted.equality()),
            },
            &mut join_budget(),
        )
        .expect("the supply joins");
    let package = admitted.package();
    let graph = package.resolved_graph();
    let compensation = graph
        .nodes
        .iter()
        .find(|candidate| {
            &*candidate.semantic_form == "compensation"
                && candidate.declaration.as_ref().is_some_and(|declaration| {
                    declaration.qualified_name.last().map(|part| &**part) == Some("Full")
                })
        })
        .expect("compensation node");
    let key: NodeKey = WireGraph::new(graph.nodes.iter().map(|n| &n.node_id))
        .expect("wire ids")
        .key(&compensation.node_id)
        .expect("key");
    let selected = bare
        .select_protocol_slot(&ProtocolSlotDescriptor::new(key.clone(), "trigger.guard", None))
        .expect("slot selects");
    let source = selected
        .sources()
        .iter()
        .find(|source| source.name() == "triggerFull")
        .expect("the trigger binder");
    let supplied = match ready {
        Some(ready) => {
            let ValueType::Composite(record) = source.value_type() else {
                panic!("the trigger is a record");
            };
            SuppliedValue::Value(
                bare.world()
                    .expect("world")
                    .types()
                    .record(
                        record.clone(),
                        vec![("ready", FieldValue::Present(Value::Boolean(ready)))],
                    )
                    .expect("Notice"),
            )
        }
        None => SuppliedValue::Unavailable(quire_semantic_value::catalog::CatalogCode::new(
            "unavailable_observation",
            "missing-required-artifact",
        )),
    };
    let slot = ProtocolSlot {
        node: key,
        member: "trigger.guard".to_owned(),
        index: None,
    };
    let supply = ProtocolSupply {
        binders: vec![(
            source.binder().clone(),
            ProtocolSuppliedValue {
                value: supplied,
                references: Vec::new(),
            },
        )],
        observation: None,
    };
    evaluate(&bare, &slot, &supply, &mut Meter::new(UNLIMITED))
        .expect("the guard evaluates")
        .outcome
}

fn admitted() -> AdmittedHandoff {
    let written = written("event_position", Corpus::None);
    let files = HandoffFiles {
        checked_package: &written.files["checked-package.json"],
        selection: &written.files["selection.json"],
        identities: &written.files["identities.json"],
        equality: &written.files["equality.json"],
    };
    admit(
        &files,
        &written.evidence,
        CheckedPackageReadLimits::bounded(),
        &mut join_budget(),
    )
    .expect("R admits")
}

/// One capture QSL left unavailable among three refuses the activation, binds
/// none and names that capture.
#[trace("TC-154", "FR-101-AC-3")]
#[test]
fn one_undefined_capture_refuses_the_whole_activation() {
    let authored = [node("c1"), node("c2"), node("c3")];
    let at_activation = boundary(10);
    let unavailable = guard_outcome(None);
    assert!(matches!(unavailable, SlotOutcome::Unavailable { .. }));
    let supplies = vec![
        supply(&authored[0], at_activation, integer(1)),
        supply(&authored[1], at_activation, unavailable),
        supply(&authored[2], at_activation, integer(3)),
    ];
    let refused = bind_captures(
        pure(),
        &activation("t"),
        at_activation,
        &authored,
        supplies,
    )
    .expect_err("refused");
    let [fault] = &refused.faults[..] else {
        panic!("exactly the undefined capture: {:?}", refused.faults);
    };
    assert_eq!((fault.ordinal, &fault.capture), (1, &authored[1]));
    assert!(matches!(
        &fault.failure,
        CaptureFailure::Undefined {
            outcome: SlotOutcome::Unavailable { .. }
        }
    ));
}

/// A value supplied for another boundary is refused, as is any supply that
/// is not the authored one in the authored place.
#[trace("TC-154", "FR-101-AC-4")]
#[test]
fn only_the_activation_boundary_binds() {
    let authored = [node("c1"), node("c2")];
    let at_activation = boundary(10);
    let refused = bind_captures(
        pure(),
        &activation("t"),
        at_activation,
        &authored,
        vec![
            supply(&authored[0], at_activation, integer(1)),
            supply(&authored[1], boundary(11), integer(2)),
        ],
    )
    .expect_err("refused");
    let [fault] = &refused.faults[..] else {
        panic!("one fault: {:?}", refused.faults);
    };
    assert_eq!(fault.ordinal, 1);
    assert!(matches!(
        fault.failure,
        CaptureFailure::WrongBoundary { supplied } if supplied == boundary(11)
    ));

    let swapped = bind_captures(
        pure(),
        &activation("t"),
        at_activation,
        &authored,
        vec![
            supply(&authored[1], at_activation, integer(2)),
            supply(&authored[0], at_activation, integer(1)),
        ],
    )
    .expect_err("refused");
    assert!(swapped
        .faults
        .iter()
        .all(|fault| matches!(fault.failure, CaptureFailure::OutOfOrder { .. })));
    assert_eq!(swapped.faults.len(), 2);

    let short = bind_captures(
        pure(),
        &activation("t"),
        at_activation,
        &authored,
        vec![supply(&authored[0], at_activation, integer(1))],
    )
    .expect_err("refused");
    assert!(matches!(short.faults[0].failure, CaptureFailure::Missing));

    let surplus = bind_captures(
        pure(),
        &activation("t"),
        at_activation,
        &authored[..1],
        vec![
            supply(&authored[0], at_activation, integer(1)),
            supply(&authored[1], at_activation, integer(2)),
        ],
    )
    .expect_err("refused");
    assert!(matches!(surplus.faults[0].failure, CaptureFailure::Unauthored));
}

/// QSL supplies a value that differs from what the authored expression gives
/// when it is evaluated locally; the bound value is the supplied one.
#[trace("TC-154", "FR-101-AC-5")]
#[test]
fn the_supplied_value_is_bound_not_a_local_result() {
    let authored = [node("c1")];
    let at_activation = boundary(10);
    let bind = |outcome| {
        bind_captures(
            pure(),
            &activation("t"),
            at_activation,
            &authored,
            vec![supply(&authored[0], at_activation, outcome)],
        )
        .expect("binds")
    };
    // What the authored guard gives for a ready trigger, computed by QSL here.
    let local = bind(guard_outcome(Some(true)));
    assert!(is_boolean(local.occurrences()[0].value(), true));
    // What QSL supplied at the activation boundary, for another trigger.
    let supplied = bind(guard_outcome(Some(false)));
    assert!(is_boolean(supplied.occurrences()[0].value(), false));
}
