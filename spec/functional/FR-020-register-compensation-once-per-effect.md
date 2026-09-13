---
id: FR-020
title: "Register compensation once per established forward effect"
type: FR
relationships:
  - target: ix://agent-ix/quire-protocol/FR-003
    type: part_of
  - target: ix://agent-ix/quire-protocol/FR-019
    type: depends_on
  - target: ix://agent-ix/quire-specification/FR-056
    type: depends_on
  - target: ix://agent-ix/quire-specification/FR-226
    type: references
  - target: ix://agent-ix/quire-specification/FR-233
    type: references
  - target: ix://agent-ix/quire-specification/FR-234
    type: references
---
# FR-020: Register compensation once per established forward effect

## Description

When an authored forward effect is established, the evaluator SHALL create exactly the compensation registrations paired with that effect and retain the source compensation definition and the forward effect identity on each.
When a send, receive, delivery or attempt lacks the required successful effect, the evaluator SHALL create no compensation registration from it.
When one established forward effect identity is observed again, the evaluator SHALL yield the same registration rather than an additional one.
When a registration is created, the evaluator SHALL NOT invoke the compensating operation.

## Inputs

An admitted compensation definition, a successfully established forward effect
typed by `ix://agent-ix/quire-specification/FR-226` with its exact
workflow/effect identity, and the triggered obligation of
[FR-019](./FR-019-derive-obligation-activation-state.md).

## Outputs

Compensation registrations typed by
`ix://agent-ix/quire-specification/FR-233`, each retaining its source definition
and forward effect identity; or a typed violation or refusal.

## Behavior

Registration is keyed, not counted. `FR-233` declares its key as the source
compensation definition identity together with the established forward effect
identity, and `FR-056` states that "Repeated observation of one effect SHALL NOT
create additional registrations" and that the evaluator "SHALL NOT register
compensation from a send, receive, delivery or attempt that lacks the required
successful effect". The replay rule of this requirement is exactly that key; it
is adopted, not restated.

An attempt with no established effect leaves the forward effect identity absent.
`FR-233` records that absence is never compared to a default or empty effect
identity, so a registration key with an absent effect component compares only to
absence.

The registration's lifecycle position is
`ix://agent-ix/quire-specification/FR-234`. This requirement establishes no
lifecycle labels of its own.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-020-AC-1 | One paired successful forward effect creates exactly one matching compensation registration, positioned after that effect. | Test (TC-003) |
| FR-020-AC-2 | A send, a receive, a duplicate delivery and a failed attempt each create no compensation registration. | Test (TC-003) |
| FR-020-AC-3 | Replaying one established forward effect identity yields the already-created registration and no second one. | Test (TC-003) |
| FR-020-AC-4 | Each registration retains its source compensation definition identity and its forward effect identity, and a registration whose forward effect identity is absent compares only to absence. | Test (TC-003) |
| FR-020-AC-5 | No compensating-operation invocation occurs on any registration path. | Test (TC-003) |

## Adopted vocabulary

| Noun | Upstream object | Identity kind |
| --- | --- | --- |
| Compensation registration | `FR-233-compensation-registration` | derived, keyed (source compensation definition identity, established forward effect identity) |
| Compensation obligation lifecycle | `FR-234-compensation-obligation-lifecycle` | value |
| Business effect | `FR-226-business-effect` | derived, keyed (static declaration path, workflow instance identity, iteration identity) |

### Where the upstream vocabulary does not yet reach

- **Whether the definition identity is a required key component.** `FR-233`'s
  open gap records that no requirement states whether one effect may pair with
  more than one compensation definition, so it is not established whether the
  definition identity participates in the key or whether the effect identity
  alone is the key. Both readings are recorded upstream and neither is selected
  here; AC-3's replay rule holds under either, AC-1's "exactly one" does not.
  The owner is `agent-ix/quire-specification`.
- **Detecting a duplicate forward effect.** `FR-226` states that its declared
  key "is constant across two effect claims at one static node within one
  workflow instance, so two duplicate effects compare equal under it and the
  duplicate the evaluator is required to detect is undetectable". AC-3
  distinguishes a replay of one effect identity from a second effect only where
  such a component exists; upstream has not supplied one.

## Dependencies

- [FR-003](./FR-003-assess-compensation-state.md) aggregates this requirement.
- [FR-019](./FR-019-derive-obligation-activation-state.md) supplies the activated
  obligation a registration is paired against.
- `ix://agent-ix/quire-specification/FR-056` establishes registration, its
  retained components, the duplicate-registration rule and the rule that the
  evaluator "does not invoke the compensating operation".
