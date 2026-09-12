---
id: FR-034
title: "Derive the E02 projection identity"
type: FR
relationships:
  - target: ix://agent-ix/quire-protocol/FR-007
    type: part_of
  - target: ix://agent-ix/quire-protocol/FR-015
    type: depends_on
  - target: ix://agent-ix/quire-protocol/FR-011
    type: depends_on
  - target: ix://agent-ix/quire-specification/FR-062
    type: depends_on
  - target: ix://agent-ix/quire-specification/FR-249
    type: references
  - target: ix://agent-ix/quire-specification/FR-240
    type: references
  - target: ix://agent-ix/quire-specification/FR-201
    type: references
---
# FR-034: Derive the E02 projection identity

## Description

When a canonical protocol result is projected, the adapter SHALL derive the `TechniqueResult.resultId` from the exact tuple `FR-249-derived-projection-result` states.
When a projection is emitted, the adapter SHALL retain exactly one unchanged canonical-result entry in `nativeArtifacts` with its source URI, recomputed digest and the media type FR-062 states.
When a projection is emitted, the adapter SHALL set every E02 target-binding field equal to its selected context.

## Inputs

One strict-reader-accepted canonical result with its identity and recomputed
digest, the target obligation, plan, task, run, profile, binding, dependency
closure and execution scope, the adapter identity and version, and the exact
E02 contract revision.

## Outputs

One derived `TechniqueResult.resultId` with its retained source reference and
provenance, or one refusal carrying a code from
`FR-283-projection-refusal-code` and no `TechniqueResult`.

## Behavior

The derived identity is `sha256-jcs:<lowercase-hex>` over the RFC 8785 canonical
projection-identity preimage `FR-062` states, whose first member is
`identityVersion` equal to
`quire.protocol.e02-projection-identity/v1-draft.1`. It is distinct from the
canonical protocol-result identity, and changing any member of the derivation
tuple changes it.

The adapter validates the canonical protocol result and recomputes its
`sha256-jcs` digest before mapping it. `nativeArtifacts` carries exactly one
canonical-result entry with the source URI, the recomputed digest and the media
type `application/vnd.agent-ix.quire-protocol-result+json;version=1-draft.1`.
The canonical-result bytes are carried unchanged; the adapter neither
recanonicalises nor re-labels them.

`provenance` identifies the protocol producer plus this adapter and their
versions. The E02 property, subject, profile, binding, dependency-closure,
execution-scope, plan, task and run fields each equal their selected context,
and FR-062 states that "the adapter SHALL NOT infer or substitute one from
display text." `bindingRefs` "uses the selected E02 semantic order and SHALL NOT
be resorted by presentation text" (FR-062); that order is semantic, not a key
order, and this requirement declares no order over derived identities.

Both digests this requirement compares carry the `quire.verification.jcs` member
that [FR-015](./FR-015-digest-domain-selection.md) selects from `FR-201`. The two
preimages differ and the domain does not: `FR-201` records under "Preimage
selection is per-use, not per-domain" that FR-062's derived result identity "is
taken over a distinct projection-identity preimage under
`quire.protocol.e02-projection-identity/v1-draft.1`". No projection-specific
digest domain label is minted here.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-034-AC-1 | The derived identity changes for each source, target-binding, contract, or adapter mutation and remains distinct from the canonical result identity. | Test (TC-007) |
| FR-034-AC-2 | `nativeArtifacts` retains exactly one unchanged canonical-result URI, digest, and media type, and provenance names both producer and adapter versions. | Test (TC-007) |
| FR-034-AC-3 | The derived identity is `sha256-jcs:` followed by lowercase hexadecimal over the RFC 8785 preimage whose `identityVersion` is `quire.protocol.e02-projection-identity/v1-draft.1`. | Test (TC-007) |
| FR-034-AC-4 | Every E02 property, subject, profile, binding, dependency-closure, execution-scope, plan, task and run field equals its selected context and is never inferred or substituted from display text. | Test (TC-007) |
| FR-034-AC-5 | `bindingRefs` retains the selected E02 semantic order and is not resorted by presentation text. | Test (TC-007) |
| FR-034-AC-6 | Both the source-result digest and the projection identity are compared under the `quire.verification.jcs` label exactly as `FR-201` publishes it, and no projection-specific domain label originates in this repository. | Inspection |
| FR-034-AC-7 | The adapter recomputes the source `sha256-jcs` digest before mapping, and a mismatch emits no `TechniqueResult`. | Test (TC-007) |

## Adopted vocabulary

| Noun | Upstream object | Identity kind |
| --- | --- | --- |
| Derived portable projection result | `FR-249-derived-projection-result` | derived, keyed (canonical result identity and digest, target obligation and execution-binding identities, target contract version, adapter identity and version) |
| Canonical protocol result (source) | `FR-240-canonical-protocol-result` | not declared upstream |
| Digest domain label | `FR-201-canonical-identity-domain` | value |

`FR-249` states the derivation directly: the identity is "distinct from the
canonical protocol-result identity and stable for the exact tuple of canonical
result identity and digest, target obligation and execution-binding identities,
target contract version, and adapter identity and version". That tuple is
adopted as the key, not restated as a local one.

### Where the upstream vocabulary does not yet reach

- **Which refusal code a recomputed-digest mismatch receives is unresolved
  upstream, and this requirement does not pick a side.** AC-7 requires the
  mismatch to emit no `TechniqueResult`; the code it carries is selected by
  [FR-035](./FR-035-map-source-states-to-e02.md), which records the two standing
  upstream assignments verbatim and leaves them unresolved. Owner:
  `quire-specification`.
- **FR-015's selection table does not yet list the projection preimage.**
  [FR-015](./FR-015-digest-domain-selection.md) records
  `quire.verification.jcs` as used for "This repository's protocol-result digest,
  over RFC 8785 canonical UTF-8 bytes", which is the enclosing-reference
  preimage, not the projection-identity preimage. `FR-201` establishes that the
  domain is shared and the preimage is per-use, so no second domain member is
  needed; extending FR-015's table to name the second preimage is local work
  this requirement does not perform, and it must not be done by minting a label.
  Owner: [FR-015](./FR-015-digest-domain-selection.md).
- **Result-identity derivation kind at the source.** `FR-240` records that "no
  requirement states whether the identity is authored by the producer or derived
  from named components." The derived projection identity is fully specified; the
  source identity it consumes is not. This requirement consumes the source
  identity as an exact opaque value and declares no kind for it. Owner:
  `quire-specification`.

## Dependencies

- [FR-007](./FR-007-project-results-to-e02.md) is the parent this requirement is
  part of.
- [FR-035](./FR-035-map-source-states-to-e02.md) carries the mapping and the
  refusal set this identity is emitted with.
- [FR-015](./FR-015-digest-domain-selection.md) supplies the digest domain
  selection.
- [FR-011](./FR-011-consumer-identity-discipline.md) supplies the identity
  discipline.
- `ix://agent-ix/quire-specification/FR-062` states the preimage, the derivation
  tuple, the native-artifact entry and the no-display-text rule.
