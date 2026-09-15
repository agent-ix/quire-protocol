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
  - target: ix://agent-ix/quire-specification/FR-300
    type: depends_on
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

A linked finite protocol representation (FR-002), QSL's strict version-two
temporal request/result facts, and QObs's authority-qualified admitted subject.
Protocol validates those owner facts against its linked definition and control,
then retains its constructor-private complete event-triggered `FR-300` activation
binding. The binding carries the exact qualified definition, declaration-local
control, clause subject, admitted semantic trigger, checked activation state,
immutable captures and evaluation anchor. Closure and completeness remain
owner-issued facts. A caller-built obligation label, control coordinate, trigger
identity, capture or anchor is not a production input.

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

The complete FR-300 binding is the sole event-triggered producer-to-consumer
bridge. Protocol validates that its qualified definition equals the linked
definition and that its control is in that graph before retaining the FR-230
instance it derives. A missing, foreign, cross-control, cross-subject,
cross-trigger or contradictory binding refuses; Protocol SHALL NOT reconstruct
one from independently supplied static and runtime fragments.

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
| Protocol-obligation activation binding | `FR-300-obligation-activation-binding` | value; selection key (qualified definition, control, clause subject, semantic trigger) |

`FR-055-AC-4` already states the substitution rule this requirement's AC-2
exercises: "Substituting O2's trigger, a changed capture or a different
evaluation anchor refuses the binding."

### Accepted owner ruling

QSpec #40 fixes the rule: a closed complete trigger scope with no admitted
trigger is `inactive`; an open scope or missing/refused trigger evidence is
`activation-unknown`. Completeness, execution and progress/closure remain
independent. A false admitted guard creates no obligation instance and no
activation value. The implementation imports that ruling and invents no
pre-report lifecycle state.

QSpec main `316743b` adds FR-300 and closes the event-triggered bridge: an
activation binding is the exact source for a dynamic FR-230 instance. This
library owns the graph-validated binding operation and does not define a
parallel QObs/QSL join. QSL `40c775d` extends FR-053's strict v2 views with the
checked activation state, immutable captures, evaluation anchor and FR-054
strict `/3` activation mapping; QSL retains
no QObs subject or protocol control, so it cannot construct the binding. The
opaque trigger remains byte-exact for comparison to QObs's qualified subject.

## Dependencies

- [FR-003](./FR-003-assess-compensation-state.md) aggregates this requirement.
- [FR-002](./FR-002-link-finite-protocol-representation.md) supplies the linked
  representation the trigger and captures are read from.
- `ix://agent-ix/quire-specification/FR-055` establishes activation, its
  independence from truth and the no-business-action rule.
- `ix://agent-ix/quire-specification/FR-300` establishes the complete
  event-triggered activation binding required before Protocol can construct an
  assessed obligation.
