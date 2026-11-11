// SPDX-License-Identifier: AGPL-3.0-or-later
//! TC-154: inclusive deadline branches settle only on complete progress
//! (FR-100) and obligation captures bind as ordered, atomic occurrences
//! (FR-101).

#[path = "support/awaiting.rs"]
#[allow(dead_code)]
mod awaiting;
#[path = "support/handoff.rs"]
#[allow(dead_code)]
mod handoff;
#[path = "support/limits.rs"]
#[allow(dead_code)]
mod limits;
#[path = "support/observation_owner.rs"]
#[allow(dead_code)]
mod observation_owner;

use std::sync::OnceLock;

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
    assess, Branch, Constraint, Contradiction, Deadline, Verdict, Wait, WaitAssessment, WaitFact,
    WaitKey, WaitRefusal,
};
use quire_protocol::dynamics::ObservationRef;
use quire_protocol::ids::{EffectId, RoleInstanceId, WorkflowInstanceId};
use quire_protocol::slot::{evaluate, SlotOutcome};
use quire_spec_language::checked_handoff::Corpus;

use awaiting::Awaiting;
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

fn awaiting() -> &'static Awaiting {
    static PACKAGE: OnceLock<Awaiting> = OnceLock::new();
    PACKAGE.get_or_init(Awaiting::new)
}

fn control() -> CheckedNodeId {
    let declarations = awaiting().linked().declarations();
    let [declaration] = declarations.awaits() else {
        panic!("one await: {:?}", declarations.awaits());
    };
    declaration.control.clone()
}

fn key(who: &str) -> WaitKey {
    WaitKey::new(
        awaiting().linked().declarations(),
        &control(),
        WorkflowInstanceId::new(who.as_bytes(), who),
        Vec::new(),
    )
    .expect("a declared await keys a wait")
}

/// The tag of the owner views every wait here is judged against; their
/// progress clock is `clock:{TAG}`.
const TAG: &str = "t154";

/// The window the package authors, `within [0,29]`, from anchor 0: the one the
/// owner fixtures' progress views name.
fn deadline() -> Deadline {
    let declarations = awaiting().linked().declarations();
    Deadline::of(
        declarations.await_at(&control()).expect("declared"),
        0,
        &format!("clock:{TAG}"),
    )
    .expect("the window fits")
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
    WaitFact::authority(at(position), &deadline(), progress, completeness).expect("the window")
}

fn response(position: u64, observed: i128) -> WaitFact {
    WaitFact::observation(at(position), observed, Constraint::Holds)
}

fn settle(who: &str, facts: &[WaitFact]) -> WaitAssessment {
    assess(pure(), key(who), deadline(), facts).expect("settles")
}

fn current(assessment: &WaitAssessment) -> &quire_protocol::deadline::Settlement {
    assessment.results().last().expect("a result").settlement()
}

fn response_at(position: u64) -> Option<Branch> {
    Some(Branch::Response {
        observation: at(position),
    })
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
    let views = scope_views(TAG);
    let closed = authority(&views, 2, Progress::Closed, Completeness::Complete);
    let at_deadline = settle("w", &[response(1, 29), closed.clone()]);
    assert_eq!(current(&at_deadline).branch().cloned(), response_at(1));
    assert_eq!(
        current(&at_deadline).execution(),
        AssessmentExecution::Completed
    );
    let after = settle("w", &[response(1, 30), closed]);
    assert_eq!(current(&after).branch(), Some(&Branch::Timeout));
    assert!(after.supersessions().is_empty());
}

/// No response: timeout needs progress `closed` and completeness `complete`;
/// open progress is pending; incomplete or contradicted completeness is
/// unavailable, also with open progress; none of those selects timeout.
#[trace("TC-154", "FR-100-AC-2")]
#[test]
fn timeout_needs_closed_progress_and_complete_input() {
    let views = scope_views(TAG);
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
        assert_eq!(settlement.truth(), truth, "{label}");
        assert_eq!(
            settlement.execution(),
            AssessmentExecution::Completed,
            "{label}"
        );
        let timeout = matches!(
            (progress, completeness),
            (Progress::Closed, Completeness::Complete)
        );
        assert_eq!(
            settlement.branch().cloned(),
            timeout.then_some(Branch::Timeout),
            "{label}"
        );
        // Progress is the owner's value, unchanged.
        let owner_progress = match progress {
            Progress::Open => &views.progress_open,
            Progress::Closed => &views.progress_closed,
        };
        assert_eq!(
            settlement.progress(),
            Some(owner_progress.payload().state()),
            "{label}"
        );
    }
    let incomplete = settle(
        "w",
        &[authority(
            &views,
            1,
            Progress::Closed,
            Completeness::Incomplete,
        )],
    );
    assert_eq!(
        current(&incomplete).completeness(),
        Some(views.completeness_incomplete.payload().state())
    );
    let contradicted = settle(
        "w",
        &[authority(
            &views,
            1,
            Progress::Closed,
            Completeness::Contradicted,
        )],
    );
    assert_eq!(
        current(&contradicted).completeness(),
        Some(views.completeness_contradicted.payload().state())
    );
    assert_ne!(
        current(&incomplete).completeness(),
        current(&contradicted).completeness()
    );
}

/// A false bound constraint selects no response and does not by itself
/// select timeout; the constraint is read per observation.
#[trace("TC-154", "FR-100-AC-3")]
#[test]
fn a_false_constraint_selects_neither_branch() {
    let views = scope_views(TAG);
    assert_eq!(Constraint::of(&SlotOutcome::Holds), Constraint::Holds);
    assert_eq!(
        Constraint::of(&SlotOutcome::DoesNotHold),
        Constraint::DoesNotHold
    );
    let false_response = WaitFact::observation(at(1), 10, Constraint::DoesNotHold);
    let closed = settle(
        "w",
        &[
            false_response.clone(),
            authority(&views, 2, Progress::Closed, Completeness::Complete),
        ],
    );
    assert_eq!(current(&closed).branch(), None);
    let open = settle(
        "w",
        &[
            false_response.clone(),
            authority(&views, 2, Progress::Open, Completeness::Complete),
        ],
    );
    assert_eq!(current(&open).branch(), None);
    assert_eq!(current(&open).truth(), Some(TruthDisposition::Pending));
    let later = settle("w", &[false_response, response(2, 11)]);
    assert_eq!(current(&later).branch().cloned(), response_at(2));
}

/// With closed progress and complete input a false constraint selects no
/// branch and carries no truth; a valueless one (an unavailable slot outcome
/// or a value that is not a Boolean) carries truth `unavailable`; both report
/// execution `completed`.
#[trace("TC-154", "FR-100-AC-7")]
#[test]
fn a_false_or_valueless_constraint_with_complete_progress_selects_no_branch() {
    let views = scope_views(TAG);
    let closed = authority(&views, 2, Progress::Closed, Completeness::Complete);
    let settled = |constraint| {
        settle(
            "w",
            &[WaitFact::observation(at(1), 10, constraint), closed.clone()],
        )
    };
    let false_constraint = settled(Constraint::DoesNotHold);
    assert_eq!(
        current(&false_constraint).verdict(),
        &Verdict::NotSelected(None)
    );
    assert_eq!(
        current(&false_constraint).execution(),
        AssessmentExecution::Completed
    );
    let unavailable = guard_outcome(None);
    assert!(matches!(unavailable, SlotOutcome::Unavailable { .. }));
    let not_boolean = SlotOutcome::NotBoolean(Value::Integer(Integer::from(1i64)));
    for outcome in [unavailable, not_boolean] {
        let constraint = Constraint::of(&outcome);
        assert_eq!(constraint, Constraint::Unestablished);
        let valueless = settled(constraint);
        assert_eq!(current(&valueless).branch(), None);
        assert_eq!(
            current(&valueless).truth(),
            Some(TruthDisposition::Unavailable)
        );
        assert_eq!(
            current(&valueless).execution(),
            AssessmentExecution::Completed
        );
    }
    // A false and a valueless observation together: the missing value wins.
    let both = settle(
        "w",
        &[
            WaitFact::observation(at(1), 10, Constraint::DoesNotHold),
            WaitFact::observation(at(2), 11, Constraint::Unestablished),
            authority(&views, 3, Progress::Closed, Completeness::Complete),
        ],
    );
    assert_eq!(current(&both).truth(), Some(TruthDisposition::Unavailable));
}

/// A matching response admitted after timeout but observed inside the window
/// supersedes it under FR-033 and leaves the earlier result as it was; one
/// observed after the deadline does not; batch and incremental runs agree on
/// both.
#[trace("TC-154", "FR-100-AC-4")]
#[test]
fn a_late_admitted_response_supersedes_without_altering_the_timeout() {
    let views = scope_views(TAG);
    let closed = authority(&views, 1, Progress::Closed, Completeness::Complete);

    let mut wait = Wait::new(pure(), key("w"), deadline());
    assert!(wait.admit(closed.clone()).expect("admits").is_none());
    let timeout = wait.current().expect("timeout").clone();
    assert_eq!(timeout.settlement().branch(), Some(&Branch::Timeout));
    let supersession = wait
        .admit(response(2, 29))
        .expect("admits")
        .expect("superseded")
        .clone();
    assert_eq!(supersession.earlier(), timeout.id());
    assert_eq!(supersession.contradicted_premise(), &at(1));
    assert_eq!(supersession.corrected_input(), &at(2));
    assert_eq!(supersession.contradiction(), Contradiction::Observation);
    assert_eq!(
        supersession.superseding().settlement().branch().cloned(),
        response_at(2)
    );
    let incremental = wait.finish();
    assert_eq!(
        incremental.results()[0],
        timeout,
        "the earlier result is unaltered"
    );
    assert_eq!(incremental.results().len(), 2);

    // Batch, with the facts presented out of order, agrees.
    let batch = settle("w", &[response(2, 29), closed.clone()]);
    assert_eq!(batch, incremental);

    // A response observed after the deadline contradicts nothing.
    let mut wait = Wait::new(pure(), key("w"), deadline());
    wait.admit(closed.clone()).expect("admits");
    assert!(wait.admit(response(2, 30)).expect("admits").is_none());
    let incremental = wait.finish();
    assert!(incremental.supersessions().is_empty());
    assert_eq!(incremental.results().len(), 1);
    assert_eq!(settle("w", &[response(2, 30), closed.clone()]), incremental);

    // The history's order is not negotiable.
    let mut wait = Wait::new(pure(), key("w"), deadline());
    wait.admit(closed).expect("admits");
    assert!(matches!(
        wait.admit(response(1, 5)),
        Err(WaitRefusal::OutOfOrder { .. })
    ));
}

/// Two waiting instances of one static branch settle independently, each keyed
/// by its own workflow instance.
#[trace("TC-154", "FR-100-AC-5")]
#[test]
fn waiting_instances_of_one_branch_settle_independently() {
    let views = scope_views(TAG);
    let closed = authority(&views, 2, Progress::Closed, Completeness::Complete);
    let answered = settle("w1", &[response(1, 29), closed.clone()]);
    let unanswered = settle("w2", &[closed]);
    assert_eq!(answered.key().branch(), unanswered.key().branch());
    assert_ne!(answered.key(), unanswered.key());
    assert!(matches!(
        current(&answered).branch(),
        Some(Branch::Response { .. })
    ));
    assert_eq!(current(&unanswered).branch(), Some(&Branch::Timeout));
    assert_ne!(
        answered.results()[0].id().wait(),
        unanswered.results()[0].id().wait()
    );
}

/// No capability is admitted to any settlement path: a surface that admits a
/// business operation yields no witness to settle under, and the settlement
/// that runs under the witness invokes none.
#[trace("TC-154", "FR-100-AC-6")]
#[test]
fn settlement_invokes_no_business_operation() {
    for capability in [
        Capability::BusinessOperation,
        Capability::CompensationOperation,
    ] {
        let refused = PureAssessment::checked(&Surface(vec![capability]));
        assert_eq!(refused.expect_err("refused").capability(), capability);
    }
    let views = scope_views(TAG);
    let assessment = settle(
        "w",
        &[authority(
            &views,
            1,
            Progress::Closed,
            Completeness::Complete,
        )],
    );
    assert_eq!(current(&assessment).branch(), Some(&Branch::Timeout));
}

/// A false or valueless observation inside the window, admitted after timeout
/// was selected, replaces the timeout under FR-033 and leaves it unaltered.
#[trace("TC-154", "FR-100-AC-8")]
#[test]
fn a_late_false_or_valueless_observation_supersedes_the_timeout() {
    let views = scope_views(TAG);
    let closed = authority(&views, 1, Progress::Closed, Completeness::Complete);
    for (constraint, truth) in [
        (Constraint::DoesNotHold, None),
        (
            Constraint::Unestablished,
            Some(TruthDisposition::Unavailable),
        ),
    ] {
        let mut wait = Wait::new(pure(), key("w"), deadline());
        wait.admit(closed.clone()).expect("admits");
        let timeout = wait.current().expect("timeout").clone();
        let supersession = wait
            .admit(WaitFact::observation(at(2), 29, constraint))
            .expect("admits")
            .expect("superseded")
            .clone();
        assert_eq!(supersession.earlier(), timeout.id());
        assert_eq!(supersession.contradicted_premise(), &at(1));
        assert_eq!(supersession.corrected_input(), &at(2));
        assert_eq!(
            supersession.superseding().settlement().verdict(),
            &Verdict::NotSelected(truth)
        );
        let incremental = wait.finish();
        assert_eq!(incremental.results()[0], timeout);
        let batch = settle(
            "w",
            &[WaitFact::observation(at(2), 29, constraint), closed.clone()],
        );
        assert_eq!(batch, incremental);
    }
}

/// A progress or completeness fact admitted after timeout that no longer
/// supports it supersedes the timeout under FR-033 and does not abort the
/// batch; support that returns selects timeout again.
#[trace("TC-154", "FR-100-AC-9")]
#[test]
fn a_regressed_premise_supersedes_the_timeout_without_aborting_the_batch() {
    let views = scope_views(TAG);
    let closed = authority(&views, 1, Progress::Closed, Completeness::Complete);
    for (progress, completeness, truth) in [
        (
            Progress::Open,
            Completeness::Complete,
            TruthDisposition::Pending,
        ),
        (
            Progress::Closed,
            Completeness::Incomplete,
            TruthDisposition::Unavailable,
        ),
        (
            Progress::Closed,
            Completeness::Contradicted,
            TruthDisposition::Unavailable,
        ),
    ] {
        let regressed = authority(&views, 2, progress, completeness);
        let mut wait = Wait::new(pure(), key("w"), deadline());
        wait.admit(closed.clone()).expect("admits");
        let timeout = wait.current().expect("timeout").clone();
        let supersession = wait
            .admit(regressed.clone())
            .expect("admits")
            .expect("superseded")
            .clone();
        assert_eq!(supersession.earlier(), timeout.id());
        assert_eq!(supersession.contradicted_premise(), &at(1));
        assert_eq!(supersession.corrected_input(), &at(2));
        assert_eq!(supersession.contradiction(), Contradiction::Authority);
        assert_eq!(supersession.superseding().settlement().truth(), Some(truth));
        let incremental = wait.finish();
        assert_eq!(incremental.results()[0], timeout);
        // The batch does not abort, and agrees with the incremental run.
        let batch = settle("w", &[regressed, closed.clone()]);
        assert_eq!(batch, incremental);
    }
    // Support that returns selects timeout again; that is a new result, not a
    // supersession of one that was never settled.
    let again = settle(
        "w",
        &[
            closed,
            authority(&views, 2, Progress::Open, Completeness::Complete),
            authority(&views, 3, Progress::Closed, Completeness::Complete),
        ],
    );
    assert_eq!(current(&again).branch(), Some(&Branch::Timeout));
    assert_eq!(again.results().len(), 3);
    assert_eq!(again.supersessions().len(), 1);
}

/// An observation before the window's lower bound is not inside the wait.
#[trace("TC-154", "FR-100-AC-10")]
#[test]
fn a_response_before_the_window_does_not_select_the_response_branch() {
    let views = scope_views(TAG);
    let declarations = awaiting().linked().declarations();
    let declaration = declarations.await_at(&control()).expect("declared");
    // The same window counted from an anchor of 10 starts at 10 and ends at 39.
    let shifted = Deadline::of(declaration, 10, "clock:t154").expect("fits");
    assert!(!shifted.admits(9) && shifted.admits(10) && shifted.admits(39) && !shifted.admits(40));
    let facts = [
        response(1, -1),
        authority(&views, 2, Progress::Closed, Completeness::Complete),
    ];
    let before = settle("w", &facts);
    assert_eq!(current(&before).branch(), Some(&Branch::Timeout));
    assert!(!deadline().admits(-1));
}

/// The progress view must be the one for this wait's window and clock: a
/// different lower bound alone, a different upper bound alone, and another
/// clock are each refused.
#[trace("TC-154", "FR-100-AC-11")]
#[test]
fn progress_through_another_boundary_or_clock_is_refused() {
    let views = scope_views(TAG);
    let declarations = awaiting().linked().declarations();
    let declaration = declarations.await_at(&control()).expect("declared");
    let clock = format!("clock:{TAG}");
    let fact = |deadline: &Deadline| {
        WaitFact::authority(
            at(1),
            deadline,
            &views.progress_closed,
            &views.completeness_complete,
        )
    };
    assert!(fact(&Deadline::of(declaration, 0, &clock).expect("fits")).is_ok());
    let mut lower_only = declaration.clone();
    lower_only.lower += 1;
    let mut upper_only = declaration.clone();
    upper_only.upper += 1;
    for (label, deadline) in [
        ("lower bound", Deadline::of(&lower_only, 0, &clock)),
        ("upper bound", Deadline::of(&upper_only, 0, &clock)),
        ("clock", Deadline::of(declaration, 0, "clock:other")),
    ] {
        assert_eq!(
            fact(&deadline.expect("fits")).err(),
            Some(WaitRefusal::ProgressNotThroughDeadline),
            "{label}"
        );
    }
}

/// A window that does not fit the clock's integer range is refused.
#[trace("TC-154", "FR-100-AC-13")]
#[test]
fn a_window_that_overflows_is_refused() {
    let declarations = awaiting().linked().declarations();
    let declaration = declarations.await_at(&control()).expect("declared");
    assert_eq!(
        Deadline::of(declaration, i128::MAX, "clock:t154").err(),
        Some(WaitRefusal::WindowOutOfRange)
    );
    assert!(Deadline::of(declaration, i128::MAX - 29, "clock:t154").is_ok());
}

/// QSL's occurrence schema keys the wait: an undeclared control, a wrong
/// number of repeat ordinals and an unkeyable workflow identity are refused.
#[trace("TC-154", "FR-100-AC-12")]
#[test]
fn the_wait_is_keyed_through_qsl_s_occurrence_schema() {
    let declarations = awaiting().linked().declarations();
    let workflow = |bytes: &[u8]| WorkflowInstanceId::new(bytes, "w");
    let elsewhere = node("NotAnAwait");
    assert!(matches!(
        WaitKey::new(declarations, &elsewhere, workflow(b"w"), Vec::new()),
        Err(WaitRefusal::UndeclaredAwait { .. })
    ));
    assert!(matches!(
        WaitKey::new(
            declarations,
            awaiting().clause(),
            workflow(b"w"),
            Vec::new()
        ),
        Err(WaitRefusal::UndeclaredAwait { .. })
    ));
    assert!(matches!(
        WaitKey::new(
            declarations,
            &control(),
            workflow(b"w"),
            vec![Integer::from(0i64)]
        ),
        Err(WaitRefusal::Key(_))
    ));
    // QSL keys a workflow by non-empty UTF-8 of at most 4096 bytes.
    for bytes in [Vec::new(), vec![0xff, 0xfe], vec![b'a'; 4097]] {
        let refused = WaitKey::new(declarations, &control(), workflow(&bytes), Vec::new());
        assert!(
            matches!(refused, Err(WaitRefusal::Key(_))),
            "{}",
            bytes.len()
        );
    }
    assert!(WaitKey::new(
        declarations,
        &control(),
        workflow(&[b'a'; 4096]),
        Vec::new()
    )
    .is_ok());
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
    // Supplies answer the capture they name, whatever order they arrive in;
    // the occurrences stay in authored order. A bound value cannot change
    // after activation by type: the fields are private and there is no
    // mutable accessor (the compile_fail doctest on `ActivationCaptures`).
    let mut shuffled = supplies([3, 1, 2]);
    shuffled.reverse();
    let reordered = bind_captures(
        pure(),
        &activation("t1"),
        at_activation,
        &authored,
        shuffled,
    )
    .expect("binds");
    for (occurrence, expected) in reordered.occurrences().iter().zip([3, 1, 2]) {
        assert!(is_integer(occurrence.value(), expected));
    }
    assert_eq!(
        reordered
            .occurrences()
            .iter()
            .map(|occurrence| occurrence.id().ordinal)
            .collect::<Vec<_>>(),
        [0, 1, 2]
    );
}

/// Two identical captures are two occurrences with distinct identities.
#[trace("TC-154", "FR-101-AC-2")]
#[test]
fn identical_captures_are_distinct_occurrences() {
    let at_activation = boundary(10);
    for authored in [[node("c1"), node("c2")], [node("c1"), node("c1")]] {
        let supplies = authored
            .iter()
            .map(|capture| supply(capture, at_activation, integer(7)))
            .collect();
        let bound = bind_captures(pure(), &activation("t"), at_activation, &authored, supplies)
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
        .select_protocol_slot(&ProtocolSlotDescriptor::new(
            key.clone(),
            "trigger.guard",
            None,
        ))
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
    let refused = bind_captures(pure(), &activation("t"), at_activation, &authored, supplies)
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

    let short = bind_captures(
        pure(),
        &activation("t"),
        at_activation,
        &authored,
        vec![supply(&authored[0], at_activation, integer(1))],
    )
    .expect_err("refused");
    assert!(matches!(short.faults[0].failure, CaptureFailure::Missing));
    assert_eq!(short.faults[0].capture, authored[1]);

    // One supply missing in the middle names that capture alone.
    let three = [node("c1"), node("c2"), node("c3")];
    let gap = bind_captures(
        pure(),
        &activation("t"),
        at_activation,
        &three,
        vec![
            supply(&three[0], at_activation, integer(1)),
            supply(&three[2], at_activation, integer(3)),
        ],
    )
    .expect_err("refused");
    let [fault] = &gap.faults[..] else {
        panic!("only the missing capture: {:?}", gap.faults);
    };
    assert_eq!((fault.ordinal, &fault.capture), (1, &three[1]));
    assert!(matches!(fault.failure, CaptureFailure::Missing));

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
    assert!(matches!(
        surplus.faults[0].failure,
        CaptureFailure::Unauthored
    ));
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
    // What the authored guard gives for a trigger that is not ready (the guard
    // is true then), computed by QSL here.
    let local = bind(guard_outcome(Some(false)));
    assert!(is_boolean(local.occurrences()[0].value(), true));
    // What QSL supplied at the activation boundary, for another trigger.
    let supplied = bind(guard_outcome(Some(true)));
    assert!(is_boolean(supplied.occurrences()[0].value(), false));
}
