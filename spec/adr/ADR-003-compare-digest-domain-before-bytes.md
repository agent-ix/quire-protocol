---
id: ADR-003
title: "Compare the digest domain before the bytes"
type: ADR
status: accepted
relationships:
  - target: ix://agent-ix/quire-protocol/FR-015
    type: relates_to
  - target: ix://agent-ix/quire-protocol/FR-001
    type: relates_to
  - target: ix://agent-ix/quire-protocol/IT-001
    type: relates_to
  - target: ix://agent-ix/quire-specification/FR-201
    type: depends_on
---
# ADR-003: Compare the digest domain before the bytes

## Status

Accepted as a rule. The vocabulary the rule selects from is upstream's and
settled; this repository's implementation enum does not yet conform (see
Consequences).

## Context

This repository holds digests it did not compute alongside digests it did. A
producer's artifact digest, a producer's source digest, a producer's model
digest and this repository's own protocol-result digest are all 32 bytes of
SHA-256 rendered as hex. Nothing in the bytes says which of those a value is.

If equality is byte equality, then a producer's model digest presented in the
slot that holds a protocol-result digest compares equal to a protocol-result
digest over the same bytes, and an identity substitution survives every check
this repository performs. `AP-001`'s
`impact-identity-substitution` scenario is exactly this impact: SR-027 states it
as "changed source, dependency, observation, profile, result, or adapter bytes
retain an accepted identity"
([spec/reviews/digest-domain-discrimination.md](../reviews/digest-domain-discrimination.md)).

Upstream had already decided the rule. `FR-201-canonical-identity-domain` is
`object: enumeration` and states in its Description that "the reader SHALL
compare the selected identity domain before the bytes, so that an identity
derived under one domain is unequal to an identity derived under another
regardless of byte equality", and in its Cross-domain rule that "two identities
bearing different domain labels are unequal. Byte equality of their digests does
not make them equal". It publishes nine members and says of itself: "This
artifact *is* the vocabulary of digest domains".

### The defect that produced this decision

Two high findings, both shipped on `main` at `681e25a`, both recorded in SR-027:

- FND-001 — "Cross-domain digest case asserted `Error::Json`, a parse failure,
  so a well-formed substitution would pass it untouched"
  (`tests/it001_native_handoff.rs:496`).
- FND-002 — "Producer identity, baseline selection state and baseline digest all
  asserted the single shared `Invalid::Selection` code"
  (`tests/it001_native_handoff.rs:344`).

SR-027's Summary records the joint effect: "four landed assertions passed for the
wrong reason", and its verdict is FAIL. The first finding is the one that bears
on this ADR. The landed test proved only that a *malformed* cross-domain value
was rejected by the JSON reader. A well-formed digest from the wrong domain —
which is the attack, and the only interesting case — was never exercised. The
measurement taken during that review is the load-bearing fact: A alone returns
`Invalid(Selection)` for such an offer, the same code as two other axes. So
domain crossing was, at that revision, both undetected by B and
indistinguishable in A's report.

### The second defect, at the vocabulary level

Remediating the rule was not enough, because this repository then authored the
wrong vocabulary for it. An earlier revision of
[FR-015](../functional/FR-015-digest-domain-selection.md) declared a closed
local four-member set — `producer-canonical`, `native-raw`, `compiled-artifact`,
`protocol-result` — on the stated ground that `FR-201` typed the domain label
kind while selecting no member. FR-015 now records that this was wrong, and why
it was worse than redundant: two of the four re-spelled members upstream already
publishes, and because both vocabularies make spelling load-bearing, a local
label compares unequal to the upstream label over identical bytes. This
repository's own result digest would have compared unequal to the same bytes
under the ecosystem's own verification label. That is the "second locally
authored profile" FR-001 exists to refuse, applied to identity domains rather
than to protocol semantics. The local set was withdrawn in commit `802e81d`.

## Decision

A digest is compared under a domain label before its bytes are compared. A
digest under one domain is unequal to a digest over identical bytes under
another, in both directions. A digest with no selected domain compares only to
absence and is never defaulted to a member.

The labels are `FR-201`'s, spelled exactly as upstream publishes them. This
repository mints none. It selects two:

| Selected member | Used for |
| --- | --- |
| `raw-artifact-digest` | The exact supplied bytes of an offered artifact, as delivered, never recanonicalised. |
| `quire.verification.jcs` | This repository's own protocol-result digest. |

A producer reference arriving already carrying an upstream label retains that
label exactly and is never re-labelled into another member.

FR-015 states the rule and the selection. FR-001 carries the intake half — it
"SHALL NOT reinterpret a producer-owned model/configuration digest through this
artifact's numeric or JCS domain". IT-001-SC-07 is the evidence: a
producer-canonical, native-raw, compiled-artifact or protocol-result digest
presented in another domain's slot returns a typed refusal, and the crossing is
exercised over raw bytes.

## Consequences

**The implementation does not conform, and this ADR does not fix it.**
`src/digest/producer.rs` declares `DigestDomain` with five members —
`ProducerArtifactBytes`, `ProducerSource`, `ProducerModel`, `ProducerConfig`,
`ProtocolResultJcs` — none spelled as `FR-201` publishes them, and the `/2`
lane's test enum names four more. FR-015 records this as an unreconciled
divergence and declines to rename either side. The rule is specified and the
code is non-conformant; that gap is real and open, and it is implementation work
owned by this repository, not a question for upstream.

**IT-001-SC-07 still names the withdrawn vocabulary.** SC-07 and IT-001's Inputs
describe "digest-domain substitution across the producer-canonical, native-raw,
compiled-artifact, and protocol-result domains" — the four withdrawn local
labels. The scenario's substance survives the withdrawal (crossing refuses), but
its four names no longer correspond to anything specified. IT-001 is owned by
another agent in this campaign and is not edited here; it is reported as a
needed correction.

**Selecting only two members means most producer digests have no domain.**
`raw-artifact-digest` covers offered artifact bytes. A producer's *source*,
*model* and *configuration* digests are not artifact bytes in that sense, and
FR-015 selects no member for them. FR-015 records that `FR-259`, `FR-261` and
`FR-263` each note that no requirement selects a domain member for the digests
they carry, and routes the question upstream rather than answering it. The
operational consequence is deliberate and blunt: an unselected domain compares
only to absence, so a comparison requiring a domain refuses rather than
guessing. Refusing is correct and it also means those comparisons cannot
currently be made at all.

**Every digest slot now needs a per-slot guard, and they are not all written.**
SR-027 FND-003 is open: "Per-source, per-dependency and per-model digest slots
are not guarded; only the three top-level selection references are"
(`src/inventory/map.rs:96`). Its disposition records why — the per-entry slots
need the independently selected per-source and per-dependency inventory that a
later fixture-builder chunk lands — and that the scope limit is stated in the
guard's own doc comment rather than left implied.

## Alternatives considered

**Byte equality, with domain separation enforced by keeping the values in
separate typed fields.** Rejected. This was effectively the pre-`681e25a` state,
and it failed: the offered package supplies values into slots, and a value
placed in the wrong slot is exactly what the guard must catch. Type separation
protects against a programming mistake on B's side; it does nothing against a
crafted offer.

**Domain-prefix the digest preimage, so a crossed digest simply does not
match.** Rejected, and the rejection is upstream's, not this repository's.
`FR-201` states that "The `sha256-jcs:` textual prefix is a domain label outside
the hashed bytes. It is not a hash preimage prefix (FR-018)". Adopting preimage
prefixing locally would change what every digest *is*, not just how it compares,
and would make this repository's digests incomparable with the rest of the
ecosystem's.

**Mint a local domain vocabulary fitted to this repository's slots.** Tried, and
withdrawn — see "The second defect" above. It is recorded here as an alternative
because it was not merely considered: it was specified, reviewed, and had to be
reverted.
