---
id: FR-018
title: "Delivery index"
type: FR
object: entity
relationships:
  - target: ix://agent-ix/quire-protocol/US-003
    type: implements
  - target: ix://agent-ix/quire-protocol/FR-010
    type: part_of
  - target: ix://agent-ix/quire-protocol/FR-011
    type: depends_on
  - target: ix://agent-ix/quire-protocol/FR-009
    type: depends_on
  - target: ix://agent-ix/quire-protocol/FR-012
    type: depends_on
  - target: ix://agent-ix/quire-specification/FR-224
    type: references
  - target: ix://agent-ix/quire-specification/FR-252
    type: references
---
# FR-018: Delivery index

## Description

When this repository pins one producer delivery, the consumer boundary SHALL retain one delivery index recording what that delivery contained, written by this repository and never by the producer.
When a delivery index is offered as the source or repair of a version-lock entry, the resolver SHALL refuse rather than reading it.

A delivery index is evidence this repository reads about what was delivered.
`FR-009` Inputs states the limit exactly: a delivery index "is evidence B reads
about what was delivered, and it can neither supply nor repair a version-lock
entry."

## This is not `ix://agent-ix/quire-specification/FR-224`

Upstream publishes an entity also called **Delivery**
(`ix://agent-ix/quire-specification/FR-224`). It is a completely different noun
and the two must not be conflated.

| | `ix://agent-ix/quire-specification/FR-224` | This artifact |
| --- | --- | --- |
| What it is | A *transport observation*: one delivery of one message on one channel, counted against that send's delivery cardinality. | A *consumer record*: this repository's index of the files one producer shipped at one pinned revision. |
| Whose subject | The producer's protocol subject. | The consumer's relationship to a producer's shipment. |
| Who authors it | Upstream, from an observed transport record. | This repository, at pin time. |
| Identity | Producer-authored FR-287 key of kind `delivery`; FR-224 gives it canonical comparison order and no causal order. | Consumer-authored, keyed below. |
| Cardinality | Many per send. | One per pinned delivery root. |

Nothing in this artifact supplies, repairs, or stands in for FR-224's key, and
no member of this index is a transport observation. The shared word
"delivery" is the only thing the two have in common.

## Properties

Every property is a component of the index this repository writes; none is
emitted by the producer.

- **producer_revision** (identity component) — the exact immutable producer
  revision the indexed bytes were emitted at. Producer-authored and copied
  byte-exactly under [FR-011](./FR-011-consumer-identity-discipline.md); this
  repository pins no intermediate commit.
- **offer** (state) — the offered artifact's path within the delivery root and
  the producer's external artifact reference, copied verbatim. The bytes at that
  path are carried through untouched and are the exact bytes submitted.
- **inputs** (state) — one entry per producer input role, each naming the role
  and the path of the original, unrecanonicalized bytes for that role. An
  unindexed role and an unreadable indexed file are distinct outcomes; the second
  is reported, never collapsed into the first.
- **temporal** (state) — the indexed temporal profile path and the indexed
  binding records this repository retains, each carrying an owner, a
  `declaration` and a path. The temporal semantics themselves are
  `ix://agent-ix/quire-specification`'s and are referenced, never restated here.
- **limits** (state) — one entry per resource limit the exactly-sufficient probe
  of [FR-016](./FR-016-resource-budget-dimension.md) is to be run at, carrying
  an exact typed dimension and bound. The dimension is either
  `Protocol(FR-016 member)` or
  `CompiledProtocol(protocol_artifact::work::Dimension member)`; the two owner
  vocabularies never share a key or substitute for one another. An earlier
  revision called these "producer-declared" limits and then reduced both
  vocabularies to opaque strings. Both were wrong: the grant is caller-supplied,
  and its typed dimension retains the accountant that owns it.
- **mutations** (state) — one entry per producer-supplied mutation fixture,
  naming the fixture, the scenario it belongs to, and its path. A scenario with
  more than one registered mutation fixture is ambiguous and is reported, not
  resolved by choosing one.

The index carries no owner component. The owner is fixed by the delivery root the
index is loaded from, not by a field of the index. Where an owner is required as
an identity component — as [FR-012](./FR-012-version-lock-entry.md)'s key
requires — it is read from the version-lock manifest entry and never from this
index, because this index "can neither supply nor repair a version-lock entry".

## The binding rule this index enforces is A's

The cardinality, uniqueness and order of the indexed binding records are the
producer's version 2 wire contract, quoted verbatim in IT-001 Preconditions:

> "There is exactly one binding for every temporal declaration and none for other
> declaration families. Bindings sort by `declaration` and cannot repeat."

That sentence, and nothing stronger, is what this repository relies on. This
repository enforces it over its **own retained index** and restates it in no
other words. In particular this artifact does not call `declaration` a "declared
key", and it does not call the sort a "declared total order": the producer uses
neither phrase, and adopting either would author the producer's semantics on its
behalf.

What the rule settles, over this index and only over this index:

| Defect | Settled by |
| --- | --- |
| `missing`, `surplus` | The cardinality clause — exactly one binding per temporal declaration, none for other declaration families. |
| `duplicate` | The no-repeat clause. Two indexed bindings naming one `declaration` are reported. |
| `reordered` | The sort clause. An indexed binding whose `declaration` is lower than its predecessor's is reported. |

Without that rule holding over this index, "duplicate" and "reordered" name
nothing here.

### Upstream settles none of it

`ix://agent-ix/quire-specification/FR-252` gives a clock binding a **value**
identity whose key is composed of the selected profile identity and the clock
components that profile admits. No declaration component appears in that key. Two bindings on two different declarations that
agree on every listed component therefore compare **equal** upstream, which is
exactly the comparison the no-repeat and sort clauses need to distinguish. This
repository cites that as the reason it enforces the producer's wire rule over its
own index rather than deriving uniqueness or order from an upstream key, and it
supplies no upstream key of its own.

## Identity and comparison

- **Identity kind** — *authored*, authority *consumer*, per
  [FR-011](./FR-011-consumer-identity-discipline.md). The index is written by
  this repository at pin time and is not emitted by the producer. Being
  consumer-authored, it crosses no boundary as authority: it appears in no
  canonical result as a producer reference, no producer accepts it as a selector,
  and it supplies, selects or repairs no version-lock entry. The producer
  revision, the external artifact reference, the indexed bytes and the declared
  limit bounds it *carries* are producer-authored and copied byte-exactly;
  carrying them does not make the index's own identity producer-authored.
- **Key** — (`delivery_root`, `producer_revision`), where `delivery_root` is the
  root the index was loaded from. Equality is componentwise over exactly those
  two. The offer, inputs, temporal records, limits and mutations are state, never
  identity: reindexing one delivery after a fixture is added leaves the same
  index. Two indices at one revision under two roots are two records, because the
  index carries no owner component to tell them apart by. A display name, a file
  name, a span or a message text never participates.
- **Key order** — lexicographic over (`delivery_root`, `producer_revision`), for
  collection comparison and duplicate detection only. It carries no causal
  meaning, is no chronology of deliveries, and expresses no precedence between
  revisions. It is emphatically not the binding order above: the binding order is
  ascending `declaration` within one index, under the producer's rule, and the
  two orders never substitute for one another.
- **Causal order** — none between indices, and none over deliveries. A later
  producer revision is not a causal successor of an earlier one, and this index
  derives no order from a timestamp, a file modification time or an ingestion
  order.
- **Absence** — every way an index can fail to be assembled is reported, and none
  is treated as a pass: a selected delivery root that is missing, an absent
  index, an index member named but not present, an index that
  does not deserialize into this repository's record shape, a scenario with more
  than one mutation fixture, a repeated binding declaration, and unsorted
  bindings. An absent index compares only to absence; it is never defaulted to an
  empty index, and a lane without one reports its absence rather than reporting
  an outcome.
- **Digest domain** — the index carries no digest of its own and synthesizes
  none. Where a digest is computed over an indexed member's bytes, its domain is
  selected under [FR-015](./FR-015-digest-domain-selection.md), and a digest from
  one domain is unequal to one from another over identical bytes. A digest an
  index carried would still not satisfy a version-lock entry's recorded digest,
  which `FR-009` recomputes over the exact delivered bytes.

## Ordering obligation over the index's collections

Under [FR-011](./FR-011-consumer-identity-discipline.md)'s ordering obligation,
each collection this index holds either declares a unique key and a total order,
or names the owner that has not:

| Collection | Unique key | Total order | Owner |
| --- | --- | --- | --- |
| `temporal.bindings` | `declaration` | Ascending `declaration` | The producer's version 2 wire contract, quoted above. |
| `inputs` | `role` | Not required; the roles are a fixed closed set and the collection admits no duplicate role. | This repository. |
| `mutations` | `scenario` | Not required; at most one fixture per scenario, and a second is reported as ambiguous. | This repository. |
| `limits` | (`accountant owner/contract version`, exact typed dimension member) | Lexicographic owner/contract key, then the owning vocabulary's declared member order; comparison only. Duplicate keys refuse. | This repository defines the tagged union and ordering; each owner retains its own dimension members. |

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-018-AC-1 | A delivery index supplies, selects or repairs no version-lock entry, and removing or editing an index changes no resolution outcome. | Test (TC-017) |
| FR-018-AC-2 | A delivery index reference does not satisfy a comparison expecting `ix://agent-ix/quire-specification/FR-224`, and no member of an index is a transport observation. | Inspection |
| FR-018-AC-3 | Two indexed bindings naming one `declaration` are reported, and an indexed binding whose `declaration` is lower than its predecessor's is reported, each naming the offending declaration. | Test (TC-017) |
| FR-018-AC-4 | The binding cardinality, uniqueness and order this repository enforces are exactly the producer's quoted wire rule, and no stronger declared key or declared total order is asserted over the producer's bindings. | Inspection |
| FR-018-AC-5 | A selected delivery root that is missing, an absent index, a missing indexed member, an unreadable index, an ambiguous mutation, a repeated declaration and unsorted bindings are each reported as a distinct absence, and none is treated as a passing lane. | Test (TC-017) |
| FR-018-AC-6 | An unindexed input role and an unreadable indexed file are distinct outcomes, and neither is collapsed into the other. | Test (TC-017) |
| FR-018-AC-7 | Two indices whose delivery root and producer revision are equal compare equal regardless of offer, inputs, temporal records, limits or mutations, and two differing in either key component compare unequal regardless of identical remaining state. | Analysis |
| FR-018-AC-8 | The indexed offer bytes are carried through untouched and are byte-equal to the bytes submitted, and no indexed producer input is recanonicalized. | Test (TC-017) |
| FR-018-AC-9 | Limit entries are keyed by accountant owner/contract version plus exact typed dimension; duplicate keys refuse, canonical ordering is deterministic and non-causal, and Protocol and compiled-protocol dimensions never substitute despite similar names. | Test (TC-017) |

## Dependencies

- [FR-009](./FR-009-lock-external-producer-revisions.md) Inputs establishes that a
  delivery index is evidence this repository reads and can neither supply nor
  repair a version-lock entry.
- [FR-012](./FR-012-version-lock-entry.md) owns the `(owner, contract version)`
  key this index does not carry and must not supply.
- [FR-011](./FR-011-consumer-identity-discipline.md) supplies the kind and
  authority axes and the ordering obligation applied above.
- [FR-015](./FR-015-digest-domain-selection.md) selects the domain of any digest
  computed over an indexed member's bytes.
- [FR-016](./FR-016-resource-budget-dimension.md) consumes the indexed limit
  bounds as the exact values its probe is run at.
- [FR-001](./FR-001-admit-selected-protocol-profile.md) admits the offered bytes
  this index points at.
- `ix://agent-ix/quire-specification/FR-224` is the upstream transport-observation
  entity this artifact is explicitly not; its accepted FR-287 key remains
  producer authority and is not reused by this consumer index.
- `ix://agent-ix/quire-specification/FR-252` types the upstream clock binding
  whose key carries no declaration component.

## Status

Implemented under Task-019/#66. Five default-running TC-017 tests exercise every
Test-designated criterion, including exact identity and bytes, typed absence and
ambiguity, producer declaration rules, typed limit ordering, transactional
resource charging, and version-lock non-authority. SR-108 and SR-109 record the
PR-time Rust and gap reviews. The superseded pre-delivery corpus remains retired;
A's `MutationManifest` is producer-owned and does not instantiate this
consumer-authored object.
