---
id: SR-2127044
title: "TL-6 PR #149 gap-analysis"
type: SpecReview
analysis: gap-analysis
review_set: subset
scope: "agent-ix/quire-protocol; TL-6; PR #149 head ecdac92; spec/functional/FR-100, spec/functional/FR-101, spec/test-cases/TC-154, spec/tests.md, src/deadline.rs, src/captures.rs, tests/tc_154_settle_deadline_captures.rs"
relationships:
  - { target: ix://agent-ix/quire-protocol/FR-100, type: reviews }
  - { target: ix://agent-ix/quire-protocol/FR-101, type: reviews }
---
## Summary

Ticket: TL-6. Planless gap analysis (Plan completion: not assessed) of FR-100 and FR-101 against the computed Test Matrix (`quire matrix --format tsv` on head ecdac92), the tagged tests and the code. Every new test uses `ix_trace_rs::trace` with `("TC-154", "FR-1nn-AC-n")`; no `/// Trace:` lines.

## Examined

- FR-100-AC-1 (spec/functional/FR-100-settle-inclusive-deadline-branches-on-complete-progress.md)
- FR-100-AC-2 (spec/functional/FR-100-settle-inclusive-deadline-branches-on-complete-progress.md)
- FR-100-AC-3 (spec/functional/FR-100-settle-inclusive-deadline-branches-on-complete-progress.md)
- FR-100-AC-4 (spec/functional/FR-100-settle-inclusive-deadline-branches-on-complete-progress.md)
- FR-100-AC-5 (spec/functional/FR-100-settle-inclusive-deadline-branches-on-complete-progress.md)
- FR-100-AC-6 (spec/functional/FR-100-settle-inclusive-deadline-branches-on-complete-progress.md)
- FR-100-AC-7 (spec/functional/FR-100-settle-inclusive-deadline-branches-on-complete-progress.md)
- FR-101-AC-1 (spec/functional/FR-101-bind-obligation-captures-as-ordered-atomic-occurrences.md)
- FR-101-AC-2 (spec/functional/FR-101-bind-obligation-captures-as-ordered-atomic-occurrences.md)
- FR-101-AC-3 (spec/functional/FR-101-bind-obligation-captures-as-ordered-atomic-occurrences.md)
- FR-101-AC-4 (spec/functional/FR-101-bind-obligation-captures-as-ordered-atomic-occurrences.md)
- FR-101-AC-5 (spec/functional/FR-101-bind-obligation-captures-as-ordered-atomic-occurrences.md)
- FR-100 (spec/functional/FR-100-settle-inclusive-deadline-branches-on-complete-progress.md)

## Verdict

**FAIL.** `quire matrix` reports FR-100-AC-1..AC-6 and FR-101-AC-1..AC-5 `tagged` and FR-100-AC-7 `untagged`. The AC-7 behaviour is asserted inside the AC-3 test but carries no AC-7 tag, so spec/tests.md's 'all 7 criteria have living traced tests' is false. FR-101-AC-5 genuinely proves the supplied value is bound: the same slot (`trigger.guard`) is evaluated by QSL for two supplies and each binding returns exactly QSL's value, so any local recomputation or alteration would fail it. FR-101-AC-3 uses a real QSL `Unavailable` outcome. No code without an owning requirement; no stubs.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | FR-100-AC-7 has no tagged test (`quire matrix`: untagged), yet spec/tests.md marks FR-100 and TC-154 Implemented with 'all 7 criteria'. The assertions for it (closed/complete with a DoesNotHold or Unestablished observation gives no selection and truth `unavailable`) sit in `a_false_constraint_selects_neither_branch`, tagged AC-3 only, and that test does not assert execution `completed` for this case. Add an AC-7-tagged test asserting selected None, truth Unavailable and execution Completed. | tests/tc_154_settle_deadline_captures.rs:252-287 |
| FND-002 | medium | The supersession bullet covers any observation at or before the deadline admitted after timeout, but only a matching response is tested. No test admits a late false/valueless observation after timeout (where the code silently replaces the result) or a regressing authority fact (which the test asserts is refused as `PremiseRegressed`, pinning the defect in place rather than the FR-033 behaviour). | tests/tc_154_settle_deadline_captures.rs:302-359 |
| FND-003 | low | `progress_through_another_boundary_is_refused` is tagged FR-100-AC-2 but asserts that a progress view through another boundary is refused, which AC-2 does not state. Remove the AC-2 tag or add an AC for the boundary refusal. | tests/tc_154_settle_deadline_captures.rs:423-441 |
| FND-004 | low | The 'stays bound' half of FR-101-AC-1 cannot fail: the test binds a second, independent activation and re-reads the first, which Rust ownership already guarantees. No admitted fact reaches `bind_captures` at all. The ordering half is real. Either state immutability as a type property (no mutating accessor, verified by inspection) or drive a later fact through the activation path that feeds captures. | tests/tc_154_settle_deadline_captures.rs:491-536 |

## New findings (disposition pass 1)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-005 | low | `progress_through_another_boundary_is_refused` shifts the anchor by +1 and -1, which moves both bounds, so it also passes on the previous upper-only comparison. The lower-bound half of `named_by`, which is what the round-1 fix added, is untested. Add a case where only the lower bound differs. | tests/tc_154_settle_deadline_captures.rs:582-603 |

## Dispositions

Round 1, fix commit 183d4f1.

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | 183d4f1 |
| FND-002 | fixed | 183d4f1 |
| FND-003 | fixed | 183d4f1 |
| FND-004 | fixed | 183d4f1 |

Round 2, fix commit fb68003.

| FND | outcome | reason |
| --- | --- | --- |
| FND-005 | fixed | fb68003 |
