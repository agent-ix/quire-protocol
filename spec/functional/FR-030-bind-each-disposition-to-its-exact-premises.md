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
| FR-030-AC-6 | A producer-authored component is byte-equal to the bytes it arrived as; normalizing, repairing or recomputing one refuses rather than producing a repaired component. | Analysis |

## Adopted vocabulary

Every component this requirement binds a disposition to is a producer's subject.
The binding rule is this repository's; the components are not.

| Noun | Upstream object | Identity kind |
| --- | --- | --- |
| Assumption state | `FR-280-assumption-state` | value |
| Channel premise (all six components) | `FR-216-channel-premise` | value |
| Role internality / visibility | `FR-217-role-internality` | value |
| Protocol subject kind discrimination | `FR-210-protocol-subject-vocabulary` | value |
| Derived portable projection result | `FR-249-derived-projection-result` | derived, keyed (canonical result identity and digest, target obligation and execution-binding identities, target contract version, adapter identity and version) |
| Projection refusal code | `FR-283-projection-refusal-code` | value |

`FR-216-AC-4` is the reason FR-030-AC-1 names all six channel-premise
components: FR-054-AC-4 makes "any channel premise" identity-bearing, so a
disposition that drops one is a disposition of a different request. `FR-249`
supplies the only derived-artifact correspondence the specification states in
full, and FR-030-AC-5 is its identity rule applied at the dispatcher: changing
any member of the derivation tuple changes the derived identity. `FR-280`
supplies FR-030-AC-4: each assumption carries its exact identity plus exactly
one of `met`, `not-met`, `unknown` or `not-assessed`, and an absent state "is
never defaulted to `not-assessed` or to `unknown`; those are two distinct
assessed outcomes". FR-030-AC-6 is `FR-011`'s authority rule, not a new one.

### Where the upstream vocabulary does not yet reach

- **The admitted fragment and the selected backend are untyped.** FR-005 and
  upstream `FR-060` both require every advanced result to identify "its admitted
  fragment, backend, assumptions and source/derived-artifact correspondence"
  (FR-060-AC-4), and the object set types neither a protocol fragment nor an
  analysis backend: no `FR-2NN` artifact exists for either. FR-030-AC-1,
  FR-030-AC-2 and FR-030-AC-3 each compare a fragment or a backend for equality
  and have no key to compare over. The owner is `agent-ix/quire-specification`.
  This repository supplies neither identity, because a fragment and a backend
  are producer subjects, not consumer-to-producer relationships
  (`FR-010`).
- **Which refusal code a digest mismatch carries is contradicted upstream, and
  both readings stand.** `FR-283`, under `invalid-source-result`: "The canonical
  protocol result did not validate, or its recomputed `sha256-jcs` digest did
  not match, before mapping was attempted (FR-062)." `FR-249`, under
  `identity-mismatch`: "An exact context or canonical-result reference is
  missing or conflicting; substituting any identity or digest refuses the
  projection (FR-062, FR-062-AC-2)." `FR-249`'s open gap states that "They are
  not reconcilable by reading: a strict reader receiving a refusal cannot tell
  from the code whether the source failed its reader or its digest disagreed, and
  a consumer matching on either code alone will miss the case", and that closing
  it "requires an owner ruling and a requirement that names the code".
  FR-030-AC-5 requires the refusal and names no code. The owner is
  `agent-ix/quire-specification`.
- **Two code-spelling conventions coexist with no namespace rule.** `FR-283`'s
  open gap records snake_case literals in FR-003-AC-2, FR-008-AC-4 and
  FR-009-AC-3 against FR-062's kebab-case literals, and states that "No
  requirement says which convention belongs to which namespace". This
  requirement selects no spelling.
- **Role internality has no normative spelling.** `FR-217` states that "No
  requirement fixes a spelling for either kind; the labels above name the
  distinction FR-050 and FR-050-AC-1 draw and are not established wire strings."
  FR-030-AC-1 retains the visibility premise and asserts no wire form for it.
- **The protocol subject membership is stated four ways.** `FR-210`'s open gap
  records four requirements each stating a closed list of distinct subjects and
  reconciles none. A disposition's source identities are drawn from those
  subjects, so what exactly must be retained varies with which list is read.

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
