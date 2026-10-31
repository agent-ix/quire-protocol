---
id: FR-034
title: "Derive the E02 projection identity"
type: FR
relationships:
  - target: ix://agent-ix/quire-protocol/FR-007
    type: part_of
  - target: ix://agent-ix/quire-protocol/FR-011
    type: depends_on
  - target: ix://agent-ix/quire-specification/FR-062
    type: depends_on
  - target: ix://agent-ix/quire-specification/FR-249
    type: references
  - target: ix://agent-ix/quire-specification/FR-240
    type: references
---
# FR-034: Bind the E02 projection identity

## Description

When a canonical protocol result is projected, the adapter SHALL emit the `TechniqueResult.resultId` required by QSpec FR-062 and FR-249.
When a projection is emitted, the adapter SHALL retain exactly one unchanged canonical-result entry in `nativeArtifacts` with its source URI and the media type FR-062 states.
When a projection is emitted, the adapter SHALL set every E02 target-binding field equal to its selected context.

## Inputs

One strict-reader-accepted canonical result with its identity, the target
obligation, plan, task, run, profile, binding, dependency
closure and execution scope, the adapter identity and version, and the exact
E02 contract revision.

## Outputs

One derived `TechniqueResult.resultId` with its retained source reference and
provenance, or one refusal carrying a code from
`FR-283-projection-refusal-code` and no `TechniqueResult`.

## Behavior

The adapter carries the `resultId` structure and source `resultKey` specified
by QSpec FR-062 and FR-249. It performs exact-structure comparisons and derives
no hash identity. The target obligation and execution context remain separately
checked against their selected values.

The adapter validates the canonical protocol result before mapping it.
`nativeArtifacts` carries exactly one canonical-result entry with the source
URI and media type
`application/vnd.agent-ix.quire-protocol-result+json;version=1-draft.1`. The
canonical-result bytes are carried unchanged; the adapter neither
recanonicalises nor re-labels them.

When the strict source reader refuses the supplied bytes, the
`invalid-source-result` refusal identifies the caller-selected source URI by
its UTF-8 bytes. The projection emits no `TechniqueResult`.

`provenance` identifies the protocol producer plus this adapter and their
versions. The E02 property, subject, profile, binding, dependency-closure,
execution-scope, plan, task and run fields each equal their selected context,
and FR-062 states that "the adapter SHALL NOT infer or substitute one from
display text." `bindingRefs` "uses the selected E02 semantic order and SHALL NOT
be resorted by presentation text" (FR-062); that order is semantic, not a key
order, and this requirement declares no order over derived identities.

The source result identity is the FR-286 structural `resultKey`; the projection
identity is the FR-062 object. The adapter compares these structures exactly
and does not compute a digest for either one.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-034-AC-1 | `TechniqueResult.resultId` has the exact FR-062/FR-249 structure and changes only when its source `resultKey` or adapter identity/version changes. | Test (TC-007) |
| FR-034-AC-2 | `nativeArtifacts` retains exactly one unchanged canonical-result URI and media type, and provenance names both producer and adapter versions. | Test (TC-007) |
| FR-034-AC-3 | The adapter compares `resultId` and its source key by exact structure and derives no identity hash. | Test (TC-007) |
| FR-034-AC-4 | Every E02 property, subject, profile, binding, dependency-closure, execution-scope, plan, task and run field equals its selected context and is never inferred or substituted from display text. | Test (TC-007) |
| FR-034-AC-5 | `bindingRefs` retains the selected E02 semantic order and is not resorted by presentation text. | Test (TC-007) |
| FR-034-AC-6 | The source result key is the structural FR-286 key, and neither it nor the FR-062 projection identity is replaced by a digest. | Test (TC-007) |
| FR-034-AC-7 | A strict source-reader refusal emits `invalid-source-result` with the exact UTF-8 bytes of the selected source URI and no `TechniqueResult`. | Test (TC-007) |

## Adopted vocabulary

| Noun | Upstream object | Identity kind |
| --- | --- | --- |
| Derived portable projection result | `FR-249-derived-projection-result` | `resultId` per FR-062 |
| Canonical protocol result (source) | `FR-240-canonical-protocol-result` | structural `resultKey` per FR-286 |

The identity structure is owned by QSpec FR-062 and FR-249 and is consumed by
reference rather than restated here.

### Resolved and opaque source details

Accepted Quire Specification FS02 fixes a source-reader refusal to
`invalid-source-result`; `identity-mismatch` applies after source validation to
a target-context mismatch.

`FR-240` states that "The prose does not state whether that identity is
  authored by the producer or derived from the result's own components, so
  this artifact declares neither." The derived projection identity is fully specified; the
  source identity it consumes is not. This requirement consumes the source
  identity as an exact opaque value and declares no kind for it. Owner:
  `quire-specification`.

## Dependencies

- [FR-007](./FR-007-project-results-to-e02.md) is the parent this requirement is
  part of.
- [FR-035](./FR-035-map-source-states-to-e02.md) carries the mapping and the
  refusal set this identity is emitted with.
- [FR-011](./FR-011-consumer-identity-discipline.md) supplies the identity
  discipline.
- `ix://agent-ix/quire-specification/FR-062` states the result identity,
  native-artifact entry and no-display-text rule.
