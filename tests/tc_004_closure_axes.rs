// SPDX-License-Identifier: AGPL-3.0-or-later
//! TC-004 (partial, chunk C-15): the five-axis closure and outcome lattice.
//!
//! Covers FR-004-AC-3 in full. The remaining TC-004 criteria (FR-004-AC-1,
//! AC-2, AC-4) need the batch and incremental evaluators and are owned by
//! chunks C-20 and C-21.
//!
//! # Why exhaustive enumeration and not random property sampling
//!
//! Two reasons, in this order.
//!
//! 1. The axis set is finite and small: 5 x 5 x 5 x 5 x 6 = 3750 closure
//!    states and 6 outcomes. Enumerating it is a *proof over the whole
//!    domain*; sampling it would only be evidence. For lattice laws — which
//!    are universally quantified equations — total enumeration is strictly
//!    stronger than any number of random draws.
//! 2. `Cargo.toml` is COLLISION-7 in the chunk plan: exactly one chunk per
//!    wave (C-11) may add a dependency, so no `proptest` harness may be
//!    introduced here, and the repository has none today.
//!
//! Associativity is enumerated exhaustively *per axis* (every ordered triple
//! of every axis: at most 6^3 = 216 triples). Because `ClosureState::meet`
//! and `ClosureState::join` are defined componentwise, per-axis associativity
//! entails product associativity; the product-level triple sweep below is
//! corroboration over a spanning basis, not the proof.

use std::collections::HashSet;

use ix_trace_rs::trace;
use quire_protocol::closure::{
    AssessmentOutcome, Axis, AxisRank, ClosureAxis, ClosureRefusal, ClosureState,
    DecisionScopeClosure, DecisionScopeProgress, GlobalConformanceClosure, ProtocolAssessment,
    SurroundingExecutionClosure, SurroundingExecutionProgress,
};

/// The one closure state in which all five axes admit complete global
/// conformance. Every single-axis test perturbs exactly one field of this.
fn all_axes_admitting() -> ClosureState {
    let mut state = ClosureState::BOTTOM;
    for axis in Axis::ALL {
        state = state.established(axis);
    }
    state
}

/// Every closure state: the full product of the five axis value sets.
fn all_states() -> Vec<ClosureState> {
    let mut states = Vec::new();
    for &decision_scope_progress in DecisionScopeProgress::VALUES {
        for &decision_scope_closure in DecisionScopeClosure::VALUES {
            for &surrounding_execution_progress in SurroundingExecutionProgress::VALUES {
                for &surrounding_execution_closure in SurroundingExecutionClosure::VALUES {
                    for &complete_global_conformance_closure in GlobalConformanceClosure::VALUES {
                        states.push(ClosureState {
                            decision_scope_progress,
                            decision_scope_closure,
                            surrounding_execution_progress,
                            surrounding_execution_closure,
                            complete_global_conformance_closure,
                        });
                    }
                }
            }
        }
    }
    states
}

// ---------------------------------------------------------------------------
// Axis set integrity
// ---------------------------------------------------------------------------

/// Check one axis's declared value set against its own `index` match.
fn check_value_set<A: ClosureAxis>() {
    let axis = A::AXIS;
    let values = A::VALUES;

    for (position, value) in values.iter().enumerate() {
        assert_eq!(
            value.index(),
            position,
            "{}: {value:?} indexes outside its declared position",
            axis.code()
        );
    }

    let distinct: HashSet<&A> = values.iter().collect();
    assert_eq!(
        distinct.len(),
        values.len(),
        "{}: declared value set repeats a state",
        axis.code()
    );

    assert_eq!(A::BOTTOM.rank(), AxisRank::Bottom, "{}", axis.code());
    assert_eq!(A::TOP.rank(), AxisRank::Top, "{}", axis.code());

    let admitting: Vec<&A> = values
        .iter()
        .filter(|value| value.admits_complete_global_conformance())
        .collect();
    assert_eq!(
        admitting.len(),
        1,
        "{}: exactly one state may admit complete global conformance, found {admitting:?}",
        axis.code()
    );
}

/// Tracing: TC-004, FR-004-AC-3.
#[test]
#[trace("TC-004", "FR-004-AC-3")]
fn tc_004_axis_value_sets_are_exhaustive() {
    check_value_set::<DecisionScopeProgress>();
    check_value_set::<DecisionScopeClosure>();
    check_value_set::<SurroundingExecutionProgress>();
    check_value_set::<SurroundingExecutionClosure>();
    check_value_set::<GlobalConformanceClosure>();

    for (position, axis) in Axis::ALL.iter().enumerate() {
        assert_eq!(
            axis.index(),
            position,
            "{axis:?} indexes out of declared order"
        );
    }
    let axis_codes: HashSet<&'static str> = Axis::ALL.iter().map(|axis| axis.code()).collect();
    assert_eq!(axis_codes.len(), Axis::ALL.len(), "two axes share a code");
}

/// Tracing: TC-004, FR-004-AC-3.
#[test]
#[trace("TC-004", "FR-004-AC-3")]
fn tc_004_axes_are_independently_settable() {
    let states = all_states();
    let expected = DecisionScopeProgress::VALUES.len()
        * DecisionScopeClosure::VALUES.len()
        * SurroundingExecutionProgress::VALUES.len()
        * SurroundingExecutionClosure::VALUES.len()
        * GlobalConformanceClosure::VALUES.len();

    assert_eq!(states.len(), expected, "the product table is incomplete");

    let distinct: HashSet<&ClosureState> = states.iter().collect();
    assert_eq!(
        distinct.len(),
        expected,
        "two distinct axis combinations are not distinguishable"
    );

    let distinct_code_tuples: HashSet<Vec<&'static str>> = states
        .iter()
        .map(|state| {
            Axis::ALL
                .iter()
                .map(|&axis| state.state_code(axis))
                .collect()
        })
        .collect();
    assert_eq!(
        distinct_code_tuples.len(),
        expected,
        "two distinct axis combinations report the same observable codes"
    );
}

/// Tracing: TC-004, FR-004-AC-3.
#[test]
#[trace("TC-004", "FR-004-AC-3")]
fn tc_004_closing_one_axis_closes_no_other() {
    let states = all_states();
    let mut pairs_checked = 0_usize;

    for &moved in &Axis::ALL {
        for &observed in &Axis::ALL {
            if moved == observed {
                continue;
            }
            pairs_checked += 1;
            for &state in &states {
                let after = state.established(moved);
                assert_eq!(
                    after.state_code(observed),
                    state.state_code(observed),
                    "establishing {} changed the observable state of {}",
                    moved.code(),
                    observed.code()
                );
                assert_eq!(
                    after.refusal_for(observed),
                    state.refusal_for(observed),
                    "establishing {} changed the refusal of {}",
                    moved.code(),
                    observed.code()
                );
            }
        }
    }

    assert_eq!(pairs_checked, 20, "not every ordered axis pair was checked");
}

// ---------------------------------------------------------------------------
// One test per axis — each asserting only its own typed variant and code
// ---------------------------------------------------------------------------

/// Tracing: TC-004, FR-004-AC-3.
#[test]
#[trace("TC-004", "FR-004-AC-3")]
fn tc_004_decision_scope_progress_refuses_on_its_own_variant() {
    for state in [
        DecisionScopeProgress::Unobserved,
        DecisionScopeProgress::NotStarted,
        DecisionScopeProgress::Stalled,
        DecisionScopeProgress::Contradicted,
    ] {
        let closure = ClosureState {
            decision_scope_progress: state,
            ..all_axes_admitting()
        };
        let refusals =
            ProtocolAssessment::new(closure, AssessmentOutcome::CompleteGlobalSuccess).refusals();

        assert_eq!(
            refusals,
            vec![ClosureRefusal::DecisionScopeProgressNotEstablished(state)],
            "a single-axis decision-scope-progress perturbation raised another axis"
        );
        assert_eq!(
            refusals[0].code(),
            "closure.decision-scope-progress.not-established"
        );
        assert_eq!(refusals[0].axis(), Some(Axis::DecisionScopeProgress));
    }
}

/// Tracing: TC-004, FR-004-AC-3.
#[test]
#[trace("TC-004", "FR-004-AC-3")]
fn tc_004_decision_scope_closure_refuses_on_its_own_variant() {
    for state in [
        DecisionScopeClosure::Unobserved,
        DecisionScopeClosure::Open,
        DecisionScopeClosure::Incomplete,
        DecisionScopeClosure::Contradicted,
    ] {
        let closure = ClosureState {
            decision_scope_closure: state,
            ..all_axes_admitting()
        };
        let refusals =
            ProtocolAssessment::new(closure, AssessmentOutcome::CompleteGlobalSuccess).refusals();

        assert_eq!(
            refusals,
            vec![ClosureRefusal::DecisionScopeNotClosed(state)],
            "a single-axis decision-scope-closure perturbation raised another axis"
        );
        assert_eq!(refusals[0].code(), "closure.decision-scope.not-closed");
        assert_eq!(refusals[0].axis(), Some(Axis::DecisionScopeClosure));
    }
}

/// Tracing: TC-004, FR-004-AC-3.
#[test]
#[trace("TC-004", "FR-004-AC-3")]
fn tc_004_surrounding_execution_progress_refuses_on_its_own_variant() {
    for state in [
        SurroundingExecutionProgress::Unobserved,
        SurroundingExecutionProgress::NotStarted,
        SurroundingExecutionProgress::Stalled,
        SurroundingExecutionProgress::Contradicted,
    ] {
        let closure = ClosureState {
            surrounding_execution_progress: state,
            ..all_axes_admitting()
        };
        let refusals =
            ProtocolAssessment::new(closure, AssessmentOutcome::CompleteGlobalSuccess).refusals();

        assert_eq!(
            refusals,
            vec![ClosureRefusal::SurroundingExecutionProgressNotEstablished(
                state
            )],
            "a single-axis surrounding-execution-progress perturbation raised another axis"
        );
        assert_eq!(
            refusals[0].code(),
            "closure.surrounding-execution-progress.not-established"
        );
        assert_eq!(refusals[0].axis(), Some(Axis::SurroundingExecutionProgress));
    }
}

/// Tracing: TC-004, FR-004-AC-3.
#[test]
#[trace("TC-004", "FR-004-AC-3")]
fn tc_004_surrounding_execution_closure_refuses_on_its_own_variant() {
    for state in [
        SurroundingExecutionClosure::Unobserved,
        SurroundingExecutionClosure::Open,
        SurroundingExecutionClosure::Incomplete,
        SurroundingExecutionClosure::Contradicted,
    ] {
        let closure = ClosureState {
            surrounding_execution_closure: state,
            ..all_axes_admitting()
        };
        let refusals =
            ProtocolAssessment::new(closure, AssessmentOutcome::CompleteGlobalSuccess).refusals();

        assert_eq!(
            refusals,
            vec![ClosureRefusal::SurroundingExecutionNotClosed(state)],
            "a single-axis surrounding-execution-closure perturbation raised another axis"
        );
        assert_eq!(
            refusals[0].code(),
            "closure.surrounding-execution.not-closed"
        );
        assert_eq!(refusals[0].axis(), Some(Axis::SurroundingExecutionClosure));
    }
}

/// Tracing: TC-004, FR-004-AC-3.
#[test]
#[trace("TC-004", "FR-004-AC-3")]
fn tc_004_complete_global_conformance_closure_refuses_on_its_own_variant() {
    for state in [
        GlobalConformanceClosure::Unobserved,
        GlobalConformanceClosure::NotRequired,
        GlobalConformanceClosure::Open,
        GlobalConformanceClosure::Incomplete,
        GlobalConformanceClosure::Contradicted,
    ] {
        let closure = ClosureState {
            complete_global_conformance_closure: state,
            ..all_axes_admitting()
        };
        let refusals =
            ProtocolAssessment::new(closure, AssessmentOutcome::CompleteGlobalSuccess).refusals();

        assert_eq!(
            refusals,
            vec![ClosureRefusal::CompleteGlobalConformanceNotClosed(state)],
            "a single-axis global-closure perturbation raised another axis"
        );
        assert_eq!(
            refusals[0].code(),
            "closure.complete-global-conformance.not-closed"
        );
        assert_eq!(
            refusals[0].axis(),
            Some(Axis::CompleteGlobalConformanceClosure)
        );
    }
}

/// Tracing: TC-004, FR-004-AC-3.
#[test]
#[trace("TC-004", "FR-004-AC-3")]
fn tc_004_no_two_refusals_share_a_code() {
    let mut refusals = Vec::new();
    for &state in DecisionScopeProgress::VALUES {
        refusals.push(state.refusal());
    }
    for &state in DecisionScopeClosure::VALUES {
        refusals.push(state.refusal());
    }
    for &state in SurroundingExecutionProgress::VALUES {
        refusals.push(state.refusal());
    }
    for &state in SurroundingExecutionClosure::VALUES {
        refusals.push(state.refusal());
    }
    for &state in GlobalConformanceClosure::VALUES {
        refusals.push(state.refusal());
    }
    for outcome in AssessmentOutcome::NON_SUCCESS {
        refusals.push(outcome.refusal().expect("a non-success outcome refuses"));
    }

    let codes: HashSet<&'static str> = refusals.iter().map(|refusal| refusal.code()).collect();
    assert_eq!(
        codes.len(),
        10,
        "expected ten distinct refusal codes, got {codes:?}"
    );

    // Every axis code is distinct from every other axis code, so no two axes
    // can be confused by matching on the code alone.
    let axis_codes: HashSet<&'static str> = refusals
        .iter()
        .filter(|refusal| refusal.axis().is_some())
        .map(|refusal| refusal.code())
        .collect();
    assert_eq!(axis_codes.len(), Axis::ALL.len());
}

/// Tracing: TC-004, FR-004-AC-3.
#[test]
#[trace("TC-004", "FR-004-AC-3")]
fn tc_004_scoped_closure_cannot_substitute_for_global_closure() {
    for scoped in [
        Axis::DecisionScopeProgress,
        Axis::DecisionScopeClosure,
        Axis::SurroundingExecutionProgress,
        Axis::SurroundingExecutionClosure,
    ] {
        for &global in GlobalConformanceClosure::VALUES {
            if global.admits_complete_global_conformance() {
                continue;
            }
            let closure = ClosureState {
                complete_global_conformance_closure: global,
                ..all_axes_admitting()
            }
            .established(scoped);

            let assessment =
                ProtocolAssessment::new(closure, AssessmentOutcome::CompleteGlobalSuccess);
            let refusals = assessment
                .complete_global_conformance()
                .expect_err("an unclosed global axis must refuse");

            assert_eq!(
                refusals,
                vec![ClosureRefusal::CompleteGlobalConformanceNotClosed(global)],
                "establishing {} substituted for global closure",
                scoped.code()
            );
        }
    }
}

// ---------------------------------------------------------------------------
// One test per non-success outcome
// ---------------------------------------------------------------------------

/// Assert one non-success outcome refuses on its own code, from a closure
/// state in which all five axes admit, so the refusal can only be the
/// outcome's own.
fn check_non_success(outcome: AssessmentOutcome, expected: ClosureRefusal, code: &'static str) {
    let assessment = ProtocolAssessment::new(all_axes_admitting(), outcome);
    let refusals = assessment
        .complete_global_conformance()
        .expect_err("a non-success outcome must never establish complete global conformance");

    assert_eq!(refusals, vec![expected]);
    assert_eq!(refusals[0].code(), code);
    assert_eq!(
        refusals[0].axis(),
        None,
        "an outcome refusal is not an axis"
    );
}

/// Tracing: TC-004, FR-004-AC-3.
#[test]
#[trace("TC-004", "FR-004-AC-3")]
fn tc_004_open_outcome_never_becomes_complete_global_success() {
    check_non_success(
        AssessmentOutcome::Open,
        ClosureRefusal::OutcomeOpen,
        "outcome.open",
    );
}

/// Tracing: TC-004, FR-004-AC-3.
#[test]
#[trace("TC-004", "FR-004-AC-3")]
fn tc_004_incomplete_outcome_never_becomes_complete_global_success() {
    check_non_success(
        AssessmentOutcome::Incomplete,
        ClosureRefusal::OutcomeIncomplete,
        "outcome.incomplete",
    );
}

/// Tracing: TC-004, FR-004-AC-3.
#[test]
#[trace("TC-004", "FR-004-AC-3")]
fn tc_004_unsupported_outcome_never_becomes_complete_global_success() {
    check_non_success(
        AssessmentOutcome::Unsupported,
        ClosureRefusal::OutcomeUnsupported,
        "outcome.unsupported",
    );
}

/// Tracing: TC-004, FR-004-AC-3.
#[test]
#[trace("TC-004", "FR-004-AC-3")]
fn tc_004_failed_outcome_never_becomes_complete_global_success() {
    check_non_success(
        AssessmentOutcome::Failed,
        ClosureRefusal::OutcomeFailed,
        "outcome.failed",
    );
}

/// Tracing: TC-004, FR-004-AC-3.
#[test]
#[trace("TC-004", "FR-004-AC-3")]
fn tc_004_exhausted_outcome_never_becomes_complete_global_success() {
    check_non_success(
        AssessmentOutcome::Exhausted,
        ClosureRefusal::OutcomeExhausted,
        "outcome.exhausted",
    );
}

/// Tracing: TC-004, FR-004-AC-3.
#[test]
#[trace("TC-004", "FR-004-AC-3")]
fn tc_004_complete_global_success_needs_every_axis_and_the_success_outcome() {
    let admitting = all_axes_admitting();

    ProtocolAssessment::new(admitting, AssessmentOutcome::CompleteGlobalSuccess)
        .complete_global_conformance()
        .expect("all five axes admitting plus a success outcome establishes conformance");

    // Exhaustive negative sweep: no other closure state, under any outcome,
    // establishes complete global conformance.
    let mut established = 0_usize;
    for state in all_states() {
        for outcome in AssessmentOutcome::ALL {
            let assessment = ProtocolAssessment::new(state, outcome);
            if assessment.complete_global_conformance().is_ok() {
                established += 1;
                assert_eq!(
                    state, admitting,
                    "a non-admitting state established conformance"
                );
                assert_eq!(outcome, AssessmentOutcome::CompleteGlobalSuccess);
            }
        }
    }
    assert_eq!(
        established, 1,
        "exactly one of the 22500 (state, outcome) rows may establish conformance"
    );
}

/// Tracing: TC-004, FR-004-AC-3.
#[test]
#[trace("TC-004", "FR-004-AC-3")]
fn tc_004_refusal_order_is_declared_not_sorted() {
    // Every axis failing at once, plus a failing outcome: the order is
    // Axis::ALL order followed by the outcome, never a sorted or
    // severity-ranked order.
    let assessment = ProtocolAssessment::new(ClosureState::TOP, AssessmentOutcome::Exhausted);
    let refusals = assessment.refusals();

    let axes: Vec<Option<Axis>> = refusals.iter().map(|refusal| refusal.axis()).collect();
    let expected: Vec<Option<Axis>> = Axis::ALL
        .iter()
        .map(|&axis| Some(axis))
        .chain(std::iter::once(None))
        .collect();
    assert_eq!(axes, expected, "refusals are not in declared order");
    assert_eq!(refusals.len(), 6, "a failing axis was masked by another");
}

// ---------------------------------------------------------------------------
// The algebra
// ---------------------------------------------------------------------------

/// Exhaustively check the bounded-lattice laws on one axis: every ordered pair
/// and every ordered triple of that axis's full value set.
fn check_axis_lattice_laws<A: ClosureAxis>() {
    let axis = A::AXIS;
    let values = A::VALUES;

    for &x in values {
        assert_eq!(x.meet(x), x, "{}: meet is not idempotent", axis.code());
        assert_eq!(x.join(x), x, "{}: join is not idempotent", axis.code());
        assert_eq!(
            x.join(A::BOTTOM),
            x,
            "{}: bottom is not a join unit",
            axis.code()
        );
        assert_eq!(x.meet(A::TOP), x, "{}: top is not a meet unit", axis.code());
        assert_eq!(
            x.meet(A::BOTTOM),
            A::BOTTOM,
            "{}: bottom absorbs meet",
            axis.code()
        );
        assert_eq!(x.join(A::TOP), A::TOP, "{}: top absorbs join", axis.code());
        assert!(
            x.is_at_or_below(x),
            "{}: order is not reflexive",
            axis.code()
        );
    }

    for &x in values {
        for &y in values {
            assert_eq!(
                x.meet(y),
                y.meet(x),
                "{}: meet is not commutative",
                axis.code()
            );
            assert_eq!(
                x.join(y),
                y.join(x),
                "{}: join is not commutative",
                axis.code()
            );
            assert_eq!(
                x.meet(x.join(y)),
                x,
                "{}: meet/join absorption fails",
                axis.code()
            );
            assert_eq!(
                x.join(x.meet(y)),
                x,
                "{}: join/meet absorption fails",
                axis.code()
            );
            assert_eq!(
                x.is_at_or_below(y),
                x.join(y) == y,
                "{}: order disagrees with join",
                axis.code()
            );
            if x.is_at_or_below(y) && y.is_at_or_below(x) {
                assert_eq!(x, y, "{}: order is not antisymmetric", axis.code());
            }
        }
    }

    for &x in values {
        for &y in values {
            for &z in values {
                assert_eq!(
                    x.meet(y).meet(z),
                    x.meet(y.meet(z)),
                    "{}: meet is not associative",
                    axis.code()
                );
                assert_eq!(
                    x.join(y).join(z),
                    x.join(y.join(z)),
                    "{}: join is not associative",
                    axis.code()
                );
                if x.is_at_or_below(y) && y.is_at_or_below(z) {
                    assert!(
                        x.is_at_or_below(z),
                        "{}: order is not transitive",
                        axis.code()
                    );
                }
            }
        }
    }
}

/// Tracing: TC-004, FR-004-AC-3.
#[test]
#[trace("TC-004", "FR-004-AC-3")]
fn tc_004_axis_lattice_laws_hold_over_every_pair_and_triple() {
    check_axis_lattice_laws::<DecisionScopeProgress>();
    check_axis_lattice_laws::<DecisionScopeClosure>();
    check_axis_lattice_laws::<SurroundingExecutionProgress>();
    check_axis_lattice_laws::<SurroundingExecutionClosure>();
    check_axis_lattice_laws::<GlobalConformanceClosure>();
}

/// Tracing: TC-004, FR-004-AC-3.
#[test]
#[trace("TC-004", "FR-004-AC-3")]
fn tc_004_product_lattice_laws_hold_over_every_state_pair() {
    let states = all_states();

    for &x in &states {
        assert_eq!(x.meet(x), x, "product meet is not idempotent");
        assert_eq!(x.join(x), x, "product join is not idempotent");
        assert_eq!(x.join(ClosureState::BOTTOM), x, "bottom is not a join unit");
        assert_eq!(x.meet(ClosureState::TOP), x, "top is not a meet unit");
        assert!(x.is_at_or_below(x), "product order is not reflexive");
    }

    for &x in &states {
        for &y in &states {
            assert_eq!(x.meet(y), y.meet(x), "product meet is not commutative");
            assert_eq!(x.join(y), y.join(x), "product join is not commutative");
            assert_eq!(x.meet(x.join(y)), x, "product meet/join absorption fails");
            assert_eq!(x.join(x.meet(y)), x, "product join/meet absorption fails");
            assert_eq!(
                x.is_at_or_below(y),
                x.join(y) == y,
                "product order disagrees with join"
            );
            if x.is_at_or_below(y) && y.is_at_or_below(x) {
                assert_eq!(x, y, "product order is not antisymmetric");
            }
        }
    }
}

/// Tracing: TC-004, FR-004-AC-3.
#[test]
#[trace("TC-004", "FR-004-AC-3")]
fn tc_004_product_associativity_holds_over_a_spanning_triple_sweep() {
    // Associativity is proved exhaustively per axis in
    // `tc_004_axis_lattice_laws_hold_over_every_pair_and_triple`; because the
    // product operations are componentwise, that entails product
    // associativity. This sweep corroborates it over a spanning basis: bottom,
    // top, and every single-axis atom, against every one of the 3750 states.
    let mut basis = vec![ClosureState::BOTTOM, ClosureState::TOP];
    for &value in DecisionScopeProgress::VALUES {
        basis.push(ClosureState {
            decision_scope_progress: value,
            ..ClosureState::BOTTOM
        });
    }
    for &value in DecisionScopeClosure::VALUES {
        basis.push(ClosureState {
            decision_scope_closure: value,
            ..ClosureState::BOTTOM
        });
    }
    for &value in SurroundingExecutionProgress::VALUES {
        basis.push(ClosureState {
            surrounding_execution_progress: value,
            ..ClosureState::BOTTOM
        });
    }
    for &value in SurroundingExecutionClosure::VALUES {
        basis.push(ClosureState {
            surrounding_execution_closure: value,
            ..ClosureState::BOTTOM
        });
    }
    for &value in GlobalConformanceClosure::VALUES {
        basis.push(ClosureState {
            complete_global_conformance_closure: value,
            ..ClosureState::BOTTOM
        });
    }

    let states = all_states();
    for &x in &states {
        for &y in &basis {
            for &z in &basis {
                assert_eq!(
                    x.meet(y).meet(z),
                    x.meet(y.meet(z)),
                    "product meet is not associative"
                );
                assert_eq!(
                    x.join(y).join(z),
                    x.join(y.join(z)),
                    "product join is not associative"
                );
            }
        }
    }
}
