---
id: ADR-004
title: "Producer immutability is an owner assertion, not a consumer check"
type: ADR
status: accepted
relationships:
  - target: ix://agent-ix/quire-protocol/FR-009
    type: relates_to
  - target: ix://agent-ix/quire-protocol/NFR-003
    type: depends_on
  - target: ix://agent-ix/quire-protocol/NFR-001
    type: depends_on
  - target: ix://agent-ix/quire-protocol/AD-001
    type: depends_on
---
# ADR-004: Producer immutability is an owner assertion, not a consumer check

## Status

Accepted. Reached by reverting a requirement that asked this repository to do
what its own architecture forbids.

## Context

FR-009 pins every external producer to an exact immutable revision. The question
this ADR settles is who establishes that the revision is immutable and
published.

An earlier revision of
[FR-009](../functional/FR-009-lock-external-producer-revisions.md) put that
establishment on this library. Two independent reviews quote the wording. SR-048
FND-001: "FR-009 requires each revision be \"reachable from its owner's published
history\", which no pure boundary can determine without VCS or network access
that AD-001's Views paragraph and NFR-003 forbid; reachability must be an
owner-supplied manifest input, not a check this library performs"
([spec/reviews/it002-it003-packet/scope-boundary.md](../reviews/it002-it003-packet/scope-boundary.md)).
SR-045 FND-001 reaches the same conclusion from the failure domain and adds the
reproducibility argument
([spec/reviews/it002-it003-packet/failure-domain.md](../reviews/it002-it003-packet/failure-domain.md)):
"Reachability is also not a function of the manifest: a force-push or a retention
policy in the owner's repository changes the resolution outcome of an unchanged
manifest, so FR-009-AC-2 is not reproducible and collides with NFR-001."

That is the whole case, and it has three independent legs.

**Capability.** [AD-001](../assurance/AD-001-protocol-library-boundaries.md)'s
Views paragraph states that no boundary "reads the filesystem, spawns a process,
accesses a network, retains evidence, or invokes a business action".
[NFR-003](../non-functional/NFR-003-preserve-rust-assurance-boundary.md)-AC-4
requires the inventory to contain no local producer runner. Determining whether
a commit is reachable from a published branch requires either a repository on
disk or a network call to a forge. Both are outside the boundary. The
requirement asked for a capability the architecture denies.

**Reproducibility.** Reachability is not a property of the inputs.
[NFR-001](../non-functional/NFR-001-reproduce-semantic-results.md) requires
results to reproduce. A force-push or a retention policy in a repository this
one does not own flips the outcome of an unchanged manifest against unchanged
bytes. A check whose answer another party can change without touching any input
cannot be part of a reproducible result.

**Authority.** Immutability of a revision is a claim about someone else's
repository, and that someone else is its owner. This repository has no standing
to adjudicate it and no way to be right about it.

## Decision

The library treats the immutability and publication of a revision as an
assertion the owner records in the manifest entry, not as a property this
library checks. Resolution reads no version-control history, spawns no process,
and accesses no network.

The boundary sits between an assertion and a verification, and each side of it
carries exactly one thing:

- **Asserted, not checked**: that the revision is immutable and published. The
  entry records what Protocol was told.
- **Checked, not asserted**: that the bytes consumed are the bytes the entry
  names. Where an entry supplies bytes, the library recomputes the digest over
  the exact consumed bytes as delivered — no canonicalization, re-encoding or
  normalization — and compares it to the recorded digest before those bytes
  reach a consumer path. The verified byte sequence is the one passed to the
  consumer, so no value is re-read between verification and use.

Where an entry names an interface rather than an artifact — the Engineering
Assurance seam, the E02 reader, the Quoin evidence interface — it carries the
revision and no digest, and the library synthesizes no digest for it.

## Why this is not the self-authorization FR-009 forbids

FR-009 elsewhere refuses to let a producer supply its own acceptance context: no
producer-emitted sidecar, display name, path, or message text can source or
repair a manifest entry, because an offer that supplies its own acceptance
context authorizes itself. An owner-recorded assertion looks superficially like
the same shape and is not, for two stated reasons.

The assertion sits in a Protocol-authored manifest that a producer cannot write. And it
is falsifiable independently: an entry whose digest does not match its bytes
refuses regardless of what the assertion claims. The assertion cannot make wrong
bytes acceptable. It can only make the right bytes attributable.

## Consequences

**A lying or mistaken owner is not detected, and that is accepted.** An owner
can record an assertion for a revision that was later rewritten. Protocol will resolve
the entry, verify the digest over whatever bytes it was handed, and proceed.
What Protocol then holds is bytes it can attribute to a claim someone made — not proof
that the claim was true. This is a genuine reduction in assurance relative to
the requirement that was reverted, and it is the price of staying pure. The
mitigation is not technical: the assertion is a named party's recorded statement,
and misattribution is traceable to them.

**The digest is the only real check, so its preimage had to be nailed down.**
SR-048 FND-002 recorded that the earlier FR-009 "requires recomputing each
artifact's content digest but never states the preimage", and that AD-001
mandates JCS for digest preimages while holding producer-owned digest domains
distinct — so an unpinned preimage would let a second canonicalizer over
producer content appear inside the manifest. That is why FR-009 now says *exact
delivered bytes, no canonicalization of any kind*, and it is why
[ADR-006](./ADR-006-confine-jcs-to-the-protocol-result-domain.md) exists.

**Interface entries have no check at all.** An entry naming a seam rather than
an artifact carries a revision and no digest. For those entries — Engineering
Assurance, E02, Quoin — the owner's assertion is the only thing standing behind
the pin. Nothing local falsifies it.

**Entry status is per contract.** FR-009's architecture-baseline table records
the QSL, QObs, Engineering Assurance and qverify contracts as pinned, and only
the Quoin Protocol scenario-intake entry as unresolved. The Protocol-owned
schema, two-status model, digest recomputation, per-key refusal, entry
independence, sidecar non-authority and bounded resolver are implemented; each
scenario still requires every entry it consumes to be pinned before execution.

## Alternatives considered

**Keep the reachability check and grant the manifest boundary filesystem or
network access.** Rejected. It contradicts AD-001's Views paragraph and
NFR-003-AC-4, and NFR-003-AC-4 is enforced by an inventory check, so the
contradiction would surface as a failing gate rather than as a design debate.
Granting the capability to one boundary also removes the property that makes
every other boundary auditable in one sentence.

**Have Engineering Assurance perform the reachability check and transport the
result.** Rejected for now, and this is the alternative that stays live. It
would keep the check outside this boundary while still producing a verified
rather than asserted answer, and it is the natural home given that EA already
owns producer invocation. It remains optional because accepted EA
`producer_execution` owns execution, not repository-governance truth.
`publish = false` does not prevent pinning an exact Git revision, but no current
requirement asks EA to attest VCS reachability.

**Drop immutability from the entry and rely on the digest alone.** Rejected. The
digest establishes that bytes are the bytes; it establishes nothing about *which
revision produced them*, which is the attribution US-002 needs. A result
attributable to bytes but not to a revision cannot be assessed without
confusing one producer's authority for another's.
