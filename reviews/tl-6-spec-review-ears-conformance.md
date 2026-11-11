---
id: SR-2127020
title: "TL-6 delta FR-100..FR-102: ears-conformance"
type: SpecReview
analysis: ears-conformance
review_set: subset
scope: "agent-ix/quire-protocol; TL-6; PR #144 head 52c68cb9d9117964c734fb32d9bd8e3768942866; spec/functional/FR-100-settle-inclusive-deadline-branches-on-complete-progress.md; spec/functional/FR-101-bind-obligation-captures-as-ordered-atomic-occurrences.md; spec/functional/FR-102-carry-unsupported-protocol-constructs-as-typed-dispositions.md; spec/test-cases/TC-154-settle-deadline-captures-and-unsupported-constructs.md; spec/tests.md"
relationships:
  - { target: ix://agent-ix/quire-protocol/FR-100, type: reviews }
  - { target: ix://agent-ix/quire-protocol/FR-101, type: reviews }
  - { target: ix://agent-ix/quire-protocol/FR-102, type: reviews }
  - { target: ix://agent-ix/quire-protocol/TC-154, type: reviews }
---
## Summary

Ticket: TL-6. EARS conformance of the FR-100..FR-102 Description and Behavior statements and testability of their ACs.

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

## Verdict

PASS WITH FINDINGS. FR-101's statements conform. FR-100 and FR-102 each carry an obligation without SHALL, and FR-100-AC-2 has an undecidable disjunction.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | FR-100's Description joins two obligations, and the second ('it selects the timeout branch only when progress is authoritatively complete ...') has no SHALL, so the central timeout rule is not normative text. Split it into its own SHALL statement. | spec/functional/FR-100-settle-inclusive-deadline-branches-on-complete-progress.md:25 |
| FND-002 | low | FR-102 Behavior 'the evaluator approximates, truncates or skips none' states the no-repair rule without SHALL NOT; it is the rule FR-102-AC-4 verifies, so it should be normative. | spec/functional/FR-102-carry-unsupported-protocol-constructs-as-typed-dispositions.md:33 |
| FND-003 | medium | FR-100-AC-2 accepts 'pending or incomplete' for no response under incomplete progress without saying which applies when, so a test can only assert 'not timeout'; an implementation returning the wrong one of the two passes. State the condition for each (for example open progress -> pending, incomplete or contradicted input -> incomplete) and assert each. | spec/functional/FR-100-settle-inclusive-deadline-branches-on-complete-progress.md:44 |

## Dispositions

Round 1, fix commit 2d40a60.

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | 2d40a60: When the awaited scope's progress (FR-284) is `closed` through the deadline and the input completeness (FR-295) is `complete`, and no matching observation was admitted, the evaluator SHALL select the timeout branch. |
| FND-002 | fixed | 2d40a60: FR-102 removed. The no-approximation rule now lives in FR-057 (PR #143): 'The library SHALL NOT approximate, truncate or repair the construct.' |
| FND-003 | fixed | 2d40a60: No response with progress `closed` through the deadline and completeness `complete` selects timeout; the same history with progress `open` is `pending`; with completeness `incomplete` it is `incomplete`; neither selects timeout. |
