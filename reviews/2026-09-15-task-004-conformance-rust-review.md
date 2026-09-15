---
id: SR-110
title: "Rust review of the Task-004 activation-derived conformance bridge"
type: SpecReview
analysis: code-review
scope: "Task-004 / quire-protocol#6, FR-004/FR-026..FR-028/FR-041 and TC-004"
review_set: subset
relationships:
  - { target: ix://agent-ix/quire-protocol/Task-004, type: reviews }
  - { target: ix://agent-ix/quire-protocol/FR-004, type: reviews }
  - { target: ix://agent-ix/quire-protocol/FR-026, type: references }
  - { target: ix://agent-ix/quire-protocol/FR-027, type: references }
  - { target: ix://agent-ix/quire-protocol/FR-028, type: references }
  - { target: ix://agent-ix/quire-protocol/FR-041, type: references }
  - { target: ix://agent-ix/quire-protocol/TC-004, type: reviews }
---
# SR-110: Rust Review of the Task-004 Activation-Derived Conformance Bridge

## Summary

Reviewed the Task-004 production boundary, tests and tracking. The final
surface accepts only exact QSL FR-300 activation facts tied to an admitted
linked graph and Engine-owned compensation snapshot. The evaluator's `Model`
is an internal kernel representation: it is neither a public API nor a domain
object callers can mint.

## Verdict

**PASS for Task-004 / quire-protocol#6.** No unresolved in-scope ownership,
identity, panic, unsafe, replay, test-quality, trace or local-gate finding
remains. This does not close Task-005, Task-006, Task-008, Plan-001 or epic
#14.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-001 | high | Resolved: external callers could construct the evaluator `Model` and obligations directly, bypassing graph/activation admission. Standalone evaluator modules are now crate-private; public evaluation is activation-derived, and model-only cases are crate unit tests. | FR-004-AC-1/2; TC-004 | implementation-bug-despite-evidence |
| FND-002 | high | Resolved: the conformance kernel held an older compensation-registration snapshot after Engine activation. The Engine now builds and validates a replacement kernel from the candidate registration snapshot before committing either state. | FR-004-AC-1/4; FR-026; TC-004 | implementation-bug-despite-evidence |
| FND-003 | medium | Resolved: replacement construction used a panic-capable `expect` despite a validated existing kernel. The internal replacement preserves the validated graph/activation configuration while exchanging only the Engine-owned candidate snapshot, so activation remains a typed, panic-free transaction. | FR-004-AC-1; TC-004 | implementation-bug-despite-evidence |

## Rust Review

| Check | Result |
| --- | --- |
| Ownership and public surface | `Engine` remains the sole mutable owner of compensation state. `Kernel` derives a private `Model`; public callers submit exact activation facts and cannot manufacture obligations, observations or a model. |
| Identity and source evidence | Activation bindings compare the linked definition, control, subject, semantic trigger and exact temporal result. Conflicting replay refuses; QObs facts correlate to the activated occurrence rather than a caller-selected subject. |
| Transactionality and replay | The Engine validates the candidate compensation and conformance snapshots before assigning either. Exact activation replay is idempotent; changed activation evidence cannot overwrite retained state. |
| Independent conformance axes | Decision-scope and surrounding-execution progress/closure, complete-global closure, observation completeness and owner non-successes remain independently observable. Open, incomplete, contradicted, refused, unsupported, resource-incomplete and failed inputs do not collapse into conformance. |
| Panic, unsafe and concurrency | Changed production code introduces no `unsafe`, lock, thread, async/blocking bridge, unchecked indexing, `unwrap` or panic-capable input path. The internal snapshot replacement has no fallible branch after initial Kernel validation. |
| Bounds and test quality | The existing finite/bounded types continue to own limits. TC-004 unit and integration tests assert concrete reports and retained state across interleavings, adverse facts, axis mutations, exact result mismatches and compensation synchronization; no test merely checks a variant name. |
| Repository constraints | Rust only; no feature, workflow, hosted CI or `resources/native-v1/` change. The QSL dependency remains pinned at `0c2f13e`. |

## Verification

The final candidate passes the required unchanged-head local gate in order:
format check, locked all-target Clippy with warnings denied, then the complete
locked test suite, each with one Cargo job, the shared heavy lock and the
worktree-local `target-codex-backends` directory. Focused TC-002 and TC-003
bridge tests also pass. No `.github` workflow file changes are present.

## Boundary

This review covers the Task-004 activation-to-conformance bridge only. It does
not claim downstream claim dispatch, canonical result composition, aggregate
resource qualification or any Plan-001 completion.
