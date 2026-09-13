---
id: FR-012
title: "Version-lock entry"
type: FR
object: entity
relationships:
  - target: ix://agent-ix/quire-protocol/ADR-009
    type: references
  - target: ix://agent-ix/quire-protocol/US-003
    type: implements
  - target: ix://agent-ix/quire-protocol/FR-010
    type: part_of
  - target: ix://agent-ix/quire-protocol/FR-011
    type: depends_on
  - target: ix://agent-ix/quire-protocol/FR-009
    type: specifies
  - target: ix://agent-ix/quire-specification/FR-201
    type: depends_on
  - target: ix://agent-ix/quire-specification/FR-204
    type: depends_on
---
# FR-012: Version-lock entry

## Description

When a version-lock entry is compared, the resolver SHALL compare its owner and contract version rather than its revision, digest, status or reason.
When two entries share one owner and contract version, the resolver SHALL refuse every integration before resolution.

## Properties

- **owner** (identity component) — the external party that publishes the pinned
  subject: one of the producers `FR-009` enumerates. The complete-global-conformance
  closure record is not an owner, because no external party can assert anything
  about it ([FR-009](./FR-009-lock-external-producer-revisions.md) Inputs).
- **contract_version** (identity component) — the published contract version of
  that owner this entry pins. One owner publishing two contract versions takes
  two entries, resolved independently (FR-009-AC-5).
- **revision** (state) — the exact immutable revision the owner is pinned at.
  Producer-authored and copied byte-exactly under
  [FR-011](./FR-011-consumer-identity-discipline.md); never derived, repaired or
  normalized here.
- **immutability_assertion** (state) — the owner's own record that the revision
  is immutable and published. It records what this repository was told, not what
  it verified (FR-009, "On the owner's assertion").
- **content_digest** (state, conditional) — the digest of the exact consumed
  bytes as delivered, present only where the producer supplies bytes. Its domain
  is selected under `FR-201`. An entry naming an interface rather than an
  artifact carries no digest.
- **status** (state) — exactly one of `pinned`, or `unresolved` with the reason
  it cannot be pinned.

### Interface-only entries

The Engineering Assurance typed producer-execution seam, the E02 strict reader
and the Quoin evidence interface name an interface, not an artifact. Such an
entry carries a revision and no digest, and no digest is synthesized for it.
Absence of a digest on such an entry is the specified state, not a defect.

### Entry independence

Changing one entry alters the resolved owner, revision, digest or status of no
other entry. Entries declare no dependency on one another, so resolution needs
no order and can contain no cycle. This is a property of resolution at run time
and says nothing about the order in which `FR-009`'s gates close.

## Identity and comparison

- **Identity kind** — *authored*, authority *consumer*. The entry is this
  repository's own manifest record; a producer cannot write it, and no
  producer-emitted sidecar, display name, path or message text supplies, selects
  or repairs one (FR-009 "Self-authorization", FR-009-AC-4). The `revision` and
  `content_digest` it *carries* are producer-authored and copied byte-exactly;
  carrying them does not make the entry's own identity producer-authored.
- **Key** — (`owner`, `contract_version`). Equality is componentwise over
  exactly those two. Revision, immutability assertion, digest, status and reason
  are state, never identity: repinning an entry to a later revision leaves the
  same entry, and `FR-204` forbids field equality from standing in for object
  identity. A display name, path, span or message text never participates.
- **Key order** — lexicographic over (`owner`, `contract_version`), for
  collection comparison and duplicate detection only. It carries no causal
  meaning and expresses no precedence between owners or between contract
  versions of one owner.
- **Causal order** — none between entries. Entry independence is exactly the
  statement that no causal order exists, so resolution is unordered and
  acyclic by construction.
- **Absence** — an absent entry compares only to absence and is never defaulted
  to another contract version of the same owner. An `unresolved` entry is present
  with a reason, and is not absence; a scenario requiring one is reported
  unexecutable rather than passing or failing (FR-009-AC-3). An absent digest on
  an interface-only entry compares only to absence and is never synthesized.
- **Digest domain** — where `content_digest` is present, its domain is selected
  under `FR-201`, and a digest from one domain is unequal to one from another
  over identical bytes. The digest is recomputed over the exact delivered bytes
  with no canonicalization, re-encoding or normalization.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-012-AC-1 | Two entries whose owner and contract version are equal compare equal regardless of revision, digest, status or reason, and two entries differing in owner or in contract version compare unequal regardless of identical remaining fields. | Test (TC-013) |
| FR-012-AC-2 | Two entries sharing one (owner, contract version) key refuse every integration before resolution, and the refusal names that key. | Test (TC-013) |
| FR-012-AC-3 | Adding, removing or repinning one entry changes the resolved owner, revision, digest and status of no other entry, and a manifest carrying two contract versions of one owner resolves each independently. | Test (TC-013) |
| FR-012-AC-4 | An entry naming an interface rather than an artifact carries a revision and no digest, and synthesizing a digest for it refuses. | Test (TC-013) |
| FR-012-AC-5 | A producer-emitted sidecar, display name, path or message text supplies, selects or repairs no entry, and removing the sidecar changes no resolution outcome. | Test (TC-013) |
| FR-012-AC-6 | An entry carries exactly one status, an `unresolved` entry carries a reason, and an `unresolved` entry is satisfied by no other contract version of the same owner. | Test (TC-013) |
| FR-012-AC-7 | Entry keys admit a total order and duplicate detection runs over that order, so no duplicate key escapes detection through collection ordering. | Analysis |
| FR-012-AC-8 | The entry's revision and digest are byte-equal to the producer bytes they arrived as, and no resolution path derives, repairs or normalizes either. | Test (TC-013) |

## Dependencies

- [FR-009](./FR-009-lock-external-producer-revisions.md) establishes the entry,
  its key, its two-status resolution model, entry independence, the
  interface-only digest exemption and the self-authorization prohibition.
- [FR-011](./FR-011-consumer-identity-discipline.md) supplies the kind and
  authority axes and the rule that a consumer-authored identity crosses no
  boundary as authority.
- [FR-008](./FR-008-return-typed-refusals.md) owns the refusal envelope returned
  when an entry is absent, unpinned, duplicated or digest-mismatched.
- `ix://agent-ix/quire-specification/FR-201` selects the digest domain of
  `content_digest`.
- `ix://agent-ix/quire-specification/FR-204` establishes that equal field values
  do not imply equal objects.

## Status

Draft. This artifact records existing meaning structurally and establishes no new
obligation. Every entry is `unresolved` today, for the reasons FR-009's "Entry
status today" table records per owner; that table is the current state of this
object's population, not a defect of this specification.
