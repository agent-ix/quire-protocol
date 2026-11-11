---
id: SR-2127023
title: "TL-6 delta FR-100..FR-102: failure-domain"
type: SpecReview
analysis: failure-domain
review_set: subset
scope: "agent-ix/quire-protocol; TL-6; PR #144 head 52c68cb9d9117964c734fb32d9bd8e3768942866; spec/functional/FR-100-settle-inclusive-deadline-branches-on-complete-progress.md; spec/functional/FR-101-bind-obligation-captures-as-ordered-atomic-occurrences.md; spec/functional/FR-102-carry-unsupported-protocol-constructs-as-typed-dispositions.md; spec/test-cases/TC-154-settle-deadline-captures-and-unsupported-constructs.md; spec/tests.md"
relationships:
  - { target: ix://agent-ix/quire-protocol/FR-100, type: reviews }
  - { target: ix://agent-ix/quire-protocol/FR-101, type: reviews }
  - { target: ix://agent-ix/quire-protocol/FR-102, type: reviews }
  - { target: ix://agent-ix/quire-protocol/TC-154, type: reviews }
---
## Summary

Ticket: TL-6. Failure-domain analysis of FR-100 (deadline settlement), FR-101 (captures) and FR-102 (unsupported constructs): axis confusion, identity confusion and boundary cases.

## Assurance Context

AP-001 (spec/assurance/AP-001-protocol-implementation.md) declares `review_policy.mode: require` but is `status: proposed`, so it is not an active profile and no enforced ix-flow run is claimed. Direct analyses run: base, integrity, ears-conformance, scope-boundary, dependency, failure-domain.

## Examined

- FR-100 (spec/functional/FR-100-settle-inclusive-deadline-branches-on-complete-progress.md)
- FR-100-AC-2 (spec/functional/FR-100-settle-inclusive-deadline-branches-on-complete-progress.md)
- FR-100-AC-4 (spec/functional/FR-100-settle-inclusive-deadline-branches-on-complete-progress.md)
- FR-101 (spec/functional/FR-101-bind-obligation-captures-as-ordered-atomic-occurrences.md)
- FR-101-AC-2 (spec/functional/FR-101-bind-obligation-captures-as-ordered-atomic-occurrences.md)
- FR-101-AC-3 (spec/functional/FR-101-bind-obligation-captures-as-ordered-atomic-occurrences.md)
- FR-102 (spec/functional/FR-102-carry-unsupported-protocol-constructs-as-typed-dispositions.md)

## Verdict

PASS WITH FINDINGS. FR-101's atomic-refusal and duplicate-occurrence edges are fully stated. FR-100 has three unstated failure modes: progress/completeness conflation, the meaning of 'late', and settlement identity across instances.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | FR-100 conditions timeout on progress being 'authoritatively complete through the deadline', but FR-028/FR-284 give progress only `open`/`closed` per scope with FR-268 authority, and completeness is the separate FR-295 axis that FR-028 forbids substituting for a progress or closure axis. State which facts are required (for example decision-scope progress `closed` through the deadline under FR-268 authority, plus input completeness `complete`), so an implementation cannot take timeout from a completeness fact alone. | spec/functional/FR-100-settle-inclusive-deadline-branches-on-complete-progress.md:25, 29, 37 |
| FND-002 | medium | 'A response after a settled timeout' does not say whether 'after' is admission order or observed time. A response observed after the deadline contradicts nothing; a response admitted late but observed at or before the deadline contradicts the progress premise and must be an FR-033 superseding or invalidating result. FR-100 names neither case nor FR-033. Separate the two cases and bind the second to FR-033. | spec/functional/FR-100-settle-inclusive-deadline-branches-on-complete-progress.md:37, 46 |
| FND-003 | medium | FR-100 settles 'one settlement per deadline branch, each naming the branch', but a branch is static: under repeated or `activation on each` waits one branch has many waiting instances. Key the settlement by the waiting instance (protocol/workflow instance, control occurrence, as QSpec FR-172 keys activations), or two instances' settlements collapse. | spec/functional/FR-100-settle-inclusive-deadline-branches-on-complete-progress.md:33 |

## New findings (disposition pass 1)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-004 | low | FR-100 Behavior names the absent-selection outcomes `pending` (FR-242) and `incomplete` without saying which axis carries `incomplete` (it is an FR-295 member, not an FR-242 or FR-241 one), and states none for completeness `contradicted`, which it forbids from selecting timeout. Name the axis for each and add the `contradicted` case. | spec/functional/FR-100-settle-inclusive-deadline-branches-on-complete-progress.md:49 |

## New findings (disposition pass 2)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-005 | medium | The round-2 fix maps completeness `contradicted` to `refused`. `refused` is an FR-241 execution value, but QSpec FR-241 defines it as 'the evaluator refused the request' (E02 invalid-input), and QSpec FR-061's result-combination rules (quire-specification 76efa51f) never derive it from an input-completeness fact. FR-061 handles a missing decision-support fact as truth `unavailable` with completeness retaining the fact, and a contradicted premise by FR-295 `contradicted` plus FR-033 supersession. The `incomplete` case likewise names no FR-242 truth value. Both no-selection cases should be execution `completed`, truth `unavailable` (FR-242), with completeness carrying the FR-295 value `incomplete` or `contradicted`. Fix FR-100 Behavior and FR-100-AC-2 to match. | spec/functional/FR-100-settle-inclusive-deadline-branches-on-complete-progress.md:49-50, 59 |

## New findings (disposition pass 3)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-006 | low | FR-100 Behavior gives truth `pending` when progress is `open` and truth `unavailable` when completeness is `incomplete` or `contradicted`, but does not say which applies when both hold (progress `open` with completeness `incomplete`). QSpec FR-061 resolves it (`pending` requires the required observations to be complete through the progress point, so `unavailable` wins), but FR-100 should say so, for example 'completeness `incomplete` or `contradicted` takes precedence over open progress', and FR-100-AC-2 should test that combination. | spec/functional/FR-100-settle-inclusive-deadline-branches-on-complete-progress.md:49-50 |

## Dispositions

Round 1, fix commit 2d40a60.

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | 2d40a60: When the awaited scope's progress (FR-284) is `closed` through the deadline and the input completeness (FR-295) is `complete`, and no matching observation was admitted, the evaluator SHALL select the timeout branch. |
| FND-002 | still-open | The fix picks arrival order and always emits an FR-033 superseding or invalidating result, which keeps the original defect. A matching response observed strictly after the deadline selects timeout under FR-100-AC-1 and contradicts no premise, so it is not an FR-033 trigger. FR-100-AC-4 nonetheless requires a superseding result when that same fact arrives after the selection, so batch and incremental runs over one history disagree (FR-041, FR-026). Supersede only when the late-arriving observation is observed at or before the deadline (it contradicts the progress premise); a response observed after the deadline leaves the timeout settled and emits nothing. |
| FND-003 | fixed | 2d40a60: Two waiting instances of one static branch, one answered and one not, settle differently and independently. |
| FND-002 | fixed | ddde4c0 (round 2): A matching response that arrives after timeout was selected, by position in the admitted history, but is observed at or before the deadline, yields a FR-033 superseding or invalidating result naming the earlier result and leaves its bytes and identity unchanged; a response observed after the deadline yields no such result, and batch and incremental runs agree on both. |
| FND-004 | fixed | ddde4c0 (round 2): Absent selection under completeness `contradicted` is `refused`, carrying the FR-295 `contradicted` value on the completeness axis. |
| FND-005 | fixed | 610d466 (round 3, rebased on main 9986cd8): Absent selection under completeness `incomplete` or `contradicted` is truth `unavailable` (FR-242) with execution `completed` (FR-241). |
| FND-006 | fixed | fa56b0f (round 4): When progress is `open` and completeness is `incomplete` or `contradicted` at once, the truth is `unavailable`, because `pending` requires complete observations (QSpec FR-061). |
