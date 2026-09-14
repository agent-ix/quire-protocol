---
id: TC-013
title: "Resolve the version-lock manifest"
type: TC
relationships:
  - target: ix://agent-ix/quire-protocol/FR-009
    type: verifies
---
# TC-013: Resolve the version-lock manifest

## Description

Verify FR-009's manifest resolution against Protocol-owned fixtures, without any
external producer delivery. Resolution is B's own logic; it must not be provable
only through an integration that waits on someone else.

## Preconditions

Protocol-owned manifest fixtures only. No fixture is an artifact emitted by an external
producer, and no assertion in this test case depends on an external revision
being published.

## Test Data

Manifests carrying: every required entry `pinned` with an owner immutability
assertion and a matching digest; an absent entry; an entry with no revision; an
entry whose recorded digest differs from the bytes; an entry with status
`unresolved` and a reason; two entries sharing one (owner, contract version)
key; two entries for one owner at different contract versions; entries naming an
interface rather than an artifact, carrying a revision and no digest; and byte
sequences whose canonical re-encoding differs from their delivered form. Each
interface fixture has an independently compiled accepted owner/contract/revision
selection. Multi-error manifests are supplied in every presentation order.

## Test Procedure

1. Resolve a manifest whose required entries are all `pinned`.
   - TC-013-SC-01: every key, revision, digest and status is retained unchanged,
     and the consumer receives the same byte sequence that was verified.
2. Resolve manifests carrying an absent, revision-less, or digest-mismatched
   required entry, one at a time.
   - TC-013-SC-02: each returns a typed refusal naming that entry's key before
     any entry's bytes are passed on, and the differing digest is not written
     back into the manifest.
3. Resolve a manifest whose required entry has status `unresolved`.
   - TC-013-SC-03: the dependent scenario is reported unexecutable, is reported
     neither passing nor failing, and is not satisfied by another contract
     version of the same owner present in the same manifest.
4. Resolve a manifest carrying two entries under one key, then one carrying two
   contract versions of one owner.
   - TC-013-SC-04: the duplicate key refuses every scenario before resolution;
     the two contract versions resolve independently, and repinning either
     changes the other's resolved owner, revision, digest and status not at all.
5. Deliver bytes whose canonical re-encoding differs from the delivered form.
   - TC-013-SC-05: the digest is computed over the delivered bytes, the
     re-encoded form fails verification, and no canonicalization, re-encoding or
     normalization is applied at any point.
6. Resolve an interface-only entry carrying a revision and no digest.
   - TC-013-SC-06: resolution succeeds without synthesizing a digest, and an
     interface entry carrying a digest is refused.
7. Resolve at exactly the granted entry-count, byte and deadline budget, then
   one unit short on each dimension, then again under a fresh budget.
   - TC-013-SC-07: the exact budget completes; one short returns that
     dimension's typed resource-incomplete outcome with no entry's bytes passed
     on; the fresh budget completes.
8. Inspect the resolution path for containment.
   - TC-013-SC-08: resolution reads no version-control history, spawns no
     process and opens no network connection, and the owner's immutability
     assertion is read from the entry rather than checked against a history.
9. Resolve interface entries against the adapter-compiled accepted selection,
   then mutate only the manifest revision and reuse a capability with another
   request/value.
   - TC-013-SC-09: only the exact compiled selection mints a private capability;
     the revision mutation and cross-request/value reuse each refuse.
10. Resolve every permutation of one manifest containing multiple bad required
    keys.
    - TC-013-SC-10: every permutation returns the same first causing key and
      partial state under lexicographic required-key validation.

## Expected Results

Every resolution outcome is determined by the manifest and the delivered bytes
alone. No outcome depends on an external revision being reachable, and no
refusal is ambiguous about which entry caused it.

## Metadata

- Priority: P0
- Type: Property
- Automation: Automated; Protocol-owned fixtures only, no external dependency

## Traceability

Verifies FR-009-AC-1 through FR-009-AC-9 and contributes to NFR-003.

This test case additionally backs criteria on the object requirements FR-012,
FR-016 and FR-018, which cite it as their verification. FR-017 composition is
owned by TC-016 after final outcomes/resources exist.

This test case additionally backs `NFR-006-AC-6`, `NFR-006-AC-7` and
`NFR-006-AC-8`, the delivery-surface criteria of
[NFR-006](../non-functional/NFR-006-fix-the-consumed-dependency-set.md), which
name it as their verification. Their negative controls — an unenumerated file in
an otherwise complete delivery root, a member's bytes swapped at the same path,
and an edited or deleted delivery index — run on the Protocol-owned fixtures above and
require no producer delivery. NFR-006's five closure criteria, `NFR-006-AC-1`
through `NFR-006-AC-5`, verify by inspection and are not backed here.
