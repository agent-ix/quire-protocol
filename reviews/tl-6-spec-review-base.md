---
id: SR-2127018
title: "TL-6 delta FR-100..FR-102: base"
type: SpecReview
analysis: base
review_set: subset
scope: "agent-ix/quire-protocol; TL-6; PR #144 head 52c68cb9d9117964c734fb32d9bd8e3768942866; spec/functional/FR-100-settle-inclusive-deadline-branches-on-complete-progress.md; spec/functional/FR-101-bind-obligation-captures-as-ordered-atomic-occurrences.md; spec/functional/FR-102-carry-unsupported-protocol-constructs-as-typed-dispositions.md; spec/test-cases/TC-154-settle-deadline-captures-and-unsupported-constructs.md; spec/tests.md"
relationships:
  - { target: ix://agent-ix/quire-protocol/FR-100, type: reviews }
  - { target: ix://agent-ix/quire-protocol/FR-101, type: reviews }
  - { target: ix://agent-ix/quire-protocol/FR-102, type: reviews }
  - { target: ix://agent-ix/quire-protocol/TC-154, type: reviews }
---
## Summary

Ticket: TL-6. Base checklist over PR #144 (diff origin/main...HEAD: FR-100, FR-101, FR-102, TC-154, spec/tests.md). `quire validate --scope .` exits 0 on the four new artifacts; spec/tests.md fails only on the pre-existing `Coverage Status` vs `Status` header assertion (line 144), which also fails on origin/main. Ids FR-100..FR-102 lie inside ADR-010's TL-6 block FR-100..FR-149; TC-154 is the next free TC id. Every AC names TC-154 or Inspection, and TC-154 verifies all three FRs.

## Assurance Context

AP-001 (spec/assurance/AP-001-protocol-implementation.md) declares `review_policy.mode: require` but is `status: proposed`, so it is not an active profile and no enforced ix-flow run is claimed. Direct analyses run: base, integrity, ears-conformance, scope-boundary, dependency, failure-domain.

## Examined

- FR-100 (spec/functional/FR-100-settle-inclusive-deadline-branches-on-complete-progress.md)
- FR-100-AC-1 (spec/functional/FR-100-settle-inclusive-deadline-branches-on-complete-progress.md)
- FR-100-AC-2 (spec/functional/FR-100-settle-inclusive-deadline-branches-on-complete-progress.md)
- FR-100-AC-3 (spec/functional/FR-100-settle-inclusive-deadline-branches-on-complete-progress.md)
- FR-100-AC-4 (spec/functional/FR-100-settle-inclusive-deadline-branches-on-complete-progress.md)
- FR-100-AC-5 (spec/functional/FR-100-settle-inclusive-deadline-branches-on-complete-progress.md)
- FR-101 (spec/functional/FR-101-bind-obligation-captures-as-ordered-atomic-occurrences.md)
- FR-101-AC-1 (spec/functional/FR-101-bind-obligation-captures-as-ordered-atomic-occurrences.md)
- FR-101-AC-2 (spec/functional/FR-101-bind-obligation-captures-as-ordered-atomic-occurrences.md)
- FR-101-AC-3 (spec/functional/FR-101-bind-obligation-captures-as-ordered-atomic-occurrences.md)
- FR-101-AC-4 (spec/functional/FR-101-bind-obligation-captures-as-ordered-atomic-occurrences.md)
- FR-102 (spec/functional/FR-102-carry-unsupported-protocol-constructs-as-typed-dispositions.md)
- FR-102-AC-1 (spec/functional/FR-102-carry-unsupported-protocol-constructs-as-typed-dispositions.md)
- FR-102-AC-2 (spec/functional/FR-102-carry-unsupported-protocol-constructs-as-typed-dispositions.md)
- FR-102-AC-3 (spec/functional/FR-102-carry-unsupported-protocol-constructs-as-typed-dispositions.md)
- FR-102-AC-4 (spec/functional/FR-102-carry-unsupported-protocol-constructs-as-typed-dispositions.md)
- TC-154 (spec/test-cases/TC-154-settle-deadline-captures-and-unsupported-constructs.md)
- spec/tests.md (spec/tests.md)

## Verdict

PASS WITH FINDINGS. ID formats, coverage links and the V1-PROT-019 / QSpec FR-049-AC-9 / FR-172 citations are accurate. spec/tests.md was only half updated.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | spec/tests.md adds the FR-100..FR-102 coverage rows but no TC-154 row in the Test Case table, and leaves the 49 FR / 24 TC / 348 AC prose totals, the 40-row/229-AC behavioural total, the FR-set breakdown and the Test Execution Summary row counts unchanged; with three FRs (13 ACs) and one TC added these figures are now wrong. | spec/tests.md:72-78, 254-277, 289-296 |
| FND-002 | low | FR-102-AC-4 is verified by Inspection, but it is testable: each G7 fixture can assert that the carried disposition's bound, loss selector and record type equal the linked declaration byte for byte and that no exploration result is returned. Inspection is not justified for an obligation a fixture already in TC-154 can fail. | spec/functional/FR-102-carry-unsupported-protocol-constructs-as-typed-dispositions.md:42 |
| FND-003 | low | FR-100-AC-1 says 'one tick later' but no artifact defines a tick; the boundary granularity is the native-temporal boundary mapping (QSpec FR-289, cited by FR-026/FR-028). Name the smallest representable instant after the inclusive deadline under that mapping. | spec/functional/FR-100-settle-inclusive-deadline-branches-on-complete-progress.md:43 |

## New findings (disposition pass 1)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-004 | low | FR-101-AC-5 ('No source in the evaluator evaluates a capture expression') is a source scan verified by Inspection, but the repository verifies 'no X in source' claims by Test (FR-047-AC-1/AC-2, TC-153 Static), and 'evaluates a capture expression' names no symbol a scan can find. A disagreeing-owner fixture fails it directly: supply a QSL outcome that differs from what the authored expression would give locally, and assert the bound value equals the supplied outcome. spec/tests.md also lists AC-5 under TC-154 (AC-1..AC-5) although it is not a test. | spec/functional/FR-101-bind-obligation-captures-as-ordered-atomic-occurrences.md:45 |

## Dispositions

Round 1, fix commit 2d40a60.

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | still-open | The TC-154 row and the FR-100/FR-101 coverage rows are added, but the prose totals (49 FRs / 24 TCs / 348 ACs, line 72-73), the FR-set breakdown, the 40-row / 229-AC behavioural total (line 193), the Test Execution Summary (lines 290-295: 40 FR rows, 24 TC rows, 85 overall) and the 344 owed-criteria row (line 311) are unchanged; with 2 FRs (11 ACs) and 1 TC added these are wrong. |
| FND-002 | fixed | 2d40a60: Verifies the TL-6 delta over FR-019 and FR-028 (ADR-010). Unsupported constructs (ADR-010 gap G7) are verified by FR-057's test case, not here. |
| FND-003 | fixed | 2d40a60: A matching response observed exactly at the deadline instant selects the response branch, and the same response at a time strictly after the deadline, with progress `closed` and completeness `complete`, selects timeout. |
| FND-001 | still-open | deferred to post-#143 by the coordinator: spec/tests.md prose totals (49 FRs / 24 TCs / 348 ACs), FR-set breakdown, the 40-row / 229-AC behavioural total, the Test Execution Summary and the 344 owed-criteria row are still unchanged at ddde4c0; they are to be recomputed once PR #143 (FR-050..FR-059) merges. |
| FND-004 | fixed | ddde4c0 (round 2): Given a QSL-supplied capture value that differs from what the authored expression would yield if computed locally, the bound value equals the supplied one. |
