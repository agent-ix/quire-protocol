---
id: FR-015
title: "Digest domain membership"
type: FR
object: enumeration
relationships:
  - target: ix://agent-ix/quire-protocol/FR-010
    type: part_of
  - target: ix://agent-ix/quire-protocol/FR-011
    type: depends_on
  - target: ix://agent-ix/quire-protocol/FR-001
    type: specifies
  - target: ix://agent-ix/quire-specification/FR-201
    type: depends_on
---
# FR-015: Digest domain membership

## Description

When this repository compares a digest to a digest slot, the consumer boundary SHALL select exactly one member of this vocabulary for that digest and compare the selected member before the bytes.
When a digest selected under one member is presented in another member's slot, the consumer boundary SHALL return a typed refusal before any dependent field is read.

`ix://agent-ix/quire-specification/FR-201` types the domain *label kind* and
supplies the cross-domain inequality rule this artifact applies. It selects no
member for any digest this repository consumes, and upstream records that
omission as an open gap rather than closing it — see **What upstream leaves
open**. The four-member set below is therefore this repository's own, and it
names the four domains that must stay separate across one selection.

## Values

| Value | Description |
| --- | --- |
| `producer-canonical` | The producer's own canonical digest over a producer-owned subject — source, model, dependency or configuration. It is copied byte-exactly and stays opaque. FR-001 forbids reinterpreting it: the library "SHALL NOT reinterpret a producer-owned model/configuration digest through this artifact's numeric or JCS domain." |
| `native-raw` | A digest over the exact delivered bytes of a native producer input, as delivered, with no canonicalization, re-encoding or normalization ([FR-009](./FR-009-lock-external-producer-revisions.md), "Digest verification"). A producer input this repository recanonicalized instead of copying byte for byte refuses rather than admitting an equivalent form (IT-001-SC-07). |
| `compiled-artifact` | A digest over the compiled-protocol artifact bytes admitted under one accepted compiled-artifact contract version ([FR-001](./FR-001-admit-selected-protocol-profile.md)). It is producer-authored and is never derived from, nor substituted for, the native inputs the artifact was compiled from. |
| `protocol-result` | The consumer-owned digest over the RFC 8785 (JCS) canonical form of a canonical protocol result ([FR-006](./FR-006-produce-canonical-protocol-results.md)). It is the only domain this repository computes rather than copies, and the only one its JCS canonicalisation covers. |

Membership is closed. This repository adds no fifth member and admits no digest
whose selected member is absent.

## Identity and comparison

- **Identity kind** — *value*, per [FR-011](./FR-011-consumer-identity-discipline.md).
  A domain label has no identity of its own and is compared componentwise as an
  exact string. Authority is not applicable to the label; the *digests* the
  labels select over are producer-authored under `producer-canonical`,
  `native-raw` and `compiled-artifact`, and consumer-authored under
  `protocol-result`.
- **Key** — the label itself. Matching is exact. A hyphenated and an underscored
  spelling are different labels and neither is a normalized alias of the other.
  A display name, source path, span, message text, locale or rendering never
  participates, and none reconstructs an absent label
  (`ix://agent-ix/quire-specification/FR-204`).
- **Key order** — lexicographic over the label, for collection comparison and
  duplicate detection only. It carries no precedence, preference or causal
  meaning between members, and it expresses no progression from a producer
  domain to the result domain.
- **Causal order** — none. A domain label participates in no causal structure.
- **Absence** — a digest supplied without a selected member has an absent
  domain. An absent domain compares only to absence and is never defaulted to
  `native-raw`, to `producer-canonical`, or to any other member. A digest whose
  domain is absent is refused rather than interpreted under an ambient
  selection.
- **Digest domain** — not applicable to the label itself. This artifact *is* the
  membership of digest domains this boundary distinguishes; it carries none.

## Cross-domain substitution refuses in both directions

Two digests bearing different members are unequal. Byte equality does not make
them equal, and the domain is fixed by *what was hashed*, never by the spelling
of the value, so a cross-domain substitution is invisible to any syntactic check
and must be decided against an independently retained selection.

The obligation is exhaustive over ordered pairs, not over unordered ones: with
four members there are **12 ordered pairs** of distinct members, and a digest of
the offered member presented in the slot member's position refuses for every one
of them. Refusing `producer-canonical` in a `protocol-result` slot does not
discharge `protocol-result` in a `producer-canonical` slot. IT-001-SC-07
exercises the substitution across the same `/2` selection over raw bytes, and
[FR-011](./FR-011-consumer-identity-discipline.md)-AC-5 states the same
obligation over identical bytes in both directions.

## What upstream leaves open

`ix://agent-ix/quire-specification/FR-201` requires the domain to be compared
before the bytes but selects no member for the digests this repository consumes.
Three upstream artifacts record that omission as an open gap and each declines
to close it:

- `ix://agent-ix/quire-specification/FR-259` — "no requirement selects a member
  for either digest. This artifact records the dependency and selects neither."
- `ix://agent-ix/quire-specification/FR-261` — "no requirement selects a member
  for this digest. This artifact records the dependency and selects no member."
- `ix://agent-ix/quire-specification/FR-263` — "No requirement states the
  identity domain of the membership, configuration or closure digests."

This artifact cites those gaps and does not close them. It selects no member for
any upstream subject. It declares only the membership this consumer boundary
distinguishes among the values it consumes and the one result digest it
produces, and it borrows the inequality rule from FR-201 rather than restating a
second one.

## Unreconciled divergence with the implementation enum

The crate's `digest::producer::DigestDomain` names five members —
`ProducerArtifactBytes`, `ProducerSource`, `ProducerModel`, `ProducerConfig`,
`ProtocolResultJcs`. The `/2` lane's `tests/support/v2::DigestDomain` names the
four specified above. Only one member corresponds across them:
`ProtocolResultJcs` and `protocol-result`.

**These are not the same axis, and neither refines the other.** Two independent
dimensions are collapsed into one enum on each side, differently:

- *Which artifact* the digest is over — artifact bytes, source, model,
  configuration. The production enum partitions on this.
- *Which canonicalization regime* produced the digested bytes — the producer's
  own canonical form, the raw native bytes as delivered, the compiled artifact's
  form, or RFC 8785/JCS. The specified membership partitions mostly on this, and
  `compiled-artifact` on the first.

So `ProducerSource` corresponds to `native-raw` or to `producer-canonical`
depending on which dimension is read, and nothing decides which. A mapping
between the two sets cannot be written down without first choosing whether a
digest domain is an artifact kind, a canonicalization regime, or an ordered pair
of both.

This requirement declares the four members it owns. It does **not** declare that
the production enum refines them, because it does not: calling it a refinement
would assert a correspondence that no artifact establishes, which is the same
defect this requirement exists to prevent one level down.

**What must be decided, and is not decided here.** Whether a digest domain is a
canonicalization regime, an artifact kind, or a pair of the two. Until that is
settled, the two enums are two different vocabularies that share one member name,
and the cross-domain inequality obligation below is stated over the four members
this requirement declares.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-015-AC-1 | Each member's value equals the exact label string in the table, an underscored spelling does not match a hyphenated one, and no fifth member is admitted. | Inspection |
| FR-015-AC-2 | A digest selected under one member, presented in another member's slot, returns a typed refusal for every one of the 12 ordered pairs of distinct members. | Test (TC-001, IT-001) |
| FR-015-AC-3 | Two digests with equal bytes and different selected members compare unequal, in both directions, for every ordered pair. | Test (TC-001, IT-001) |
| FR-015-AC-4 | A digest with an absent selected member compares only to absence, receives no default member, and is refused rather than interpreted under an ambient selection. | Test (TC-001) |
| FR-015-AC-5 | A `producer-canonical`, `native-raw` or `compiled-artifact` digest is copied byte-exactly and reaches no path that recanonicalizes, re-encodes or normalizes it, and no producer digest is reinterpreted through this repository's numeric or JCS domain. | Test (TC-001, IT-001) |
| FR-015-AC-6 | A producer input recanonicalized instead of copied byte for byte refuses rather than admitting an equivalent form, exercised with a form byte-distinguishable from the as-delivered form. | Test (IT-001) |
| FR-015-AC-7 | No member is selected here for any digest whose domain an upstream artifact records as an open gap. | Inspection |

## Dependencies

- `ix://agent-ix/quire-specification/FR-201` types the domain label kind and
  supplies the cross-domain inequality rule.
- `ix://agent-ix/quire-specification/FR-204` establishes that equal field values
  do not imply equal objects.
- `ix://agent-ix/quire-specification/FR-259`,
  `ix://agent-ix/quire-specification/FR-261` and
  `ix://agent-ix/quire-specification/FR-263` record the unselected-member gap
  this artifact cites rather than closes.
- [FR-011](./FR-011-consumer-identity-discipline.md) supplies the kind and
  authority axes and the both-directions inequality obligation.
- [FR-001](./FR-001-admit-selected-protocol-profile.md) forbids reinterpreting a
  producer-owned digest through this repository's numeric or JCS domain.
- [FR-009](./FR-009-lock-external-producer-revisions.md) recomputes an entry's
  digest over the exact delivered bytes with no canonicalization.
- [FR-008](./FR-008-return-typed-refusals.md) owns the refusal envelope returned
  on a cross-domain substitution.

## Status

Draft. This artifact records existing meaning structurally and establishes no new
obligation. Where an upstream artifact does not select a member, that absence is
cited here rather than supplied.
