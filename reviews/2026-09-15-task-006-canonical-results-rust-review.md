---
id: SR-116
title: "Rust review of the Task-006 atomic canonical-result orchestration"
type: SpecReview
analysis: code-review
scope: "Task-006 / quire-protocol#8, FR-006/FR-008/FR-031..FR-033/FR-036..FR-040 and TC-006/TC-011"
review_set: subset
relationships:
  - { target: ix://agent-ix/quire-protocol/Task-006, type: reviews }
  - { target: ix://agent-ix/quire-protocol/FR-006, type: reviews }
  - { target: ix://agent-ix/quire-protocol/FR-008, type: reviews }
  - { target: ix://agent-ix/quire-protocol/TC-006, type: reviews }
  - { target: ix://agent-ix/quire-protocol/TC-011, type: reviews }
---
# SR-116: Rust Review of the Task-006 Atomic Canonical-Result Orchestration

## Summary

Reviewed the Task-006 Engine boundary, canonical result producer/reader,
provenance and truth derivation, shared accounting, typed refusal routing,
runtime-reference carrier and migrated semantic corpus. All findings identified
in this scoped review and the independent PR-head re-review were fixed in the
candidate.

## Verdict

**PASS for Task-006 / quire-protocol#8.** One public Engine operation now owns
conformance, dispatch, private result production, strict readback and bundle
commit under one accountant. Callers cannot substitute a foreign dispatch
report, complete result-owner input or raw canonical producer. This does not
close Task-007, Task-008, Task-010, Plan-001 or epic #14.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-001 | high | Resolved: public `construct_canonical_outcome` accepted any sealed dispatch report, so conformance/dispatch and result assembly were not one atomic Engine transition. `assess_activated` now executes and retains the exact stages together; compile-fail evidence seals the removed seam. | FR-006-AC-1/9; TC-006 | implementation-bug-despite-evidence |
| FND-002 | high | Resolved: `CanonicalRowFacts` let callers restate conformance axes, completeness, truth, global closure and complete provenance. The Engine now reuses exact stage values, derives truth, validates selected model/runtime identities and constructs provenance itself. | FR-006; FR-031; FR-032; TC-006 | implementation-bug-despite-evidence |
| FND-003 | high | Resolved: public `result::produce` and `produce_bounded` let external callers bypass the Engine with a complete wire-shaped input. Both are crate-private, their semantic corpus is in-crate, and a compile-fail doctest rejects external import. | FR-006-AC-1/9; TC-006 | implementation-bug-despite-evidence |
| FND-004 | medium | Resolved: separate stage entry points reset accounting between conformance, dispatch and result work. The atomic operation carries one `ResourceAccountant`; a cumulative allocation test proves prior-stage consumption is retained. | FR-006-AC-9; NFR-002; TC-006 | implementation-bug-despite-evidence |
| FND-005 | medium | Resolved: accounting invariant faults were collapsed into generic invalid-kernel/selection refusals. Every fault variant now retains its exact site or unit, interior, actual stage/prerequisite and complete Protocol usage dimensions under registered typed codes. | FR-008-AC-1; FR-036-AC-1; FR-039-AC-1; TC-011 | implementation-bug-despite-evidence |
| FND-006 | medium | Resolved: provenance accepted caller-selected model/input identities and did not use the implemented runtime-reference carrier. The Engine validates both identities against its immutable selection and binding sets, then emits the reversible `quire.protocol.runtime-reference/1` value. | FR-031; TC-006 | correct-requirement-no-evidence |
| FND-007 | low | Resolved: hostile runtime-reference JSON reached guarded `expect` calls after a shape predicate. Those conversions now return `Error::Shape`; no input-controlled panic remains. | FR-031; TC-006 | implementation-bug-despite-evidence |
| FND-008 | low | Resolved during the full gate: one static API test self-matched its forbidden literal and the intake inventory used ambiguous `v2::read(` text that matched a temporal helper after the corpus moved in-crate. The first oracle now composes independent fragments; the second inspects the fully qualified owner reader call. | TC-006; IT-001 | implementation-bug-despite-evidence |
| FND-009 | high | Resolved by independent PR-head review: public `CanonicalRowFacts` still let callers fabricate settlement, participation, both adequacy domains, support, limitations and same-package owner selections. The public type is replaced by field-private `CanonicalRowControls`; the complete facts value is private and derived inside the Engine from authenticated QSL/QObs, conformance and dispatch outputs. | FR-006; FR-031; FR-032; TC-006 | implementation-bug-despite-evidence |
| FND-010 | high | Resolved by independent PR-head review: package equality alone admitted an unrelated checked predicate, and an arbitrary runtime binding could be selected as provenance. Result startup now requires the exact temporal declaration/request identity, its unique checked leaf, the activated subject's membership in the runtime binding set and that exact activated subject as the retained input. A valid same-package foreign predicate refuses. | FR-006-AC-1/4; FR-031-AC-5; TC-006 | implementation-bug-despite-evidence |
| FND-011 | high | Resolved by independent PR-head review: result-owner ceiling exhaustion returned a claim-local semantic refusal. Every result-owner `ResourceIncomplete` now maps to the shared pre-row resource outcome with its exact registered dimension and site; the bundle is not committed. | FR-006-AC-9; NFR-002; TC-006 | implementation-bug-despite-evidence |
| FND-012 | medium | Resolved by independent PR-head review: public constructors exposed raw `BundleRefusal`, bypassing the shared operation envelope, and dispatch accepted a permutation of the Engine's requested claims. Raw constructors/refusal are crate-private, public startup returns `OperationOutcome`, and request order must equal the Engine order exactly. | FR-006-AC-9; FR-008; TC-006/TC-011 | implementation-bug-despite-evidence |
| FND-013 | low | Resolved during remediation: the retargeting compile-fail example also contained an undefined variable, so it could pass without proving the intended API absence. The example is now otherwise well-formed and fails on the removed construction method. | FR-006-AC-1/9; TC-006 | implementation-bug-despite-evidence |

## Review Checks

| Check | Result |
| --- | --- |
| Ownership | `Engine::assess_activated` owns public orchestration; raw dispatch evaluation and canonical production remain crate-private semantic boundaries. |
| Stage identity | Exact conformance axes/completeness/global closure and sealed dispatch premises flow into result construction without caller restatement. Truth and provenance are derived from those values. |
| Accounting | One accountant spans every atomic stage. Resource incompleteness remains structurally separate; accounting faults retain all seven interiors and the actual one of four routed stages. |
| Identity and provenance | The static closure must select one model; the exact activated runtime subject must belong to the runtime binding set. The temporal request/view must select one unique checked leaf. Runtime subject bytes round-trip through the strict canonical carrier. |
| Panic, unsafe and concurrency | No changed production `unsafe`, callback, thread, lock or blocking bridge. Hostile runtime carrier input returns a typed error; remaining `expect` sites describe compile-time registries/static non-empty values. |
| Test quality | TC-006/TC-011 cover atomic composition, same-package foreign owner selection, private constructors/producer surface, exact request order, strict readback, owner and cumulative accounting, every accounting-fault variant/stage, carrier round-trip and the FR-008 child topology. |

## Verification

Focused TC-006/TC-011 and TC-012 checks, warnings-denied all-target Clippy,
Quire validation/coverage, compile-fail doctests and the complete exact NFR-004
local gate pass on one unchanged worktree state.
