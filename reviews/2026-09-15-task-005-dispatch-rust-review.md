---
id: SR-114
title: "Rust review of the Task-005 activation-derived claim dispatch"
type: SpecReview
analysis: code-review
scope: "Task-005 / quire-protocol#7, FR-005/FR-029/FR-030 and TC-005"
review_set: subset
relationships:
  - { target: ix://agent-ix/quire-protocol/Task-005, type: reviews }
  - { target: ix://agent-ix/quire-protocol/FR-005, type: reviews }
  - { target: ix://agent-ix/quire-protocol/FR-029, type: references }
  - { target: ix://agent-ix/quire-protocol/FR-030, type: references }
  - { target: ix://agent-ix/quire-protocol/TC-005, type: reviews }
---
# SR-114: Rust Review of the Task-005 Activation-Derived Claim Dispatch

## Summary

Reviewed Task-005's public Engine boundary, closed dispatcher, retained premise
model, migration of the semantic tests and the related tracking reconciliation.
All findings identified in this scoped review were fixed in the candidate.

## Verdict

**PASS for Task-005 / quire-protocol#7.** The public path dispatches only the
Engine's activation-derived conformance report, preserves one independent row
per requested claim, and permits no caller callback or alternate backend to
promote a result. This does not close Task-006, Task-008, Plan-001 or epic #14.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-001 | high | Resolved: public raw dispatch accepted an arbitrary `ConformanceReport`. It is crate-private now; `Engine::dispatch_activated` obtains the report from exact activation/graph state first. | FR-005-AC-1/3; TC-005 | implementation-bug-despite-evidence |
| FND-002 | high | Resolved: finite conformance accepted any non-empty backend bytes. The closed table admits only `qprotocol:conformance/v1`; substituted identity/version refuses claim-locally. | FR-005-AC-4; FR-030-AC-3; TC-005 | implementation-bug-despite-evidence |
| FND-003 | medium | Resolved: integration tests could still exercise the raw helper as an external API. They are now crate-internal semantic tests, and a compile-fail proof prevents external import. | FR-005-AC-4; TC-005 | correct-requirement-no-evidence |
| FND-004 | medium | Resolved: inherited Task-004 reviews reused Task-009's SR-110/SR-111 identifiers. Task-004 is renumbered SR-112/SR-113; this review starts the unique next pair. | Task-004; Task-005 | correct-requirement-no-evidence |

## Review Checks

| Check | Result |
| --- | --- |
| Ownership | Engine owns public conformance-to-dispatch composition; the raw evaluator and bounded semantic harness are crate-private. |
| Backend boundary | The only supported capability is exact finite conformance. Advanced claims remain `Unsupported`; a failed execution and an unsupported sibling remain distinct. |
| Identity and premises | Request order, identity, strength and all premise components are retained. Duplicate request, missing/substituted component, circular assumption and correspondence mismatch refuse only that claim. |
| Panic, unsafe and concurrency | No changed production `unsafe`, panic-capable input path, callback, thread, lock or blocking bridge. Test panic sites are fixture assertions only. |
| Test quality | TC-005 covers total ordered rows, global conformance mapping, all premise axes, duplicate identity, exact correspondence, resource charging, alternate-backend refusal and failed/unsupported independence. A real FR-300 activation fixture proves the public Engine path. |

## Verification

Focused TC-005, the real activation-to-dispatch proof, TC-012, doc compile-fail
proof, warnings-denied all-target Clippy and the final local gate pass on QSL
default/current Protocol pin `0c2f13e261c0ffeaf89f0638931b573ec038e99b`.
