// SPDX-License-Identifier: AGPL-3.0-or-later
//! TC-004 (partial, chunk C-15): the five closure axes and the execution state.
//!
//! Covers FR-004-AC-3 in full. The remaining TC-004 criteria (FR-004-AC-1,
//! AC-2, AC-4) need the batch and incremental evaluators and are owned by
//! chunks C-20 and C-21.
//!
//! # Every vocabulary asserted here is an owner's
//!
//! The four scope axes take `open` or `closed` from
//! `FR-284`; the fifth takes one of five
//! members from `FR-247`; the execution state takes one of five from `FR-241`.
//! The assertions below compare against those published spellings, never
//! against another local label — an assertion that compares two of this crate's
//! own labels to each other cannot fail.
//!
//! # Why exhaustive enumeration and not random property sampling
//!
//! The state space is finite and small: 2 x 2 x 2 x 2 x 5 = 80 closure states
//! and 5 execution states. Enumerating it is a *proof over the whole domain*;
//! sampling it would only be evidence. `Cargo.toml` is COLLISION-7 in the chunk
//! plan, so no `proptest` harness may be introduced here in any case.
//!
//! # What this file no longer tests, and why
//!
//! An earlier revision enumerated lattice laws (associativity, commutativity,
//! idempotence, absorption) over per-axis and product `meet`/`join`. Those
//! operations are gone. `FR-284` gives the labels no causal order and no
//! progression from one to another, and a later contradiction is reported by
//! supersession rather than by a closure label. The lattice combined two
//! disagreeing facts into a third value that no vocabulary publishes.
//! Supersession is chunk C-26's obligation.

use std::collections::HashSet;

use ix_trace_rs::trace;
use quire_protocol::closure::{
    AssessmentExecution, Axis, ClosureRefusal, ClosureState, DecisionScopeClosure,
    DecisionScopeProgress, GlobalConformanceClosure, ProtocolAssessment, ScopeAxis, ScopeState,
    SurroundingExecutionClosure, SurroundingExecutionProgress,
};

/// The one closure state in which all five axes admit complete global
/// conformance. Every single-axis test perturbs exactly one field of this.
fn all_axes_admitting() -> ClosureState {
    state(
        ScopeState::Closed,
        ScopeState::Closed,
        ScopeState::Closed,
        ScopeState::Closed,
        GlobalConformanceClosure::Closed,
    )
}

/// Build a complete state; every axis present, so this never refuses.
fn state(
    one: ScopeState,
    two: ScopeState,
    three: ScopeState,
    four: ScopeState,
    five: GlobalConformanceClosure,
) -> ClosureState {
    ClosureState::new(
        Some(DecisionScopeProgress::of(one)),
        Some(DecisionScopeClosure::of(two)),
        Some(SurroundingExecutionProgress::of(three)),
        Some(SurroundingExecutionClosure::of(four)),
        Some(five),
    )
    .expect("every axis supplied")
}

/// Every closure state: the full product of the five axis value sets.
fn all_states() -> Vec<ClosureState> {
    let mut states = Vec::new();
    for one in ScopeState::ALL {
        for two in ScopeState::ALL {
            for three in ScopeState::ALL {
                for four in ScopeState::ALL {
                    for five in GlobalConformanceClosure::ALL {
                        states.push(state(one, two, three, four, five));
                    }
                }
            }
        }
    }
    states
}

// ---------------------------------------------------------------------------
// The adopted vocabularies
// ---------------------------------------------------------------------------

/// The published label strings, asserted literally.
///
/// Tracing: TC-004, FR-004-AC-3.
#[test]
#[trace("TC-004", "FR-004-AC-3")]
fn tc_004_axis_labels_are_the_published_spellings() {
    // FR-284 publishes exactly these two for all four scope axes.
    assert_eq!(ScopeState::Open.label(), "open");
    assert_eq!(ScopeState::Closed.label(), "closed");
    assert_eq!(ScopeState::ALL.len(), 2);

    // FR-247 publishes exactly these five for the fifth axis.
    assert_eq!(
        GlobalConformanceClosure::ALL.map(GlobalConformanceClosure::label),
        [
            "closed",
            "open",
            "incomplete",
            "contradicted",
            "not-required"
        ]
    );

    // FR-241 publishes exactly these five for the execution state.
    assert_eq!(
        AssessmentExecution::ALL.map(AssessmentExecution::label),
        [
            "completed",
            "refused",
            "unsupported",
            "resource-incomplete",
            "failed"
        ]
    );
}

/// `incomplete`, `contradicted` and `not-required` belong to the fifth axis
/// alone; the labels named by no requirement are absent everywhere.
///
/// FR-284-AC-3 requires exactly this separation.
///
/// Tracing: TC-004, FR-004-AC-3.
#[test]
#[trace("TC-004", "FR-028-AC-4")]
fn tc_004_the_fifth_axis_members_are_not_scope_axis_members() {
    let scope: HashSet<&str> = ScopeState::ALL.iter().map(|s| s.label()).collect();
    for extra in ["incomplete", "contradicted", "not-required"] {
        assert!(
            !scope.contains(extra),
            "{extra} is a fifth-axis member only (FR-247), not a scope-axis member"
        );
    }
    // The labels that belong to no adopted vocabulary.
    for withdrawn in ["not-started", "advanced", "stalled", "unobserved"] {
        assert!(
            !scope.contains(withdrawn),
            "{withdrawn} is named by no requirement and must not be reintroduced"
        );
        assert!(
            !GlobalConformanceClosure::ALL
                .iter()
                .any(|value| value.label() == withdrawn),
            "{withdrawn} is named by no requirement and must not be reintroduced"
        );
    }
}

/// Exactly one state per axis admits complete global conformance.
///
/// Tracing: TC-004, FR-004-AC-3.
#[test]
#[trace("TC-004", "FR-004-AC-3")]
fn tc_004_exactly_one_state_per_axis_admits() {
    fn check<A: ScopeAxis>() {
        let admitting: Vec<A> = A::values()
            .into_iter()
            .filter(|value| value.admits_complete_global_conformance())
            .collect();
        assert_eq!(
            admitting.len(),
            1,
            "{}: exactly one state may admit, found {admitting:?}",
            A::AXIS.code()
        );
        assert_eq!(admitting[0].state(), ScopeState::Closed);
    }
    check::<DecisionScopeProgress>();
    check::<DecisionScopeClosure>();
    check::<SurroundingExecutionProgress>();
    check::<SurroundingExecutionClosure>();

    let admitting: Vec<GlobalConformanceClosure> = GlobalConformanceClosure::ALL
        .into_iter()
        .filter(|value| value.admits_complete_global_conformance())
        .collect();
    assert_eq!(admitting, vec![GlobalConformanceClosure::Closed]);
}

/// `not-required` on the fifth axis does not establish the claim.
///
/// Where no global-conformance claim applies there is no claim to establish,
/// and FR-059 refuses substituting any other fact for a missing or non-closed
/// global premise.
///
/// Tracing: TC-004, FR-004-AC-3.
#[test]
#[trace("TC-004", "FR-028-AC-4")]
fn tc_004_not_required_does_not_establish_global_conformance() {
    assert!(!GlobalConformanceClosure::NotRequired.admits_complete_global_conformance());
    let mut closure = all_axes_admitting();
    closure.complete_global_conformance_closure = GlobalConformanceClosure::NotRequired;
    let refusals = ProtocolAssessment::new(closure, AssessmentExecution::Completed)
        .complete_global_conformance()
        .expect_err("not-required does not establish the claim");
    assert_eq!(
        refusals,
        vec![ClosureRefusal::CompleteGlobalConformanceNotClosed(
            GlobalConformanceClosure::NotRequired
        )]
    );
}

// ---------------------------------------------------------------------------
// Independence
// ---------------------------------------------------------------------------

/// Every axis takes every value independently of the other four.
///
/// Tracing: TC-004, FR-004-AC-3.
#[test]
#[trace("TC-004", "FR-004-AC-3", "FR-028-AC-1")]
fn tc_004_axes_are_independently_settable() {
    let states = all_states();
    assert_eq!(states.len(), 80, "2 x 2 x 2 x 2 x 5");
    let distinct: HashSet<&ClosureState> = states.iter().collect();
    assert_eq!(distinct.len(), states.len(), "a state repeats");

    // Each axis takes each of its values somewhere in the product.
    for value in ScopeState::ALL {
        assert!(states
            .iter()
            .any(|state| state.decision_scope_progress.state() == value));
        assert!(states
            .iter()
            .any(|state| state.decision_scope_closure.state() == value));
        assert!(states
            .iter()
            .any(|state| state.surrounding_execution_progress.state() == value));
        assert!(states
            .iter()
            .any(|state| state.surrounding_execution_closure.state() == value));
    }
    for value in GlobalConformanceClosure::ALL {
        assert!(states
            .iter()
            .any(|state| state.complete_global_conformance_closure == value));
    }
}

/// Changing one axis changes no other axis's established status.
///
/// Under `ix://agent-ix/quire-specification/FR-061` no completeness,
/// assessment-execution or other progress/closure fact may substitute for a
/// closure axis.
///
/// Tracing: TC-004, FR-004-AC-3.
#[test]
#[trace("TC-004", "FR-004-AC-3", "FR-028-AC-1", "FR-028-AC-3")]
fn tc_004_changing_one_axis_changes_no_other() {
    let base = all_axes_admitting();
    for axis in Axis::ALL {
        let mut perturbed = base;
        match axis {
            Axis::DecisionScopeProgress => {
                perturbed.decision_scope_progress = DecisionScopeProgress::of(ScopeState::Open);
            }
            Axis::DecisionScopeClosure => {
                perturbed.decision_scope_closure = DecisionScopeClosure::of(ScopeState::Open);
            }
            Axis::SurroundingExecutionProgress => {
                perturbed.surrounding_execution_progress =
                    SurroundingExecutionProgress::of(ScopeState::Open);
            }
            Axis::SurroundingExecutionClosure => {
                perturbed.surrounding_execution_closure =
                    SurroundingExecutionClosure::of(ScopeState::Open);
            }
            Axis::CompleteGlobalConformanceClosure => {
                perturbed.complete_global_conformance_closure = GlobalConformanceClosure::Open;
            }
        }
        assert!(!perturbed.established(axis), "{}", axis.code());
        for other in Axis::ALL.into_iter().filter(|other| *other != axis) {
            assert!(
                perturbed.established(other),
                "opening {} disturbed {}",
                axis.code(),
                other.code()
            );
        }
    }
}

/// A closed scope axis does not substitute for the global closure axis.
///
/// Under `ix://agent-ix/quire-specification/FR-284`, a label equal to `closed`
/// or `open` on one of the four axes and on the fifth is one label string in
/// two distinct vocabularies.
///
/// Tracing: TC-004, FR-004-AC-3.
#[test]
#[trace("TC-004", "FR-004-AC-3", "FR-028-AC-3")]
fn tc_004_scoped_closure_cannot_substitute_for_global_closure() {
    let closure = state(
        ScopeState::Closed,
        ScopeState::Closed,
        ScopeState::Closed,
        ScopeState::Closed,
        GlobalConformanceClosure::Open,
    );
    // Four axes carry the label `closed`; the fifth still refuses.
    for axis in Axis::ALL
        .into_iter()
        .filter(|axis| *axis != Axis::CompleteGlobalConformanceClosure)
    {
        assert_eq!(closure.label(axis), "closed");
        assert!(closure.established(axis));
    }
    assert_eq!(
        closure.refusals(),
        vec![ClosureRefusal::CompleteGlobalConformanceNotClosed(
            GlobalConformanceClosure::Open
        )]
    );
}

// ---------------------------------------------------------------------------
// Absence is not a value
// ---------------------------------------------------------------------------

/// An omitted axis refuses and is never defaulted.
///
/// Under `ix://agent-ix/quire-specification/FR-284` an absent axis compares
/// only to absence and is never defaulted to `open` or to `closed`, and
/// `ix://agent-ix/quire-specification/FR-061` requires a strict reader to
/// refuse an omitted required axis.
///
/// Tracing: TC-004, FR-004-AC-3.
#[test]
#[trace("TC-004", "FR-028-AC-6")]
fn tc_004_an_omitted_axis_refuses_and_is_never_defaulted() {
    let closed = ScopeState::Closed;
    let refusals = ClosureState::new(
        None,
        Some(DecisionScopeClosure::of(closed)),
        Some(SurroundingExecutionProgress::of(closed)),
        Some(SurroundingExecutionClosure::of(closed)),
        Some(GlobalConformanceClosure::Closed),
    )
    .expect_err("an omitted axis refuses");
    assert_eq!(
        refusals,
        vec![ClosureRefusal::AxisOmitted(Axis::DecisionScopeProgress)]
    );
}

/// Every omitted axis is reported, in declared order; the first does not mask
/// the rest.
///
/// Tracing: TC-004, FR-004-AC-3.
#[test]
#[trace("TC-004", "FR-028-AC-6")]
fn tc_004_every_omitted_axis_is_reported_in_declared_order() {
    let refusals = ClosureState::new(None, None, None, None, None).expect_err("all omitted");
    assert_eq!(
        refusals,
        Axis::ALL
            .into_iter()
            .map(ClosureRefusal::AxisOmitted)
            .collect::<Vec<_>>()
    );
}

// ---------------------------------------------------------------------------
// Refusal identity
// ---------------------------------------------------------------------------

/// Each axis refuses on its own variant, carrying its own observed value.
///
/// Tracing: TC-004, FR-004-AC-3.
#[test]
#[trace("TC-004", "FR-004-AC-3", "FR-028-AC-1")]
fn tc_004_each_axis_refuses_on_its_own_variant() {
    let open = ScopeState::Open;
    let expected = [
        (
            Axis::DecisionScopeProgress,
            ClosureRefusal::DecisionScopeProgressNotClosed(DecisionScopeProgress::of(open)),
        ),
        (
            Axis::DecisionScopeClosure,
            ClosureRefusal::DecisionScopeNotClosed(DecisionScopeClosure::of(open)),
        ),
        (
            Axis::SurroundingExecutionProgress,
            ClosureRefusal::SurroundingExecutionProgressNotClosed(
                SurroundingExecutionProgress::of(open),
            ),
        ),
        (
            Axis::SurroundingExecutionClosure,
            ClosureRefusal::SurroundingExecutionNotClosed(SurroundingExecutionClosure::of(open)),
        ),
        (
            Axis::CompleteGlobalConformanceClosure,
            ClosureRefusal::CompleteGlobalConformanceNotClosed(GlobalConformanceClosure::Open),
        ),
    ];

    for (axis, refusal) in expected {
        let mut perturbed = all_axes_admitting();
        match axis {
            Axis::DecisionScopeProgress => {
                perturbed.decision_scope_progress = DecisionScopeProgress::of(open);
            }
            Axis::DecisionScopeClosure => {
                perturbed.decision_scope_closure = DecisionScopeClosure::of(open);
            }
            Axis::SurroundingExecutionProgress => {
                perturbed.surrounding_execution_progress = SurroundingExecutionProgress::of(open);
            }
            Axis::SurroundingExecutionClosure => {
                perturbed.surrounding_execution_closure = SurroundingExecutionClosure::of(open);
            }
            Axis::CompleteGlobalConformanceClosure => {
                perturbed.complete_global_conformance_closure = GlobalConformanceClosure::Open;
            }
        }
        assert_eq!(perturbed.refusals(), vec![refusal], "{}", axis.code());
        assert_eq!(refusal.axis(), Some(axis));
    }
}

/// No two refusals share a code.
///
/// One shared code across every negative row is the defect this checks for:
/// bypassing one axis must not fall through to another under the same code.
///
/// Tracing: TC-004, FR-004-AC-3.
#[test]
#[trace("TC-004", "FR-004-AC-3")]
fn tc_004_no_two_refusals_share_a_code() {
    let mut refusals: Vec<ClosureRefusal> = Axis::ALL
        .into_iter()
        .map(ClosureRefusal::AxisOmitted)
        .collect();
    let open = ScopeState::Open;
    refusals.extend([
        ClosureRefusal::DecisionScopeProgressNotClosed(DecisionScopeProgress::of(open)),
        ClosureRefusal::DecisionScopeNotClosed(DecisionScopeClosure::of(open)),
        ClosureRefusal::SurroundingExecutionProgressNotClosed(SurroundingExecutionProgress::of(
            open,
        )),
        ClosureRefusal::SurroundingExecutionNotClosed(SurroundingExecutionClosure::of(open)),
        ClosureRefusal::CompleteGlobalConformanceNotClosed(GlobalConformanceClosure::Open),
    ]);
    refusals.extend(
        AssessmentExecution::NON_COMPLETING
            .into_iter()
            .map(|execution| execution.refusal().expect("non-completing refuses")),
    );

    // The five AxisOmitted variants share one code by design; every other
    // refusal has its own. Count distinct codes over the non-omitted set.
    let codes: Vec<&str> = refusals
        .iter()
        .filter(|refusal| !matches!(refusal, ClosureRefusal::AxisOmitted(_)))
        .map(|refusal| refusal.code())
        .collect();
    let distinct: HashSet<&&str> = codes.iter().collect();
    assert_eq!(
        distinct.len(),
        codes.len(),
        "a refusal code repeats: {codes:?}"
    );
    assert_eq!(codes.len(), 9, "five axes plus four non-completing states");
}

// ---------------------------------------------------------------------------
// Execution state
// ---------------------------------------------------------------------------

/// No non-completing execution state becomes complete global success, even
/// with every axis closed.
///
/// Tracing: TC-004, FR-004-AC-3.
#[test]
#[trace("TC-004", "FR-004-AC-3", "FR-028-AC-2")]
fn tc_004_no_non_completing_execution_becomes_complete_global_success() {
    for execution in AssessmentExecution::NON_COMPLETING {
        let refusals = ProtocolAssessment::new(all_axes_admitting(), execution)
            .complete_global_conformance()
            .expect_err("a non-completing execution state never establishes the claim");
        assert_eq!(
            refusals,
            vec![execution.refusal().expect("non-completing refuses")],
            "{} refused on the wrong identity",
            execution.label()
        );
    }
}

/// Complete global success needs every axis closed *and* `completed`.
///
/// Tracing: TC-004, FR-004-AC-3.
#[test]
#[trace("TC-004", "FR-004-AC-3", "FR-028-AC-2")]
fn tc_004_complete_global_success_needs_every_axis_and_completed() {
    ProtocolAssessment::new(all_axes_admitting(), AssessmentExecution::Completed)
        .complete_global_conformance()
        .expect("every axis closed and the evaluator completed");

    for closure in all_states()
        .into_iter()
        .filter(|closure| Axis::ALL.into_iter().any(|axis| !closure.established(axis)))
    {
        assert!(
            ProtocolAssessment::new(closure, AssessmentExecution::Completed)
                .complete_global_conformance()
                .is_err(),
            "an unclosed axis established the claim: {closure:?}"
        );
    }
}

/// Refusal order is declared, never sorted: the five axes in `Axis::ALL` order,
/// then the execution state.
///
/// Tracing: TC-004, FR-004-AC-3.
#[test]
#[trace("TC-004", "FR-004-AC-3")]
fn tc_004_refusal_order_is_declared_not_sorted() {
    let open = ScopeState::Open;
    let closure = state(open, open, open, open, GlobalConformanceClosure::Open);
    let refusals = ProtocolAssessment::new(closure, AssessmentExecution::Failed).refusals();

    let axes: Vec<Axis> = refusals
        .iter()
        .filter_map(|refusal| refusal.axis())
        .collect();
    assert_eq!(axes, Axis::ALL.to_vec(), "axes out of declared order");
    assert_eq!(
        refusals.last().map(|refusal| refusal.code()),
        Some("execution.failed"),
        "the execution refusal comes last"
    );
    assert_eq!(refusals.len(), 6);

    // Sorting by code would reorder these; the declared order is not sorted.
    let codes: Vec<&str> = refusals.iter().map(|refusal| refusal.code()).collect();
    let mut sorted = codes.clone();
    sorted.sort_unstable();
    assert_ne!(codes, sorted, "declared order coincides with sorted order");
}
