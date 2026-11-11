---
id: SR-2127023
title: "Failure-domain analysis of TL-11 FR-053..FR-056"
type: SpecReview
analysis: failure-domain
scope: "agent-ix/quire-protocol; spec/functional/FR-050..FR-059, spec/test-cases/TC-050..TC-055, spec/tests.md (PR #143, head 00836cf)"
review_set: subset
date: 2026-11-11
---

## Summary

Ticket: TL-11. Unstated failure modes in the four assessment FRs, checked against the QSL constructs ADR-010 lists. Identity handling (no minting, branch counted once) and order invariance are stated well. Four edge cases are unspecified or over-refused.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-001 | medium | QSL's `Unsupported::UnruledOutstanding` is 'a join whose policy may leave branches running, with no `outstanding` rule' (ADR-010 G3). An `all` join can never leave branches running, yet FR-055 and AC-5 return the FR-057 disposition for any 'join with no `outstanding` rule', so a well-formed `all` join without the rule is never assessed. Restrict the unsupported path to policies that may leave branches outstanding (`any`, `quorum(n)`). | FR-055-AC-5; FR-055; FR-057; spec/functional/FR-055-assess-join-settlement-per-policy.md:51 | wrong-requirement |
| FND-002 | medium | A loop with no finite maximum is admissible (QSL `DomainKind::Loop`: 'a loop admitted with an invariant and a well-founded variant and no finite maximum'), and FR-052 links a loop's maximum without saying it is always present. FR-056 does not say what the loop assessment reports when the maximum is absent: no violation with the bound reported absent (as FR-053-AC-4 does for roles), or a disposition. Add the case and a criterion. | FR-056; FR-058; FR-052; spec/functional/FR-056-assess-loop-iteration-bound.md:21 | missing-requirement |
| FND-003 | medium | Lifetime violations are defined only as activity 'before its spawn or after its retirement'. In QSL a `workflow` lifetime ends with its holding instance and a `scope` lifetime ends when no running thread can reach a node the role owns (qsl-eval roles.rs); neither is a `retire` observation, and `until(p)` would need p evaluated. State how the end of a `scope`, `workflow` and `until(p)` lifetime is recognised in an observed history, or which kinds return an FR-057 disposition. | FR-053; FR-053-AC-2; FR-050; spec/functional/FR-053-assess-role-lifetime-and-population.md:35 | wrong-requirement |
| FND-004 | low | AC-3 judges `loss head` and `loss tail` outcomes per policy but does not exclude an `unordered` channel, where QSL refuses a head/tail selector (`Unsupported::UnorderedLossSelector`, ADR-010 G7). State that such a channel returns the FR-057 disposition in assessment as well, or restrict AC-3 to `fifo` channels. | FR-054-AC-3; FR-054; FR-057; spec/functional/FR-054-assess-queue-capacity-and-overflow-outcome.md:49 | wrong-requirement |

## Verdict

Findings above must be fixed or dispositioned before merge. Units examined:

- `FR-053` (examined)
- `FR-054` (examined)
- `FR-055` (examined)
- `FR-056` (examined)
- `FR-053-AC-1` (examined)
- `FR-053-AC-2` (examined)
- `FR-053-AC-3` (examined)
- `FR-053-AC-4` (examined)
- `FR-053-AC-5` (examined)
- `FR-054-AC-1` (examined)
- `FR-054-AC-2` (examined)
- `FR-054-AC-3` (examined)
- `FR-054-AC-4` (examined)
- `FR-054-AC-5` (examined)
- `FR-055-AC-1` (examined)
- `FR-055-AC-2` (examined)
- `FR-055-AC-3` (examined)
- `FR-055-AC-4` (examined)
- `FR-055-AC-5` (examined)
- `FR-056-AC-1` (examined)
- `FR-056-AC-2` (examined)
- `FR-056-AC-3` (examined)
- `FR-057` (context_only)

## New findings (disposition pass 1)

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-005 | medium | The new `until(p)` end rule ('at the observation at which `p` is observed to hold') contradicts QSL. In qsl-eval roles.rs an `until(p)` lifetime 'ends in a `retire` step': p holding only enables a `retire` (role_steps), and the instance stays live until that retire is taken. Activity between p first holding and the retire is permitted, so FR-053-AC-6 would report false lifetime violations. The rule also makes the library evaluate p, which FR-053 never says how. Fix: an `until(p)` lifetime ends at its observed retire, and a retire observed while p is not known to hold is not judged here, or returns FR-057. | FR-053; FR-053-AC-6; TC-051; spec/functional/FR-053-assess-role-lifetime-and-population.md:35 | wrong-requirement |
| FND-006 | medium | The `scope` end rule ('no branch able to reach a node the role owns') drops half of QSL's RR-4 (roles.rs retire_scoped): a scope role also stays live while a running thread can reach the `effect` of a compensation template the role owns, or while a registration of such a template is Registered or Activated. Under the FR-053 rule, a compensation attempt by a scope role after its branches move on is reported as a lifetime violation that QSL permits. It also restates step semantics, which FR-053 says it does not. Fix: cite ADR-027 RR-4 for the end of a `scope` lifetime, including owned compensation templates and open registrations, and add that case to AC-6. | FR-053-AC-6; FR-053; TC-051; spec/functional/FR-053-assess-role-lifetime-and-population.md:52 | wrong-requirement |

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | Round 1, commit 5ef24e3: Over an `all` join with no `outstanding` rule, a history settling with every branch complete reports no violation and no outstanding branch, and a history settling with a branch missing reports a join violation. |
| FND-002 | fixed | Round 1, commit 5ef24e3: Over a loop with no authored maximum and a history of any iteration count, the assessment reports the observed count, reports the maximum as absent and reports no loop-bound violation. |
| FND-003 | fixed | Round 1, commit 5ef24e3: - The evaluator shall take the end of a `workflow` lifetime at the observation that ends the workflow instance holding the role, the end of a `scope` lifetime at the first observation after which the linked graph leaves no branch able to reach a node the role owns, and the end of an `until(p)` lifet |
| FND-004 | fixed | Round 1, commit 5ef24e3: Over an `unordered` channel, any delivery order of the same messages reports no order violation; over an `unordered` channel whose overflow loses `head` or `tail`, the assessment returns the FR-057 disposition for that channel. |
| FND-005 | fixed | Round 2, commit 34ec96c: - The evaluator shall take the end of a `workflow` lifetime at the observation that ends the workflow instance holding the role, the end of a `scope` lifetime at the observation of the step that QSL ADR-027 RR-4 ends it by (no running thread can reach a node the role owns, the effect of a compensati |
| FND-006 | fixed | Round 2, commit 34ec96c: Over one role per lifetime kind, activity after the observation that ends the workflow, after the observed end of a `scope` lifetime and after the observed `retire` of an `until(p)` instance is reported as a lifetime violation for `workflow`, `scope` and `until(p)` respectively, and activity before  |
| FND-007 | fixed | Round 3, df16bc9: ...the checked role node, the workflow instance the role instance belongs to, the role instance identity the observation names... - The evaluator shall judge each role once for each workflow instance the history names: the authored `max` bounds the live instances of one workflow instance, and a workflow end ends only that workflow instance's role instances. (FR-053-AC-11) |

## New findings (quire-protocol#147 spec delta)

Spec delta of quire-protocol#147 at ecee04fb29b155ea52a4c4f576882992d03f85da (`git diff b9c6e01..ecee04f -- spec/`): FR-050, FR-051, FR-053..FR-056, TC-050..TC-054, spec/tests.md.

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-007 | medium | Amended FR-053 Inputs define RoleFact without a workflow instance, merging population and workflow-end across workflow instances | spec/functional/FR-053-assess-role-lifetime-and-population.md:Inputs |
