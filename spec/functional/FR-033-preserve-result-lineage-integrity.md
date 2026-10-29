---
id: FR-033
title: "Preserve result lineage integrity"
type: FR
relationships:
  - target: ix://agent-ix/quire-protocol/FR-006
    type: part_of
  - target: ix://agent-ix/quire-specification/FR-061
    type: depends_on
  - target: ix://agent-ix/quire-specification/FR-248
    type: references
  - target: ix://agent-ix/quire-specification/FR-282
    type: references
  - target: ix://agent-ix/quire-specification/FR-240
    type: references
---
# FR-033: Preserve result lineage integrity

## Description

When a decision-support record is retained, the result producer SHALL carry exactly one dependence relation from `FR-282` alongside that record's exact oracle and observation identities.
When later input contradicts a retained premise of a settled result, the result producer SHALL emit a typed superseding or invalidating result identifying the earlier result, the contradicted premise and the corrected input identity.
When a result has been superseded or invalidated, the result producer SHALL preserve every byte and identity of that earlier result.

## Inputs

Retained decision-support records with their oracle and observation identities
and dependence relations, the applicable prior result, and the later
observation, progress or closure record that contradicts a retained premise.

## Outputs

One result whose support lineage is acyclic and fully identified, or one new
linked superseding or invalidating result, or one typed strict-reader refusal.

## Behavior

A `derived` relation identifies its exact predecessor and a `shared` relation
identifies its dependence group. A strict reader refuses a missing, cyclic or
self-referential predecessor. FR-061 additionally forbids inferring
independence from distinct display names or implementations, so `independent`
is never reached by omission and an absent relation compares only to absence.
`shared` and `derived` relations are retained as themselves and are never
collapsed into `independent` or into each other.

A premise contradiction is not an admitted continuation under the earlier input
identity. The earlier result's bytes and its result identity remain unchanged;
the producer does not rewrite, reseal or silently replace them. The new result
strictly matches the prior-result identity and the contradicted premise identity
rather than describing them in prose, and it makes dependent package success
unavailable while recomputing truth only under the corrected exact input
identity.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-033-AC-1 | Support lineage rejects missing predecessors, cycles, self-reference, and invented independence while retaining shared and derived relations. | Test (TC-006) |
| FR-033-AC-2 | A later contradiction emits a new linked result and cannot rewrite or reseal the earlier artifact. | Test (TC-006) |
| FR-033-AC-3 | Each retained decision-support record carries exactly one `FR-282` member plus its exact oracle and observation identities, and an absent relation is never read as `independent`. | Test (TC-006) |
| FR-033-AC-4 | The superseding or invalidating result strictly matches the prior-result identity, the contradicted premise identity and the corrected input identity. | Test (TC-006) |
| FR-033-AC-5 | A premise contradiction is not admitted as a continuation under the earlier input identity, and it makes dependent package success unavailable. | Test (TC-006) |

## Adopted vocabulary

| Noun | Upstream object | Identity kind |
| --- | --- | --- |
| Decision-support record | `FR-248` | value, keyed (oracle identity, observation identity, dependence relation, and the predecessor or dependence group that relation requires) |
| Dependence relation | `FR-282` | value |
| Canonical protocol result | `FR-240` | derived under FR-286 |

`FR-282`'s four members are adopted by reference and not restated. `FR-248`'s
key is exactly this requirement's lineage rule: the predecessor or dependence
group is part of the record's key, so a `derived` record without an identified
predecessor has an incomplete key rather than a defaulted one.

### Accepted owner ruling

QSpec #29 fixes result identity under FR-286 and refuses same identity with
different canonical bytes. Supersession is an explicit predecessor relation,
never a total order derived from key order, timestamp or source position. This
requirement preserves that accepted relation and invents no global result order.

## Dependencies

- [FR-006](./FR-006-produce-canonical-protocol-results.md) is the parent this
  requirement is part of.
- `ix://agent-ix/quire-specification/FR-061` states the dependence relations,
  the lineage refusals and the supersession rule.
