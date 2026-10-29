---
id: ADR-006
title: "Confine RFC 8785 JCS to results this repository produces"
type: ADR
status: accepted
relationships:
  - target: ix://agent-ix/quire-protocol/FR-006
    type: relates_to
  - target: ix://agent-ix/quire-protocol/FR-001
    type: relates_to
  - target: ix://agent-ix/quire-protocol/FR-009
    type: relates_to
  - target: ix://agent-ix/quire-protocol/FR-015
    type: depends_on
---
# ADR-006: Confine RFC 8785 JCS to results this repository produces

## Status

Accepted for intake, and **contested for output**. The rule as stated below —
JCS on production, never on intake — is settled and enforced. Whether JCS is
confined to *one* domain is not: FR-007 and AD-001 each say otherwise, and
neither is reconciled. Both readings are recorded below rather than merged.

## Context

This repository owns a canonicaliser for its result identity. [FR-006](../functional/FR-006-produce-canonical-protocol-results.md)
requires canonical protocol result bytes and an FR-286 identity; the external
ArtifactRef carries no digest.

Owning a canonicaliser next to a stream of externally produced bytes creates a
standing temptation: run the producer's bytes through it, and then two inputs
that differ only in whitespace or key order become the same input. That is
exactly the wrong behaviour for a consumer whose whole job is to detect that an
offer differs from a selection.

The general form is that canonicalisation is an equivalence-class collapse. It
is correct when this repository *is* the authority for the class — its own
result bytes, where JCS is how the result's identity is defined. It is a defect
when some other party is the authority, because the collapse discards exactly
the byte-level difference that would have refused.

## Decision

RFC 8785 JCS is applied to bytes this repository produces. It is never applied
to bytes this repository received.

Three requirements carry the intake half:

- [FR-001](../functional/FR-001-admit-selected-protocol-profile.md) states that
  the library "SHALL NOT reinterpret a producer-owned model/configuration digest
  through this artifact's numeric or JCS domain".
- [FR-009](../functional/FR-009-lock-external-producer-revisions.md) requires the
  entry digest to be recomputed "over the exact consumed bytes as delivered,
  with no canonicalization, re-encoding, or normalization of any kind", and the
  verified byte sequence to be the one passed to the consumer.
- [FR-015](../functional/FR-015-digest-domain-selection.md) publishes digest
  domains; this consumer does not use one to seal an external ArtifactRef.
  `quire.verification.jcs` remains the domain for this repository's result
  identity.

IT-001-SC-07 is the negative control, and it was written to be non-vacuous: the
recanonicalization probe uses a pretty-printed JSON re-encoding of the original
model bytes, byte-distinguishable from the as-delivered form, so a producer input
that is already canonical cannot make the scenario pass for free.

## The contested part: whether JCS is confined to one domain

The claim that JCS is used in exactly one place does not survive contact with
two other artifacts in this repository. Neither is edited here; both readings
are quoted.

**[AD-001](../assurance/AD-001-protocol-library-boundaries.md) says JCS is
universal.** Its Decisions section: "Use RFC 8785 JCS for every digest or
semantic identity preimage, admit bare JSON numbers only within the exact
interoperable integer interval, and require contract-defined canonical
non-number encodings for wider integers and exact rationals. Producer-owned
digest domains remain distinct typed inputs."

That is the pre-remediation position, and it is directly contradicted by
FR-009's "no canonicalization, re-encoding, or normalization of any kind" over a
producer-supplied entry. SR-048 FND-002 named the hazard before FR-009 was
fixed: an unpinned preimage, against an AD-001 that mandates JCS, "lets a second
canonicalizer over producer content appear inside the manifest — the exact
crossing the intake section forbids". FR-009 was corrected; AD-001's Decisions
bullet was not. **Unresolved.** AD-001's owner decides whether that bullet is
narrowed to first-party preimages or whether FR-009 and FR-015 are wrong.

**[FR-007](../functional/FR-007-project-results-to-e02.md) uses JCS in a second
domain.** It requires the adapter to "compute a distinct RFC 8785 projection
identity over the complete FR-062 preimage", and FR-007-AC-2 requires that
identity to remain distinct from the canonical result identity. Upstream
`FR-201` records the domain that identity is taken under: for FR-062, "the
derived result identity is taken over a distinct projection-identity preimage
under `quire.protocol.e02-projection-identity/v1-draft.1`".

So JCS is the canonicalisation for at least two distinct output identities, under
two distinct domain labels, and FR-015's selection table names only one of them.
That is not a contradiction in behaviour — both are first-party output, both are
domain-separated, and the projection identity is *required* to differ from the
result identity — but it does falsify the flat statement that JCS is confined to
the protocol-result domain. **FR-015's selection table is incomplete**: it
selects no member for the E02 projection identity that FR-007 computes. The
member exists upstream and is named above; selecting it is a correction to
FR-015, reported rather than made here.

`FR-201` also records that preimage selection is per-use rather than per-domain:
three upstream requirements independently state what is inside the
`quire.verification.jcs` preimage and no requirement cross-references the
others. It states that it "does not reconcile them; the selection of a preimage
belongs to the requirement that selects it". So even within one domain, "JCS"
does not name one preimage.

## Consequences

**Byte-identical-after-canonicalisation inputs refuse, and callers will call
that a bug.** A producer that reformats its output without changing its meaning
produces bytes this repository refuses. There is no tolerant mode and no
normalise-then-compare fallback. The correct fix is on the producer's side or in
a repinned entry, and no amount of caller pressure changes that, because the
alternative is a consumer that cannot tell a reformat from a substitution.

**The canonicaliser is deliberately incomplete, and stays that way.** SR-027
FND-010 recorded that "The RFC 8785 canonicaliser refuses JSON numbers rather
than implementing ECMAScript number serialisation" (`src/digest/jcs.rs:60`). Its
disposition is "open by design", with the reason stated: "Approximating
ECMAScript number serialisation would produce a wrong canonical form that still
looks well formed, which is the failure class this crate exists to refuse." A
refusal is a correct answer; a plausible-looking wrong canonical form is not.
The consequence is that any result field needing a JSON number outside what the
canonicaliser accepts cannot be canonicalised at all, and FR-001's numeric rules
push those values into contract-defined non-number encodings instead.

**Two first-party identities must stay distinct, and only one is specified.**
FR-007-AC-2 requires the projection identity to differ from the canonical result
identity across every source, target-binding, contract and adapter mutation.
Under [ADR-003](./ADR-003-compare-digest-domain-before-bytes.md)'s rule the
domain label is what makes them unequal — and FR-015 selects a label for one of
the two. Until FR-015 selects the projection domain, that distinctness rests on
preimage difference alone, which is weaker than the domain-first rule this
repository adopted.

## Alternatives considered

**Canonicalise producer bytes on intake and compare canonical forms.** Rejected.
It discards the difference the comparison exists to find, and it would make this
repository an authority over the canonical form of someone else's artifact. It
is also the specific crossing SR-048 FND-002 identified as reachable through an
unstated preimage, so it is a live failure mode rather than a hypothetical one.

**Use one JCS domain for every first-party identity.** Rejected, and the
rejection is not this repository's to make: FR-007-AC-2 requires the projection
identity to be distinct from the result identity, and upstream `FR-062` already
selects a separate projection-identity domain for it. One shared domain would
make two identities that must differ compare in the same space.

**Adopt a different canonicalisation for intake comparison — a tolerant JSON
equality, say.** Rejected. Any equivalence weaker than byte equality is a
policy about which producer differences are meaningless, and that policy belongs
to the producer's contract, not to its consumer.
