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

Accepted for producer-owned artifact digests consumed by this repository.
Result identities are structural under FR-286; this repository does not
select or compute a digest domain for them.

## Context

This repository holds producer-owned artifact digests it did not compute.
Nothing in the bytes says which artifact domain a value belongs to.

If equality is byte equality, then a producer's model digest presented in an
artifact slot compares equal to an artifact digest over the same bytes, and an
identity substitution survives every check this repository performs. `AP-001`'s
`impact-identity-substitution` scenario is exactly this impact: SR-027 states it
as "changed source, dependency, observation, profile, result, or adapter bytes
retain an accepted identity"
([spec/reviews/digest-domain-discrimination.md](../reviews/digest-domain-discrimination.md)).

The earlier `FR-201-canonical-identity-domain` contract supplied that rule and
vocabulary. QSpec #520 withdrew its digest-domain criteria and now defines
structural identities; this ADR's remaining selection concerns only the
producer-owned artifact digest described by QP FR-015.

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
label compares unequal to the upstream label over identical bytes. That is the "second locally
authored profile" FR-001 exists to refuse, applied to identity domains rather
than to protocol semantics. The local set was withdrawn in commit `802e81d`.

## Decision

A digest is compared under a domain label before its bytes are compared. A
digest under one domain is unequal to a digest over identical bytes under
another, in both directions. A digest with no selected domain compares only to
absence and is never defaulted to a member.

This repository mints no digest-domain label. Its current artifact selection
is:

| Selected member | Used for |
| --- | --- |
| `raw-artifact-digest` | The exact supplied bytes of an offered artifact, as delivered, never recanonicalised. |

A producer reference arriving already carrying an upstream label retains that
label exactly and is never re-labelled into another member.

FR-015 states the rule and the selection. FR-001 carries the intake half — it
"SHALL NOT reinterpret a producer-owned model/configuration digest through this
artifact's numeric or JCS domain". IT-001-SC-07 is the evidence: a
producer digest presented under an unselected label returns a typed refusal,
and the crossing is exercised over raw bytes.

## Consequences

**The implementation selects only the upstream artifact domain.**
`src/digest/producer.rs` exposes `RawArtifact` with the exact `FR-015` label
`raw-artifact-digest`. Result identities use the structural key required by
FR-286; no result digest is derived.

**The remaining digest selection is upstream-owned.** This repository selects
`raw-artifact-digest` only for offered artifact bytes. Result identities and
other semantic identities follow their current upstream contracts; this
repository does not assign them a local digest domain. An unselected artifact
digest slot still refuses rather than borrowing the `raw-artifact-digest`
selection.

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
The digest's domain label is distinct from its bytes; locally prefixing a label
into the preimage would change the selected digest contract. The adopted rule
compares the selected label before comparing bytes.

**Mint a local domain vocabulary fitted to this repository's slots.** Tried, and
withdrawn — see "The second defect" above. It is recorded here as an alternative
because it was not merely considered: it was specified, reviewed, and had to be
reverted.
