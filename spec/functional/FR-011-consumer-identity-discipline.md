---
id: FR-011
title: "Consumer identity discipline"
type: FR
object: value_object
relationships:
  - target: ix://agent-ix/quire-protocol/FR-010
    type: part_of
  - target: ix://agent-ix/quire-specification/FR-200
    type: depends_on
  - target: ix://agent-ix/quire-specification/FR-201
    type: depends_on
  - target: ix://agent-ix/quire-specification/FR-204
    type: depends_on
---
# FR-011: Consumer identity discipline

## Description

When this repository specifies an object of its own, that object SHALL declare its identity under the same three kinds `FR-200-quire-assessment-domain` establishes.
Each such object SHALL declare separately which side authored that identity.

## Properties

- **kind** (identity component) — one of *authored*, *derived* or *value*,
  exactly as `FR-200` establishes. This repository adds no fourth kind.
- **authority** (identity component) — *producer* or *consumer*. Which side
  brought the identity into existence. Orthogonal to kind, and declared for all
  three kinds including *value*: an identity may be authored by a producer or
  authored by this repository, and the kind is the same in both cases. For a
  *value* identity nothing assigns the identity — the value *is* the identity —
  so there authority names the side whose vocabulary the value is drawn from,
  which is what decides whether the value may be recomputed locally and whether
  it may cross the boundary as authority. A single object may declare authority
  per key component where its components originate on different sides.
- **key** (ordered tuple) — the components that constitute identity under the
  declared kind.
- **domain** (reference to `FR-201-canonical-identity-domain`) — where a key
  component is a digest.

### Reconciliation with the upstream trichotomy

An earlier local draft proposed four kinds — `producer-assigned`, `structural`,
`value`, `consumer-assigned`. It is superseded, and the reconciliation is
recorded here rather than left implicit, because every adopted upstream
reference reads against upstream's vocabulary:

| Superseded local kind | Upstream kind | Authority |
| --- | --- | --- |
| `producer-assigned` | *authored* | producer |
| `structural` | *derived* | either |
| `value` | *value* | either |
| `consumer-assigned` | *authored* | consumer |

`consumer-assigned` was not a fourth kind. It was *authored* with a different
author, and collapsing it revealed the rule that actually matters: authority is
independent of kind, and the constraint belongs on authority, not on kind.

An earlier revision of this table marked authority *not applicable* for kind
*value*, which contradicted both `FR-011-AC-1` below and the four *value*-kind
objects in this repository that declare one. The contradiction is resolved in
favour of declaring authority for every kind, because the authority rule stated
next — byte-exact copy on one side, never crossing the boundary on the other —
is what the declaration is *for*, and that rule does not become vacuous merely
because the identity happens to be the value itself. A digest-domain label drawn
from an upstream vocabulary and a refusal identity minted here are both *value*
identities, and only authority distinguishes the one this repository may not
re-spell from the one it may not export.

### The authority rule

A **producer-authored** identity is copied byte-exactly. This repository never
derives, repairs, normalizes or recomputes one. Where a producer identity is
absent or invalid, the operation refuses; it does not reconstruct.

A **consumer-authored** identity is local evidence. It never crosses a boundary
as authority: it appears in no canonical result as a producer reference, and no
producer accepts it as a selector. This is the rule that makes the version-lock
entry and the scenario evidence record safe to own — they are consumer-authored
and therefore inert outside this boundary.

## Identity and comparison

- **Identity kind** — *value*, authority *consumer*. This artifact types the
  kind/authority pair itself, which is compared componentwise. The pair
  vocabulary is this repository's own reconciliation of the upstream trichotomy,
  so it crosses no boundary as authority: no producer accepts a kind/authority
  pair as a selector.
- **Key** — (`kind`, `authority`). Equality is componentwise over exactly those
  two. A display name, source path, span, message text, locale or rendering never
  participates, and none may reconstruct an absent component
  (`FR-204-declared-object-identity`).
- **Key order** — lexicographic over (`kind`, `authority`), for collection
  comparison only. It carries no causal meaning and expresses no precedence
  between kinds or between the two authorities.
- **Causal order** — none.
- **Absence** — an absent kind or authority compares only to absence and is never
  defaulted. An object that declares neither has no identity discipline, and an
  operation requiring one refuses rather than assuming a default.
- **Digest domain** — not applicable to the pair itself. Where an object's key
  contains a digest, that component's domain is selected under `FR-201`, and a
  component from one domain is unequal to one from another over identical bytes.

## Ordering obligation

An object in this context whose collections admit `missing`, `surplus`,
`duplicate` or `reordered` as defects SHALL declare a unique key and a total
order over that key, or SHALL name the owner that has not declared one.

Where the owner is not this repository, this repository does not supply the
missing key. Naming the owner and the exact unsettled question is the required
outcome; inventing a key would author another party's semantics.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-011-AC-1 | Every object specified in this repository declares one of the three upstream kinds and one authority, and declares no fourth kind. | Inspection |
| FR-011-AC-2 | Changing only a display name, source path, span, message text, locale or rendering changes no object's identity, equality, or ordering position. | Test (TC-014) |
| FR-011-AC-3 | A producer-authored identity is byte-equal to the producer bytes it arrived as; deriving, repairing, normalizing or recomputing one refuses rather than producing a repaired identity. | Test (TC-014) |
| FR-011-AC-4 | A consumer-authored identity appears in no canonical result as a producer reference, and is accepted by no producer as a selector. | Test (TC-014) |
| FR-011-AC-5 | A key component from one digest domain is unequal to a component from another domain over identical bytes, in both directions, for every ordered pair of domains. | Test (TC-014) |
| FR-011-AC-6 | An absent kind, authority or key component compares equal only to absence, never to a default, an empty value, or a present component. | Test (TC-014) |
| FR-011-AC-7 | An object whose collections admit duplicate, surplus, missing or reordered defects either declares a unique key and a total order, or names the owner that has not declared one and the exact unsettled question. | Inspection |

## Dependencies

- `ix://agent-ix/quire-specification/FR-200` establishes the three kinds.
- `ix://agent-ix/quire-specification/FR-201` selects digest domains.
- `ix://agent-ix/quire-specification/FR-204` establishes that equal field values
  do not imply equal objects, and that structural record equality never replaces
  object identity.
- [FR-008](./FR-008-return-typed-refusals.md) owns the refusal returned when an
  identity is absent or invalid.
