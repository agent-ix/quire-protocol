---
id: FR-055
title: "Assess join settlement per policy"
type: FR
relationships:
  - { target: ix://agent-ix/quire-protocol/US-001, type: implements }
  - { target: ix://agent-ix/quire-protocol/ADR-010, type: depends_on }
  - { target: ix://agent-ix/quire-protocol/FR-004, type: references }
  - { target: ix://agent-ix/quire-protocol/FR-027, type: references }
  - { target: ix://agent-ix/quire-protocol/FR-028, type: references }
  - { target: ix://agent-ix/quire-protocol/FR-041, type: references }
  - { target: ix://agent-ix/quire-specification/FR-170, type: references }
  - { target: ix://agent-ix/quire-specification/FR-171, type: references }
  - { target: ix://agent-ix/quire-spec-language/ADR-027, type: references }
  - { target: ix://agent-ix/quire-protocol/FR-052, type: depends_on }
  - { target: ix://agent-ix/quire-protocol/FR-057, type: depends_on }
  - { target: ix://agent-ix/quire-spec-language/FR-384, type: depends_on }
---
# FR-055: Assess join settlement per policy

## Description

When an admitted observation history is assessed against a linked protocol, the evaluator SHALL judge each observed join's settlement against its authored policy and threshold. The evaluator SHALL report the outcome of the branches left outstanding.

## Inputs

A linked protocol with its join declarations (FR-052), and an admitted observation fact set, given to the assessment as the library's `JoinFact` values: each names the observation, the `parallel` control, the workflow instance, one repeat ordinal for each enclosing `repeat` as QSL's occurrence key binds them, and one of branch completed, settled or outstanding branch outcome.

## Outputs

Per join occurrence: the set of observed completed branches, whether the join settled, whether the observed settlement is permitted by the authored policy and threshold, and the observed outcome of each outstanding branch; plus the typed adverse observations.

## Behavior

- The evaluator shall treat `all` as permitting settlement only when every branch completed, `any` when at least one completed, and `quorum(n)` when at least n distinct branches completed.
- The evaluator shall count a branch once.
- The evaluator shall not count a branch instance the history names twice as two.
- The evaluator shall report settlement before the threshold was observed as a join violation naming the join, policy, threshold and observed count.
- The evaluator shall judge each outstanding branch against the authored `outstanding` rule (`continue` or `cancel`).
- The evaluator shall report a branch whose observed outcome differs as an outstanding violation.
- For a join of policy `any` or `quorum(n)` with no authored `outstanding` rule, the evaluator shall return the disposition of FR-057.
- For an `all` join with no authored `outstanding` rule, the evaluator shall assess settlement, because settlement requires every branch and none remains outstanding.
- For a `predicate` join the evaluator shall return the disposition of FR-057.
- For a `predicate` join the evaluator shall not evaluate the predicate.
- The evaluator shall keep one occurrence for each workflow instance and repeat ordinals.
- The evaluator shall refuse a history whose occurrence key QSL's occurrence schema refuses, naming QSL's refusal.
- The evaluator shall refuse a history whose branch label the join does not list.
- The evaluator shall refuse a history that names a node that is not a declared join, naming the node.
- The evaluator shall refuse a history whose workflow instance identity QSL's occurrence key cannot take.
- QSL takes a workflow identity as non-empty UTF-8 text of at most 4096 bytes, while this library's identities are exact bytes; the evaluator shall pass the bytes as text where they are UTF-8.
- The evaluator shall not repair or lossily convert a workflow identity that is not such text.
- This requirement states no step semantics. QSL owns the exploration semantics (QSL ADR-027); this library assesses one admitted observation history (ADR-010 Decision 2).

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-055-AC-1 | Over an `all` join, a history settling with every branch complete reports no violation, and a history settling with one branch missing reports a join violation. | Test (TC-053) |
| FR-055-AC-2 | Over an `any` join, settlement after the first completed branch reports no violation, and settlement with none completed reports a join violation. | Test (TC-053) |
| FR-055-AC-3 | Over a `quorum(n)` join, histories with n-1, n and n+1 distinct completed branches report a violation only for n-1; a branch counted twice in the history is counted once. | Test (TC-053) |
| FR-055-AC-4 | Over joins authored `outstanding continue` and `outstanding cancel`, a history whose outstanding branches behave as authored reports none, and one whose branches behave otherwise reports an outstanding violation. | Test (TC-053) |
| FR-055-AC-5 | Over a `predicate` join and over an `any` and a `quorum(n)` join with no `outstanding` rule, the assessment returns the FR-057 disposition and reports neither a settled nor an unsettled join. | Test (TC-053) |
| FR-055-AC-6 | Over an `all` join with no `outstanding` rule, a history settling with every branch complete reports no violation and no outstanding branch, and a history settling with a branch missing reports a join violation. | Test (TC-053) |
| FR-055-AC-7 | A fact naming a branch label the join does not list refuses the history with the join and label named. | Test (TC-053) |
| FR-055-AC-8 | Over a join inside a `repeat`, histories settling in two iterations are two occurrences, each judged alone, and an ordinal outside the repeat's maximum or a missing ordinal refuses the history. | Test (TC-053) |
| FR-055-AC-9 | A workflow instance identity that is not UTF-8, is empty or exceeds 4096 bytes refuses the history with QSL's refusal or the non-text identity named, and an identity of exactly 4096 UTF-8 bytes is accepted. | Test (TC-053) |
| FR-055-AC-10 | A fact naming a node that is not a declared join refuses the history with the node named. | Test (TC-053) |

## Dependencies

- [ADR-010](../adr/ADR-010-consume-qsl-protocol-analysis-own-assessment-and-dispatch.md) fixes what this library owns for dynamic roles, bounded queues, joins and loops, and what QSL owns.
- QSL ADR-027 states the step semantics this requirement does not restate; QSpec FR-170 and FR-171 state the thresholds and overflow rules the criteria cite.
- QSL FR-384 states the occurrence schema and the runtime key this requirement reads and binds against; QSL owns both.
