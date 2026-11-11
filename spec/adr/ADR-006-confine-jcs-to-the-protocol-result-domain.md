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

Accepted for intake and output bytes. Once a canonicaliser is adopted (through
quire-canonical), RFC 8785 JCS applies only to bytes this repository produces;
it does not derive a result identity or digest. Intake preserves producer bytes
as delivered. This repository currently contains no canonicaliser.

## Context

This repository has no canonicaliser of its own; result-byte canonicalisation
is to come from quire-canonical.
[FR-006](../functional/FR-006-produce-canonical-protocol-results.md) requires
canonical protocol result bytes and an FR-286 structural identity; the
external ArtifactRef carries no digest.

Owning a canonicaliser next to a stream of externally produced bytes creates a
standing temptation: run the producer's bytes through it, and then two inputs
that differ only in whitespace or key order become the same input. That is
exactly the wrong behaviour for a consumer whose whole job is to detect that an
offer differs from a selection.

The general form is that canonicalisation is an equivalence-class collapse. It
is correct when this repository *is* the authority for the class — its own
result bytes, whose serialization is canonicalized with JCS. It is a defect
when some other party is the authority, because the collapse discards exactly
the byte-level difference that would have refused.

## Decision

RFC 8785 JCS is applied to bytes this repository produces. It is never applied
to bytes this repository received. It does not produce a digest identity;
QSpec FR-286 defines the structural result key, and FR-062/FR-249 define the
structural projection identity.

Three requirements carry the intake half:

- [FR-001](../functional/FR-001-admit-selected-protocol-profile.md) states that
  the library "SHALL NOT reinterpret a producer-owned model/configuration digest
  through this artifact's numeric or JCS domain".
- [FR-009](../functional/FR-009-lock-external-producer-revisions.md) requires the
  entry digest to be recomputed "over the exact consumed bytes as delivered,
  with no canonicalization, re-encoding, or normalization of any kind", and the
  verified byte sequence to be the one passed to the consumer.
- [FR-015](../functional/FR-015-digest-domain-selection.md) selects the
  upstream `raw-artifact-digest` member for artifact bytes; this consumer does
  not use it to seal an external ArtifactRef.

IT-001-SC-07 is the negative control, and it was written to be non-vacuous: the
recanonicalization probe uses a pretty-printed JSON re-encoding of the original
model bytes, byte-distinguishable from the as-delivered form, so a producer input
that is already canonical cannot make the scenario pass for free.

## Result identities

The earlier implementation derived result and projection identities from JCS
preimages. QSpec #520 removed those digest-domain selections. This repository
now carries the FR-286 result key and the FR-062/FR-249 projection identity as
structural objects, compared by exact structure. JCS is used for canonical
result bytes only.

FR-007 delegates the projection identity structure to the QSpec contract. The
adapter compares that structure exactly; it does not hash the source key,
adapter, context, or result bytes to derive an identity.

## Consequences

**Byte-identical-after-canonicalisation inputs refuse, and callers will call
that a bug.** A producer that reformats its output without changing its meaning
produces bytes this repository refuses. There is no tolerant mode and no
normalise-then-compare fallback. The correct fix is on the producer's side or in
a repinned entry, and no amount of caller pressure changes that, because the
alternative is a consumer that cannot tell a reformat from a substitution.

**A canonicaliser, once adopted, must refuse JSON numbers it cannot serialise
exactly.** SR-027 FND-010 recorded the rule for the earlier implementation, which
was deleted (TL-251): refuse JSON numbers rather than approximate ECMAScript
number serialisation. Approximating it would produce a wrong canonical form that
still looks well formed, which is the failure class this crate exists to refuse.
A refusal is a correct answer; a plausible-looking wrong canonical form is not.
The consequence is that any result field needing a JSON number outside what the
adopted canonicaliser accepts cannot be canonicalised at all, and FR-001's
numeric rules push those values into contract-defined non-number encodings
instead.

**The structural identities stay distinct by their specified structure.**
QSpec FR-286 defines the canonical result key, and FR-062/FR-249 define the
projection identity. This repository carries those structures without
deriving a digest from either one.

## Alternatives considered

**Canonicalise producer bytes on intake and compare canonical forms.** Rejected.
It discards the difference the comparison exists to find, and it would make this
repository an authority over the canonical form of someone else's artifact. It
is also the specific crossing SR-048 FND-002 identified as reachable through an
unstated preimage, so it is a live failure mode rather than a hypothetical one.

**Derive a digest for each result identity.** Rejected. The upstream contracts
define structural identities, and this repository carries and compares those
structures exactly.

**Adopt a different canonicalisation for intake comparison — a tolerant JSON
equality, say.** Rejected. Any equivalence weaker than byte equality is a
policy about which producer differences are meaningless, and that policy belongs
to the producer's contract, not to its consumer.
