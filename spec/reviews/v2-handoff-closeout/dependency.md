---
id: SR-061
title: "Dependency review of the published version 2 handoff closeout"
type: SpecReview
analysis: dependency
scope: "Task-012 and IT-001 SC-06..SC-09 at qprotocol e53fb55 against qspec-language 93c411b"
review_set: subset
relationships:
  - { target: ix://agent-ix/quire-protocol/IT-001, type: reviews }
  - { target: ix://agent-ix/quire-protocol/Task-012, type: reviews }
---
## Summary

The closeout dependency chain is acyclic and fully enabled. The owner crate
contains the versioned reader, typed refusals, handoff types, immutable corpus,
and exported addressing constants on `main`. The consumer needs one pin update
and no local copy of producer artifacts or protocol vocabulary.

## Classification

| Item | Class | State and evidence |
| --- | --- | --- |
| Version 2 reader and typed refusal API | External enablement | Delivered by qspec-language PR #75 and present on `main` at `93c411b`. |
| Published handoff and 28-case manifest | External enablement | Delivered under A's crate source and covered by its published `SHA256SUMS`. |
| Handoff address/member/format constants | External enablement | Delivered by qspec-language PR #92 at `93c411b`; this resolves issue #78's addressing gap. |
| Exact dependency pin | Consumer enablement | qprotocol still pins `04254a3`; Task-012 updates it to `93c411b` before compiling the closeout tests. |
| Default-gate execution, cross-version calls, and resource probes | Consumer feature work | Unblocked and wholly owned by qprotocol issue #48. |
| Pre-delivery corpus retirement and tracking reconciliation | Consumer closeout | Unblocked; deletion is safe because A's manifest is now the sole version 2 corpus authority. |

## Dependency order

1. Pin `quire-spec-language` `93c411b` and import its exported constants.
2. Make the published handoff execute in the default lane and add the missing
   cross-version and per-dimension resource assertions.
3. Retire the superseded local corpus.
4. Reconcile current specs, matrix, plan, and ticket tracking; run the local Rust
   gate and PR-time reviews.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No external blocker remains. The only stale dependency fact is qprotocol's `04254a3` Cargo pin, and updating it is the first subtask rather than a reason to defer implementation. | `Cargo.toml`; Task-012 Dependency state |
| FND-002 | low | Agent B's TL-* lane is neither a dependency nor an implementation surface for this ticket. The closeout consumes qspec-language's already merged handoff interface and must not modify or duplicate TL work. | Task-012 Boundary |

## Disposition

Proceed in the order above. If the exact pin fails to expose the published
bundle or any constant, report the concrete mismatch on issue #48 rather than
inventing a path or literal.
