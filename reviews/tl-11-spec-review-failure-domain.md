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
