// SPDX-License-Identifier: AGPL-3.0-or-later
//! FR-100: settlement of an inclusive deadline branch.
//!
//! A waiting instance (an `await` control of one workflow instance, with its
//! repeat ordinals) is judged against one admitted history. The response
//! branch is selected by a matching observation at or before the deadline,
//! counting the deadline instant itself. The timeout branch is selected only
//! when the awaited scope's progress is `closed` through the deadline and the
//! input completeness is `complete`, two separate owner facts that never stand
//! in for each other (FR-028).
//!
//! Nothing here evaluates a constraint, a clock or an expression: the
//! constraint outcome is QSL's slot outcome, progress and completeness are the
//! QObs owner views, and the deadline is the owner's FR-289 boundary. A
//! settlement is keyed by the waiting instance, never by the static branch.
//! Results are immutable; a late matching observation is reported as an
//! FR-033 supersession that names the earlier result and leaves it unaltered.

use qsl_package::WorkflowInstanceIdentity;
use quire_contract_model::CheckedNodeId;
use quire_exact::Integer;
use quire_observation::authority::{
    completeness, progress, BoundaryRef, OpenClosed, TemporalBoundary,
};

use crate::closure::{AssessmentExecution, TruthDisposition};
use crate::compensation::purity::PureAssessment;
use crate::dynamics::{HistoryRefusal, ObservationRef};
use crate::ids::WorkflowInstanceId;
use crate::slot::SlotOutcome;

/// Why a wait cannot be settled. Nothing is guessed or repaired.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WaitRefusal {
    /// The waiting instance's workflow identity is not a key QSL accepts
    /// (non-UTF-8, empty or longer than 4096 bytes).
    Key(HistoryRefusal),
    /// The progress view does not name the wait's deadline as its boundary.
    ProgressNotThroughDeadline,
    /// A fact is not after the last admitted one in the history's order.
    OutOfOrder {
        /// The last admitted fact.
        last: ObservationRef,
        /// The offered fact.
        offered: ObservationRef,
    },
    /// Progress or completeness regressed after timeout was selected; that is
    /// an owner-side contradiction of a settled premise, not an input here.
    PremiseRegressed {
        /// The settled result whose premise regressed.
        selected: Box<ResultId>,
        /// The offending fact.
        at: ObservationRef,
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
    /// The waiting instance of `branch` in `workflow`.
    ///
    /// # Errors
    ///
    /// [`WaitRefusal::Key`] when the workflow identity is not text QSL keys an
    /// occurrence by: this library's identities are exact bytes, QSL's are
    /// non-empty UTF-8 of at most 4096 bytes.
    pub fn new(
        branch: CheckedNodeId,
        workflow: WorkflowInstanceId,
        repeat_ordinals: Vec<Integer>,
    ) -> Result<Self, WaitRefusal> {
        let text = std::str::from_utf8(workflow.identity_bytes()).map_err(|_| {
            WaitRefusal::Key(HistoryRefusal::WorkflowNotText {
                workflow: workflow.identity_bytes().into(),
            })
        })?;
        WorkflowInstanceIdentity::new(text)
            .map_err(|error| WaitRefusal::Key(HistoryRefusal::Occurrence(error)))?;
        Ok(Self {
            branch,
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

/// The inclusive deadline: the owner's FR-289 boundary of the wait.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Deadline {
    boundary: TemporalBoundary,
}

impl Deadline {
    /// The deadline of `boundary`.
    #[must_use]
    pub const fn new(boundary: TemporalBoundary) -> Self {
        Self { boundary }
    }

    fn upper(self) -> i128 {
        match self.boundary {
            TemporalBoundary::EventPosition {
                upper_inclusive, ..
            }
            | TemporalBoundary::FixedSample {
                upper_inclusive, ..
            } => i128::from(upper_inclusive),
            TemporalBoundary::TimestampedEvent {
                upper_inclusive_nanos,
                ..
            } => upper_inclusive_nanos,
        }
    }

    /// Whether an observation at `observed`, in the unit of the deadline's
    /// clock family, is inside the wait. The deadline instant is inside.
    #[must_use]
    pub fn admits(self, observed: i128) -> bool {
        observed <= self.upper()
    }

    fn named_by(self, reference: BoundaryRef<'_>) -> bool {
        match (self.boundary, reference) {
            (
                TemporalBoundary::EventPosition {
                    upper_inclusive, ..
                },
                BoundaryRef::EventPosition {
                    upper_inclusive: text,
                    ..
                },
            )
            | (
                TemporalBoundary::FixedSample {
                    upper_inclusive, ..
                },
                BoundaryRef::FixedSample {
                    upper_inclusive: text,
                    ..
                },
            ) => upper_inclusive.to_string() == text,
            (
                TemporalBoundary::TimestampedEvent {
                    upper_inclusive_nanos,
                    ..
                },
                BoundaryRef::TimestampedEvent {
                    upper_inclusive_nanos: text,
                    ..
                },
            ) => upper_inclusive_nanos.to_string() == text,
            _ => false,
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
        progress: OpenClosed,
        completeness: completeness::State,
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

    /// The awaited scope's progress and the input completeness, read from the
    /// owner's views.
    ///
    /// # Errors
    ///
    /// [`WaitRefusal::ProgressNotThroughDeadline`] when the progress view's
    /// boundary is not the deadline's.
    pub fn authority(
        at: ObservationRef,
        deadline: Deadline,
        progress: &progress::View,
        completeness: &completeness::View,
    ) -> Result<Self, WaitRefusal> {
        if !deadline.named_by(progress.payload().boundary()) {
            return Err(WaitRefusal::ProgressNotThroughDeadline);
        }
        Ok(Self {
            at,
            event: WaitEvent::Authority {
                progress: progress.payload().state(),
                completeness: completeness.payload().state(),
            },
        })
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

/// What one settlement says. A selected branch carries execution `completed`
/// and no truth (an absent truth compares only to absence, FR-242); with no
/// selection the truth is `pending` or `unavailable`. The progress and
/// completeness axes are carried unchanged and absent until observed.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Settlement {
    /// The selected branch, if any.
    pub selected: Option<Branch>,
    /// The truth disposition, carried when no branch is selected.
    pub truth: Option<TruthDisposition>,
    /// The assessment execution disposition.
    pub execution: AssessmentExecution,
    /// Progress through the deadline, as the owner reported it.
    pub progress: Option<OpenClosed>,
    /// Input completeness, as the owner reported it.
    pub completeness: Option<completeness::State>,
}

/// The identity of one result: the waiting instance and the fact that
/// produced it.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResultId {
    /// The waiting instance.
    pub wait: WaitKey,
    /// The fact that produced the result.
    pub produced_at: ObservationRef,
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

/// The FR-033 result for a matching observation admitted after timeout was
/// selected but observed at or before the deadline.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Supersession {
    /// The earlier result, unaltered.
    pub earlier: ResultId,
    /// The contradicted premise: the authority fact that closed the wait.
    pub contradicted_premise: ObservationRef,
    /// The corrected input: the late observation.
    pub corrected_input: ObservationRef,
    /// The new result, a response selection.
    pub superseding: SettledResult,
}

/// Every result and supersession of one wait, in history order.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WaitAssessment {
    /// The waiting instance.
    pub key: WaitKey,
    /// Each result, earliest first; the last is current.
    pub results: Vec<SettledResult>,
    /// Each supersession, earliest first.
    pub supersessions: Vec<Supersession>,
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
    deadline: Deadline,
    last: Option<ObservationRef>,
    seen: Vec<Seen>,
    progress: Option<OpenClosed>,
    completeness: Option<completeness::State>,
    results: Vec<SettledResult>,
    supersessions: Vec<Supersession>,
}

impl Wait {
    /// A wait on `deadline`, assessed under the pure-assessment witness: no
    /// business operation is reachable from any settlement path.
    #[must_use]
    pub const fn new(_pure: PureAssessment, key: WaitKey, deadline: Deadline) -> Self {
        Self {
            key,
            deadline,
            last: None,
            seen: Vec::new(),
            progress: None,
            completeness: None,
            results: Vec::new(),
            supersessions: Vec::new(),
        }
    }

    /// The current result, absent until a fact decides one.
    #[must_use]
    pub fn current(&self) -> Option<&SettledResult> {
        self.results.last()
    }

    /// Admit the next fact of the history.
    ///
    /// # Errors
    ///
    /// [`WaitRefusal::OutOfOrder`] when the fact is not after the last one;
    /// [`WaitRefusal::PremiseRegressed`] when progress or completeness stops
    /// supporting a selected timeout.
    pub fn admit(&mut self, fact: WaitFact) -> Result<Option<&Supersession>, WaitRefusal> {
        if let Some(last) = &self.last {
            if fact.at <= *last {
                return Err(WaitRefusal::OutOfOrder {
                    last: last.clone(),
                    offered: fact.at,
                });
            }
        }
        let before = self.current().cloned();
        match fact.event {
            WaitEvent::Observation {
                observed,
                constraint,
            } => self.seen.push(Seen {
                observed,
                constraint,
                at: fact.at.clone(),
            }),
            WaitEvent::Authority {
                progress,
                completeness,
            } => {
                if let Some(selected) = before.as_ref().filter(|result| {
                    result.settlement.selected == Some(Branch::Timeout)
                        && !(progress == OpenClosed::Closed
                            && completeness == completeness::State::Complete)
                }) {
                    return Err(WaitRefusal::PremiseRegressed {
                        selected: Box::new(selected.id.clone()),
                        at: fact.at,
                    });
                }
                self.progress = Some(progress);
                self.completeness = Some(completeness);
            }
        }
        self.last = Some(fact.at.clone());
        let Some(settlement) = self.judge() else {
            return Ok(None);
        };
        if before.as_ref().map(|result| &result.settlement) == Some(&settlement) {
            return Ok(None);
        }
        let result = SettledResult {
            id: ResultId {
                wait: self.key.clone(),
                produced_at: fact.at.clone(),
            },
            settlement,
        };
        let supersedes = before.filter(|earlier| {
            earlier.settlement.selected == Some(Branch::Timeout)
                && matches!(result.settlement.selected, Some(Branch::Response { .. }))
        });
        self.results.push(result.clone());
        match supersedes {
            Some(earlier) => {
                self.supersessions.push(Supersession {
                    contradicted_premise: earlier.id.produced_at.clone(),
                    earlier: earlier.id,
                    corrected_input: fact.at,
                    superseding: result,
                });
                Ok(self.supersessions.last())
            }
            None => Ok(None),
        }
    }

    fn judge(&self) -> Option<Settlement> {
        let settlement = |selected, truth| Settlement {
            selected,
            truth,
            execution: AssessmentExecution::Completed,
            progress: self.progress,
            completeness: self.completeness,
        };
        if let Some(first) = self
            .seen
            .iter()
            .find(|seen| self.deadline.admits(seen.observed) && seen.constraint.matches())
        {
            return Some(settlement(
                Some(Branch::Response {
                    observation: first.at.clone(),
                }),
                None,
            ));
        }
        let (progress, completeness) = (self.progress?, self.completeness?);
        Some(match (completeness, progress) {
            (completeness::State::Incomplete | completeness::State::Contradicted, _) => {
                settlement(None, Some(TruthDisposition::Unavailable))
            }
            (completeness::State::Complete, OpenClosed::Open) => {
                settlement(None, Some(TruthDisposition::Pending))
            }
            (completeness::State::Complete, OpenClosed::Closed) => {
                let blocked = self
                    .seen
                    .iter()
                    .any(|seen| self.deadline.admits(seen.observed) && !seen.constraint.matches());
                if blocked {
                    settlement(None, Some(TruthDisposition::Unavailable))
                } else {
                    settlement(Some(Branch::Timeout), None)
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
/// As [`Wait::admit`]; a repeated position is [`WaitRefusal::OutOfOrder`].
pub fn assess(
    pure: PureAssessment,
    key: WaitKey,
    deadline: Deadline,
    facts: &[WaitFact],
) -> Result<WaitAssessment, WaitRefusal> {
    let mut ordered = facts.to_vec();
    ordered.sort_by(|left, right| left.at.cmp(&right.at));
    let mut wait = Wait::new(pure, key, deadline);
    for fact in ordered {
        wait.admit(fact)?;
    }
    Ok(wait.finish())
}
