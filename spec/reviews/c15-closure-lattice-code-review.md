---
id: SR-024
title: "Code review of the five-axis closure and outcome lattice"
type: SpecReview
analysis: code-review
scope: "src/closure.rs, tests/tc_004_closure_axes.rs, spec/tests.md (FR-004 and TC-004 rows)"
review_set: subset
evaluated_revision: "48db0398d60523f15eb97978d919673cf27cde40"
review_date: "2026-09-11"
relationships:
  - target: ix://agent-ix/quire-protocol/FR-004
    type: reviews
  - target: ix://agent-ix/quire-protocol/TC-004
    type: reviews
---
## Summary

Reviewed chunk C-15: the B-owned five-axis closure and outcome lattice in
`src/closure.rs` and its exhaustive TC-004 evidence in
`tests/tc_004_closure_axes.rs`. The change satisfies FR-004-AC-3 in full — the
five axes are separate types, each raises its own refusal variant and its own
stable code, no axis is computed from another, and no non-success outcome can
reach complete global conformance. The lattice algebra is proved by total
enumeration rather than sampling. Three low findings remain, all of them named
follow-ups rather than defects in the landed behaviour.

## Verdict

**CONDITIONAL** — no high or medium findings; three low findings carry forward
to named downstream chunks.

## Assurance Context

- **Profile:** AP-001 Protocol implementation assurance profile
  (`spec/assurance/AP-001-protocol-implementation.md`, `profile_version: 0.2`,
  `status: proposed`). It applies: `review_policy.mode` is `require` and
  `code-review` is a listed operation.
- **Applicable impact assessment:** `impact-false-conformance` (severity
  critical, `control_ref` FR-004) — "an invalid, open, incomplete, failed, or
  exhausted protocol assessment is reported as complete conformance". This is
  exactly the behaviour C-15 implements and gates.
- **Evaluated revision:** `48db0398d60523f15eb97978d919673cf27cde40` plus the uncommitted C-15 working tree
  on branch `agent-b/c15-closure-lattice`, based on `agent-b/c00-lib-facade`.
- **Context examined:** FR-004, FR-006 (closure-axis behaviour only), TC-004,
  TM-001, and the AP-001 impact assessments.
- **Unavailable context:** no architecture selection record, no measurement
  evidence and no independence evidence exist for this repository yet; the
  producer-reliance record for observation and temporal facts is unavailable
  because epic gate 4 is open. No structural-gap analysis is available for
  `src/closure.rs` because the modules that consume it (C-20, C-26) do not
  exist yet.
- **Active exceptions:** none.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-001 | low | `ClosureAxis::VALUES` is hand-declared and stable Rust cannot compile-assert that it lists every variant, so a state added to an axis and deliberately omitted from `VALUES` would be classified by the four exhaustive matches but never enumerated by a test. Mitigation in place: adding a state produces four compile errors (`index`, `rank`, `code`, `admits_complete_global_conformance`), verified by probe. Closing it needs a derive macro, which needs a `Cargo.toml` dependency, which COLLISION-7 serialises to one chunk per wave. | src/closure.rs:77 | correct-requirement-no-evidence |
| FND-002 | low | `ProtocolAssessment::complete_global_conformance` refuses with `Vec<ClosureRefusal>` rather than the B-owned `ProtocolRefusal` envelope, because chunk C-01 has not landed the envelope yet. The refusal codes are already stable and per-axis distinct, so the later mapping is additive. | src/closure.rs:1046 | correct-requirement-no-evidence |
| FND-003 | low | `GlobalConformanceClosure::NotRequired` does not admit complete global conformance, which is correct — a claim that did not require global closure never established it — but a consumer could misread `closure.complete-global-conformance.not-closed` on a non-global claim as a defect. C-26/C-27 must branch on the claim kind before treating that refusal as adverse. | src/closure.rs:497 | correct-requirement-no-evidence |

## Checklist results

| Check | Result |
| --- | --- |
| Repo idioms | `#![forbid(unsafe_code)]` unchanged; `clippy::all = "deny"` honoured; `///` on every public item and a `//!` module header naming what the module owns. No new `#[allow]`. |
| Variant set is the API | Ten refusal conditions, ten typed variants, ten distinct stable codes. No variant carries a human-readable string as its discriminant. Proved by `tc_004_no_two_refusals_share_a_code`. |
| Exhaustiveness | No `match` in `src/closure.rs` has a `_` arm; the module denies `clippy::wildcard_enum_match_arm`. Probed empirically: adding one axis state produced 4 `E0004` errors; adding a sixth axis produced 10 errors including `E0027` on the `refusals()` destructure. |
| Independent assertability | One `#[test]` per axis, each asserting its own variant, its own code, its own `Axis`, and `refusals == vec![one]` — so a perturbation that fell through to another axis fails the test. Five more tests, one per non-success outcome. |
| Tautology check | `tc_004_axis_value_sets_are_exhaustive` compares `VALUES` order against the independently hand-written `index()` match; `tc_004_axes_are_independently_settable` compares the product size against the observable code tuples, which fails if two axes share a code namespace. Neither asserts a value it set itself. |
| Trace tags | Every test carries a `/// Tracing:` doc line and a bare `#[trace("TC-004", "FR-004-AC-3")]` binding, imported unqualified. No test claims FR-006-AC-3 or an IT-002 scenario, because C-15 proves only the closure-axis fraction of those. |
| Panic surface | `expect` appears only in tests. The library has no `unwrap`, no indexing, no arithmetic, no I/O, no allocation beyond one bounded `Vec` of at most six refusals. |
| Boundary (G-BOUNDARY) | No grammar, parser, wire decoder, canonicalizer, runner, evidence store or producer semantics added. The module holds closure algebra only. |
| Matrix honesty | FR-004 and TC-004 rows annotate that FR-004-AC-3 landed and keep the `🚧 Planned` status glyph, so the Test Execution Summary totals stay consistent for the per-wave bookkeeping pass (COLLISION-4). |

## Validation and limits

Gates were run locally, serially, with one Cargo job and warnings denied, each
with `CARGO_TARGET_DIR` unset so the shared target directory could not serve a
stale artifact. `cargo fmt --all -- --check` clean; `cargo clippy -j 1
--all-targets -- -D warnings` clean; `cargo test -j 1 -- --test-threads=1`
reported 20 passed in `tc_004_closure_axes`, 2 compile-fail doc-tests passed,
0 failed, and the 9 pre-existing IT-001 tests ignored for want of A's producer
fixture.

The `--features quire-extraction` lane named in G-GATES was not run: this
crate's `Cargo.toml` declares no `[features]` table, and inventing one is out
of scope for this chunk. No hosted workflow was dispatched.

This review establishes FR-004-AC-3 only. FR-004-AC-1, AC-2 and AC-4 need the
batch and incremental evaluators (chunks C-20 and C-21) and remain planned.
