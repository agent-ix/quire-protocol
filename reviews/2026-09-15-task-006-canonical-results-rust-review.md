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
runtime-reference carrier and migrated semantic corpus. Local activation and
resource-boundary findings were remediated, but the independent exact-head
re-review found two owner-capability gaps that Protocol cannot safely synthesize.

## Verdict

**BLOCKED for Task-006 / quire-protocol#8.** One public Engine operation now owns
conformance, dispatch, private result production, strict readback and bundle
commit under one accountant. Callers cannot substitute a foreign dispatch
report, result-owner input or raw canonical producer. The pinned owners do not
publish the authenticated oracle/dependence support or correction views needed
to complete FR-032/FR-033, so this review cannot pass Task-006. This also does
not close Task-007, Task-008, Task-010, Plan-001 or epic #14.

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
| FND-014 | high | Resolved by final independent review: an Engine selected for subject/request A accepted activation facts produced for B, allowing B's conformance semantics to be reported with A's provenance. Every fact is now checked against the Engine's exact definition, QObs subject and strict QSL request bytes before conformance. | FR-006-AC-1/4; FR-031; TC-006 | implementation-bug-despite-evidence |
| FND-015 | high | Open owner-capability blocker: an obligation identity had been substituted for an unavailable oracle identity, and any nonempty observation set could become decisive support. The local fabrication is removed and orchestration fails closed, but the pinned QSL/QObs owners publish no authenticated oracle/dependence capability. `unavailable` is safe interim behavior, not completion evidence. | FR-032-AC-2; FR-033-AC-3; TC-006 | correct-requirement-no-evidence |
| FND-016 | high | Resolved by final independent review: participation invented `qsl-role-instance:*` strings and sorted them as identities. Role occurrences now resolve only through exact QObs-authored FR-287 subjects bound to QSL `role_instance` requirements, use the FR-287 key order, and emit a typed claim-local owner-evidence refusal when the binding is absent. | FR-006; FR-032; TC-006 | implementation-bug-despite-evidence |
| FND-017 | high | Open owner-capability blocker: public controls accepted caller-authored correction relation, premise and corrected-input identity. The fabrication seam is removed, but applicable correction cannot execute through the public Engine path until an owner-authenticated contradiction/correction capability exists. | FR-006; FR-033; TC-006 | correct-requirement-no-evidence |
| FND-018 | high | Resolved by final independent review: unrelated row-assembly failures collapsed to one `result.invalid-selection` envelope after diagnostic prose was discarded. Exhaustive private row-assembly variants now mint distinct registered `assessment-result.*` codes, axes, causes and affected identities, with an injectivity test over the observed envelopes. | FR-006-AC-9; FR-008-AC-1; FR-014; TC-006/TC-011 | implementation-bug-despite-evidence |
| FND-019 | high | Resolved by final independent review: owner-limit routing guessed a resource site from aggregate usage, compared raw rather than effective limits and bypassed the shared accountant. The producer now charges output bytes, maximum retained string, depth and visited fields at four reconciled sites; depth/field sites are distinct, owner clamps execute through `ResourceAccountant`, and the heuristic/direct constructor is deleted. | FR-006-AC-9; NFR-002; TC-006/TC-009 | implementation-bug-despite-evidence |
| FND-020 | medium | Resolved by final independent review: owner selection rejected valid temporal subjects with more than one reachable Boolean leaf. Selection now requires exact declaration-qualified membership in QSL's published leaf set, which still refuses foreign or unreachable leaves without re-deriving QSL semantics. | FR-006-AC-1/4; FR-031-AC-5; TC-006 | implementation-bug-despite-evidence |

## Review Checks

| Check | Result |
| --- | --- |
| Ownership | `Engine::assess_activated` owns public orchestration; raw dispatch evaluation and canonical production remain crate-private semantic boundaries. |
| Stage identity | Exact conformance axes/completeness/global closure and sealed dispatch premises flow into result construction without caller restatement. Facts must match the complete Engine activation. Missing oracle/dependence authority yields explicit unavailable truth rather than invented support, leaving FR-032 incomplete. |
| Accounting | One accountant spans every atomic stage. Result output, retained strings, depth and field visits charge their own reconciled sites; owner/caller exhaustion remains structurally separate from accounting faults. |
| Identity and provenance | The static closure selects one model; the exact activated runtime subject belongs to the runtime binding set. A checked predicate must be a declaration-qualified member of QSL's reachable leaf set. Runtime and role-instance subjects round-trip through the strict canonical carrier. |
| Panic, unsafe and concurrency | No changed production `unsafe`, callback, thread, lock or blocking bridge. Hostile runtime carrier input returns a typed error; remaining `expect` sites describe compile-time registries/static non-empty values. |
| Test quality | TC-006/TC-009/TC-011 cover atomic composition, foreign activation facts, same-package foreign owner selection, exact role-instance carriers, fail-closed unavailable support, private constructors/producer surface, exact request order, strict readback, effective owner clamps, producer structural accounting, row-refusal injectivity, every accounting-fault variant/stage and FR-008 topology. |

## Verification

Local focused tests cover the remediated activation and resource boundaries.
The review remains blocked until the owner capabilities land, the public
authenticated-support/correction scenarios execute, and the complete unchanged
candidate gate is rerun.
