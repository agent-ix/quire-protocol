---
id: FR-009
title: "Lock every external producer revision in one manifest"
type: FR
relationships:
  - target: ix://agent-ix/quire-protocol/ADR-004
    type: references
  - target: ix://agent-ix/quire-protocol/ADR-006
    type: references
  - target: ix://agent-ix/quire-protocol/ADR-009
    type: references
  - target: ix://agent-ix/quire-protocol/US-003
    type: implements
  - target: ix://agent-ix/quire-protocol/US-002
    type: implements
  - target: ix://agent-ix/quire-protocol/FR-008
    type: depends_on
---
# FR-009: Lock every external producer revision in one manifest

## Description

When an integration consumes a value produced outside this repository,
the library SHALL resolve that producer through one version-lock manifest
pinning every external producer to an exact immutable revision and, where the
producer supplies bytes, to the content digest of those bytes.
If a manifest entry required by that integration is absent, unpinned,
declared unresolved, or resolves to different bytes, then the library SHALL
refuse that integration.

This requirement descends from `StR-002-attribute-results-to-pinned-producer-revisions`
through `US-003-attribute-an-outcome-to-pinned-revisions`: a result whose
producer revision is unknown cannot be attributed, and an unattributable result
cannot be assessed without confusing one producer's authority for another's.

An earlier revision of this paragraph said the requirement was "a cross-cutting
constraint derived from EPIC #14 rather than from a single user story", serving
`US-002` only indirectly. That was the honest answer to `SR-049` FND-001 at the
time -- the finding recorded that this requirement descended from no story, and
the available remedies were to author the missing story or to record the
indirect descent. The story now exists, so the first remedy is taken and the
paragraph is corrected. The `US-002` edge is retained alongside `US-003`
because the indirect service it names is still real.

## Inputs

- One version-lock manifest. It is B-authored and never producer-supplied. It is
  a distinct artifact from any delivery index a producer ships beside its bytes:
  a delivery index is evidence B reads about what was delivered, and it can
  neither supply nor repair a version-lock entry. Each entry is keyed by the pair (owner, contract
  version) and carries that key, an exact immutable revision, the owner's own
  assertion that the revision is immutable and published, the content digest of
  the bytes consumed from it where the producer supplies bytes, and its
  resolution status.
- The external producers: the A-owned compiled-protocol artifact contract at each
  published version, the E-owned temporal profile, the F-owned observation and
  output-mapping contracts, the Engineering Assurance typed producer-execution
  seam, the E02 strict reader, the selected diagnostic catalog whose identity,
  revision and digest FR-008 binds, and the Quoin evidence interface. The
  complete-global-conformance closure record is not among them: it is B-authored
  in `quire-specification` and takes no entry, because no external owner can
  assert anything about it.
- The set of manifest entries each integration scenario requires, declared by
  that scenario in its own integration test, not by the manifest.

## Outputs

- One resolved producer set in which every entry required by the integration is
  pinned and, where it supplies bytes, digest-verified; or one typed refusal
  naming the entry that is not.

## Behavior

### Resolution

The library SHALL resolve every manifest entry an integration requires before it
reads any producer value for that integration.
The library SHALL treat the immutability and publication of a revision as an
assertion the owner records in the entry, not as a property this library checks.
The library SHALL NOT read a version-control history, spawn a process, or access
a network to resolve an entry, because AD-001 and NFR-003 place those capabilities
outside this boundary.

### Digest verification

Where an entry supplies bytes, the library SHALL recompute that entry's digest
over the exact consumed bytes as delivered, with no canonicalization,
re-encoding, or normalization of any kind, and SHALL compare it to the recorded
digest before those bytes reach a consumer path.
The library SHALL pass to the consumer the same byte sequence it verified, so no
value is re-read between verification and use.
Where an entry names an interface rather than an artifact — the Engineering
Assurance seam, the E02 reader, the Quoin evidence interface — the entry SHALL
carry the immutable revision and no digest, and the library SHALL NOT synthesize
a digest for it.

### Entry identity and independence

The manifest SHALL carry at most one entry per (owner, contract version) key.
If two entries share that key, then the library SHALL refuse every integration
before resolution, because no refusal can name an ambiguous entry.
The library SHALL NOT let a change to one entry alter the resolved owner,
revision, digest or status of another entry.
The manifest SHALL declare no dependency between entries, so resolution needs no
order and can contain no cycle.

### Resolution status

Each entry SHALL carry exactly one status: `pinned`, or `unresolved` with the
reason it cannot be pinned.
The library SHALL refuse every scenario that requires an `unresolved` entry, and
that scenario SHALL be reported as unexecutable rather than passing or failing.
An `unresolved` status SHALL NOT be satisfied by substituting a different entry,
including another contract version of the same owner.

### Self-authorization

The library SHALL NOT accept a producer-emitted sidecar, display name, path, or
message text as the source or repair of a manifest entry, because an offer that
supplies its own acceptance context authorizes itself.
The manifest SHALL NOT declare which scenarios require which entries, because a
document that declares its own requiredness can unblock a scenario by editing
itself rather than by a producer change.

### Bounds

Resolution SHALL consume a caller-supplied budget covering the entry count, the
total bytes digested, and a deadline.
If resolution exceeds any of those, then the library SHALL return that
dimension's typed resource-incomplete outcome and SHALL read no producer value.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-009-AC-1 | A manifest whose required entries are each `pinned`, carry the owner's immutability assertion, and whose recorded digest equals the digest recomputed over the exact delivered bytes, resolves with every key, revision, digest and status retained unchanged, and the consumer receives the same byte sequence that was verified. | Test (TC-013, IT-002, IT-003) |
| FR-009-AC-2 | An absent, unpinned or digest-mismatched required entry returns a typed refusal naming that entry's (owner, contract version) key before any producer value for the dependent scenario is read, and the differing digest is not written back. | Test (TC-013, IT-002, IT-003) |
| FR-009-AC-3 | A scenario requiring an entry whose status is `unresolved` is reported unexecutable, is not reported as passing or failing, and cannot be satisfied by another contract version of the same owner. | Test (TC-013, IT-002, IT-003) |
| FR-009-AC-4 | A producer-emitted sidecar, display name, path, or message text cannot supply, select, or repair an entry; removing the sidecar changes no resolution outcome; and the manifest cannot change which entries a scenario requires. | Test (TC-013, IT-002, IT-003) |
| FR-009-AC-5 | Two entries sharing one (owner, contract version) key refuse every integration before resolution; otherwise adding, removing or repinning one entry changes the resolved owner, revision, digest and status of no other, and a manifest carrying two versions of one producer resolves each independently. | Test (TC-013, IT-002, IT-003) |
| FR-009-AC-6 | Resolution at exactly the granted entry-count, byte and deadline budget completes; one unit short on any dimension returns that dimension's typed resource-incomplete outcome with no producer value read; and the same manifest under a fresh budget completes. | Test (TC-013, IT-002, IT-003) |
| FR-009-AC-7 | Neither resolution nor digest verification reads a version-control history, spawns a process, or accesses a network, and no entry's digest is computed over anything but the exact delivered bytes. | Test (TC-010, TC-013) |

## Dependencies

Local ownership: [FR-001](./FR-001-admit-selected-protocol-profile.md) admits the
compiled-protocol artifact after its entry resolves, so FR-001 depends on this
requirement and not the reverse.
[FR-008](./FR-008-return-typed-refusals.md) owns the refusal envelope,
[NFR-002](../non-functional/NFR-002-enforce-resource-bounds.md) the
resource-incomplete outcome, and
[NFR-003](../non-functional/NFR-003-preserve-rust-assurance-boundary.md) the
containment resolution runs under.

### Gate order

The gates close in this order. Each stage is blocked by the one above it, and
this section is not satisfied while any stage above an entry is still moving.
Entries carrying no dependency on each other is a property of resolution at run
time; it says nothing about the order in which these gates close.

1. **B, no external dependency.** The manifest schema, the two-status resolution
   model, digest recomputation over exact delivered bytes, per-key refusal, entry
   independence, sidecar non-authority, and the resolution budget. TC-013
   verifies all of it on B-owned fixtures. Nothing below can be built until this
   exists, and no manifest artifact exists in the tree today.
2. **Per owner: publication.** An owner publishes an immutable revision and
   records its assertion. Until then that owner's entry is `unresolved`.
3. **Per entry: pinning.** The entry moves from `unresolved` to `pinned`.
4. **Per scenario: execution.** A scenario reads a producer value only after
   every entry it requires is `pinned`.
5. **NFR-005.** A scenario's evidence record retains the resolution state, so it
   cannot exist before stage 3 produces one.

### Entry status today

Every entry is `unresolved`. None can be pinned, and each names why.

| Entry | Gate | Why unresolved |
| --- | --- | --- |
| A `/1` compiled protocol | quire-spec-language#40 | The current pin `5690c2b` is not an ancestor of A's `origin/main`; it exists only on `agent-a/handoff-numeric-domains`. A has no tags and no releases. |
| A `/2` compiled protocol | quire-spec-language#40 | Implemented and under final integration, present only in an unpushed local worktree. No published revision exists to assert. |
| E temporal profile | Accepted temporal contract | No Rust interface exists at any state; E's contribution is merged prose in `quire-specification`. |
| F observation / output mapping | F-owned FR-115/FR-116, TC-145/TC-146, IT-051 | `quire-observation` is a real Rust crate but is `publish = false`, and this crate declares no dependency on it. |
| Engineering Assurance seam | engineering-assurance#34 | `producer_execution` is merged on `origin/main` but `publish = false`, no release contains it, and the cited acceptance revision is not reachable after its PR was squashed. |
| E02 strict reader | quire-verification FR-007 | Zero tags, zero releases; the interfaces are declared provisional across repositories and the code sits on a branch. |
| Diagnostic catalog | quire-specification FR-270 | The only catalog offered is the proposed native diagnostic interpretation, which states of itself "Proposed, not adopted or implemented"; FR-270 calls it source material, not an adopted selection. |
| Quoin evidence interface | quire-research#62 | Quoin exposes no Rust-callable boundary and contains no `Cargo.toml`; the containment contract is open with every acceptance box unchecked. |

Stage 1 is unblocked and owes no external party. Stages 2 through 5 are blocked
on all seven gates above.

### On the owner's assertion

Making publication an owner-recorded assertion is not the self-authorization
this requirement forbids elsewhere. The forbidden shape is an *offer* supplying
the context that accepts it: the producer's own bytes selecting their own
expected values. Here the assertion sits in a B-authored manifest that a producer
cannot write, and it is falsifiable independently — an entry whose digest does
not match its bytes refuses regardless of what the assertion claims. The
assertion records what B was told, not what B verified, and the table above is
the record of B not having been told it yet by anyone.
