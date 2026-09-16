---
id: SR-120
title: "Rust review of Task-008 resource and reproduction closure"
type: SpecReview
analysis: code-review
scope: "Task-008 / quire-protocol#10, NFR-001/NFR-002 and TC-008/TC-009"
review_set: subset
relationships:
  - { target: ix://agent-ix/quire-protocol/Task-008, type: reviews }
  - { target: ix://agent-ix/quire-protocol/NFR-001, type: reviews }
  - { target: ix://agent-ix/quire-protocol/NFR-002, type: reviews }
  - { target: ix://agent-ix/quire-protocol/TC-008, type: reviews }
  - { target: ix://agent-ix/quire-protocol/TC-009, type: reviews }
---
# SR-120: Rust Review of Task-008 Resource and Reproduction Closure

## Summary

Reviewed the final caller-budget architecture, source-bound inventory, monotonic
deadline path, owner-accounting boundary, canonical-result reproduction,
producer-integration permutations and E02 projection accounting. All in-scope
Rust findings were fixed before this artifact was finalized.

## Verdict

**PASS.** `ResourceAccountant` is the sole supported Protocol request
accountant. Every final caller-controlled production site is reconciled to one
inventory row and boundary case; E02 projection participates in the same
accounting and deadline path; equivalent presentations reproduce; and semantic
identity mutations remain discriminating.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-001 | high | Resolved: `bounds` still exported a complete unused `CountGuard`/`BoundedRun` architecture beside `ResourceAccountant`. Only legacy tests consumed it, so public callers could select two incompatible accounting models. The duplicate registry, admission/reservation, precheck/refusal, run-composition and mirrored QSL-dimension helpers are removed; the deadline primitive remains crate-private beneath `ResourceAccountant`. | NFR-002-AC-1..AC-3; Task-008 | implementation-bug-despite-evidence |
| FND-002 | high | Resolved: `project_e02` strict-read source bytes and constructed, canonicalized, reread and copied E02 output without the Protocol accountant. It now runs as a declared deadline unit, uses bounded source reading, charges derived/item/operation work, charges each logical allocation before it occurs, and charges the exact canonical output length before output allocation. Eight source sites and independent inventory rows cover the path. | NFR-002; FR-007; TC-009 | correct-requirement-no-evidence |
| FND-003 | medium | Resolved: retiring the duplicate guard left eight `bounds.precheck.*`/`bounds.deadline.*` catalog registrations that no production source could emit. The dead code family and registry module are removed, so the refusal-catalog live-entry test again proves every registered code has an emitter. | FR-008; TC-011 | implementation-bug-despite-evidence |
| FND-004 | medium | Resolved: the final reproduction corpus stopped at canonical results. TC-008 now compares byte-identical and typed-equivalent E02 results across reverse map insertion, discriminates the projection adapter identity, and binds the legal IT-002 batch/incremental arrival permutation to NFR-001. | NFR-001-AC-1..AC-3; TC-008 | correct-requirement-no-evidence |
| FND-005 | low | Resolved: the merged frozen source-module list ordered `projection.rs` before `producer_contracts.rs`, unlike the deterministic directory census. The inventory now matches the actual sorted source population. | NFR-004; TC-012 | implementation-bug-despite-evidence |

## Rust Review

| Check | Result |
| --- | --- |
| Public architecture | `ResourceAccountant`, `Budget`, typed `ResourceIncomplete`, inventory values and the public clock seam remain. The old count/run guards, registration API, allocation token and their refusal codes have no public or private production surface. |
| Owner boundaries | QSL/QObs/qverify values and owner usage stay typed and versioned. Protocol does not mirror QSL's accumulation enum/table or add owner usage into its own caller dimensions. E02 wire types, validation and RFC 8785 operations remain qverify-owned. |
| Pre-work accounting | Static increments use checked accountant arithmetic and transactional multi-site charging. Projection charges candidate construction before allocation, computes canonical length without an output buffer, then charges output allocation/bytes and the final source copy before each allocation. |
| Deadline discipline | One monotonic clock trait feeds crate-private before/after observation. Expiry carries measured elapsed time and overshoot; clock regression is an accounting fault; no wall-clock value enters the API. |
| Errors and panics | Budget exhaustion remains structurally distinct from domain refusal and accounting defect. Changed request paths add no unsafe code, unchecked narrowing, input-controlled indexing, recursion, locks, background tasks or blocking async bridge. Production `expect` sites remain validated static/request invariants. |
| Determinism | Canonical JCS bytes, ordered typed E02 output and unchanged source bytes agree across equivalent map presentations. Locale, allocation layout, independent arrival order and every named result identity retain their existing independent controls. Elapsed observations are not compared as reproducible semantic data. |
| Tests | TC-009 exercises every production inventory row below/at/above through the accountant, both authored/caller ceilings, deadline before/after/overshoot/regression, fresh budgets and every inventory mutation. Projection additionally executes real exact, one-short and fresh-exact dimension paths. TC-008 adds E02 and producer-integration permutations to the existing full identity matrix. |

## Verification

Formatting, locked all-target Clippy with warnings denied, the complete locked
test suite, strict Quire grammar validation, targeted Quire coverage and native
Quoin validation pass on the reconciled candidate. Native Quoin
`0.23.1-86-gc721d71` reports no finding. Quire's known multiline Rust trace
scanner regression remains tracked in
[quire-cli#90](https://github.com/agent-ix/quire-cli/issues/90); the targeted
bindings use short compiler-checked one-line attributes.
