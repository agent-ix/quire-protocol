---
id: FR-022
title: "Enforce the authored finite compensation retry bound"
type: FR
relationships:
  - target: ix://agent-ix/quire-protocol/FR-003
    type: part_of
  - target: ix://agent-ix/quire-protocol/FR-020
    type: depends_on
  - target: ix://agent-ix/quire-specification/FR-058
    type: depends_on
  - target: ix://agent-ix/quire-specification/FR-235
    type: references
  - target: ix://agent-ix/quire-specification/FR-239
    type: references
---
# FR-022: Enforce the authored finite compensation retry bound

## Description

When a compensation attempt would exceed the authored finite retry bound of its registration, the evaluator SHALL refuse that attempt as a violation distinct from every ordering violation.
When an attempt is refused for exceeding the bound, the evaluator SHALL leave the attempts already retained for that registration unchanged.
When an attempt lies beyond the bound, the evaluator SHALL NOT produce recovery success from it.
When a registration carries no authored retry bound, the evaluator SHALL treat the bound as absent rather than as zero, as unbounded or as an ambient default.

## Inputs

A compensation registration typed by
`ix://agent-ix/quire-specification/FR-233`, its retry policy typed by
`ix://agent-ix/quire-specification/FR-239`, and the compensation attempts typed
by `ix://agent-ix/quire-specification/FR-235`.

## Outputs

Per-attempt admission or refusal against the bound, with the retained attempt
set unchanged by a refusal, and a bound-refusal violation held distinct from the
ordering violations of
[FR-021](./FR-021-distinguish-commit-ordering-violations.md).

## Behavior

`FR-058` states that the evaluator "SHALL refuse attempts beyond the authored
finite retry bound", and `FR-058-AC-4` that "Unknown or missing population
membership and attempts beyond the retry bound cannot produce recovery success".
`FR-239-AC-2` states that "Refusing an attempt beyond the bound leaves the
previously retained attempts unchanged".

The bound counts the first attempt. `FR-049-AC-6` states, verbatim, "Retry bounds
count the initial attempt". This requirement adopts that counting rule and
defines no other.

The bound is not an order. `FR-239` records that no order over retry policies is
declared and that a larger bound is not to be read as a later or stronger
policy.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-022-AC-1 | An attempt sequence exactly at the authored finite bound is admitted, and the next attempt is refused as a bound violation distinct from every ordering violation. | Test (TC-003) |
| FR-022-AC-2 | The bound counts the initial attempt, so a bound of n admits n attempts rather than n retries after a first. | Test (TC-003) |
| FR-022-AC-3 | A refused over-bound attempt changes none of the attempts already retained for that registration. | Test (TC-003) |
| FR-022-AC-4 | An attempt beyond the bound produces no recovery success on any path. | Test (TC-003) |
| FR-022-AC-5 | A registration with no authored retry bound compares only to absence and is never evaluated against zero, an unbounded policy or a default. | Test (TC-003) |

## Adopted vocabulary

| Noun | Upstream object | Identity kind |
| --- | --- | --- |
| Compensation retry policy | `FR-239-compensation-retry-policy` | value, keyed (authored finite retry bound) |
| Compensation attempt | `FR-235-compensation-attempt` | authored exact FR-287 `compensation-attempt` key |

### Accepted owner ruling

Quire Specification #28/#31 defines the attempt as an authored FR-287 key. The
bound counts distinct admitted keys under one registration, including the
initial attempt and every admitted outcome. Exact replay counts once. An
over-bound candidate refuses before retention and leaves the prior count
unchanged. Only the authored predecessor/successor relation orders retries.

## Dependencies

- [FR-003](./FR-003-assess-compensation-state.md) aggregates this requirement.
- [FR-020](./FR-020-register-compensation-once-per-effect.md) supplies the
  registration the retry policy is bound to.
- `ix://agent-ix/quire-specification/FR-058` establishes the bounded retry policy
  and the refusal rule.
- `ix://agent-ix/quire-specification/FR-239` types the policy and exact counting
  rule.
