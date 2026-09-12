---
id: FR-019
title: "Derive obligation activation state"
type: FR
relationships:
  - target: ix://agent-ix/quire-protocol/FR-003
    type: part_of
  - target: ix://agent-ix/quire-protocol/FR-002
    type: depends_on
  - target: ix://agent-ix/quire-specification/FR-055
    type: depends_on
  - target: ix://agent-ix/quire-specification/FR-230
    type: references
  - target: ix://agent-ix/quire-specification/FR-231
    type: references
  - target: ix://agent-ix/quire-specification/FR-232
    type: references
  - target: ix://agent-ix/quire-specification/FR-244
    type: references
  - target: ix://agent-ix/quire-specification/FR-253
    type: references
---
# FR-019: Derive obligation activation state

## Description

When a linked protocol carries an admitted trigger observation for an obligation, the evaluator SHALL report that obligation's activation as exactly one member of the `ix://agent-ix/quire-specification/FR-244` vocabulary.
When the trigger scope is authoritatively closed and complete with no admitted trigger, the evaluator SHALL report `inactive` rather than absence, a default or a truth value.
When the trigger scope is open, or trigger evidence is missing or refused, the evaluator SHALL report `activation-unknown` rather than `inactive`.
When any of the obligation's workflow, role, effect, predicate, capture or evaluation-anchor identities is substituted, the evaluator SHALL refuse the activation binding.
When activation is `inactive` or `activation-unknown`, the evaluator SHALL NOT report the obligation as satisfied or demonstrated.
When activation is derived, the evaluator SHALL execute no business action.

## Inputs

A linked finite protocol representation (FR-002), admitted trigger observations,
immutable activation captures typed by
`ix://agent-ix/quire-specification/FR-232`, the evaluation anchor typed by
`ix://agent-ix/quire-specification/FR-253`, and the closure and completeness
authorities owned upstream.

## Outputs

One activation state per obligation instance, drawn from
`ix://agent-ix/quire-specification/FR-244`, carrying the obligation's exact
retained dependency identities; or a refusal naming the substituted identity
axis.

## Behavior

Activation is independent from truth. `FR-055` states that the evaluator "SHALL
keep activation independent from truth" and that it "SHALL NOT treat an inactive
or unknown obligation as satisfied or demonstrated"; `FR-055` Outputs adds that
"no business action is executed". This requirement adopts those obligations for
the consumer and adds none.

The obligation instance being reported on is
`ix://agent-ix/quire-specification/FR-230`; its activation label vocabulary is
`ix://agent-ix/quire-specification/FR-244`; the machine that enters each label is
`ix://agent-ix/quire-specification/FR-231`. This requirement re-spells none of
them.

Absence is not a state. An obligation whose activation has not been reported has
an absent activation state, and absence is never defaulted to `inactive`, which
is itself a positive assertion about closure and completeness.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-019-AC-1 | An established admitted trigger reports `triggered`, an absent trigger under authoritatively closed and complete input reports `inactive`, and an unavailable trigger under open or incomplete input reports `activation-unknown`. | Test (TC-003) |
| FR-019-AC-2 | Independently substituting the workflow, role, effect, predicate, capture or evaluation-anchor identity refuses the activation binding and names the substituted axis. | Test (TC-003) |
| FR-019-AC-3 | An `inactive` or `activation-unknown` activation is readable as neither satisfied nor demonstrated, and neither is evidence that recovery behavior was exercised. | Test (TC-003) |
| FR-019-AC-4 | An obligation with no reported activation compares only to absence and receives no default activation value. | Test (TC-003) |
| FR-019-AC-5 | No business-action invocation occurs on any activation path, including the refusing ones. | Test (TC-003) |

## Adopted vocabulary

| Noun | Upstream object | Identity kind |
| --- | --- | --- |
| Protocol obligation instance | `FR-230-protocol-obligation` | derived, keyed (clause subject, semantic trigger-event identity) |
| Obligation activation state machine | `FR-231-obligation-activation-state` | value |
| Activation state label vocabulary | `FR-244-activation-state` | value |
| Immutable activation capture | `FR-232-activation-capture` | value |
| Temporal evaluation anchor | `FR-253-temporal-evaluation-anchor` | value |

`FR-055-AC-4` already states the substitution rule this requirement's AC-2
exercises: "Substituting O2's trigger, a changed capture or a different
evaluation anchor refuses the binding."

### Where the upstream vocabulary does not yet reach

- **Whether an unavailable required trigger is an activation value or a
  completeness value.** `FR-231`'s open gap records three requirements that do
  not agree, and states that "The selection belongs to an owner ruling, not to
  this artifact." `FR-055` Behavior reads one axis with two values: "An absent
  trigger under complete closed input yields inactive; an unavailable required
  trigger yields activation-unknown or incomplete." `FR-061`, as quoted by
  `FR-231`, reads two retained facts: "Activation is activation-unknown | Truth
  is unavailable and completeness is open or incomplete with the missing trigger
  dependencies." AC-1 above carries FR-003's existing wording unchanged and
  selects neither reading. The owner is `agent-ix/quire-specification`.
- **The pre-report state of an obligation.** `FR-231`'s open gap states that
  "`scope_linked` is authored scaffolding that no requirement establishes", and
  that no requirement names a state an obligation occupies before its activation
  is reported. This requirement therefore reports activation without naming a
  prior state.
- **The key of a whole-execution-origin obligation instance.** `FR-230`'s open
  gap records that it is not established whether such an instance carries a
  trigger-event identity component at all. AC-4's absence comparison rests on a
  key upstream has not fixed for that case.

## Dependencies

- [FR-003](./FR-003-assess-compensation-state.md) aggregates this requirement.
- [FR-002](./FR-002-link-finite-protocol-representation.md) supplies the linked
  representation the trigger and captures are read from.
- `ix://agent-ix/quire-specification/FR-055` establishes activation, its
  independence from truth and the no-business-action rule.
