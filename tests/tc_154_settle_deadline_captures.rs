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
use quire_observation::authority::{observation, TemporalBoundary};
use quire_observation::{Anchor, ClockRange, QualifiedSubject, SubjectIdentity, SubjectKind};
use quire_protocol::captures::{bind_captures, CaptureFailure, CaptureSupply};
use quire_protocol::checked_package::{admit, AdmittedHandoff, HandoffFiles};
use quire_protocol::closure::{AssessmentExecution, TruthDisposition};
use quire_protocol::compensation::activate::Binding;
use quire_protocol::compensation::purity::{Capability, CapabilityInventory, PureAssessment};
use quire_protocol::deadline::{
    assess, AfterAnchor, Branch, ClockFamily, Constraint, Contradiction, Deadline, Verdict, Wait,
    WaitAssessment, WaitFact, WaitKey, WaitRefusal,
};
use quire_protocol::declarations::AwaitDeclaration;
use quire_protocol::dynamics::ObservationRef;
use quire_protocol::ids::{EffectId, RoleInstanceId, WorkflowInstanceId};
use quire_protocol::observation_link::{Link, RoleMapping, Unlinked};
use quire_protocol::slot::{evaluate, SlotOutcome};
use quire_spec_language::checked_handoff::Corpus;

use awaiting::Awaiting;
use handoff::{join_budget, written};
use limits::UNLIMITED;
use observation_owner::{
    admitted_producer, after_observation, qualified_subject, scope_views, ScopeViews,
};

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

/// The wait of the workflow instance the observations tagged `who` are of: the
/// subject of those records is `order:{who}`.
fn key(who: &str) -> WaitKey {
    WaitKey::new(
        awaiting().linked().declarations(),
        &control(),
        WorkflowInstanceId::new(format!("order:{who}").as_bytes(), who),
        Vec::new(),
    )
    .expect("a declared await keys a wait")
}

/// The tag of the owner views every wait here is judged against; their
/// progress clock is `clock:{TAG}`.
const TAG: &str = "t154";

fn declaration() -> &'static AwaitDeclaration {
    awaiting()
        .linked()
        .declarations()
        .await_at(&control())
        .expect("declared")
}

fn clock() -> String {
    clock_of(TAG)
}

fn clock_of(tag: &str) -> String {
    format!("clock:{tag}")
}

/// The declared role mapping: the binding of the records tagged `tag` resolves
/// to the node the checked await names as its anchor.
/// The FR-287 subject key of the workflow instance the records tagged `tag`
/// are of.
fn subject(tag: &str) -> QualifiedSubject {
    qualified_subject(&admitted_producer(), tag)
}

fn link(tag: &str) -> Link {
    Link::new(mapping(tag), subject(tag))
}

fn mapping(tag: &str) -> RoleMapping {
    RoleMapping::new().resolving(&format!("binding:{tag}"), [declaration().anchor.clone()])
}

/// The admitted `after` observation of a timestamped event at `instant`.
fn after_view(instant: i128) -> observation::View {
    after_view_on(TAG, instant)
}

/// As [`after_view`], on the clock `clock:{tag}`.
fn after_view_on(tag: &str, instant: i128) -> observation::View {
    after_observation(
        tag,
        (
            Anchor::TimestampNanos(instant),
            ClockRange::Timestamp {
                start_nanos: -10,
                end_nanos: 40,
            },
        ),
    )
}

fn after_fact(position: u64, instant: i128) -> WaitFact {
    after_fact_on(TAG, position, instant)
}

fn after_fact_on(tag: &str, position: u64, instant: i128) -> WaitFact {
    WaitFact::after(at(position), &after_view_on(tag, instant)).expect("an anchor")
}

/// The window the package authors, `within [0,29]`, from anchor 0: the one the
/// owner fixtures' progress views name.
fn deadline() -> Deadline {
    let anchor = AfterAnchor::of(&after_view(0)).expect("an anchor");
    Deadline::of(declaration(), &anchor, &clock(), ClockFamily::Timestamped)
        .expect("the window fits")
}

fn wait_in(family: ClockFamily, who: &str, declaration: &AwaitDeclaration, clock: &str) -> Wait {
    Wait::new(pure(), key(who), declaration, &link(who), clock, family).expect("a declared await")
}

fn wait_on(who: &str, declaration: &AwaitDeclaration, clock: &str) -> Wait {
    wait_in(ClockFamily::Timestamped, who, declaration, clock)
}
/// A wait whose `after` observation, at position 0 and instant 0, is admitted.
fn anchored_wait() -> Wait {
    let mut wait = wait_on(TAG, declaration(), &clock());
    wait.admit(after_fact(0, 0))
        .expect("the anchor opens the window");
    wait
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
    WaitFact::authority(at(position), progress, completeness)
}

fn response(position: u64, observed: i128) -> WaitFact {
    WaitFact::observation(at(position), observed, Constraint::Holds)
}

/// The facts, with the `after` observation at position 0 and instant 0.
fn settle(who: &str, facts: &[WaitFact]) -> WaitAssessment {
    let mut anchored = vec![after_fact_on(who, 0, 0)];
    anchored.extend_from_slice(facts);
    settle_unanchored(who, &anchored)
}

fn settle_unanchored(who: &str, facts: &[WaitFact]) -> WaitAssessment {
    assess(
        pure(),
        key(who),
        declaration(),
        &link(who),
        &clock_of(who),
        ClockFamily::Timestamped,
        facts,
    )
    .expect("settles")
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
    let at_deadline = settle(TAG, &[response(1, 29), closed.clone()]);
    assert_eq!(current(&at_deadline).branch().cloned(), response_at(1));
    assert_eq!(
        current(&at_deadline).execution(),
        AssessmentExecution::Completed
    );
    let after = settle(TAG, &[response(1, 30), closed]);
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
        let assessment = settle(TAG, &[authority(&views, 1, progress, completeness)]);
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
        TAG,
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
        TAG,
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
        TAG,
        &[
            false_response.clone(),
            authority(&views, 2, Progress::Closed, Completeness::Complete),
        ],
    );
    assert_eq!(current(&closed).branch(), None);
    let open = settle(
        TAG,
        &[
            false_response.clone(),
            authority(&views, 2, Progress::Open, Completeness::Complete),
        ],
    );
    assert_eq!(current(&open).branch(), None);
    assert_eq!(current(&open).truth(), Some(TruthDisposition::Pending));
    let later = settle(TAG, &[false_response, response(2, 11)]);
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
            TAG,
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
        TAG,
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

    let mut wait = anchored_wait();
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
    let batch = settle(TAG, &[response(2, 29), closed.clone()]);
    assert_eq!(batch, incremental);

    // A response observed after the deadline contradicts nothing.
    let mut wait = anchored_wait();
    wait.admit(closed.clone()).expect("admits");
    assert!(wait.admit(response(2, 30)).expect("admits").is_none());
    let incremental = wait.finish();
    assert!(incremental.supersessions().is_empty());
    assert_eq!(incremental.results().len(), 1);
    assert_eq!(settle(TAG, &[response(2, 30), closed.clone()]), incremental);

    // The history's order is not negotiable.
    let mut wait = anchored_wait();
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
    let closed = |tag| {
        authority(
            &scope_views(tag),
            2,
            Progress::Closed,
            Completeness::Complete,
        )
    };
    let answered = settle("w1", &[response(1, 29), closed("w1")]);
    let unanswered = settle("w2", &[closed("w2")]);
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
        TAG,
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
        let mut wait = anchored_wait();
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
            TAG,
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
        let mut wait = anchored_wait();
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
        let batch = settle(TAG, &[regressed, closed.clone()]);
        assert_eq!(batch, incremental);
    }
    // Support that returns selects timeout again; that is a new result, not a
    // supersession of one that was never settled.
    let again = settle(
        TAG,
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
    let facts = [
        response(1, -1),
        authority(&views, 2, Progress::Closed, Completeness::Complete),
    ];
    let before = settle(TAG, &facts);
    assert_eq!(current(&before).branch(), Some(&Branch::Timeout));
    assert!(!deadline().admits(-1));
}

/// The progress view must be the one for this wait's window and clock: a
/// different lower bound alone, a different upper bound alone, and another
/// clock are each refused, whether the progress view or the `after`
/// observation is admitted first.
#[trace("TC-154", "FR-100-AC-11")]
#[test]
fn progress_through_another_boundary_or_clock_is_refused() {
    let views = scope_views(TAG);
    let closed = || authority(&views, 1, Progress::Closed, Completeness::Complete);
    let mut lower_only = declaration().clone();
    lower_only.lower += 1;
    let mut upper_only = declaration().clone();
    upper_only.upper += 1;
    assert!(anchored_wait().admit(closed()).is_ok());
    for (label, declaration, clock) in [
        ("lower bound", lower_only, clock()),
        ("upper bound", upper_only, clock()),
        ("clock", declaration().clone(), "clock:other".to_owned()),
    ] {
        let tag = clock.strip_prefix("clock:").expect("a clock identity");
        let after = |position| after_fact_on(tag, position, 0);
        let mut anchored = wait_on(tag, &declaration, &clock);
        anchored.admit(after(0)).expect("anchors");
        assert_eq!(
            anchored.admit(closed()).err(),
            Some(WaitRefusal::ProgressNotThroughDeadline),
            "{label}"
        );
        let mut progressed = wait_on(tag, &declaration, &clock);
        progressed
            .admit(WaitFact::authority(
                at(1),
                &views.progress_closed,
                &views.completeness_complete,
            ))
            .expect("no window to compare yet");
        assert_eq!(
            progressed.admit(after(2)).err(),
            Some(WaitRefusal::ProgressNotThroughDeadline),
            "{label} against an earlier progress view"
        );
    }
}

/// The admitted `after` observation of a fixed sample at `index`, whose epoch
/// is far from the index so that reading the epoch for the index fails a test.
fn fixed_view(index: u64) -> observation::View {
    const EPOCH: i128 = 1_000_000_007;
    after_observation(
        TAG,
        (
            Anchor::FixedSample {
                index,
                epoch_nanos: EPOCH,
                period_nanos: 5,
            },
            ClockRange::FixedSample {
                start: 0,
                end_exclusive: u64::MAX,
                epoch_nanos: EPOCH,
                period_nanos: 5,
            },
        ),
    )
}

/// A window that overflows, or lies outside the integer range of the clock
/// family, is refused, for each clock family.
#[trace("TC-154", "FR-100-AC-13")]
#[test]
fn a_window_outside_the_clock_range_is_refused() {
    let window = |view: &observation::View, family| {
        let anchor = AfterAnchor::of(view).expect("an anchor");
        Deadline::of(declaration(), &anchor, "clock:t154", family)
    };
    let signed = |instant: i128| {
        let view = after_observation(
            TAG,
            (
                Anchor::TimestampNanos(instant),
                ClockRange::Timestamp {
                    start_nanos: -10,
                    end_nanos: i128::MAX,
                },
            ),
        );
        window(&view, ClockFamily::Timestamped)
    };
    // i128 overflow, with the widest range.
    assert_eq!(
        signed(i128::MAX - 1).err(),
        Some(WaitRefusal::WindowOutOfRange)
    );
    assert!(signed(i128::MAX - 29).is_ok());
    // A negative instant is fine on the signed timestamped family.
    assert!(signed(-1).is_ok());
    // Event positions are unsigned 64-bit.
    let unsigned = |position: u64| {
        let view = after_observation(
            TAG,
            (
                Anchor::EventPosition(position),
                ClockRange::EventPosition {
                    start: 0,
                    end_exclusive: u64::MAX,
                },
            ),
        );
        window(&view, ClockFamily::EventPosition)
    };
    assert!(unsigned(0).is_ok());
    assert!(unsigned(u64::MAX - 29).is_ok());
    assert_eq!(
        unsigned(u64::MAX - 28).err(),
        Some(WaitRefusal::WindowOutOfRange)
    );
    // Fixed samples are unsigned 64-bit, counted by index.
    let sampled = |index: u64| window(&fixed_view(index), ClockFamily::FixedSample);
    assert!(sampled(u64::MAX - 29).is_ok());
    assert_eq!(
        sampled(u64::MAX - 28).err(),
        Some(WaitRefusal::WindowOutOfRange)
    );
}

/// The window is counted from the anchor of the admitted `after` observation.
#[trace("TC-154", "FR-100-AC-14")]
#[test]
fn the_window_is_counted_from_the_admitted_after_observation() {
    let selected = |anchor: i128, observed: i128| {
        let mut wait = wait_on(TAG, declaration(), &clock());
        wait.admit(after_fact(1, anchor)).expect("anchors");
        wait.admit(response(2, observed)).expect("admits");
        wait.finish()
            .results()
            .last()
            .and_then(|result| result.settlement().branch().cloned())
    };
    // Anchored at 10, the window is [10, 39].
    assert_eq!(selected(10, 39), response_at(2));
    assert_eq!(selected(10, 40), None, "past the upper bound");
    assert_eq!(selected(10, 9), None, "before the lower bound");
    // Anchored at 0, 39 is past the upper bound.
    assert_eq!(selected(0, 39), None);
    assert_eq!(selected(0, 29), response_at(2));
}

/// Without the `after` observation the wait is not triggered: no branch is
/// selected and the truth is unavailable, whatever the progress and the
/// completeness.
#[trace("TC-154", "FR-100-AC-15")]
#[test]
fn no_branch_is_selected_without_the_after_observation() {
    let views = scope_views(TAG);
    for progress in [Progress::Open, Progress::Closed] {
        for completeness in [
            Completeness::Complete,
            Completeness::Incomplete,
            Completeness::Contradicted,
        ] {
            let assessment = settle_unanchored(
                TAG,
                &[
                    response(1, 10),
                    authority(&views, 2, progress, completeness),
                ],
            );
            let settlement = current(&assessment);
            let label = format!("{progress:?} {completeness:?}");
            assert_eq!(settlement.branch(), None, "{label}");
            assert_eq!(
                settlement.truth(),
                Some(TruthDisposition::Unavailable),
                "{label}"
            );
            assert_eq!(settlement.execution(), AssessmentExecution::Completed);
        }
    }
}

/// An `after` observation admitted after the response and the authority facts
/// opens the window over them, in a batch and incrementally, and supersedes
/// the earlier result that rested on its absence.
#[trace("TC-154", "FR-100-AC-16")]
#[test]
fn a_late_after_observation_supersedes_the_result_that_lacked_it() {
    let views = scope_views(TAG);
    let closed = authority(&views, 2, Progress::Closed, Completeness::Complete);
    let mut wait = wait_on(TAG, declaration(), &clock());
    wait.admit(response(1, 29)).expect("admits");
    wait.admit(closed.clone()).expect("admits");
    let earlier = wait.current().expect("unavailable").clone();
    assert_eq!(
        earlier.settlement().truth(),
        Some(TruthDisposition::Unavailable)
    );
    let supersession = wait
        .admit(after_fact(3, 0))
        .expect("anchors")
        .expect("superseded")
        .clone();
    assert_eq!(supersession.earlier(), earlier.id());
    assert_eq!(supersession.corrected_input(), &at(3));
    assert_eq!(
        supersession.contradiction(),
        Contradiction::AfterObservation
    );
    let incremental = wait.finish();
    assert_eq!(
        incremental.results()[1],
        earlier,
        "the earlier result is unaltered"
    );
    assert_eq!(
        current(&incremental).branch().cloned(),
        response_at(1),
        "the response at the upper bound is inside the late window"
    );
    let batch = settle_unanchored(TAG, &[response(1, 29), closed, after_fact(3, 0)]);
    assert_eq!(batch, incremental);
}

/// A second `after` observation for one waiting instance is refused and
/// changes nothing.
#[trace("TC-154", "FR-100-AC-17")]
#[test]
fn a_second_after_observation_is_refused() {
    let views = scope_views(TAG);
    let closed = authority(&views, 3, Progress::Closed, Completeness::Complete);
    let mut wait = anchored_wait();
    wait.admit(closed).expect("admits");
    let before = wait.clone().finish();
    assert_eq!(
        wait.admit(after_fact(4, 5)).err(),
        Some(WaitRefusal::AmbiguousAnchor {
            first: at(0),
            offered: at(4),
        })
    );
    assert_eq!(wait.finish(), before);
}

/// An `after` anchor of another clock family is refused.
#[trace("TC-154", "FR-100-AC-18")]
#[test]
fn an_after_observation_of_another_clock_family_is_refused() {
    let view = after_observation(
        TAG,
        (
            Anchor::EventPosition(3),
            ClockRange::EventPosition {
                start: 0,
                end_exclusive: 30,
            },
        ),
    );
    let mut wait = wait_on(TAG, declaration(), &clock());
    assert_eq!(
        wait.admit(WaitFact::after(at(1), &view).expect("an anchor"))
            .err(),
        Some(WaitRefusal::AnchorNotOnClockFamily)
    );
}

/// An `after` observation on another clock of the same family is refused.
#[trace("TC-154", "FR-100-AC-19")]
#[test]
fn an_after_observation_on_another_clock_is_refused() {
    let view = after_observation(
        "other",
        (
            Anchor::TimestampNanos(0),
            ClockRange::Timestamp {
                start_nanos: -10,
                end_nanos: 40,
            },
        ),
    );
    let mut wait = wait_on("other", declaration(), &clock());
    assert_eq!(
        wait.admit(WaitFact::after(at(1), &view).expect("an anchor"))
            .err(),
        Some(WaitRefusal::AnchorNotOnClock)
    );
}

/// A fixed-sample window is counted from the sample index, not the epoch.
#[trace("TC-154", "FR-100-AC-20")]
#[test]
fn a_fixed_sample_window_is_counted_from_the_sample_index() {
    let selected = |observed: i128| {
        let mut wait = wait_in(ClockFamily::FixedSample, TAG, declaration(), &clock());
        wait.admit(WaitFact::after(at(1), &fixed_view(10)).expect("an anchor"))
            .expect("anchors");
        wait.admit(response(2, observed)).expect("admits");
        wait.finish()
            .results()
            .last()
            .and_then(|result| result.settlement().branch().cloned())
    };
    assert_eq!(selected(39), response_at(2));
    assert_eq!(selected(40), None);
    assert_eq!(selected(9), None);
}

/// Another waiting instance of the same control keeps its own anchor.
#[trace("TC-154", "FR-100-AC-21")]
#[test]
fn waiting_instances_keep_their_own_anchors() {
    let mut first = wait_on("w1", declaration(), &clock_of("w1"));
    first.admit(after_fact_on("w1", 0, 0)).expect("anchors");
    first.admit(response(1, 39)).expect("admits");
    let mut other = wait_on("w2", declaration(), &clock_of("w2"));
    other.admit(after_fact_on("w2", 0, 10)).expect("anchors");
    other.admit(response(1, 39)).expect("admits");
    assert!(first.finish().results().is_empty());
    assert_eq!(
        current(&other.finish()).branch().cloned(),
        response_at(1),
        "the second instance is anchored at 10"
    );
}

/// An `after` observation whose window is not the boundary of an earlier
/// admitted progress view is refused.
#[trace("TC-154", "FR-100-AC-22")]
#[test]
fn an_after_observation_against_an_earlier_progress_view_is_refused() {
    let views = scope_views(TAG);
    let mut progressed = wait_on(TAG, declaration(), &clock());
    progressed
        .admit(authority(
            &views,
            1,
            Progress::Closed,
            Completeness::Complete,
        ))
        .expect("admits");
    assert_eq!(
        progressed.admit(after_fact(2, 10)).err(),
        Some(WaitRefusal::ProgressNotThroughDeadline)
    );
}

/// The anchor is the target of the `after` that QSL's checked await accessor
/// publishes on the `await` node.
#[trace("TC-154", "FR-100-AC-23")]
#[test]
fn the_anchor_is_read_from_the_checked_await() {
    assert!(
        declaration().anchor.path.iter().any(|segment| matches!(
            segment,
            CheckedNodePathSegment::Name(name) if name.contains("Armed")
        )),
        "the await follows the Armed event: {:?}",
        declaration().anchor
    );
}

/// An observation is the `after` observation of a wait only when it links, by
/// derivation, to the anchor node and the waiting instance. One that does not
/// is refused with its cause and leaves the wait unaltered: a linked
/// observation at the same position still anchors it.
#[trace("TC-154", "FR-100-AC-24")]
#[test]
fn an_unlinked_observation_is_unavailable_as_the_after_observation() {
    let other = "other";
    let elsewhere = || vec![node("SomewhereElse")];
    let binding = format!("binding:{other}");
    let at_anchor = || vec![declaration().anchor.clone()];
    let cases = [
        (mapping(TAG), Unlinked::BindingUnresolved),
        (
            mapping(TAG).resolving(&binding, elsewhere()),
            Unlinked::NotAnchorNode,
        ),
        (
            mapping(TAG).resolving(&binding, at_anchor()),
            Unlinked::OtherInstance,
        ),
    ];
    for (mapping, cause) in cases {
        let mut wait = Wait::new(
            pure(),
            key(TAG),
            declaration(),
            &Link::new(mapping, subject(TAG)),
            &clock(),
            ClockFamily::Timestamped,
        )
        .expect("a declared await");
        assert_eq!(
            wait.admit(after_fact_on(other, 1, 0)).err(),
            Some(WaitRefusal::ObservationUnlinked(cause))
        );
        wait.admit(after_fact(1, 0))
            .expect("the refused observation changed nothing");
    }
}

/// The FR-287 subject key is the whole of the instance: the same identity
/// bytes under another kind are another instance, and a subject that does not
/// carry the waiting instance's identity keys no wait.
#[trace("TC-154", "FR-100-AC-24")]
#[test]
fn the_subject_key_is_compared_whole() {
    let producer = admitted_producer();
    let kind = SubjectKind::new(b"ix://example/other-kind".to_vec()).expect("a kind");
    let same_bytes = QualifiedSubject::new(
        &producer,
        kind.clone(),
        SubjectIdentity::new(format!("order:{TAG}").into_bytes()).expect("an identity"),
    );
    let mut wait = Wait::new(
        pure(),
        key(TAG),
        declaration(),
        &Link::new(mapping(TAG), same_bytes),
        &clock(),
        ClockFamily::Timestamped,
    )
    .expect("the identity bytes are the instance's");
    assert_eq!(
        wait.admit(after_fact(1, 0)).err(),
        Some(WaitRefusal::ObservationUnlinked(Unlinked::OtherInstance))
    );
    let foreign = QualifiedSubject::new(
        &producer,
        kind,
        SubjectIdentity::new(b"another".to_vec()).expect("an identity"),
    );
    assert_eq!(
        Wait::new(
            pure(),
            key(TAG),
            declaration(),
            &Link::new(mapping(TAG), foreign),
            &clock(),
            ClockFamily::Timestamped,
        )
        .err(),
        Some(WaitRefusal::SubjectNotInstance)
    );
}

/// QSL's occurrence schema keys the wait: an undeclared control, a wrong
/// number of repeat ordinals and an unkeyable workflow identity are refused.
#[trace("TC-154", "FR-100-AC-12")]
#[test]
fn the_wait_is_keyed_through_qsl_s_occurrence_schema() {
    // A wait is built only for the declaration its key names.
    let mut elsewhere_declaration = declaration().clone();
    elsewhere_declaration.control = node("NotAnAwait");
    assert!(matches!(
        Wait::new(
            pure(),
            key(TAG),
            &elsewhere_declaration,
            &link(TAG),
            &clock(),
            ClockFamily::Timestamped
        ),
        Err(WaitRefusal::UndeclaredAwait { .. })
    ));
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
