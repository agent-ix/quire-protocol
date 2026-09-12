---
id: FR-021
title: "Distinguish compensation commit-ordering violations"
type: FR
relationships:
  - target: ix://agent-ix/quire-protocol/FR-003
    type: part_of
  - target: ix://agent-ix/quire-protocol/FR-020
    type: depends_on
  - target: ix://agent-ix/quire-specification/FR-057
    type: depends_on
  - target: ix://agent-ix/quire-specification/FR-233
    type: references
  - target: ix://agent-ix/quire-specification/FR-234
    type: references
  - target: ix://agent-ix/quire-specification/FR-237
    type: references
---
# FR-021: Distinguish compensation commit-ordering violations

## Description

When a compensation registration is evaluated against the authored commit ordering, the evaluator SHALL report registration after a forbidding commit, compensation before its paired forward effect, and compensation after a forbidding commit as three distinct typed ordering violations.
When compensation is observed before its paired forward effect, the evaluator SHALL NOT let that observation validate the registration retroactively.
When a commit observation required by the ordering check is absent, the evaluator SHALL report incomplete or pending rather than an ordering result.
When the commit ordering is read, the evaluator SHALL take it from the authored causal order rather than from a key order, a timestamp or a source position.

## Inputs

A compensation registration typed by
`ix://agent-ix/quire-specification/FR-233`, commit observations against the
commit boundary typed by `ix://agent-ix/quire-specification/FR-237`, compensation
observations, and the authored causal order carried by the linked representation
of [FR-002](./FR-002-link-finite-protocol-representation.md).

## Outputs

Commit facts held separately from attempt, effect and recovery facts, carrying
one of the three distinct ordering violations, an incomplete or pending outcome,
or no ordering violation.

## Behavior

The three violations are named upstream and are not merged here. `FR-057-AC-3`
states that "Registration after commit and compensation before the forward
effect or after a forbidding commit are distinct ordering violations", and
`FR-056-AC-4` that compensation observed before its forward effect "cannot
retroactively validate registration".

The ordering is causal, never a key order. `FR-237` records that "the commit
ordering checked by FR-057 is a causal order, not a key order, and must not be
read as one".

Absence yields incompleteness, not a permitting default. `FR-057-AC-4` states
that "Missing state, commit or compensation-effect inputs yield
incomplete/pending rather than recovery success", and `FR-237` records that an
absent commit "is never compared to a default, an empty commit or an implicit
permitting commit".

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-021-AC-1 | Registration after a forbidding commit, compensation before its paired forward effect, and compensation after a forbidding commit are reported as three mutually distinguishable typed violations. | Test (TC-003) |
| FR-021-AC-2 | Compensation observed before its paired forward effect leaves the registration invalid and does not retroactively validate it. | Test (TC-003) |
| FR-021-AC-3 | An absent commit observation yields incomplete or pending, compares only to absence, and is not read as a permitting commit. | Test (TC-003) |
| FR-021-AC-4 | Reordering equal-time observations, changing a key order or changing a source position adds no ordering violation and removes none. | Test (TC-003) |

## Adopted vocabulary

| Noun | Upstream object | Identity kind |
| --- | --- | --- |
| Commit boundary | `FR-237-commit-boundary` | authored in ordering only; key not established upstream |
| Compensation registration | `FR-233-compensation-registration` | derived, keyed (source compensation definition identity, established forward effect identity) |
| Compensation obligation lifecycle | `FR-234-compensation-obligation-lifecycle` | value |

### Where the upstream vocabulary does not yet reach

`FR-237`'s open gap lists, verbatim:

- "**Identity** — whether a commit boundary is a distinct identified subject or
  only a position in the authored order"
- "**Key** — which components identify one commit boundary or one commit
  observation"
- "**Order** — no key order follows, since the components are unfixed."

Without a commit-observation key, AC-1 can distinguish the three violations by
their reported kind but cannot name *which* commit forbade a compensation. This
requirement names that gap and supplies no key. The owner is
`agent-ix/quire-specification`.

`FR-237`'s open gap also records a contradiction in the forbidding condition:
FR-057 Behavior reads "registration after a forbidding commit", while
FR-057-AC-3 reads "Registration after commit". `FR-237` states that "Whether
every commit forbids registration, or only a commit with a forbidding
disposition, is not stated." AC-1 above carries FR-003's existing wording
unchanged and selects neither reading.

## Dependencies

- [FR-003](./FR-003-assess-compensation-state.md) aggregates this requirement.
- [FR-020](./FR-020-register-compensation-once-per-effect.md) supplies the
  registration whose ordering is checked.
- `ix://agent-ix/quire-specification/FR-057` establishes the commit-ordering
  check and the three distinct violations.
