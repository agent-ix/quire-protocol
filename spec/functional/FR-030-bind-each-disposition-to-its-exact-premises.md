---
id: FR-030
title: "Bind each disposition to its exact premises"
type: FR
relationships:
  - target: ix://agent-ix/quire-protocol/FR-005
    type: part_of
  - target: ix://agent-ix/quire-protocol/US-001
    type: implements
  - target: ix://agent-ix/quire-protocol/FR-010
    type: references
  - target: ix://agent-ix/quire-protocol/FR-011
    type: depends_on
  - target: ix://agent-ix/quire-specification/FR-060
    type: depends_on
  - target: ix://agent-ix/quire-specification/FR-210
    type: references
  - target: ix://agent-ix/quire-specification/FR-216
    type: references
  - target: ix://agent-ix/quire-specification/FR-217
    type: references
  - target: ix://agent-ix/quire-specification/FR-249
    type: references
  - target: ix://agent-ix/quire-specification/FR-280
    type: references
  - target: ix://agent-ix/quire-specification/FR-283
    type: references
---
# FR-030: Bind each disposition to its exact premises

## Description

When a claim disposition is returned, the dispatcher SHALL retain that disposition's exact admitted fragment, selected backend, assumption identities and states, communication, visibility and environment premises, source identities and derived-artifact correspondence.

## Inputs

One returned claim disposition together with the admitted fragment and backend
it was selected against, its assumption set, its channel and visibility
premises, its source identities and the identity of any artifact derived from
it.

## Outputs

The disposition with every retained component intact, or one refusal of that
disposition naming the absent, substituted or duplicated component.

## Behavior

When any retained component is absent, substituted or duplicated, the dispatcher SHALL refuse that disposition.
The dispatcher SHALL leave every independent disposition unchanged when it refuses one.
The dispatcher SHALL NOT replace a failed selected backend with an unreviewed fallback.
The dispatcher SHALL NOT reconstruct an absent producer-authored component from a display name, source path, span, message text or backend availability.

A refusal here is scoped to the disposition whose premise broke.
Premise-binding and independence are separate obligations: `FR-029` keeps the
dispositions apart, and this requirement keeps each one attached to what
produced it.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-030-AC-1 | Every claim disposition retains its exact admitted fragment, backend, assumptions, communication/visibility/environment premises and source/derived-artifact correspondence. | Test (TC-005) |
| FR-030-AC-2 | Removing or substituting any one retained component refuses that disposition without changing independent results. | Test (TC-005) |
| FR-030-AC-3 | A failed selected backend invokes no fallback, and no returned disposition is attributed to a backend other than the one selected for its claim. | Test (TC-005) |
| FR-030-AC-4 | Each retained assumption carries its exact assumption identity and exactly one assumption state, and an assumption retained without a state compares only to absence rather than being defaulted. | Test (TC-005) |
| FR-030-AC-5 | A derived-artifact identity that disagrees with the identity derived from its exact selected tuple refuses that disposition. | Test (TC-005) |
| FR-030-AC-6 | A producer-authored component is byte-equal to the bytes it arrived as; normalizing, repairing or recomputing one refuses rather than producing a repaired component. | Test (TC-005) |

## Adopted vocabulary

Every component this requirement binds a disposition to is a producer's subject.
The binding rule is this repository's; the components are not.

| Noun | Upstream object | Identity kind |
| --- | --- | --- |
| Assumption state | `FR-280-assumption-state` | value |
| Channel premise (all five components) | `FR-216-channel-premise` | value |
| Role internality / visibility | `FR-217-role-internality` | value |
| Protocol subject kind discrimination | `FR-210-protocol-subject-vocabulary` | value |
| Derived portable projection result | `FR-249-derived-projection-result` | derived, keyed (canonical result identity and digest, target obligation and execution-binding identities, target contract version, adapter identity and version) |
| Projection refusal code | `FR-283-projection-refusal-code` | value |

`FR-216-AC-4` is the reason FR-030-AC-1 names all five channel-premise
components: FR-054-AC-4 makes "any channel premise" identity-bearing, so a
disposition that drops one is a disposition of a different request. `FR-249`
supplies the only derived-artifact correspondence the specification states in
full, and FR-030-AC-5 is its identity rule applied at the dispatcher: changing
any member of the derivation tuple changes the derived identity. `FR-280`
supplies FR-030-AC-4: each assumption carries its exact identity plus exactly
one of `met`, `not-met`, `unknown` or `not-assessed`, and an absent state "is
never defaulted to `not-assessed` or to `unknown`; those are two distinct
assessed outcomes". FR-030-AC-6 is `FR-011`'s authority rule, not a new one.

### Accepted owner rulings

Quire Specification #29/#31/#40 supplies the exact contracts this requirement
needs: FR-290 claim kind, FR-291 admitted fragment, FR-292 analysis backend,
FR-217 role internality/participant binding, FR-210/FR-287 subject membership,
and the FR-283 projection-refusal namespace. A recomputed invalid source digest
is `invalid-source-result`; target-context or derived-identity mismatch is
`identity-mismatch`. Native producer and E02 spellings remain distinct typed
namespaces and are never normalized across them.

## Dependencies

- [FR-005](./FR-005-dispatch-independent-protocol-claims.md) is the parent this
  requirement is part of.
- [FR-011](./FR-011-consumer-identity-discipline.md) supplies the authority rule
  FR-030-AC-6 applies: a producer-authored identity is copied byte-exactly and
  never repaired.
- `ix://agent-ix/quire-specification/FR-060` requires every advanced result to
  identify its admitted fragment, backend, assumptions and source/derived-artifact
  correspondence, and forbids replacing a failed backend with an unreviewed
  fallback.
