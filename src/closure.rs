// SPDX-License-Identifier: AGPL-3.0-or-later
//! Five-axis closure and outcome lattice. Owner chunk: **C-15**.
//!
//! FR-004-AC-3 requires that decision-scope progress, decision-scope closure,
//! surrounding-execution progress, surrounding-execution closure and
//! complete-global-conformance closure stay **five independent observable
//! axes**, and that `open | incomplete | unsupported | failed | exhausted`
//! assessments can never become complete global success.
//!
//! # Why the axes are a lattice and not a scalar
//!
//! Each axis records an independently admitted fact. Two admitted facts about
//! the same axis have to be combinable without either one repairing the other,
//! so every axis carries its own **flat (information) lattice**:
//!
//! * `Unobserved` is the bottom element — no fact has been admitted.
//! * Each remaining state is an atom; distinct atoms are incomparable, because
//!   no admitted state is evidence for any other admitted state.
//! * `Contradicted` is the top element — two admitted facts disagreed. This is
//!   the spec-named `contradicted` state, not a synthesised one.
//!
//! `join` is "everything both observers committed to" widened to
//! `Contradicted` when they disagree; `meet` is "what the two observers agree
//! on" narrowed to `Unobserved` when they do not. [`ClosureState`] is the
//! **product** of the five axis lattices, ordered and combined componentwise,
//! so an operation on one axis is by construction invisible to the other four.
//!
//! # Why nothing here can silently absorb a new state
//!
//! This module denies `clippy::wildcard_enum_match_arm`, and no `match` in it
//! carries a `_` arm. Adding a state to any axis breaks compilation in four
//! places at once ([`ClosureAxis::index`], [`ClosureAxis::rank`],
//! [`ClosureAxis::code`] and
//! [`ClosureAxis::admits_complete_global_conformance`]), so a new state can
//! never default into "not admitting" without an author saying so. Adding a
//! sixth axis breaks compilation too: [`ClosureState::refusals`] destructures
//! every field with no `..` rest pattern, and [`Axis`] is matched exhaustively.
//!
//! # Known limit of the declared value sets
//!
//! [`ClosureAxis::VALUES`] is hand-declared, and stable Rust has no way to
//! assert at compile time that it lists every variant. The four exhaustive
//! matches above are the real defence: a new state cannot compile until an
//! author has classified it. Closing the remaining gap needs a derive macro,
//! which needs a `Cargo.toml` dependency, which is serialised to one chunk per
//! wave (COLLISION-7) — so it is a follow-up, not a silent omission.
//!
//! # Boundary
//!
//! This module owns closure/outcome algebra only. It does not evaluate
//! conformance, hold obligations, carry premise identities, or produce a
//! protocol result; those belong to chunks C-20, C-26 and C-27.

#![deny(clippy::wildcard_enum_match_arm)]

use core::fmt::Debug;
use core::hash::Hash;

// ---------------------------------------------------------------------------
// Axis lattice machinery
// ---------------------------------------------------------------------------

/// Position of an axis state in its flat lattice.
///
/// `Bottom` is "no fact admitted", `Top` is "admitted facts contradicted each
/// other", and every genuinely observed state is an incomparable `Atom`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum AxisRank {
    /// The `Unobserved` element: below every other state on the axis.
    Bottom,
    /// An admitted observation. Distinct atoms are incomparable.
    Atom,
    /// The `Contradicted` element: above every other state on the axis.
    Top,
}

/// One closure axis: a bounded flat lattice of independently admitted states.
///
/// Every method is written as an exhaustive `match` with no `_` arm, so a new
/// state on any implementing axis fails to compile rather than defaulting.
pub trait ClosureAxis: Copy + Debug + Eq + Hash + Sized + 'static {
    /// The axis this state belongs to.
    const AXIS: Axis;
    /// Every state of this axis, in declared order.
    const VALUES: &'static [Self];
    /// The `Unobserved` bottom element.
    const BOTTOM: Self;
    /// The `Contradicted` top element.
    const TOP: Self;

    /// Declared position of this state within [`Self::VALUES`].
    ///
    /// Adding a state without extending `VALUES` is caught by
    /// `tc_004_axis_value_sets_are_exhaustive`; adding one without extending
    /// this match does not compile.
    fn index(self) -> usize;

    /// Lattice rank of this state.
    fn rank(self) -> AxisRank;

    /// The stable code naming this exact state. Distinct across all axes.
    fn code(self) -> &'static str;

    /// Whether this state, on its own axis alone, permits complete global
    /// conformance. Exactly one state per axis may return `true`.
    fn admits_complete_global_conformance(self) -> bool;

    /// The refusal this axis raises when [`Self::admits_complete_global_conformance`]
    /// is false. Each axis returns its own [`ClosureRefusal`] variant.
    fn refusal(self) -> ClosureRefusal;

    /// Greatest lower bound: what two independent observers both commit to.
    fn meet(self, other: Self) -> Self {
        if self == other {
            return self;
        }
        match (self.rank(), other.rank()) {
            (AxisRank::Bottom, AxisRank::Bottom) => self,
            (AxisRank::Bottom, AxisRank::Atom) => self,
            (AxisRank::Bottom, AxisRank::Top) => self,
            (AxisRank::Atom, AxisRank::Bottom) => other,
            (AxisRank::Atom, AxisRank::Atom) => Self::BOTTOM,
            (AxisRank::Atom, AxisRank::Top) => self,
            (AxisRank::Top, AxisRank::Bottom) => other,
            (AxisRank::Top, AxisRank::Atom) => other,
            (AxisRank::Top, AxisRank::Top) => self,
        }
    }

    /// Least upper bound: the two observations combined, `Contradicted` when
    /// they disagree.
    fn join(self, other: Self) -> Self {
        if self == other {
            return self;
        }
        match (self.rank(), other.rank()) {
            (AxisRank::Bottom, AxisRank::Bottom) => self,
            (AxisRank::Bottom, AxisRank::Atom) => other,
            (AxisRank::Bottom, AxisRank::Top) => other,
            (AxisRank::Atom, AxisRank::Bottom) => self,
            (AxisRank::Atom, AxisRank::Atom) => Self::TOP,
            (AxisRank::Atom, AxisRank::Top) => other,
            (AxisRank::Top, AxisRank::Bottom) => self,
            (AxisRank::Top, AxisRank::Atom) => self,
            (AxisRank::Top, AxisRank::Top) => self,
        }
    }

    /// The lattice order: `self` is at or below `other`.
    fn is_at_or_below(self, other: Self) -> bool {
        self.meet(other) == self
    }
}

// ---------------------------------------------------------------------------
// Axis 1 — decision-scope progress
// ---------------------------------------------------------------------------

/// Axis 1: progress observed inside the decision scope.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum DecisionScopeProgress {
    /// No progress fact was admitted for the decision scope.
    Unobserved,
    /// The decision scope has admitted no progress point yet.
    NotStarted,
    /// The decision scope advanced past an admitted progress point.
    Advanced,
    /// The decision scope admitted a progress point and did not pass it.
    Stalled,
    /// Two admitted decision-scope progress facts disagreed.
    Contradicted,
}

impl ClosureAxis for DecisionScopeProgress {
    const AXIS: Axis = Axis::DecisionScopeProgress;
    const VALUES: &'static [Self] = &[
        Self::Unobserved,
        Self::NotStarted,
        Self::Advanced,
        Self::Stalled,
        Self::Contradicted,
    ];
    const BOTTOM: Self = Self::Unobserved;
    const TOP: Self = Self::Contradicted;

    fn index(self) -> usize {
        match self {
            Self::Unobserved => 0,
            Self::NotStarted => 1,
            Self::Advanced => 2,
            Self::Stalled => 3,
            Self::Contradicted => 4,
        }
    }

    fn rank(self) -> AxisRank {
        match self {
            Self::Unobserved => AxisRank::Bottom,
            Self::NotStarted => AxisRank::Atom,
            Self::Advanced => AxisRank::Atom,
            Self::Stalled => AxisRank::Atom,
            Self::Contradicted => AxisRank::Top,
        }
    }

    fn code(self) -> &'static str {
        match self {
            Self::Unobserved => "decision-scope-progress/unobserved",
            Self::NotStarted => "decision-scope-progress/not-started",
            Self::Advanced => "decision-scope-progress/advanced",
            Self::Stalled => "decision-scope-progress/stalled",
            Self::Contradicted => "decision-scope-progress/contradicted",
        }
    }

    fn admits_complete_global_conformance(self) -> bool {
        match self {
            Self::Unobserved => false,
            Self::NotStarted => false,
            Self::Advanced => true,
            Self::Stalled => false,
            Self::Contradicted => false,
        }
    }

    fn refusal(self) -> ClosureRefusal {
        ClosureRefusal::DecisionScopeProgressNotEstablished(self)
    }
}

// ---------------------------------------------------------------------------
// Axis 2 — decision-scope closure
// ---------------------------------------------------------------------------

/// Axis 2: closure of the decision scope.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum DecisionScopeClosure {
    /// No closure fact was admitted for the decision scope.
    Unobserved,
    /// The decision scope is admitted open.
    Open,
    /// The decision scope is admitted incomplete.
    Incomplete,
    /// The decision scope is admitted closed.
    Closed,
    /// Two admitted decision-scope closure facts disagreed.
    Contradicted,
}

impl ClosureAxis for DecisionScopeClosure {
    const AXIS: Axis = Axis::DecisionScopeClosure;
    const VALUES: &'static [Self] = &[
        Self::Unobserved,
        Self::Open,
        Self::Incomplete,
        Self::Closed,
        Self::Contradicted,
    ];
    const BOTTOM: Self = Self::Unobserved;
    const TOP: Self = Self::Contradicted;

    fn index(self) -> usize {
        match self {
            Self::Unobserved => 0,
            Self::Open => 1,
            Self::Incomplete => 2,
            Self::Closed => 3,
            Self::Contradicted => 4,
        }
    }

    fn rank(self) -> AxisRank {
        match self {
            Self::Unobserved => AxisRank::Bottom,
            Self::Open => AxisRank::Atom,
            Self::Incomplete => AxisRank::Atom,
            Self::Closed => AxisRank::Atom,
            Self::Contradicted => AxisRank::Top,
        }
    }

    fn code(self) -> &'static str {
        match self {
            Self::Unobserved => "decision-scope-closure/unobserved",
            Self::Open => "decision-scope-closure/open",
            Self::Incomplete => "decision-scope-closure/incomplete",
            Self::Closed => "decision-scope-closure/closed",
            Self::Contradicted => "decision-scope-closure/contradicted",
        }
    }

    fn admits_complete_global_conformance(self) -> bool {
        match self {
            Self::Unobserved => false,
            Self::Open => false,
            Self::Incomplete => false,
            Self::Closed => true,
            Self::Contradicted => false,
        }
    }

    fn refusal(self) -> ClosureRefusal {
        ClosureRefusal::DecisionScopeNotClosed(self)
    }
}

// ---------------------------------------------------------------------------
// Axis 3 — surrounding-execution progress
// ---------------------------------------------------------------------------

/// Axis 3: progress observed in the execution surrounding the decision scope.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SurroundingExecutionProgress {
    /// No progress fact was admitted for the surrounding execution.
    Unobserved,
    /// The surrounding execution has admitted no progress point yet.
    NotStarted,
    /// The surrounding execution advanced past an admitted progress point.
    Advanced,
    /// The surrounding execution admitted a progress point and did not pass it.
    Stalled,
    /// Two admitted surrounding-execution progress facts disagreed.
    Contradicted,
}

impl ClosureAxis for SurroundingExecutionProgress {
    const AXIS: Axis = Axis::SurroundingExecutionProgress;
    const VALUES: &'static [Self] = &[
        Self::Unobserved,
        Self::NotStarted,
        Self::Advanced,
        Self::Stalled,
        Self::Contradicted,
    ];
    const BOTTOM: Self = Self::Unobserved;
    const TOP: Self = Self::Contradicted;

    fn index(self) -> usize {
        match self {
            Self::Unobserved => 0,
            Self::NotStarted => 1,
            Self::Advanced => 2,
            Self::Stalled => 3,
            Self::Contradicted => 4,
        }
    }

    fn rank(self) -> AxisRank {
        match self {
            Self::Unobserved => AxisRank::Bottom,
            Self::NotStarted => AxisRank::Atom,
            Self::Advanced => AxisRank::Atom,
            Self::Stalled => AxisRank::Atom,
            Self::Contradicted => AxisRank::Top,
        }
    }

    fn code(self) -> &'static str {
        match self {
            Self::Unobserved => "surrounding-execution-progress/unobserved",
            Self::NotStarted => "surrounding-execution-progress/not-started",
            Self::Advanced => "surrounding-execution-progress/advanced",
            Self::Stalled => "surrounding-execution-progress/stalled",
            Self::Contradicted => "surrounding-execution-progress/contradicted",
        }
    }

    fn admits_complete_global_conformance(self) -> bool {
        match self {
            Self::Unobserved => false,
            Self::NotStarted => false,
            Self::Advanced => true,
            Self::Stalled => false,
            Self::Contradicted => false,
        }
    }

    fn refusal(self) -> ClosureRefusal {
        ClosureRefusal::SurroundingExecutionProgressNotEstablished(self)
    }
}

// ---------------------------------------------------------------------------
// Axis 4 — surrounding-execution closure
// ---------------------------------------------------------------------------

/// Axis 4: closure of the execution surrounding the decision scope.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SurroundingExecutionClosure {
    /// No closure fact was admitted for the surrounding execution.
    Unobserved,
    /// The surrounding execution is admitted open.
    Open,
    /// The surrounding execution is admitted incomplete.
    Incomplete,
    /// The surrounding execution is admitted closed.
    Closed,
    /// Two admitted surrounding-execution closure facts disagreed.
    Contradicted,
}

impl ClosureAxis for SurroundingExecutionClosure {
    const AXIS: Axis = Axis::SurroundingExecutionClosure;
    const VALUES: &'static [Self] = &[
        Self::Unobserved,
        Self::Open,
        Self::Incomplete,
        Self::Closed,
        Self::Contradicted,
    ];
    const BOTTOM: Self = Self::Unobserved;
    const TOP: Self = Self::Contradicted;

    fn index(self) -> usize {
        match self {
            Self::Unobserved => 0,
            Self::Open => 1,
            Self::Incomplete => 2,
            Self::Closed => 3,
            Self::Contradicted => 4,
        }
    }

    fn rank(self) -> AxisRank {
        match self {
            Self::Unobserved => AxisRank::Bottom,
            Self::Open => AxisRank::Atom,
            Self::Incomplete => AxisRank::Atom,
            Self::Closed => AxisRank::Atom,
            Self::Contradicted => AxisRank::Top,
        }
    }

    fn code(self) -> &'static str {
        match self {
            Self::Unobserved => "surrounding-execution-closure/unobserved",
            Self::Open => "surrounding-execution-closure/open",
            Self::Incomplete => "surrounding-execution-closure/incomplete",
            Self::Closed => "surrounding-execution-closure/closed",
            Self::Contradicted => "surrounding-execution-closure/contradicted",
        }
    }

    fn admits_complete_global_conformance(self) -> bool {
        match self {
            Self::Unobserved => false,
            Self::Open => false,
            Self::Incomplete => false,
            Self::Closed => true,
            Self::Contradicted => false,
        }
    }

    fn refusal(self) -> ClosureRefusal {
        ClosureRefusal::SurroundingExecutionNotClosed(self)
    }
}

// ---------------------------------------------------------------------------
// Axis 5 — complete-global-conformance closure
// ---------------------------------------------------------------------------

/// Axis 5: the complete-global-conformance closure record.
///
/// FR-006 requires a global-conformance claim to carry one of `closed`,
/// `open`, `incomplete` or `contradicted`, and every other claim to carry
/// `not-required`. `NotRequired` is a genuine observation and therefore an
/// atom, but it does **not** admit complete global conformance: a claim that
/// did not require global closure never established it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum GlobalConformanceClosure {
    /// No complete-global-conformance closure record was admitted.
    Unobserved,
    /// The claim is not a global-conformance claim.
    NotRequired,
    /// Complete global conformance is admitted open.
    Open,
    /// Complete global conformance is admitted incomplete.
    Incomplete,
    /// Complete global conformance is admitted closed.
    Closed,
    /// Two admitted complete-global-conformance closure facts disagreed.
    Contradicted,
}

impl ClosureAxis for GlobalConformanceClosure {
    const AXIS: Axis = Axis::CompleteGlobalConformanceClosure;
    const VALUES: &'static [Self] = &[
        Self::Unobserved,
        Self::NotRequired,
        Self::Open,
        Self::Incomplete,
        Self::Closed,
        Self::Contradicted,
    ];
    const BOTTOM: Self = Self::Unobserved;
    const TOP: Self = Self::Contradicted;

    fn index(self) -> usize {
        match self {
            Self::Unobserved => 0,
            Self::NotRequired => 1,
            Self::Open => 2,
            Self::Incomplete => 3,
            Self::Closed => 4,
            Self::Contradicted => 5,
        }
    }

    fn rank(self) -> AxisRank {
        match self {
            Self::Unobserved => AxisRank::Bottom,
            Self::NotRequired => AxisRank::Atom,
            Self::Open => AxisRank::Atom,
            Self::Incomplete => AxisRank::Atom,
            Self::Closed => AxisRank::Atom,
            Self::Contradicted => AxisRank::Top,
        }
    }

    fn code(self) -> &'static str {
        match self {
            Self::Unobserved => "complete-global-conformance-closure/unobserved",
            Self::NotRequired => "complete-global-conformance-closure/not-required",
            Self::Open => "complete-global-conformance-closure/open",
            Self::Incomplete => "complete-global-conformance-closure/incomplete",
            Self::Closed => "complete-global-conformance-closure/closed",
            Self::Contradicted => "complete-global-conformance-closure/contradicted",
        }
    }

    fn admits_complete_global_conformance(self) -> bool {
        match self {
            Self::Unobserved => false,
            Self::NotRequired => false,
            Self::Open => false,
            Self::Incomplete => false,
            Self::Closed => true,
            Self::Contradicted => false,
        }
    }

    fn refusal(self) -> ClosureRefusal {
        ClosureRefusal::CompleteGlobalConformanceNotClosed(self)
    }
}

// ---------------------------------------------------------------------------
// The axis set
// ---------------------------------------------------------------------------

/// The five closure axes.
///
/// Adding a sixth variant fails to compile in [`Axis::index`],
/// [`Axis::code`], [`ClosureState::refusal_for`], [`ClosureState::state_code`]
/// and [`ClosureState::established`].
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
    pub fn index(self) -> usize {
        match self {
            Self::DecisionScopeProgress => 0,
            Self::DecisionScopeClosure => 1,
            Self::SurroundingExecutionProgress => 2,
            Self::SurroundingExecutionClosure => 3,
            Self::CompleteGlobalConformanceClosure => 4,
        }
    }

    /// The stable code naming this axis.
    pub fn code(self) -> &'static str {
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
// Refusals — one typed variant and one stable code per axis and per outcome
// ---------------------------------------------------------------------------

/// A single reason complete global conformance was not established.
///
/// Every axis and every non-success outcome has its own variant carrying its
/// own observed state, and its own stable code. No two variants share a code,
/// so bypassing one axis cannot fall through to another under the same code.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ClosureRefusal {
    /// Axis 1 did not establish decision-scope progress.
    DecisionScopeProgressNotEstablished(DecisionScopeProgress),
    /// Axis 2 did not close the decision scope.
    DecisionScopeNotClosed(DecisionScopeClosure),
    /// Axis 3 did not establish surrounding-execution progress.
    SurroundingExecutionProgressNotEstablished(SurroundingExecutionProgress),
    /// Axis 4 did not close the surrounding execution.
    SurroundingExecutionNotClosed(SurroundingExecutionClosure),
    /// Axis 5 did not close complete global conformance.
    CompleteGlobalConformanceNotClosed(GlobalConformanceClosure),
    /// The assessment outcome is `open`.
    OutcomeOpen,
    /// The assessment outcome is `incomplete`.
    OutcomeIncomplete,
    /// The assessment outcome is `unsupported`.
    OutcomeUnsupported,
    /// The assessment outcome is `failed`.
    OutcomeFailed,
    /// The assessment outcome is `exhausted`.
    OutcomeExhausted,
}

impl ClosureRefusal {
    /// The stable code of this refusal. Unique across every variant.
    pub fn code(self) -> &'static str {
        match self {
            Self::DecisionScopeProgressNotEstablished(_) => {
                "closure.decision-scope-progress.not-established"
            }
            Self::DecisionScopeNotClosed(_) => "closure.decision-scope.not-closed",
            Self::SurroundingExecutionProgressNotEstablished(_) => {
                "closure.surrounding-execution-progress.not-established"
            }
            Self::SurroundingExecutionNotClosed(_) => "closure.surrounding-execution.not-closed",
            Self::CompleteGlobalConformanceNotClosed(_) => {
                "closure.complete-global-conformance.not-closed"
            }
            Self::OutcomeOpen => "outcome.open",
            Self::OutcomeIncomplete => "outcome.incomplete",
            Self::OutcomeUnsupported => "outcome.unsupported",
            Self::OutcomeFailed => "outcome.failed",
            Self::OutcomeExhausted => "outcome.exhausted",
        }
    }

    /// The axis this refusal came from, or `None` for an outcome refusal.
    pub fn axis(self) -> Option<Axis> {
        match self {
            Self::DecisionScopeProgressNotEstablished(_) => Some(Axis::DecisionScopeProgress),
            Self::DecisionScopeNotClosed(_) => Some(Axis::DecisionScopeClosure),
            Self::SurroundingExecutionProgressNotEstablished(_) => {
                Some(Axis::SurroundingExecutionProgress)
            }
            Self::SurroundingExecutionNotClosed(_) => Some(Axis::SurroundingExecutionClosure),
            Self::CompleteGlobalConformanceNotClosed(_) => {
                Some(Axis::CompleteGlobalConformanceClosure)
            }
            Self::OutcomeOpen => None,
            Self::OutcomeIncomplete => None,
            Self::OutcomeUnsupported => None,
            Self::OutcomeFailed => None,
            Self::OutcomeExhausted => None,
        }
    }
}

// ---------------------------------------------------------------------------
// Outcomes — five independent non-success states
// ---------------------------------------------------------------------------

/// The reported assessment outcome.
///
/// `Open`, `Incomplete`, `Unsupported`, `Failed` and `Exhausted` are five
/// independent non-success states. None of them can be promoted to
/// [`AssessmentOutcome::CompleteGlobalSuccess`]: the only way to obtain a
/// [`CompleteGlobalConformance`] witness is
/// [`ProtocolAssessment::complete_global_conformance`], which requires the
/// outcome to already be `CompleteGlobalSuccess` **and** all five axes to
/// admit.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum AssessmentOutcome {
    /// Complete global success was reported.
    CompleteGlobalSuccess,
    /// The assessment is still open.
    Open,
    /// The assessment is incomplete.
    Incomplete,
    /// The assessment requested something this evaluator does not support.
    Unsupported,
    /// The assessment failed.
    Failed,
    /// The assessment exhausted its declared budget.
    Exhausted,
}

impl AssessmentOutcome {
    /// Every outcome, in declared order.
    pub const ALL: [Self; 6] = [
        Self::CompleteGlobalSuccess,
        Self::Open,
        Self::Incomplete,
        Self::Unsupported,
        Self::Failed,
        Self::Exhausted,
    ];

    /// The five non-success outcomes, in declared order.
    pub const NON_SUCCESS: [Self; 5] = [
        Self::Open,
        Self::Incomplete,
        Self::Unsupported,
        Self::Failed,
        Self::Exhausted,
    ];

    /// The refusal this outcome raises, or `None` for complete global success.
    pub fn refusal(self) -> Option<ClosureRefusal> {
        match self {
            Self::CompleteGlobalSuccess => None,
            Self::Open => Some(ClosureRefusal::OutcomeOpen),
            Self::Incomplete => Some(ClosureRefusal::OutcomeIncomplete),
            Self::Unsupported => Some(ClosureRefusal::OutcomeUnsupported),
            Self::Failed => Some(ClosureRefusal::OutcomeFailed),
            Self::Exhausted => Some(ClosureRefusal::OutcomeExhausted),
        }
    }
}

// ---------------------------------------------------------------------------
// The product lattice
// ---------------------------------------------------------------------------

/// The five axes observed together: the product of the five axis lattices.
///
/// Order, [`ClosureState::meet`] and [`ClosureState::join`] are componentwise,
/// so an operation applied to one axis is invisible to the other four by
/// construction, not by convention.
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
    /// Axis 5.
    pub complete_global_conformance_closure: GlobalConformanceClosure,
}

impl ClosureState {
    /// The bottom of the product lattice: nothing observed on any axis.
    pub const BOTTOM: Self = Self {
        decision_scope_progress: DecisionScopeProgress::Unobserved,
        decision_scope_closure: DecisionScopeClosure::Unobserved,
        surrounding_execution_progress: SurroundingExecutionProgress::Unobserved,
        surrounding_execution_closure: SurroundingExecutionClosure::Unobserved,
        complete_global_conformance_closure: GlobalConformanceClosure::Unobserved,
    };

    /// The top of the product lattice: every axis contradicted.
    pub const TOP: Self = Self {
        decision_scope_progress: DecisionScopeProgress::Contradicted,
        decision_scope_closure: DecisionScopeClosure::Contradicted,
        surrounding_execution_progress: SurroundingExecutionProgress::Contradicted,
        surrounding_execution_closure: SurroundingExecutionClosure::Contradicted,
        complete_global_conformance_closure: GlobalConformanceClosure::Contradicted,
    };

    /// Componentwise greatest lower bound.
    pub fn meet(self, other: Self) -> Self {
        Self {
            decision_scope_progress: self
                .decision_scope_progress
                .meet(other.decision_scope_progress),
            decision_scope_closure: self
                .decision_scope_closure
                .meet(other.decision_scope_closure),
            surrounding_execution_progress: self
                .surrounding_execution_progress
                .meet(other.surrounding_execution_progress),
            surrounding_execution_closure: self
                .surrounding_execution_closure
                .meet(other.surrounding_execution_closure),
            complete_global_conformance_closure: self
                .complete_global_conformance_closure
                .meet(other.complete_global_conformance_closure),
        }
    }

    /// Componentwise least upper bound.
    pub fn join(self, other: Self) -> Self {
        Self {
            decision_scope_progress: self
                .decision_scope_progress
                .join(other.decision_scope_progress),
            decision_scope_closure: self
                .decision_scope_closure
                .join(other.decision_scope_closure),
            surrounding_execution_progress: self
                .surrounding_execution_progress
                .join(other.surrounding_execution_progress),
            surrounding_execution_closure: self
                .surrounding_execution_closure
                .join(other.surrounding_execution_closure),
            complete_global_conformance_closure: self
                .complete_global_conformance_closure
                .join(other.complete_global_conformance_closure),
        }
    }

    /// The product order: `self` is at or below `other` on every axis.
    pub fn is_at_or_below(self, other: Self) -> bool {
        self.meet(other) == self
    }

    /// The refusal raised by one named axis, or `None` if that axis admits.
    ///
    /// This is the per-axis read that makes each axis independently
    /// assertable: it can only ever return that axis's own variant.
    pub fn refusal_for(self, axis: Axis) -> Option<ClosureRefusal> {
        match axis {
            Axis::DecisionScopeProgress => axis_refusal(self.decision_scope_progress),
            Axis::DecisionScopeClosure => axis_refusal(self.decision_scope_closure),
            Axis::SurroundingExecutionProgress => axis_refusal(self.surrounding_execution_progress),
            Axis::SurroundingExecutionClosure => axis_refusal(self.surrounding_execution_closure),
            Axis::CompleteGlobalConformanceClosure => {
                axis_refusal(self.complete_global_conformance_closure)
            }
        }
    }

    /// The stable code of one named axis's current state.
    pub fn state_code(self, axis: Axis) -> &'static str {
        match axis {
            Axis::DecisionScopeProgress => self.decision_scope_progress.code(),
            Axis::DecisionScopeClosure => self.decision_scope_closure.code(),
            Axis::SurroundingExecutionProgress => self.surrounding_execution_progress.code(),
            Axis::SurroundingExecutionClosure => self.surrounding_execution_closure.code(),
            Axis::CompleteGlobalConformanceClosure => {
                self.complete_global_conformance_closure.code()
            }
        }
    }

    /// This state with one named axis moved to the single state that admits
    /// complete global conformance on that axis, and every other axis left
    /// byte-identical.
    pub fn established(self, axis: Axis) -> Self {
        match axis {
            Axis::DecisionScopeProgress => Self {
                decision_scope_progress: DecisionScopeProgress::Advanced,
                ..self
            },
            Axis::DecisionScopeClosure => Self {
                decision_scope_closure: DecisionScopeClosure::Closed,
                ..self
            },
            Axis::SurroundingExecutionProgress => Self {
                surrounding_execution_progress: SurroundingExecutionProgress::Advanced,
                ..self
            },
            Axis::SurroundingExecutionClosure => Self {
                surrounding_execution_closure: SurroundingExecutionClosure::Closed,
                ..self
            },
            Axis::CompleteGlobalConformanceClosure => Self {
                complete_global_conformance_closure: GlobalConformanceClosure::Closed,
                ..self
            },
        }
    }

    /// Every axis refusal this state raises, in [`Axis::ALL`] order.
    ///
    /// The destructuring below has no `..` rest pattern on purpose: a sixth
    /// axis added to [`ClosureState`] fails to compile here.
    pub fn refusals(self) -> Vec<ClosureRefusal> {
        let Self {
            decision_scope_progress,
            decision_scope_closure,
            surrounding_execution_progress,
            surrounding_execution_closure,
            complete_global_conformance_closure,
        } = self;

        [
            axis_refusal(decision_scope_progress),
            axis_refusal(decision_scope_closure),
            axis_refusal(surrounding_execution_progress),
            axis_refusal(surrounding_execution_closure),
            axis_refusal(complete_global_conformance_closure),
        ]
        .into_iter()
        .flatten()
        .collect()
    }
}

/// Lift one axis state into its own refusal, or `None` when it admits.
fn axis_refusal<A: ClosureAxis>(state: A) -> Option<ClosureRefusal> {
    if state.admits_complete_global_conformance() {
        None
    } else {
        Some(state.refusal())
    }
}

// ---------------------------------------------------------------------------
// The assessment and its witness
// ---------------------------------------------------------------------------

/// Proof that complete global conformance was established.
///
/// This type has no public constructor. The only way to obtain one is
/// [`ProtocolAssessment::complete_global_conformance`], so no caller can
/// promote a non-success outcome or an unclosed axis into complete global
/// success:
///
/// ```compile_fail
/// let _ = quire_protocol::closure::CompleteGlobalConformance {};
/// ```
///
/// Nor can one axis's state be substituted for another's — the axes are
/// unrelated types:
///
/// ```compile_fail
/// use quire_protocol::closure::{DecisionScopeClosure, GlobalConformanceClosure};
/// let _: GlobalConformanceClosure = DecisionScopeClosure::Closed;
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct CompleteGlobalConformance {
    /// Private field: blocks construction outside this module.
    witness: (),
}

/// One assessment: the five closure axes plus the reported outcome.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ProtocolAssessment {
    /// The five independently observed closure axes.
    pub closure: ClosureState,
    /// The reported outcome.
    pub outcome: AssessmentOutcome,
}

impl ProtocolAssessment {
    /// Build an assessment from its closure state and outcome.
    pub fn new(closure: ClosureState, outcome: AssessmentOutcome) -> Self {
        Self { closure, outcome }
    }

    /// Every reason this assessment is not complete global conformance, in
    /// declared order: the five axes in [`Axis::ALL`] order, then the outcome.
    ///
    /// The order is declared, never sorted, and no refusal is masked by any
    /// other — every failing axis is reported.
    pub fn refusals(self) -> Vec<ClosureRefusal> {
        let mut refusals = self.closure.refusals();
        refusals.extend(self.outcome.refusal());
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
