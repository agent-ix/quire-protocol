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

Registration is keyed, not counted. `FR-233` owns the registration key — the
source compensation definition identity together with the established forward
effect identity — and `FR-056` owns the registration rule. This requirement
adopts both: repeated observation of one effect creates no additional
registration here, and no registration is created from a send, receive, delivery
or attempt lacking the required successful effect. The replay rule of this
requirement is exactly that key; it is adopted, not restated.

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
| Compensation registration | `FR-233` | derived, keyed (source compensation definition identity, established forward effect identity) |
| Compensation obligation lifecycle | `FR-234` | value |
| Business effect | `FR-226` | derived, keyed (static declaration path, workflow instance identity, iteration identity) |

### Accepted owner ruling

QSpec #28/#31 fixes registration to the qualified FR-288 compensation
definition plus the exact FR-287 effect key. Exact replay is idempotent; a
distinct effect remains distinct even at the same static node. The
implementation imports those keys and never detects duplication from payload,
timestamp, source position or display text.

## Dependencies

- [FR-003](./FR-003-assess-compensation-state.md) aggregates this requirement.
- [FR-019](./FR-019-derive-obligation-activation-state.md) supplies the activated
  obligation a registration is paired against.
- `ix://agent-ix/quire-specification/FR-056` establishes registration, its
  retained components, the duplicate-registration rule, and that assessing it
  invokes no compensating operation.
