// SPDX-License-Identifier: AGPL-3.0-or-later
//! FR-100: settlement of an inclusive deadline branch.
//!
//! A waiting instance (an `await` control of one workflow instance, with its
//! repeat ordinals) is judged against one admitted history. The response
//! branch is selected by a matching observation inside the authored window
//! `within [lower, upper]`, both bounds inside, counted from the anchor of the
//! admitted `after` observation of that wait: the window does not exist until
//! that observation is admitted. The timeout branch is selected only
//! when the awaited scope's progress is `closed` through the deadline and the
//! input completeness is `complete`, two separate owner facts that never stand
//! in for each other (FR-028).
//!
//! Nothing here evaluates a constraint, a clock or an expression: the
//! constraint outcome is QSL's slot outcome, progress and completeness are the
//! QObs owner views, and the window is the one QSL publishes on the checked
//! `await` node, which the progress view's FR-289 boundary and clock must name. A
//! settlement is keyed by the waiting instance, never by the static branch.
//! Results are immutable; a settled timeout that a later observation, progress
//! or completeness fact contradicts is reported as an FR-033 supersession that
//! names the earlier result and leaves it unaltered.

use core::ops::RangeInclusive;

use quire_contract_model::CheckedNodeId;
use quire_exact::Integer;
use quire_observation::authority::{completeness, observation, progress, BoundaryRef, OpenClosed};
use quire_observation::Anchor;

use crate::closure::{AssessmentExecution, TruthDisposition};
use crate::compensation::purity::PureAssessment;
use crate::declarations::{AwaitDeclaration, Declarations};
use crate::dynamics::{bind_occurrence, HistoryRefusal, ObservationRef};
use crate::ids::WorkflowInstanceId;
use crate::slot::SlotOutcome;

/// Why a wait cannot be settled. Nothing is guessed or repaired.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WaitRefusal {
    /// QSL's occurrence schema does not key this wait: the workflow identity,
    /// the control or the repeat ordinals are refused.
    Key(HistoryRefusal),
    /// The control is not an `await` the linked protocol declares.
    UndeclaredAwait {
        /// The offered control.
        control: CheckedNodeId,
    },
    /// The progress view's boundary is not the wait's window, or it is on
    /// another clock.
    ProgressNotThroughDeadline,
    /// The `after` observation's anchor is not of the clock family the wait is
    /// judged through.
    AnchorNotOnClockFamily,
    /// The `after` observation is on another clock than the wait's.
    AnchorNotOnClock,
    /// The owner's anchor text is not an integer of its clock family. A
    /// validated owner view never reports this.
    AnchorMalformed,
    /// A second observation was offered as the `after` event of one waiting
    /// instance.
    AmbiguousAnchor {
        /// The `after` observation already admitted.
        first: ObservationRef,
        /// The refused observation.
        offered: ObservationRef,
    },
    /// The window counted from the anchor does not fit the integer range of
    /// the clock family.
    WindowOutOfRange,
    /// A fact is not after the last admitted one in the history's order.
    OutOfOrder {
        /// The last admitted fact.
        last: ObservationRef,
        /// The offered fact.
        offered: ObservationRef,
    },
}

/// The identity of one waiting instance: the `await` control, the workflow
/// instance and the repeat ordinals of its enclosing loops.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WaitKey {
    branch: CheckedNodeId,
    workflow: WorkflowInstanceId,
    repeat_ordinals: Vec<Integer>,
}

impl WaitKey {
    /// The waiting instance of the `await` at `control` in `workflow`, keyed
    /// through QSL's occurrence schema (QSL FR-384): this library's identities
    /// are exact bytes, QSL's workflow identity is non-empty UTF-8 of at most
    /// 4096 bytes, and there is one repeat ordinal for each enclosing `repeat`.
    ///
    /// # Errors
    ///
    /// [`WaitRefusal::UndeclaredAwait`] for a control that is not a declared
    /// `await`; [`WaitRefusal::Key`] when QSL's schema refuses the key.
    pub fn new(
        declarations: &Declarations,
        control: &CheckedNodeId,
        workflow: WorkflowInstanceId,
        repeat_ordinals: Vec<Integer>,
    ) -> Result<Self, WaitRefusal> {
        let declaration =
            declarations
                .await_at(control)
                .ok_or_else(|| WaitRefusal::UndeclaredAwait {
                    control: control.clone(),
                })?;
        bind_occurrence(declarations, &declaration.key, &workflow, &repeat_ordinals)
            .map_err(WaitRefusal::Key)?;
        Ok(Self {
            branch: control.clone(),
            workflow,
            repeat_ordinals,
        })
    }

    /// The `await` control.
    #[must_use]
    pub fn branch(&self) -> &CheckedNodeId {
        &self.branch
    }

    /// The workflow instance.
    #[must_use]
    pub fn workflow(&self) -> &WorkflowInstanceId {
        &self.workflow
    }

    /// The repeat ordinals.
    #[must_use]
    pub fn repeat_ordinals(&self) -> &[Integer] {
        &self.repeat_ordinals
    }
}

/// The window of a wait: the authored `within [lower, upper]` counted from the
/// anchor instant, closed at both ends, on one clock. Observations and the
/// progress boundary are in the unit of that clock.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Deadline {
    lower: i128,
    upper: i128,
    authored_clock: Box<str>,
    clock_identity: Box<str>,
}

impl Deadline {
    /// The window `declaration` authors, counted from the anchor of the
    /// admitted `after` observation, on the QObs clock `clock_identity` that
    /// the caller's clock selection binds to the authored clock, in `family`.
    ///
    /// # Errors
    ///
    /// [`WaitRefusal::AnchorNotOnClockFamily`] when the anchor is of another
    /// family; [`WaitRefusal::AnchorNotOnClock`] when it is on another
    /// clock; [`WaitRefusal::WindowOutOfRange`] when a bound overflows or lies
    /// outside the integer range of the family.
    pub fn of(
        declaration: &AwaitDeclaration,
        after: &AfterAnchor,
        clock_identity: &str,
        family: ClockFamily,
    ) -> Result<Self, WaitRefusal> {
        if after.family() != family {
            return Err(WaitRefusal::AnchorNotOnClockFamily);
        }
        if &*after.clock != clock_identity {
            return Err(WaitRefusal::AnchorNotOnClock);
        }
        let admitted = family.admitted_range();
        let bound = |offset: u64| {
            after
                .instant()
                .checked_add(i128::from(offset))
                .filter(|instant| admitted.contains(instant))
                .ok_or(WaitRefusal::WindowOutOfRange)
        };
        Ok(Self {
            lower: bound(declaration.lower)?,
            upper: bound(declaration.upper)?,
            authored_clock: declaration.clock.clone(),
            clock_identity: clock_identity.into(),
        })
    }

    /// The authored clock name.
    #[must_use]
    pub fn authored_clock(&self) -> &str {
        &self.authored_clock
    }

    /// Whether an observation at `observed` is inside the wait. Both bounds
    /// are inside.
    #[must_use]
    pub const fn admits(&self, observed: i128) -> bool {
        self.lower <= observed && observed <= self.upper
    }

    fn names(&self, window: &ProgressWindow) -> bool {
        window.clock == self.clock_identity
            && self.lower.to_string() == *window.lower
            && self.upper.to_string() == *window.upper
    }
}

/// The clock family a wait is judged through, which fixes the integer range of
/// its instants.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ClockFamily {
    /// Event positions: unsigned 64-bit.
    EventPosition,
    /// Fixed samples, counted by sample index: unsigned 64-bit.
    FixedSample,
    /// Timestamped events: signed 128-bit nanoseconds.
    Timestamped,
}

impl ClockFamily {
    const fn admitted_range(self) -> RangeInclusive<i128> {
        match self {
            Self::EventPosition | Self::FixedSample => 0..=u64::MAX as i128,
            Self::Timestamped => i128::MIN..=i128::MAX,
        }
    }
}

/// The instant of an admitted `after` observation, read from the owner's
/// observation view: the position of an event position, the sample index of a
/// fixed sample, the signed nanosecond instant of a timestamped event, on the
/// clock the view names.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AfterAnchor {
    anchor: Anchor,
    clock: Box<str>,
}

impl AfterAnchor {
    /// The anchor of `view`.
    ///
    /// # Errors
    ///
    /// [`WaitRefusal::AnchorMalformed`] when the owner's anchor text is not
    /// an integer of its clock family.
    pub fn of(view: &observation::View) -> Result<Self, WaitRefusal> {
        let malformed = |_| WaitRefusal::AnchorMalformed;
        let anchor = match view.payload().anchor() {
            observation::AnchorRef::EventPosition { position } => {
                Anchor::EventPosition(position.parse().map_err(malformed)?)
            }
            observation::AnchorRef::FixedSample {
                index,
                epoch_nanos,
                period_nanos,
            } => Anchor::FixedSample {
                index: index.parse().map_err(malformed)?,
                epoch_nanos: epoch_nanos.parse().map_err(malformed)?,
                period_nanos: period_nanos.parse().map_err(malformed)?,
            },
            observation::AnchorRef::Timestamp { instant_nanos } => {
                Anchor::TimestampNanos(instant_nanos.parse().map_err(malformed)?)
            }
        };
        Ok(Self {
            anchor,
            clock: view.payload().clock().identity.into(),
        })
    }

    /// The anchor instant in the unit of its clock: a position, a sample
    /// index or nanoseconds.
    #[must_use]
    pub const fn instant(&self) -> i128 {
        match self.anchor {
            Anchor::EventPosition(position) => position as i128,
            Anchor::FixedSample { index, .. } => index as i128,
            Anchor::TimestampNanos(instant) => instant,
        }
    }

    /// The clock family of the anchor.
    #[must_use]
    pub const fn family(&self) -> ClockFamily {
        match self.anchor {
            Anchor::EventPosition(_) => ClockFamily::EventPosition,
            Anchor::FixedSample { .. } => ClockFamily::FixedSample,
            Anchor::TimestampNanos(_) => ClockFamily::Timestamped,
        }
    }
}

/// The boundary and clock a progress view is read through, as text.
#[derive(Clone, Debug, Eq, PartialEq)]
struct ProgressWindow {
    lower: Box<str>,
    upper: Box<str>,
    clock: Box<str>,
}

impl ProgressWindow {
    fn of(view: &progress::View) -> Self {
        let (lower, upper) = match view.payload().boundary() {
            BoundaryRef::EventPosition {
                lower,
                upper_inclusive,
                ..
            }
            | BoundaryRef::FixedSample {
                lower,
                upper_inclusive,
                ..
            } => (lower, upper_inclusive),
            BoundaryRef::TimestampedEvent {
                lower_nanos,
                upper_inclusive_nanos,
                ..
            } => (lower_nanos, upper_inclusive_nanos),
        };
        Self {
            lower: lower.into(),
            upper: upper.into(),
            clock: view.payload().clock_identity().into(),
        }
    }
}

/// How a response's bound constraint stands, read from QSL's slot outcome.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Constraint {
    /// The awaited message binds no constraint.
    Unconstrained,
    /// The constraint evaluated to true.
    Holds,
    /// The constraint evaluated to false.
    DoesNotHold,
    /// The constraint has no value (unavailable, undefined, refused,
    /// incomplete, unsupported or not Boolean); it is never read as a Boolean.
    Unestablished,
}

impl Constraint {
    /// The constraint outcome `outcome` reports.
    #[must_use]
    pub fn of(outcome: &SlotOutcome) -> Self {
        match outcome {
            SlotOutcome::Holds => Self::Holds,
            SlotOutcome::DoesNotHold => Self::DoesNotHold,
            SlotOutcome::NotBoolean(_)
            | SlotOutcome::Unavailable { .. }
            | SlotOutcome::Undefined(_)
            | SlotOutcome::Refused(_)
            | SlotOutcome::Incomplete(_)
            | SlotOutcome::Family(_) => Self::Unestablished,
        }
    }

    const fn matches(self) -> bool {
        matches!(self, Self::Unconstrained | Self::Holds)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum WaitEvent {
    Observation {
        observed: i128,
        constraint: Constraint,
    },
    Authority {
        window: ProgressWindow,
        progress: OpenClosed,
        completeness: completeness::State,
    },
    After {
        anchor: AfterAnchor,
    },
}

/// One fact of the wait's admitted history, at its position.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WaitFact {
    at: ObservationRef,
    event: WaitEvent,
}

impl WaitFact {
    /// A response observation at `observed` (in the deadline's unit).
    #[must_use]
    pub const fn observation(at: ObservationRef, observed: i128, constraint: Constraint) -> Self {
        Self {
            at,
            event: WaitEvent::Observation {
                observed,
                constraint,
            },
        }
    }

    /// The admitted observation of the awaited `after` event of this wait,
    /// whose anchor opens the window. The caller attributes the observation
    /// to the wait's `after` event; the owner's view names neither.
    ///
    /// # Errors
    ///
    /// As [`AfterAnchor::of`].
    pub fn after(at: ObservationRef, view: &observation::View) -> Result<Self, WaitRefusal> {
        Ok(Self {
            at,
            event: WaitEvent::After {
                anchor: AfterAnchor::of(view)?,
            },
        })
    }

    /// The awaited scope's progress and the input completeness, read from the
    /// owner's views. Whether the progress view is the wait's window is
    /// checked when the fact is admitted to the wait.
    #[must_use]
    pub fn authority(
        at: ObservationRef,
        progress: &progress::View,
        completeness: &completeness::View,
    ) -> Self {
        Self {
            at,
            event: WaitEvent::Authority {
                window: ProgressWindow::of(progress),
                progress: progress.payload().state(),
                completeness: completeness.payload().state(),
            },
        }
    }

    /// The fact's place in the admitted history.
    #[must_use]
    pub const fn at(&self) -> &ObservationRef {
        &self.at
    }
}

/// A selected branch.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Branch {
    /// The response branch, by the matching observation.
    Response {
        /// The first matching observation in the admitted history.
        observation: ObservationRef,
    },
    /// The timeout branch.
    Timeout,
}

/// What a settlement decides. A branch and a truth disposition cannot both be
/// present: a selected branch carries its execution disposition only, and
/// which truth, if any, a selection carries is an open question for the
/// owner of FR-242 (see FR-100).
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Verdict {
    /// A branch is selected.
    Selected(Branch),
    /// No branch is selected, with the truth FR-100 assigns, if any.
    NotSelected(Option<TruthDisposition>),
}

/// One settlement. The progress and completeness axes are carried unchanged
/// and are absent until observed.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Settlement {
    verdict: Verdict,
    progress: Option<OpenClosed>,
    completeness: Option<completeness::State>,
}

impl Settlement {
    /// What the settlement decides.
    #[must_use]
    pub const fn verdict(&self) -> &Verdict {
        &self.verdict
    }

    /// The selected branch, if any.
    #[must_use]
    pub const fn branch(&self) -> Option<&Branch> {
        match &self.verdict {
            Verdict::Selected(branch) => Some(branch),
            Verdict::NotSelected(_) => None,
        }
    }

    /// The truth disposition, when one is assigned.
    #[must_use]
    pub const fn truth(&self) -> Option<TruthDisposition> {
        match &self.verdict {
            Verdict::Selected(_) => None,
            Verdict::NotSelected(truth) => *truth,
        }
    }

    /// The assessment execution disposition: every settlement here completed.
    #[must_use]
    pub const fn execution(&self) -> AssessmentExecution {
        AssessmentExecution::Completed
    }

    /// Progress through the deadline, as the owner reported it.
    #[must_use]
    pub const fn progress(&self) -> Option<OpenClosed> {
        self.progress
    }

    /// Input completeness, as the owner reported it.
    #[must_use]
    pub const fn completeness(&self) -> Option<completeness::State> {
        self.completeness
    }
}

/// The identity of one result: the waiting instance and the fact that
/// produced it.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResultId {
    wait: WaitKey,
    produced_at: ObservationRef,
}

impl ResultId {
    /// The waiting instance.
    #[must_use]
    pub const fn wait(&self) -> &WaitKey {
        &self.wait
    }

    /// The fact that produced the result.
    #[must_use]
    pub const fn produced_at(&self) -> &ObservationRef {
        &self.produced_at
    }
}

/// One immutable result of a wait.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SettledResult {
    id: ResultId,
    settlement: Settlement,
}

impl SettledResult {
    /// The result's identity.
    #[must_use]
    pub const fn id(&self) -> &ResultId {
        &self.id
    }

    /// The settlement.
    #[must_use]
    pub const fn settlement(&self) -> &Settlement {
        &self.settlement
    }
}

/// What contradicted an earlier result.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Contradiction {
    /// An observation inside the window, admitted late.
    Observation,
    /// The `after` observation, admitted after a result that rested on its
    /// absence.
    AfterObservation,
    /// A progress and completeness fact that no longer supports timeout.
    Authority,
}

/// The FR-033 result for an earlier result that a later fact contradicts.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Supersession {
    earlier: ResultId,
    contradicted_premise: ObservationRef,
    corrected_input: ObservationRef,
    contradiction: Contradiction,
    superseding: SettledResult,
}

impl Supersession {
    /// The earlier result, unaltered.
    #[must_use]
    pub const fn earlier(&self) -> &ResultId {
        &self.earlier
    }

    /// The contradicted premise: the authority fact that closed the wait.
    #[must_use]
    pub const fn contradicted_premise(&self) -> &ObservationRef {
        &self.contradicted_premise
    }

    /// The corrected input: the fact that contradicted it.
    #[must_use]
    pub const fn corrected_input(&self) -> &ObservationRef {
        &self.corrected_input
    }

    /// What kind of fact contradicted it.
    #[must_use]
    pub const fn contradiction(&self) -> Contradiction {
        self.contradiction
    }

    /// The new result.
    #[must_use]
    pub const fn superseding(&self) -> &SettledResult {
        &self.superseding
    }
}

/// Every result and supersession of one wait, in history order.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WaitAssessment {
    key: WaitKey,
    results: Vec<SettledResult>,
    supersessions: Vec<Supersession>,
}

impl WaitAssessment {
    /// The waiting instance.
    #[must_use]
    pub const fn key(&self) -> &WaitKey {
        &self.key
    }

    /// Each result, earliest first; the last is current.
    #[must_use]
    pub fn results(&self) -> &[SettledResult] {
        &self.results
    }

    /// Each supersession, earliest first.
    #[must_use]
    pub fn supersessions(&self) -> &[Supersession] {
        &self.supersessions
    }
}

#[derive(Clone, Debug)]
struct Anchoring {
    declaration: AwaitDeclaration,
    clock_identity: Box<str>,
    family: ClockFamily,
}

#[derive(Clone, Debug)]
struct Seen {
    observed: i128,
    constraint: Constraint,
    at: ObservationRef,
}

/// One waiting instance, settled fact by fact.
#[derive(Clone, Debug)]
pub struct Wait {
    key: WaitKey,
    anchoring: Anchoring,
    deadline: Option<Deadline>,
    anchored_by: Option<ObservationRef>,
    unanchored: Vec<ProgressWindow>,
    last: Option<ObservationRef>,
    seen: Vec<Seen>,
    progress: Option<OpenClosed>,
    completeness: Option<completeness::State>,
    results: Vec<SettledResult>,
    supersessions: Vec<Supersession>,
}

impl Wait {
    /// A wait on the `await` that `declaration` authors, keyed by `key`, assessed under the
    /// pure-assessment witness: no business operation is reachable from any
    /// settlement path. The window opens when the `after` observation is
    /// admitted, on the QObs clock `clock_identity` in `family`.
    ///
    /// # Errors
    ///
    /// [`WaitRefusal::UndeclaredAwait`] when `key` is not the wait of
    /// `declaration`.
    pub fn new(
        _pure: PureAssessment,
        key: WaitKey,
        declaration: &AwaitDeclaration,
        clock_identity: &str,
        family: ClockFamily,
    ) -> Result<Self, WaitRefusal> {
        if declaration.control != key.branch {
            return Err(WaitRefusal::UndeclaredAwait {
                control: key.branch,
            });
        }
        Ok(Self {
            key,
            anchoring: Anchoring {
                declaration: declaration.clone(),
                clock_identity: clock_identity.into(),
                family,
            },
            deadline: None,
            anchored_by: None,
            unanchored: Vec::new(),
            last: None,
            seen: Vec::new(),
            progress: None,
            completeness: None,
            results: Vec::new(),
            supersessions: Vec::new(),
        })
    }

    /// The current result, absent until a fact decides one.
    #[must_use]
    pub fn current(&self) -> Option<&SettledResult> {
        self.results.last()
    }

    /// Admit the next fact of the history. A fact that changes a settled
    /// timeout returns the FR-033 supersession it produced. A refused fact
    /// changes nothing.
    ///
    /// # Errors
    ///
    /// [`WaitRefusal::OutOfOrder`] when the fact is not after the last one;
    /// [`WaitRefusal::AmbiguousAnchor`] for a second `after` observation;
    /// [`WaitRefusal::AnchorNotOnClockFamily`] and
    /// [`WaitRefusal::WindowOutOfRange`] for an `after` observation the clock
    /// family cannot window; [`WaitRefusal::ProgressNotThroughDeadline`] for a
    /// progress view that is not the window.
    pub fn admit(&mut self, fact: WaitFact) -> Result<Option<&Supersession>, WaitRefusal> {
        if let Some(last) = &self.last {
            if fact.at <= *last {
                return Err(WaitRefusal::OutOfOrder {
                    last: last.clone(),
                    offered: fact.at,
                });
            }
        }
        let contradiction = match fact.event {
            WaitEvent::After { anchor } => {
                if let Some(first) = &self.anchored_by {
                    return Err(WaitRefusal::AmbiguousAnchor {
                        first: first.clone(),
                        offered: fact.at,
                    });
                }
                let deadline = Deadline::of(
                    &self.anchoring.declaration,
                    &anchor,
                    &self.anchoring.clock_identity,
                    self.anchoring.family,
                )?;
                if !self.unanchored.iter().all(|window| deadline.names(window)) {
                    return Err(WaitRefusal::ProgressNotThroughDeadline);
                }
                self.deadline = Some(deadline);
                self.anchored_by = Some(fact.at.clone());
                self.unanchored.clear();
                Contradiction::AfterObservation
            }
            WaitEvent::Observation {
                observed,
                constraint,
            } => {
                self.seen.push(Seen {
                    observed,
                    constraint,
                    at: fact.at.clone(),
                });
                Contradiction::Observation
            }
            WaitEvent::Authority {
                window,
                progress,
                completeness,
            } => {
                match &self.deadline {
                    Some(deadline) if !deadline.names(&window) => {
                        return Err(WaitRefusal::ProgressNotThroughDeadline);
                    }
                    Some(_) => {}
                    None => self.unanchored.push(window),
                }
                self.progress = Some(progress);
                self.completeness = Some(completeness);
                Contradiction::Authority
            }
        };
        self.last = Some(fact.at.clone());
        let Some(settlement) = self.judge() else {
            return Ok(None);
        };
        let before = self.results.last().cloned();
        // The `after` observation contradicts the premise of every earlier
        // result, so a new result follows it even when the settlement is equal.
        if contradiction != Contradiction::AfterObservation
            && before.as_ref().map(|result| &result.settlement) == Some(&settlement)
        {
            return Ok(None);
        }
        let result = SettledResult {
            id: ResultId {
                wait: self.key.clone(),
                produced_at: fact.at.clone(),
            },
            settlement,
        };
        self.results.push(result.clone());
        let Some(earlier) = before.filter(|earlier| {
            contradiction == Contradiction::AfterObservation
                || earlier.settlement.branch() == Some(&Branch::Timeout)
        }) else {
            return Ok(None);
        };
        self.supersessions.push(Supersession {
            contradicted_premise: earlier.id.produced_at.clone(),
            earlier: earlier.id,
            corrected_input: fact.at,
            contradiction,
            superseding: result,
        });
        Ok(self.supersessions.last())
    }

    fn judge(&self) -> Option<Settlement> {
        let settlement = |verdict| Settlement {
            verdict,
            progress: self.progress,
            completeness: self.completeness,
        };
        let unselected = |truth| settlement(Verdict::NotSelected(truth));
        // Without the `after` observation the wait is not triggered, and
        // `pending` requires activation `triggered` (QSpec FR-242, FR-061).
        let Some(deadline) = &self.deadline else {
            return Some(unselected(Some(TruthDisposition::Unavailable)));
        };
        let inside = || {
            self.seen
                .iter()
                .filter(|seen| deadline.admits(seen.observed))
        };
        if let Some(first) = inside().find(|seen| seen.constraint.matches()) {
            return Some(settlement(Verdict::Selected(Branch::Response {
                observation: first.at.clone(),
            })));
        }
        let (progress, completeness) = (self.progress?, self.completeness?);
        Some(match (completeness, progress) {
            (completeness::State::Incomplete | completeness::State::Contradicted, _) => {
                unselected(Some(TruthDisposition::Unavailable))
            }
            (completeness::State::Complete, OpenClosed::Open) => {
                unselected(Some(TruthDisposition::Pending))
            }
            (completeness::State::Complete, OpenClosed::Closed) => {
                if inside().any(|seen| seen.constraint == Constraint::Unestablished) {
                    unselected(Some(TruthDisposition::Unavailable))
                } else if inside().any(|seen| seen.constraint == Constraint::DoesNotHold) {
                    unselected(None)
                } else {
                    settlement(Verdict::Selected(Branch::Timeout))
                }
            }
        })
    }

    /// The wait's results and supersessions so far.
    #[must_use]
    pub fn finish(self) -> WaitAssessment {
        WaitAssessment {
            key: self.key,
            results: self.results,
            supersessions: self.supersessions,
        }
    }
}

/// Settle a whole admitted history at once. The facts are ordered by their
/// place in the history before they are read, so the order they are presented
/// in changes no value (FR-026), and the outcome equals admitting them one by
/// one in that order (FR-041).
///
/// # Errors
///
/// As [`Wait::new`] and [`Wait::admit`]; a repeated position is [`WaitRefusal::OutOfOrder`].
pub fn assess(
    pure: PureAssessment,
    key: WaitKey,
    declaration: &AwaitDeclaration,
    clock_identity: &str,
    family: ClockFamily,
    facts: &[WaitFact],
) -> Result<WaitAssessment, WaitRefusal> {
    let mut ordered = facts.to_vec();
    ordered.sort_by(|left, right| left.at.cmp(&right.at));
    let mut wait = Wait::new(pure, key, declaration, clock_identity, family)?;
    for fact in ordered {
        wait.admit(fact)?;
    }
    Ok(wait.finish())
}
