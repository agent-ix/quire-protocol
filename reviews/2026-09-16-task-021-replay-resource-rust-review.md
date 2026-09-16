---
id: SR-129
title: "Rust review of Task-021 replay and resource completion"
type: SpecReview
analysis: code-review
scope: "Task-021 / quire-protocol#12, IT-002 SC-05 and SC-06"
review_set: subset
relationships:
  - { target: ix://agent-ix/quire-protocol/Task-021, type: reviews }
  - { target: ix://agent-ix/quire-protocol/IT-002, type: reviews }
  - { target: ix://agent-ix/quire-protocol/FR-004, type: reviews }
  - { target: ix://agent-ix/quire-protocol/FR-026, type: reviews }
  - { target: ix://agent-ix/quire-protocol/FR-041, type: reviews }
  - { target: ix://agent-ix/quire-protocol/NFR-002, type: reviews }
---
# SR-129: Rust Review of Task-021 Replay and Resource Completion

## Summary

Reviewed Task-021's QSL/QObs evidence correlation, activation-derived model,
batch and settled-incremental paths, occurrence identity, composed resource
accounting and IT-002 SC-05/SC-06 evidence against the repository conventions
and the Rust-review checklist. Three independent pre-promotion reviews found
blocking boundary, accounting, identity and test-oracle defects in the first
candidate. Every finding below was fixed before this artifact was finalized.

## Verdict

**PASS for the Task-021 code subset and promotion.** No unresolved correctness,
Rust-idiom, panic, unsafe, conversion, owner-boundary, resource, determinism,
trace or test-quality finding remains. This verdict does not close IT-002:
SC-07a/SC-07b and Task-018 remain open.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | Resolved: the first candidate hashed five caller-sized QObs documents and allocated/derived activation models before the shared accountant guarded that work. Five independent production source sites now pre-charge evidence bytes plus model nodes, framed identity bytes, bounded comparisons and allocation population before traversal or retention; each active dimension has exact, one-short and fresh-exact evidence. | `src/conformance/kernel.rs`; `src/bounds/inventory.rs`; IT-002-SC-06 |
| FND-002 | high | Resolved: the composed Engine checked temporal evidence, but its public activation-derived batch and incremental semantic frontends could be reached through `conformance()` without that check. Batch now checks every fact's exact QSL request; incremental push checks each fact and finish checks the last admitted request. Empty incremental finish and empty derived models refuse. | `src/conformance/kernel.rs`; TC-002; IT-002-SC-05 |
| FND-003 | high | Resolved: SC-05 reused one QObs observation, did not establish a genuinely non-singleton equal-time population, and compared paths with a weaker local oracle. It now carries three distinct owner observations and workflow occurrences under one admitted time key, uses the production semantic comparator, proves early settlement, checks both nontrivial permutations, repeats the authored-premise mutation through both paths and requires typed path divergence. | `tests/it002_observation_temporal.rs`; IT-002-SC-05a..SC-05d |
| FND-004 | medium | Resolved: one all-foreign negative case could pass after the first short-circuited axis. Independent tests now substitute decision progress, decision closure, surrounding progress, surrounding closure, completeness and execution one at a time. A focused unit test mutates each of the nine evidence-reference coordinates independently. | `tests/it002_observation_temporal.rs`; `src/conformance/kernel.rs` |
| FND-005 | medium | Resolved: the first derived obligation identity did not retain workflow or repeat-ordinal identity and used ambiguous component concatenation. The identity is now a fixed SHA-256 over canonical length-framed document, subject, trigger and workflow bytes plus declaration/control handles and every signed repeat ordinal; tests discriminate ordinal and framing mutations. | `src/conformance/kernel.rs` |
| FND-006 | medium | Resolved: the Task-021 trace and current specification text credited the wrong FR-026 criteria, omitted FR-041 and retained stale QSL `0c2f13e` pins. SC-05 now traces only the exercised FR-004/FR-026/FR-041 criteria, current dependency statements select accepted QSL `2736b62`, and SC-07's integration-wide identity comparison remains explicitly open. | IT-002; TM-001; Task-021; `spec/spec.md` |
| FND-007 | low | Resolved: adding the kernel unit-test module changed the repository's exact conditional-site inventory, and the full local gate killed the stale oracle. TC-012 now records the new site rather than weakening conditional coverage. | `tests/tc_012_local_gate.rs`; NFR-004-AC-8 |

## Rust Review

| Check | Result |
| --- | --- |
| Owner and semantic boundaries | Protocol compares QSL's borrowed evidence tuple to exact QObs owner views. It does not parse, normalize, repair or mirror an owner payload. QSL's `Execution` remains narrower than Protocol's assessment disposition; only exact representable Completed/Failed pairs correlate, and nonrepresentable substitutions refuse. |
| Identity and ordering | Derived IDs include source document, declaration/control, subject, semantic trigger, workflow and repeat ordinals. The derived model is canonically ordered by that ID, so equal-time presentation order changes neither obligation order nor results. Authored non-repeat causal edges remain the only predecessor source. |
| Errors and safety | Evidence mismatch is a closed typed kernel/refusal axis. No new production `unwrap`, `expect`, panic, `unsafe`, unchecked narrowing conversion, async bridge, lock or background work was added. Digest comparison is allocation-free and length-checked before indexing. |
| Bounds and allocation | Evidence documents are charged before hashing. Population scanning is preceded by an operation charge; node, exact framed hash input, bounded pair work and stable allocations are pre-charged before model construction. Existing batch, dispatch, result and read charges remain cumulative in the same request accountant. |
| Test quality and trace | SC-05 uses distinct real QObs observations, non-singleton equal-time permutations, production comparison and an adverse premise mutation. SC-06 exercises seven active dimensions. Coordinate and repeat-ordinal tests target production helpers, while TC-002 proves owner-matched open/incomplete/failed requests rather than depending on a bypass. |

## Verification

The repository-defined serialized gate passes unchanged: formatting; locked
all-target Clippy with warnings denied; and the full locked default test suite.
The suite passes 284 unit/integration tests plus 43 doc-tests, with zero failed
or ignored. Native Quoin `0.23.1-87-gb3d02d4` strict validation and the review
artifact validation pass with no finding.

## Boundary

This review covers Task-021's SC-05/SC-06 allocation only. It does not claim
IT-002 SC-07, final Protocol scenario evidence, Task-018/#63, Plan-001 or epic
#14 complete.
