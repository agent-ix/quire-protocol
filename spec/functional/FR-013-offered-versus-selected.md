---
id: FR-013
title: "Offered versus selected discrimination"
type: FR
object: value_object
relationships:
  - target: ix://agent-ix/quire-protocol/FR-010
    type: part_of
  - target: ix://agent-ix/quire-protocol/FR-011
    type: depends_on
  - target: ix://agent-ix/quire-protocol/FR-001
    type: specifies
  - target: ix://agent-ix/quire-protocol/FR-009
    type: specifies
  - target: ix://agent-ix/quire-specification/FR-204
    type: depends_on
---
# FR-013: Offered versus selected discrimination

## Description

When a value is compared against this repository's retained expectation, the comparator SHALL compare provenance together with bytes rather than bytes alone.
An offered value and a selected value that are byte-identical SHALL compare unequal.
The comparator SHALL compare a value whose provenance is absent against no expectation.

## Properties

- **provenance** (identity component) — one of *selected* or *offered*.
  *Selected* is what this repository independently retained before it inspected
  any offer; *offered* is what the external producer presented. The distinction
  is the one `tests/support/v2/mod.rs` states on `OfferedAxis`: "Distinct from
  `SelectionAxis`: those change what B independently selected, these change what
  A offered. Both must refuse, for different reasons."
- **subject** (identity component) — the thing the value is a value *of*: the
  identity within its provenance side that the comparison is about.
- **value** (state) — the bytes or the retained identity components themselves.
  Never an identity component of this object on their own.

### The independence condition

The consumer selects and retains the compiler contract, baseline, producer,
native-source, model-source and dependency inputs **before** it inspects an
offered package ([IT-001](../integration/IT-001-consume-native-protocol-artifact.md)
Preconditions). Retention before inspection is what makes the two sides
independent; a selection derived from the offer is not a selection.

### Self-authorization

An offer that supplies its own acceptance context authorizes itself, and is
refused. A producer-emitted sidecar carrying the expected selection — an
`expected.json` beside the artifact, or any equivalent — is never this
repository's selection source ([FR-009](./FR-009-lock-external-producer-revisions.md)
"Self-authorization", IT-001 Dependencies). The recipe by which an expectation
is constructed is a derivation this repository implements, not a file it reads.
A contract version is likewise never inferred from the offered bytes
([FR-001](./FR-001-admit-selected-protocol-profile.md) Behavior).

### What is typed here, and what is not

This artifact types the **discrimination** — the provenance-bearing key and the
rule that byte equality never collapses the two sides. It does not type the axis
lists. `SelectionAxis`'s thirteen members and `OfferedAxis`'s six members are
this repository's test corpus for the discrimination, enumerated per contract in
the integration lane, and adding or removing a member changes no obligation
stated here.

## Identity and comparison

- **Identity kind** — *value*. This artifact types the provenance-tagged value
  itself, compared componentwise. Authority is *consumer* for the *selected*
  side and *producer* for the *offered* side; the tag recording which is
  consumer-authored, and a producer accepts it as no selector
  ([FR-011](./FR-011-consumer-identity-discipline.md)).
- **Key** — (`provenance`, `subject`, `value`). Provenance is a key component,
  not an annotation: that is precisely what prevents byte equality alone from
  collapsing an offered value into a selected one. `FR-204` states the general
  form — "Equal fields do not imply equal objects" — and this key is its
  application at the consumer boundary. A display name, path, span, message text
  or locale never participates.
- **Key order** — lexicographic over (`provenance`, `subject`, `value`), for
  collection comparison only. Ordering `provenance` first groups each side
  together and expresses no precedence: neither side is authoritative over the
  other, and the order carries no causal meaning.
- **Causal order** — one only, and it is not between key components: retention
  of the *selected* side precedes inspection of the *offered* side. A selected
  value that exists only after an offer was inspected is not independent, and
  the comparison it feeds is void.
- **Absence** — an absent provenance compares only to absence and is never
  defaulted to either side. A value carrying no provenance is compared against
  no expectation and refuses, because defaulting it to *selected* would let an
  offer authorize itself. An absent selected value is never filled from the
  offered side, and an absent offered value is never filled from the selected
  side.
- **Digest domain** — not applicable to the pair itself. Where a `value` is a
  digest, its domain membership is
  `FR-015-digest-domain-membership`'s subject, and a digest from one
  domain presented in another domain's slot is unequal in both directions.

## Symmetric refusal, asymmetric reason

A mutation on the selected side and a mutation on the offered side both refuse,
and they refuse for different reasons and under different typed outcome
identities ([FR-014](./FR-014-typed-outcome-identity.md)). A selected-side
mutation means this repository's retained expectation no longer describes the
offer. An offered-side mutation means the producer's artifact no longer matches
what was independently selected. Reporting either under the other's identity
erases which side moved, which is the whole content of this discrimination.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-013-AC-1 | An offered value and a selected value that are byte-identical over the same subject compare unequal, in both directions. | Test (TC-001, IT-001) |
| FR-013-AC-2 | Mutating one selected-side component and mutating one offered-side component each refuse, and the two refusals carry different typed outcome identities. | Test (TC-001, IT-001) |
| FR-013-AC-3 | An offer supplying its own acceptance context — a producer-emitted sidecar, display name, path or message text — is refused as a selection source, and removing it changes no admission outcome. | Test (TC-001, IT-001) |
| FR-013-AC-4 | A contract version is inferred from no offered bytes, and bytes of one contract version offered at another version's entry point refuse with no detection, upgrade, downgrade or repair. | Test (IT-001) |
| FR-013-AC-5 | A value carrying no provenance is compared against no expectation and refuses, rather than being defaulted to either side. | Test (TC-001) |
| FR-013-AC-6 | Changing a display name, path, span, message text or locale on either side changes no equality verdict and no refusal identity. | Test (TC-001, IT-001) |
| FR-013-AC-7 | Every selected component is retained before any offered byte for that integration is inspected. | Inspection |
| FR-013-AC-8 | Adding or removing a member of either axis list changes no obligation this requirement states. | Analysis |

## Dependencies

- [FR-001](./FR-001-admit-selected-protocol-profile.md) establishes the exact
  match between the selected profile and the offered artifact, and the
  per-contract-version entry points.
- [FR-009](./FR-009-lock-external-producer-revisions.md) establishes the
  self-authorization prohibition this artifact generalizes.
- [FR-011](./FR-011-consumer-identity-discipline.md) supplies the kind and
  authority axes.
- [FR-014](./FR-014-typed-outcome-identity.md) owns the distinct typed identities
  the two sides' refusals carry.
- [IT-001](../integration/IT-001-consume-native-protocol-artifact.md) exercises
  the discrimination over both axis corpora.
- `ix://agent-ix/quire-specification/FR-204` establishes that equal field values
  do not imply equal objects.

## Status

Draft. This artifact records existing meaning structurally and establishes no new
obligation. It has no upstream counterpart, because a producer has no concept of
a selection retained before its bytes were inspected.
