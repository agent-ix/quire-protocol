---
id: FR-031
title: "Bind canonical result identity"
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
# FR-031: Bind canonical result identity

## Description

When a canonical protocol result is emitted, the result producer SHALL bind the contract version, kind and result identity that `FR-240-canonical-protocol-result` states.

## Inputs

One constructed result fact set, the selected
digest domain member, and the exact source, profile, model, protocol, request,
binding, input, backend and producer identities plus the exact deterministic
seed, toolchain identity and corpus identity carried as reproduction provenance.

## Outputs

One canonical result bearing its contract version, kind and result identity, or
one typed strict-reader refusal.

## Behavior

The result producer SHALL emit `contractVersion` and `kind` exactly as
`FR-240-canonical-protocol-result` records them, and a result identity, without
restating those values as a local vocabulary.

The digest domain is the `quire.verification.jcs` member that
[FR-015](./FR-015-digest-domain-selection.md) selects from
`FR-201-canonical-identity-domain`'s published vocabulary. This requirement
selects no member of its own and spells none of its own. RFC 8785
canonicalization is confined to this result-identity preimage. Offered artifact
bytes are read and compared under their selected input contract; this requirement
does not attach a content digest to their ArtifactRef.

Every provenance identity the result carries — source, profile, model, protocol,
request, binding, input, backend and producer — is recomputed or strictly
matched against its selected input. None is inferred from display name, source
path, span, message text, locale or rendering, and none is substituted from
another identity of the same shape.

The same provenance record carries the complete `NFR-001` reproduction tuple:
the caller-selected deterministic seed and the exact toolchain and corpus
identity bytes. The canonical JSON spells the seed as its unsigned decimal
string and each arbitrary byte identity as lowercase hexadecimal, so neither a
JSON number conversion nor a lossy text decode can alter the run identity.
Display text is excluded. An omitted, empty, over-limit or malformed member is
refused rather than inferred.

Where the string-only canonical result must carry an opaque runtime identity,
the Protocol-owned `quire.protocol.runtime-reference/1` encoding is a fixed
`quire-runtime-v1:` prefix followed by lowercase hexadecimal of the RFC 8785
canonical object containing the complete authority key, producer-local kind
bytes and identity bytes. The decoder verifies the prefix, even-length lowercase
hexadecimal, canonical object bytes and byte-exact round trip. This encoding is
only a reversible carrier for a Protocol result field; it neither changes the
producer identity nor becomes authority at an upstream boundary.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-031-AC-1 | Every emitted result carries the `FR-240` contract version and kind plus a result identity, and a result with a stale identity is refused by the strict reader. | Test (TC-006) |
| FR-031-AC-4 | The protocol-result identity carries the `quire.verification.jcs` label exactly as `FR-201` publishes it, and no producer-domain value can substitute for it. | Test (TC-006) |
| FR-031-AC-5 | Every source, profile, model, protocol, request, binding, input, backend and producer identity plus the deterministic seed and exact toolchain and corpus identity bytes on the result is recomputed or strictly matched; changing any one changes the result identity or refuses, and none is inferred from display text or substituted from another identity. | Test (TC-006, TC-008) |
| FR-031-AC-6 | RFC 8785 canonicalization is applied to the protocol-result identity preimage only. | Test (TC-006) |
| FR-031-AC-7 | Every opaque runtime identity carried in a string-only result field round-trips byte-exactly through `quire.protocol.runtime-reference/1`; wrong prefix, uppercase/odd/non-hex content, non-canonical object bytes, missing authority/kind/identity bytes or a changed decoded byte refuses. | Test (TC-006) |

## Adopted vocabulary

| Noun | Upstream object | Identity kind |
| --- | --- | --- |
| Canonical protocol result | `FR-240-canonical-protocol-result` | not declared upstream; see the gap below |
| Digest domain label | `FR-201-canonical-identity-domain` | value |

The member selection is local and is
[FR-015](./FR-015-digest-domain-selection.md)'s; the vocabulary selected from is
`FR-201`'s. `FR-201` states of itself: "This artifact *is* the vocabulary of
digest domains." Nothing here extends it.

### Accepted owner ruling

Quire Specification #29 / PR #38 defines the FR-240 result identity as derived
from the complete canonical fact set and exact producer/profile/request tuple.
Same identity with different canonical bytes is refused. Protocol consumes that
accepted rule and owns only the reversible runtime-reference carrier used by its
own string-only result contract.

## Dependencies

- [FR-006](./FR-006-produce-canonical-protocol-results.md) is the parent this
  requirement is part of.
- [FR-015](./FR-015-digest-domain-selection.md) supplies the digest domain
  selection.
- [FR-011](./FR-011-consumer-identity-discipline.md) supplies the identity
  discipline.
- `ix://agent-ix/quire-specification/FR-061` states the contract version, kind
  and identity rules.
