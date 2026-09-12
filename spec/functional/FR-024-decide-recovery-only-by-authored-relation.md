---
id: FR-024
title: "Decide recovery only by the authored recovery relation"
type: FR
relationships:
  - target: ix://agent-ix/quire-protocol/FR-003
    type: part_of
  - target: ix://agent-ix/quire-protocol/FR-023
    type: depends_on
  - target: ix://agent-ix/quire-specification/FR-057
    type: depends_on
  - target: ix://agent-ix/quire-specification/FR-058
    type: depends_on
  - target: ix://agent-ix/quire-specification/FR-238
    type: references
  - target: ix://agent-ix/quire-specification/FR-263
    type: references
  - target: ix://agent-ix/quire-specification/FR-264
    type: references
  - target: ix://agent-ix/quire-specification/FR-268
    type: references
---
# FR-024: Decide recovery only by the authored recovery relation

## Description

When a compensation's recovery is decided, the evaluator SHALL evaluate only that compensation's exact authored typed recovery relation over the retained target state and achieved state.
When a compensation operation or a compensation attempt succeeds, the evaluator SHALL NOT infer recovery success from that success.
When a recovery relation aggregates related effects, the evaluator SHALL require the exact declared population scope, membership rule and completeness authority before reporting success.
When a target state, achieved state, commit, compensation effect, population member, membership authority or completeness authority required by the relation is missing, the evaluator SHALL report incomplete or pending rather than recovery success.
When an achieved state satisfies an authored partial-recovery relation, the evaluator SHALL report that satisfaction independently of whether full restoration holds.

## Inputs

A compensation registration and its authored recovery relation typed by
`ix://agent-ix/quire-specification/FR-238`, the retained target and achieved
states, the immutable captures of
`ix://agent-ix/quire-specification/FR-232`, the declared finite population of
`ix://agent-ix/quire-specification/FR-263` with its membership rule
`ix://agent-ix/quire-specification/FR-264`, and the progress and closure
authority of `ix://agent-ix/quire-specification/FR-268`.

## Outputs

Recovery facts held separately from attempt, effect and commit facts: a recovery
decision produced by the authored relation, or an incomplete or pending outcome
naming the missing input.

## Behavior

`FR-057` states that the evaluator "SHALL NOT infer recovery success from
compensation-operation success" and that it "SHALL retain the target and achieved
states used by the recovery relation". `FR-057-AC-1` states that "A full refund
effect before commit satisfies a full-restoration relation only when the captured
amount and target state match", and `FR-057-AC-2` that "A successful partial
refund can satisfy an authored partial-recovery relation while failing full
restoration".

Full restoration and partial recovery are separate authored relations, not two
points on a scale. `FR-238-AC-2` states that neither ranks above the other.

`FR-058` states that "Where recovery aggregates related effects, it SHALL require
the exact declared population scope, membership and completeness authorities
before success", and `FR-058-AC-3` that "Two related partial refunds satisfy the
captured amount only under a complete declared population with exact
arithmetic". `FR-058-AC-4` states that "Unknown or missing population membership
and attempts beyond the retry bound cannot produce recovery success", and
`FR-057-AC-4` that "Missing state, commit or compensation-effect inputs yield
incomplete/pending rather than recovery success".

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-024-AC-1 | One achieved state satisfies an authored partial-recovery relation while failing an authored full-restoration relation over the same captured target, and neither relation ranks above the other. | Test (TC-003) |
| FR-024-AC-2 | A recovery success requires the exact authored relation, the exact captured target state and the exact achieved state; independently varying any of the three withdraws the success. | Test (TC-003) |
| FR-024-AC-3 | An aggregating recovery relation reports success only under a complete declared population with its exact membership rule, exact arithmetic over the captured amount, and applicable completeness authority. | Test (TC-003) |
| FR-024-AC-4 | A missing target state, achieved state, commit, compensation effect, population member, membership authority or completeness authority yields incomplete or pending rather than recovery success. | Test (TC-003) |
| FR-024-AC-5 | A successful compensating-operation invocation and a `successful` attempt outcome each produce no recovery success on their own. | Test (TC-003) |

## Adopted vocabulary

| Noun | Upstream object | Identity kind |
| --- | --- | --- |
| Authored recovery relation | `FR-238-recovery-relation` | authored; key not established upstream |
| Declared finite population | `FR-263-declared-population` | entity, retained per FR-112's identity set |
| Declared membership rule | `FR-264-membership-rule` | value, keyed (rule identity, rule digest) |
| Progress and late-data authority | `FR-268-progress-and-late-data-authority` | value |
| Immutable activation capture | `FR-232-activation-capture` | value |

### Where the upstream vocabulary does not yet reach

- **Recovery relation key.** `FR-238`'s open gap states that the prose
  establishes a recovery relation is authored, typed, per compensation and
  evaluated under selected profiles, and then that "It does not establish:" the
  key components that identify one, nor whether the selected state and temporal
  profile identities participate. AC-2 varies the authored relation
  without a key that says when two relations are the same relation. The owner is
  `agent-ix/quire-specification`.
- **Whether relation types are a closed set.** `FR-238` records that FR-057 names
  a full-restoration relation and an authored partial-recovery relation inside
  acceptance criteria, and that "No requirement states that these are a closed
  set of relation types, so they are not enumerated here." AC-1 exercises those
  two without asserting they are the only two.
- **Duplicate effect inside an aggregating population.** `FR-226` states that
  two duplicate effects compare equal under its declared key, so an aggregating
  relation summing related effects cannot tell a duplicated effect from a second
  member. AC-3's exact arithmetic rests on a member identity upstream has not
  supplied.

## Dependencies

- [FR-003](./FR-003-assess-compensation-state.md) aggregates this requirement.
- [FR-023](./FR-023-preserve-attempt-and-effect-distinctness.md) supplies the
  attempt and effect facts a recovery decision reads but never infers from.
- `ix://agent-ix/quire-specification/FR-057` establishes the typed recovery
  relation and its retained states.
- `ix://agent-ix/quire-specification/FR-058` establishes the declared-population
  requirement for aggregating recovery.
