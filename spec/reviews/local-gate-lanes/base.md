---
id: SR-028
title: "Base review of the local verification gate definition"
type: SpecReview
analysis: base
scope: "NFR-004 and TC-012 at 681e25a"
review_set: subset
relationships:
  - { target: ix://agent-ix/quire-protocol/NFR-004, type: reviews }
---
## Summary

Reviewed the new gate-definition requirement and its test case against the
checklist. Identifier formats are well-formed and sequential: `NFR-004` follows
`NFR-003` and `TC-012` follows `TC-011`, with no gap and no duplicate across any
branch of this repository. Every acceptance criterion carries a verification
method and resolves to a single observable condition with its own negative
control in TC-012. Two defects were found in the first draft and remediated
before this review was recorded.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The first draft of NFR-004-AC-10 restated the hosted-run prohibition already owned by NFR-003-AC-1, creating two owners for one obligation. Remediated: AC-10 now asserts only unchanged-head binding and delegates the hosted-run question by reference. | NFR-004-AC-10; NFR-003-AC-1 |
| FND-002 | medium | The `cargo test` command's stated risk is not met today, because the repository is specification-only and its one test file is ignored. Remediated: the Scope section now records that `fmt` and `clippy` are fully effective while `test` accrues coverage as the suite lands, rather than implying the gate detects regression it cannot yet detect. | NFR-004 Scope; spec.md section 14 |
| FND-003 | low | No coverage gap. NFR-004 has exactly one owning test case, TC-012 traces back with a `verifies` relationship, and TM-001 carries the requirement row, the test-case row and the corrected execution totals. | NFR-004; TC-012; TM-001 |
| FND-004 | low | Each criterion is independently assertable. AC-6 and AC-8 are distinct rather than redundant: AC-6 governs an instruction naming a feature the crate does not declare, AC-8 governs a feature the crate declares with no conditionally compiled item behind it. | NFR-004-AC-6; NFR-004-AC-8 |
| FND-005 | high | SpecReview ids collided across concurrent agents. Four branches independently scanned every ref, each correctly found SR-023 as the maximum, and each minted from there, because none could see the others' unpushed work. This review set was reallocated from SR-024..SR-026 to SR-028..SR-030. Scanning is not a sound allocator under concurrency: it reads committed state while the contended resource is the set of in-flight intentions. The durable fix is one allocator for the wave, or staking an id by pushing its commit immediately. | SR-028; SR-029; SR-030 |

## Coverage Assessment

| Rule | Status | Evidence |
| --- | --- | --- |
| Every requirement has a test case | Pass | NFR-004 to TC-012 in TM-001 |
| Every test case traces to a requirement | Pass | TC-012 `verifies` NFR-004 |
| Every acceptance criterion is reachable | Pass | AC-1 through AC-10 each have a procedure step and an expected result in TC-012 |
| Negative and error paths are specified | Pass | Eight negative controls, one per guarded condition |
| Identifiers are unique and sequential | Pass | NFR-004 and TC-012 were free at every scan; the maxima were NFR-003 and TC-011. The SpecReview ids were reallocated from SR-024..SR-026 to SR-028..SR-030 after a concurrent-wave collision, recorded in FND-005 |
| Matrix totals reconcile | Pass | Static/Inspection 1 to 2, Overall 14 to 15 |
