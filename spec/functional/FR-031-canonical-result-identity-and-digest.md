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

When a canonical protocol result is emitted, the result producer SHALL bind the contract version, kind and structural result key that `FR-240-canonical-protocol-result` and QSpec FR-286 state.

## Inputs

One constructed result fact set and the exact source, profile, model, protocol, request,
binding, input, backend and producer identities plus the exact deterministic
seed, toolchain identity and corpus identity carried as reproduction provenance.

## Outputs

One canonical result bearing its contract version, kind and structural `resultKey`, or
one typed strict-reader refusal.

## Behavior

The result producer SHALL emit `contractVersion` and `kind` exactly as
`FR-240-canonical-protocol-result` records them, and the supplied `resultKey`, without
restating those values as a local vocabulary.

The `resultKey` is the structural `{producer, request, ordinal}` key defined by
QSpec FR-286. The producer supplies the ordinal for this result in its own
request-scoped result stream. The reader compares the complete key exactly;
this requirement derives no result identity and computes no result digest.
Canonical JSON encoding of the emitted result remains governed by FR-240.
Offered artifact bytes are read under their selected input contract; this
requirement does not attach a content digest to their ArtifactRef.

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
| FR-031-AC-1 | Every emitted result carries the `FR-240` contract version and kind plus its FR-286 structural `resultKey`; a missing or incomplete key is refused by the strict reader. | Test (TC-006) |
| FR-031-AC-4 | The emitted `resultKey` is the complete FR-286 structure; the reader compares it exactly and does not derive a digest from it or from result bytes. | Test (TC-006) |
| FR-031-AC-5 | Every source, profile, model, protocol, request, binding, input, backend and producer identity plus the deterministic seed and exact toolchain and corpus identity bytes on the result is recomputed or strictly matched; the `resultKey` changes only with its FR-286 key members, and no provenance value is inferred from display text or substituted from another identity. | Test (TC-006, TC-008) |
| FR-031-AC-6 | RFC 8785 canonicalization determines emitted result bytes only; it is not used to derive a result identity or digest. | Test (TC-006) |
| FR-031-AC-7 | Every opaque runtime identity carried in a string-only result field round-trips byte-exactly through `quire.protocol.runtime-reference/1`; wrong prefix, uppercase/odd/non-hex content, non-canonical object bytes, missing authority/kind/identity bytes or a changed decoded byte refuses. | Test (TC-006) |

## Adopted vocabulary

| Noun | Upstream object | Identity kind |
| --- | --- | --- |
| Canonical protocol result | `FR-240-canonical-protocol-result` | `resultKey` per QSpec FR-286 |

QSpec FR-286 defines the result key and its exact comparison. This requirement
does not restate that key's structure.

## Dependencies

- [FR-006](./FR-006-produce-canonical-protocol-results.md) is the parent this
  requirement is part of.
- [FR-011](./FR-011-consumer-identity-discipline.md) supplies the identity
  discipline.
- `ix://agent-ix/quire-specification/FR-061` states the contract version and kind.
- `ix://agent-ix/quire-specification/FR-286` states the structural result key.
