// SPDX-License-Identifier: AGPL-3.0-or-later
//! Five-axis closure and assessment-execution states. Owner chunk: **C-15**.
//!
//! Every vocabulary here is adopted, not declared. This module holds the
//! algebra over them and nothing else.
//!
//! | Concept | Vocabulary | Owner |
//! | --- | --- | --- |
//! | the four scope axes | `open`, `closed` | `FR-284-scope-progress-and-closure-state` |
//! | complete-global-conformance closure | five members | `FR-247-complete-global-conformance-closure-state` |
//! | assessment execution | five members | `FR-241-assessment-execution-disposition` |
//!
//! # Correction: this module previously declared three vocabularies of its own
//!
//! An earlier revision minted 20 labels across the four scope axes
//! (`not-started`, `advanced`, `stalled`, plus `unobserved` and `contradicted`
//! throughout), re-spelled `FR-247`'s five members under a local
//! `complete-global-conformance-closure/` prefix, and fused three separate core
//! axes into one local outcome enum. `FR-284` now names four of those labels
//! directly: "`not-started`, `advanced`, `stalled`, `unobserved` — named by no
//! requirement. They are not members here and are not to be reintroduced by a
//! consumer."
//!
//! # Why there is no join lattice
//!
//! The previous revision gave every axis a flat information lattice with
//! `Unobserved` as bottom and `Contradicted` as top, and combined two admitted
//! facts with `meet`/`join`. Both of those elements were invented, and the
//! construction contradicts the adopted vocabularies in two ways.
//!
//! `FR-284` states of its key order that "`closed` sorting before `open` is an
//! artefact of that ordering and carries no progression from `open` towards
//! `closed`, no precedence and no causal meaning", and gives the axes no causal
//! order at all. A join lattice manufactures exactly that progression.
//!
//! And disagreement is not a state on these axes. `FR-284`: "A retained premise
//! that is later contradicted is reported by supersession, not by a closure
//! label: FR-061-AC-9 requires a superseding or invalidating result that
//! preserves the earlier bytes." So two disagreeing facts do not combine into a
//! third value here; they are two results, and the later one supersedes while
//! retaining the earlier bytes. That is chunk C-26's obligation, not this
//! module's.
//!
//! What remains is what the vocabularies actually support: each axis is a value
//! compared as an exact string, the five are independent, and none substitutes
//! for another.
//!
//! # Why absence is not a value
//!
//! `FR-284`: "an absent axis compares only to absence. It is never defaulted to
//! `open` or to `closed`: FR-061 requires that 'A strict reader SHALL refuse an
//! omitted required axis'. Absence is not a member of this vocabulary."
//!
//! So [`crate::closure::ClosureState::new`] takes an `Option` per axis and refuses an omitted
//! one, rather than admitting a bottom element standing for "not yet observed".
//!
//! # Why one label string is not one type
//!
//! `closed` and `open` are members of the four scope axes and also of the
//! fifth. `FR-284` is explicit that this is a coincidence of spelling: "A label
//! equal to `closed` or `open` on one of the four axes and on the fifth is one
//! label string in two distinct vocabularies. Equality of the label does not
//! make the axes equal or substitutable."
//!
//! The four scope axes are therefore four distinct types over one shared
//! vocabulary type ([`crate::closure::ScopeState`]), and the fifth is unrelated to all of them,
//! so a substitution between any two is a compile error rather than a runtime
//! check.
//!
//! # Why nothing here can silently absorb a new state
//!
//! This module denies `clippy::wildcard_enum_match_arm` and no `match` in it
//! carries a `_` arm, so a state added to any adopted vocabulary fails to
//! compile until an author has classified it. Adding a sixth axis fails too:
//! [`crate::closure::ClosureState::refusals`] destructures every field with no `..` rest
//! pattern, and [`crate::closure::Axis`] is matched exhaustively.
//!
//! # Boundary
//!
//! This module owns the closure and execution algebra only. It does not
//! evaluate conformance, hold obligations, carry premise identities, supersede
//! a result, or produce a protocol result; those belong to chunks C-20, C-26
//! and C-27.

#![deny(clippy::wildcard_enum_match_arm)]

use core::fmt::Debug;
use core::hash::Hash;

// ---------------------------------------------------------------------------
// The adopted scope vocabulary (FR-284)
// ---------------------------------------------------------------------------

/// The two members `FR-284-scope-progress-and-closure-state` publishes.
///
/// This crate selects the vocabulary and mints nothing. `FR-284` states of the
/// pair: "Both label strings are used by the accepted requirements for both
/// scopes and for progress. This artifact mints no new spelling."
///
/// Identity kind *value*: compared componentwise as an exact string, with no
/// causal order over the labels.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ScopeState {
    /// `open`. On a progress axis the retained progress point does not yet
    /// cover the scope's required boundary; on a closure axis the scope's exact
    /// closure authority has not closed it.
    Open,
    /// `closed`. On a progress axis the retained progress point covers the
    /// required boundary; on a closure axis the exact closure authority has
    /// closed it.
    Closed,
}

impl ScopeState {
    /// Both members, in the order `FR-284` tabulates them.
    pub const ALL: [Self; 2] = [Self::Open, Self::Closed];

    /// The published member label, spelled exactly as `FR-284` publishes it.
    pub const fn label(self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::Closed => "closed",
        }
    }
}

/// One of the four independent scope facts `FR-061` decomposes.
///
/// Implemented by four distinct types over [`ScopeState`] so that no axis's
/// state can be passed where another's belongs.
pub trait ScopeAxis: Copy + Debug + Eq + Hash + Sized + 'static {
    /// Which of the four axes this type carries.
    const AXIS: Axis;

    /// Wrap a state as this axis's fact.
    fn of(state: ScopeState) -> Self;

    /// The carried state.
    fn state(self) -> ScopeState;

    /// Every value of this axis, in `FR-284`'s order.
    fn values() -> [Self; 2] {
        [Self::of(ScopeState::Open), Self::of(ScopeState::Closed)]
    }

    /// Whether this state, on its own axis alone, permits complete global
    /// conformance.
    ///
    /// Only `closed` does. `FR-061` forbids reading across axes: "The result
    /// producer SHALL NOT use completeness, assessment execution or any other
    /// progress/closure fact as a substitute for any closure axis."
    fn admits_complete_global_conformance(self) -> bool {
        match self.state() {
            ScopeState::Closed => true,
            ScopeState::Open => false,
        }
    }

    /// The refusal this axis raises when it does not admit.
    fn refusal(self) -> ClosureRefusal;
}

macro_rules! scope_axis {
    ($(#[$meta:meta])* $name:ident, $axis:ident, $refusal:ident) => {
        $(#[$meta])*
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
        pub struct $name(ScopeState);

        impl ScopeAxis for $name {
            const AXIS: Axis = Axis::$axis;

            fn of(state: ScopeState) -> Self {
                Self(state)
            }

            fn state(self) -> ScopeState {
                self.0
            }

            fn refusal(self) -> ClosureRefusal {
                ClosureRefusal::$refusal(self)
            }
        }
    };
}

scope_axis!(
    /// Axis 1: progress observed inside the decision scope.
    DecisionScopeProgress,
    DecisionScopeProgress,
    DecisionScopeProgressNotClosed
);
scope_axis!(
    /// Axis 2: closure of the decision scope by its exact closure authority.
    DecisionScopeClosure,
    DecisionScopeClosure,
    DecisionScopeNotClosed
);
scope_axis!(
    /// Axis 3: progress observed in the surrounding execution.
    SurroundingExecutionProgress,
    SurroundingExecutionProgress,
    SurroundingExecutionProgressNotClosed
);
scope_axis!(
    /// Axis 4: closure of the surrounding execution by its exact closure
    /// authority.
    SurroundingExecutionClosure,
    SurroundingExecutionClosure,
    SurroundingExecutionNotClosed
);

// ---------------------------------------------------------------------------
// The fifth axis (FR-247) — a distinct vocabulary
// ---------------------------------------------------------------------------

/// The five members `FR-247-complete-global-conformance-closure-state`
/// publishes.
///
/// Unrelated to [`ScopeState`] as a type, deliberately. `FR-284`: "Its extra
/// three members belong to that axis alone", and `FR-061` "refuses the
/// cross-wire directly — 'another closure axis relabelled `not-required` is
/// refused'".
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum GlobalConformanceClosure {
    /// `closed`. The global-conformance premise is closed.
    Closed,
    /// `open`. It is not closed.
    Open,
    /// `incomplete`. A member of this axis alone.
    Incomplete,
    /// `contradicted`. A member of this axis alone.
    Contradicted,
    /// `not-required`. A member of this axis alone; no global-conformance claim
    /// applies.
    NotRequired,
}

impl GlobalConformanceClosure {
    /// Every member, in the order `FR-247` tabulates them.
    pub const ALL: [Self; 5] = [
        Self::Closed,
        Self::Open,
        Self::Incomplete,
        Self::Contradicted,
        Self::NotRequired,
    ];

    /// The published member label, spelled exactly as `FR-247` publishes it.
    pub const fn label(self) -> &'static str {
        match self {
            Self::Closed => "closed",
            Self::Open => "open",
            Self::Incomplete => "incomplete",
            Self::Contradicted => "contradicted",
            Self::NotRequired => "not-required",
        }
    }

    /// Whether this state permits complete global conformance.
    ///
    /// Only `closed` does. `not-required` does not: where no global-conformance
    /// claim applies there is no claim to establish, and `FR-059` refuses the
    /// substitution of any other fact for "a missing or non-closed global
    /// premise".
    pub const fn admits_complete_global_conformance(self) -> bool {
        match self {
            Self::Closed => true,
            Self::Open | Self::Incomplete | Self::Contradicted | Self::NotRequired => false,
        }
    }

    /// The refusal this axis raises when it does not admit.
    pub const fn refusal(self) -> ClosureRefusal {
        ClosureRefusal::CompleteGlobalConformanceNotClosed(self)
    }
}

// ---------------------------------------------------------------------------
// The axis set
// ---------------------------------------------------------------------------

/// The five closure axes `FR-061` decomposes.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Axis {
    /// Progress inside the decision scope.
    DecisionScopeProgress,
    /// Closure of the decision scope.
    DecisionScopeClosure,
    /// Progress in the surrounding execution.
    SurroundingExecutionProgress,
    /// Closure of the surrounding execution.
    SurroundingExecutionClosure,
    /// The complete-global-conformance closure record.
    CompleteGlobalConformanceClosure,
}

impl Axis {
    /// Every axis, in declared order. This order is the refusal order.
    pub const ALL: [Self; 5] = [
        Self::DecisionScopeProgress,
        Self::DecisionScopeClosure,
        Self::SurroundingExecutionProgress,
        Self::SurroundingExecutionClosure,
        Self::CompleteGlobalConformanceClosure,
    ];

    /// Declared position of this axis within [`Axis::ALL`].
    pub const fn index(self) -> usize {
        match self {
            Self::DecisionScopeProgress => 0,
            Self::DecisionScopeClosure => 1,
            Self::SurroundingExecutionProgress => 2,
            Self::SurroundingExecutionClosure => 3,
            Self::CompleteGlobalConformanceClosure => 4,
        }
    }

    /// The name `FR-061` and `FR-240` use for this axis.
    pub const fn code(self) -> &'static str {
        match self {
            Self::DecisionScopeProgress => "decision-scope-progress",
            Self::DecisionScopeClosure => "decision-scope-closure",
            Self::SurroundingExecutionProgress => "surrounding-execution-progress",
            Self::SurroundingExecutionClosure => "surrounding-execution-closure",
            Self::CompleteGlobalConformanceClosure => "complete-global-conformance-closure",
        }
    }
}

// ---------------------------------------------------------------------------
// Refusals — one typed variant and one code per axis and per execution state
// ---------------------------------------------------------------------------

/// A single reason complete global conformance was not established.
///
/// Every axis and every non-completing execution state has its own variant
/// carrying its own observed value, and its own code. No two variants share a
/// code, so bypassing one axis cannot fall through to another under the same
/// code.
///
/// These codes are this crate's own. `FR-271-diagnostic-code-vocabulary`
/// governs *producer*-minted codes and a consumer's refusal codes are a
/// separate namespace that never resolves to a member there.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ClosureRefusal {
    /// An axis required for the claim was omitted entirely.
    ///
    /// `FR-061` via `FR-284`: "A strict reader SHALL refuse an omitted required
    /// axis". Absence is not a value on any axis, so it is a refusal.
    AxisOmitted(Axis),
    /// Axis 1 is not closed.
    DecisionScopeProgressNotClosed(DecisionScopeProgress),
    /// Axis 2 is not closed.
    DecisionScopeNotClosed(DecisionScopeClosure),
    /// Axis 3 is not closed.
    SurroundingExecutionProgressNotClosed(SurroundingExecutionProgress),
    /// Axis 4 is not closed.
    SurroundingExecutionNotClosed(SurroundingExecutionClosure),
    /// Axis 5 is not closed.
    CompleteGlobalConformanceNotClosed(GlobalConformanceClosure),
    /// The evaluator refused the request (`FR-241` `refused`).
    ExecutionRefused,
    /// The requested capability is unsupported (`FR-241` `unsupported`).
    ExecutionUnsupported,
    /// A resource ceiling stopped the assessment (`FR-241`
    /// `resource-incomplete`).
    ExecutionResourceIncomplete,
    /// The selected analysis backend failed for this exact claim (`FR-241`
    /// `failed`).
    ExecutionFailed,
}

impl ClosureRefusal {
    /// The code of this refusal. Unique across every variant.
    pub const fn code(self) -> &'static str {
        match self {
            Self::AxisOmitted(_) => "closure.axis.omitted",
            Self::DecisionScopeProgressNotClosed(_) => "closure.decision-scope-progress.not-closed",
            Self::DecisionScopeNotClosed(_) => "closure.decision-scope.not-closed",
            Self::SurroundingExecutionProgressNotClosed(_) => {
                "closure.surrounding-execution-progress.not-closed"
            }
            Self::SurroundingExecutionNotClosed(_) => "closure.surrounding-execution.not-closed",
            Self::CompleteGlobalConformanceNotClosed(_) => {
                "closure.complete-global-conformance.not-closed"
            }
            Self::ExecutionRefused => "execution.refused",
            Self::ExecutionUnsupported => "execution.unsupported",
            Self::ExecutionResourceIncomplete => "execution.resource-incomplete",
            Self::ExecutionFailed => "execution.failed",
        }
    }

    /// The axis this refusal came from, or `None` for an execution refusal.
    pub const fn axis(self) -> Option<Axis> {
        match self {
            Self::AxisOmitted(axis) => Some(axis),
            Self::DecisionScopeProgressNotClosed(_) => Some(Axis::DecisionScopeProgress),
            Self::DecisionScopeNotClosed(_) => Some(Axis::DecisionScopeClosure),
            Self::SurroundingExecutionProgressNotClosed(_) => {
                Some(Axis::SurroundingExecutionProgress)
            }
            Self::SurroundingExecutionNotClosed(_) => Some(Axis::SurroundingExecutionClosure),
            Self::CompleteGlobalConformanceNotClosed(_) => {
                Some(Axis::CompleteGlobalConformanceClosure)
            }
            Self::ExecutionRefused
            | Self::ExecutionUnsupported
            | Self::ExecutionResourceIncomplete
            | Self::ExecutionFailed => None,
        }
    }
}

// ---------------------------------------------------------------------------
// Assessment execution (FR-241)
// ---------------------------------------------------------------------------

/// The five members `FR-241-assessment-execution-disposition` publishes.
///
/// This axis records **whether the requested evaluator ran** — not whether
/// conformance succeeded. `FR-241`: "no state other than `completed` carries a
/// successful truth value."
///
/// An earlier revision of this crate fused three separate core axes into one
/// local enum here: a `CompleteGlobalSuccess` member that is not an execution
/// state at all, an `Open` member belonging to the closure axes (`FR-284`), and
/// an `Incomplete` member belonging to input completeness. `refused` was
/// missing entirely.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum AssessmentExecution {
    /// `completed`. The requested evaluator completed. Required for truth
    /// `satisfied`, `violated` or `pending`.
    Completed,
    /// `refused`. The evaluator refused the request; truth and settlement basis
    /// are unavailable.
    Refused,
    /// `unsupported`. The requested capability is unsupported. One unsupported
    /// request does not remove an independent supported result.
    Unsupported,
    /// `resource-incomplete`. A resource ceiling stopped the assessment; an
    /// independently settled sibling result is unchanged.
    ResourceIncomplete,
    /// `failed`. The selected analysis backend failed for this exact claim,
    /// without invoking a fallback or changing independent results.
    Failed,
}

impl AssessmentExecution {
    /// Every member, in the order `FR-241` tabulates them.
    pub const ALL: [Self; 5] = [
        Self::Completed,
        Self::Refused,
        Self::Unsupported,
        Self::ResourceIncomplete,
        Self::Failed,
    ];

    /// The four states that do not carry a successful truth value.
    pub const NON_COMPLETING: [Self; 4] = [
        Self::Refused,
        Self::Unsupported,
        Self::ResourceIncomplete,
        Self::Failed,
    ];

    /// The published member label, spelled exactly as `FR-241` publishes it.
    pub const fn label(self) -> &'static str {
        match self {
            Self::Completed => "completed",
            Self::Refused => "refused",
            Self::Unsupported => "unsupported",
            Self::ResourceIncomplete => "resource-incomplete",
            Self::Failed => "failed",
        }
    }

    /// The refusal this state raises, or `None` for `completed`.
    pub const fn refusal(self) -> Option<ClosureRefusal> {
        match self {
            Self::Completed => None,
            Self::Refused => Some(ClosureRefusal::ExecutionRefused),
            Self::Unsupported => Some(ClosureRefusal::ExecutionUnsupported),
            Self::ResourceIncomplete => Some(ClosureRefusal::ExecutionResourceIncomplete),
            Self::Failed => Some(ClosureRefusal::ExecutionFailed),
        }
    }
}

// ---------------------------------------------------------------------------
// Truth disposition (FR-242)
// ---------------------------------------------------------------------------

/// The four members `FR-242-truth-disposition` publishes.
///
/// No Rust crate publishes this vocabulary, so it is adopted here the way
/// [`AssessmentExecution`] adopts `FR-241`: the labels are the owner's and are
/// compared as exact strings. Neither `pending` nor `unavailable` is a Boolean
/// success, and an absent truth is `None`, never `unavailable` (`FR-242-AC-3`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TruthDisposition {
    /// `satisfied`.
    Satisfied,
    /// `violated`.
    Violated,
    /// `pending`: execution `completed`, decision-scope progress open.
    Pending,
    /// `unavailable`: a required decision-support fact is missing.
    Unavailable,
}

impl TruthDisposition {
    /// Every member, in the order `FR-242` tabulates them.
    pub const ALL: [Self; 4] = [
        Self::Satisfied,
        Self::Violated,
        Self::Pending,
        Self::Unavailable,
    ];

    /// The published member label, spelled exactly as `FR-242` publishes it.
    pub const fn label(self) -> &'static str {
        match self {
            Self::Satisfied => "satisfied",
            Self::Violated => "violated",
            Self::Pending => "pending",
            Self::Unavailable => "unavailable",
        }
    }
}

// ---------------------------------------------------------------------------
// The five axes observed together
// ---------------------------------------------------------------------------

/// The five independent closure facts of one canonical protocol result.
///
/// A product of five independent values, not a lattice: `FR-284` gives the
/// labels no causal order and no progression, and reports a later
/// contradiction by supersession rather than by a combined value.
///
/// A sixth axis cannot be added silently: [`ClosureState::refusals`]
/// destructures every field with no `..` rest pattern.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ClosureState {
    /// Axis 1.
    pub decision_scope_progress: DecisionScopeProgress,
    /// Axis 2.
    pub decision_scope_closure: DecisionScopeClosure,
    /// Axis 3.
    pub surrounding_execution_progress: SurroundingExecutionProgress,
    /// Axis 4.
    pub surrounding_execution_closure: SurroundingExecutionClosure,
    /// Axis 5, carrying its own distinct vocabulary.
    pub complete_global_conformance_closure: GlobalConformanceClosure,
}

impl ClosureState {
    /// Admit five observed axes, refusing any that was omitted.
    ///
    /// `FR-284` forbids defaulting an absent axis: "It is never defaulted to
    /// `open` or to `closed`: FR-061 requires that 'A strict reader SHALL refuse
    /// an omitted required axis'."
    ///
    /// Every omitted axis is reported, in [`Axis::ALL`] order — a first failure
    /// does not mask the rest.
    pub fn new(
        decision_scope_progress: Option<DecisionScopeProgress>,
        decision_scope_closure: Option<DecisionScopeClosure>,
        surrounding_execution_progress: Option<SurroundingExecutionProgress>,
        surrounding_execution_closure: Option<SurroundingExecutionClosure>,
        complete_global_conformance_closure: Option<GlobalConformanceClosure>,
    ) -> Result<Self, Vec<ClosureRefusal>> {
        let mut omitted = Vec::new();
        if decision_scope_progress.is_none() {
            omitted.push(ClosureRefusal::AxisOmitted(Axis::DecisionScopeProgress));
        }
        if decision_scope_closure.is_none() {
            omitted.push(ClosureRefusal::AxisOmitted(Axis::DecisionScopeClosure));
        }
        if surrounding_execution_progress.is_none() {
            omitted.push(ClosureRefusal::AxisOmitted(
                Axis::SurroundingExecutionProgress,
            ));
        }
        if surrounding_execution_closure.is_none() {
            omitted.push(ClosureRefusal::AxisOmitted(
                Axis::SurroundingExecutionClosure,
            ));
        }
        if complete_global_conformance_closure.is_none() {
            omitted.push(ClosureRefusal::AxisOmitted(
                Axis::CompleteGlobalConformanceClosure,
            ));
        }
        match (
            decision_scope_progress,
            decision_scope_closure,
            surrounding_execution_progress,
            surrounding_execution_closure,
            complete_global_conformance_closure,
        ) {
            (Some(one), Some(two), Some(three), Some(four), Some(five)) => Ok(Self {
                decision_scope_progress: one,
                decision_scope_closure: two,
                surrounding_execution_progress: three,
                surrounding_execution_closure: four,
                complete_global_conformance_closure: five,
            }),
            _ => Err(omitted),
        }
    }

    /// The state of one axis, as its published label.
    pub const fn label(&self, axis: Axis) -> &'static str {
        match axis {
            Axis::DecisionScopeProgress => self.decision_scope_progress.0.label(),
            Axis::DecisionScopeClosure => self.decision_scope_closure.0.label(),
            Axis::SurroundingExecutionProgress => self.surrounding_execution_progress.0.label(),
            Axis::SurroundingExecutionClosure => self.surrounding_execution_closure.0.label(),
            Axis::CompleteGlobalConformanceClosure => {
                self.complete_global_conformance_closure.label()
            }
        }
    }

    /// Whether one axis, on its own, permits complete global conformance.
    pub fn established(&self, axis: Axis) -> bool {
        match axis {
            Axis::DecisionScopeProgress => self
                .decision_scope_progress
                .admits_complete_global_conformance(),
            Axis::DecisionScopeClosure => self
                .decision_scope_closure
                .admits_complete_global_conformance(),
            Axis::SurroundingExecutionProgress => self
                .surrounding_execution_progress
                .admits_complete_global_conformance(),
            Axis::SurroundingExecutionClosure => self
                .surrounding_execution_closure
                .admits_complete_global_conformance(),
            Axis::CompleteGlobalConformanceClosure => self
                .complete_global_conformance_closure
                .admits_complete_global_conformance(),
        }
    }

    /// The refusal one axis raises, or `None` when that axis admits.
    pub fn refusal_for(&self, axis: Axis) -> Option<ClosureRefusal> {
        if self.established(axis) {
            return None;
        }
        Some(match axis {
            Axis::DecisionScopeProgress => self.decision_scope_progress.refusal(),
            Axis::DecisionScopeClosure => self.decision_scope_closure.refusal(),
            Axis::SurroundingExecutionProgress => self.surrounding_execution_progress.refusal(),
            Axis::SurroundingExecutionClosure => self.surrounding_execution_closure.refusal(),
            Axis::CompleteGlobalConformanceClosure => {
                self.complete_global_conformance_closure.refusal()
            }
        })
    }

    /// Every axis that does not admit complete global conformance, in
    /// [`Axis::ALL`] order. No refusal masks any other.
    pub fn refusals(&self) -> Vec<ClosureRefusal> {
        // Destructured with no `..` rest pattern: a sixth axis breaks this.
        let Self {
            decision_scope_progress: _,
            decision_scope_closure: _,
            surrounding_execution_progress: _,
            surrounding_execution_closure: _,
            complete_global_conformance_closure: _,
        } = self;
        Axis::ALL
            .into_iter()
            .filter_map(|axis| self.refusal_for(axis))
            .collect()
    }
}

// ---------------------------------------------------------------------------
// The assessment and its witness
// ---------------------------------------------------------------------------

/// Proof that complete global conformance was established.
///
/// This type has no public constructor. The only way to obtain one is
/// [`ProtocolAssessment::complete_global_conformance`], so no caller can
/// promote a non-completing execution state or an unclosed axis into complete
/// global success:
///
/// ```compile_fail
/// let _ = quire_protocol::closure::CompleteGlobalConformance {};
/// ```
///
/// Nor can one axis's state be substituted for another's. `FR-284` requires
/// this: one label string in two vocabularies does not make the axes
/// substitutable, so the types are unrelated and substitution does not compile:
///
/// ```compile_fail
/// use quire_protocol::closure::{DecisionScopeClosure, GlobalConformanceClosure};
/// let _: GlobalConformanceClosure = DecisionScopeClosure::of(ScopeState::Closed);
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct CompleteGlobalConformance {
    /// Private field: blocks construction outside this module.
    witness: (),
}

/// One assessment: the five closure axes plus the execution state.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ProtocolAssessment {
    /// The five independently observed closure axes.
    pub closure: ClosureState,
    /// Whether the requested evaluator ran.
    pub execution: AssessmentExecution,
}

impl ProtocolAssessment {
    /// Build an assessment from its closure state and execution state.
    pub const fn new(closure: ClosureState, execution: AssessmentExecution) -> Self {
        Self { closure, execution }
    }

    /// Every reason this assessment is not complete global conformance, in
    /// declared order: the five axes in [`Axis::ALL`] order, then the execution
    /// state.
    ///
    /// The order is declared, never sorted, and no refusal is masked by any
    /// other — every failing axis is reported.
    pub fn refusals(self) -> Vec<ClosureRefusal> {
        let mut refusals = self.closure.refusals();
        refusals.extend(self.execution.refusal());
        refusals
    }

    /// Establish complete global conformance, or refuse with every reason.
    pub fn complete_global_conformance(
        self,
    ) -> Result<CompleteGlobalConformance, Vec<ClosureRefusal>> {
        let refusals = self.refusals();
        if refusals.is_empty() {
            Ok(CompleteGlobalConformance { witness: () })
        } else {
            Err(refusals)
        }
    }
}
