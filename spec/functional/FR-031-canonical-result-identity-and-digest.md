---
id: FR-031
title: "Bind canonical result identity and digest"
type: FR
relationships:
  - target: ix://agent-ix/quire-protocol/FR-006
    type: part_of
  - target: ix://agent-ix/quire-protocol/FR-015
    type: depends_on
  - target: ix://agent-ix/quire-protocol/FR-011
    type: depends_on
  - target: ix://agent-ix/quire-specification/FR-061
    type: depends_on
  - target: ix://agent-ix/quire-specification/FR-240
    type: references
  - target: ix://agent-ix/quire-specification/FR-201
    type: references
---
# FR-031: Bind canonical result identity and digest

## Description

When a canonical protocol result is emitted, the result producer SHALL bind the contract version, kind and result identity that `FR-240-canonical-protocol-result` states.
When a canonical protocol result is emitted, the result producer SHALL carry the enclosing artifact reference's digest over the complete RFC 8785 canonical result bytes and outside those bytes.
When a canonical protocol result is read, the strict reader SHALL recompute the enclosing artifact reference's digest before accepting the result.

## Inputs

One constructed result fact set, the enclosing artifact reference, the selected
digest domain member, and the exact source, profile, model, protocol, request,
binding, input, backend and producer identities carried as provenance.

## Outputs

One canonical result bearing its contract version, kind, result identity and a
non-self-referential `sha256-jcs` enclosing artifact reference, or one typed
strict-reader refusal.

## Behavior

The result producer SHALL emit `contractVersion` and `kind` exactly as
`FR-240-canonical-protocol-result` records them, and a result identity, without
restating those values as a local vocabulary.

The enclosing artifact reference's digest is `sha256-jcs` over the complete RFC
8785 canonical result bytes. The digest is not a field inside those bytes, so it
is never self-referential, and a strict reader recomputes it before accepting the
result. A recomputed digest that differs from the carried one refuses the result.

The digest domain is the `quire.verification.jcs` member that
[FR-015](./FR-015-digest-domain-selection.md) selects from
`FR-201-canonical-identity-domain`'s published vocabulary. This requirement
selects no member of its own and spells none of its own. RFC 8785
canonicalization is confined to this preimage: an offered artifact's supplied
bytes are digested under the `raw-artifact-digest` member FR-015 selects for
them and are never recanonicalised for comparison.

Every provenance identity the result carries — source, profile, model, protocol,
request, binding, input, backend and producer — is recomputed or strictly
matched against its selected input. None is inferred from display name, source
path, span, message text, locale or rendering, and none is substituted from
another identity of the same shape.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-031-AC-1 | Every emitted result carries the `FR-240` contract version and kind plus a result identity, and a result with a stale identity is refused by the strict reader. | Test (TC-006) |
| FR-031-AC-2 | The enclosing artifact reference's digest is computed over the complete RFC 8785 canonical result bytes and appears nowhere inside those bytes. | Test (TC-006) |
| FR-031-AC-3 | A strict reader recomputes the enclosing digest before accepting, and refuses a result whose recomputed digest differs from the carried one. | Test (TC-006) |
| FR-031-AC-4 | The protocol-result digest carries the `quire.verification.jcs` label exactly as `FR-201` publishes it, and no digest domain label used by this requirement originates in this repository. | Inspection |
| FR-031-AC-5 | Every source, profile, model, protocol, request, binding, input, backend and producer identity on the result is recomputed or strictly matched, and none is inferred from display text or substituted from another identity. | Test (TC-006) |
| FR-031-AC-6 | RFC 8785 canonicalization is applied to the protocol-result digest preimage only; an offered artifact's supplied bytes are digested as delivered under the member FR-015 selects for them. | Inspection |

## Adopted vocabulary

| Noun | Upstream object | Identity kind |
| --- | --- | --- |
| Canonical protocol result | `FR-240-canonical-protocol-result` | not declared upstream; see the gap below |
| Digest domain label | `FR-201-canonical-identity-domain` | value |

The member selection is local and is
[FR-015](./FR-015-digest-domain-selection.md)'s; the vocabulary selected from is
`FR-201`'s. `FR-201` states of itself: "This artifact *is* the vocabulary of
digest domains." Nothing here extends it.

### Where the upstream vocabulary does not yet reach

- **Result-identity derivation kind.** `FR-240` records under Open gap that
  FR-061 requires "a result identity" and FR-061-AC-4 refuses a "stale
  identity", but "no requirement states whether the identity is authored by the
  producer or derived from named components." This requirement therefore obliges
  the identity to be carried and strictly matched, and declares neither kind.
  The identity kind is `quire-specification`'s to state, in FR-061 or FR-240.
- **Identity versus bytes.** `FR-240` records: "No requirement states whether
  two results bearing the same result identity and different canonical bytes are
  admissible." This requirement does not settle it, and so states no admissibility
  rule for that pair. Owner: `quire-specification`.

## Dependencies

- [FR-006](./FR-006-produce-canonical-protocol-results.md) is the parent this
  requirement is part of.
- [FR-015](./FR-015-digest-domain-selection.md) supplies the digest domain
  selection.
- [FR-011](./FR-011-consumer-identity-discipline.md) supplies the identity
  discipline.
- `ix://agent-ix/quire-specification/FR-061` states the contract version, kind,
  identity, digest and recomputation rules.
