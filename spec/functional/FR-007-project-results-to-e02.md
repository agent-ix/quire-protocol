---
id: FR-007
title: "Project canonical results to E02"
type: FR
relationships:
  - target: ix://agent-ix/quire-protocol/ADR-006
    type: references
  - target: ix://agent-ix/quire-protocol/US-002
    type: implements
  - target: ix://agent-ix/quire-protocol/FR-006
    type: depends_on
  - target: ix://agent-ix/quire-protocol/FR-034
    type: contains
  - target: ix://agent-ix/quire-protocol/FR-035
    type: contains
  - target: ix://agent-ix/quire-specification/FR-062
    type: depends_on
  - target: ix://agent-ix/quire-specification/FR-201
    type: references
  - target: ix://agent-ix/quire-specification/FR-240
    type: references
  - target: ix://agent-ix/quire-specification/FR-241
    type: references
  - target: ix://agent-ix/quire-specification/FR-242
    type: references
  - target: ix://agent-ix/quire-specification/FR-244
    type: references
  - target: ix://agent-ix/quire-specification/FR-245
    type: references
  - target: ix://agent-ix/quire-specification/FR-246
    type: references
  - target: ix://agent-ix/quire-specification/FR-248
    type: references
  - target: ix://agent-ix/quire-specification/FR-249
    type: references
  - target: ix://agent-ix/quire-specification/FR-280
    type: references
  - target: ix://agent-ix/quire-specification/FR-282
    type: references
  - target: ix://agent-ix/quire-specification/FR-283
    type: references
  - target: ix://agent-ix/quire-verification/FR-007
    type: depends_on
---
# FR-007: Project canonical results to E02

## Description

When a canonical protocol result and exact portable evidence context are submitted, the adapter SHALL emit one source-bound `quire-verification/e02-draft-2` `TechniqueResult` through the atomic requirements FR-034 and FR-035 that together discharge source requirement FR-062.
When either atomic requirement FR-034 or FR-035 is unsatisfied, the adapter SHALL emit one stable typed refusal and no output result.

## Inputs

One strict-reader-accepted canonical result, target obligation, plan/task/run,
profile, binding, dependency closure, execution scope, adapter identity/version,
and exact E02 contract revision.

## Outputs

One derived E02 `TechniqueResult`, or one refusal from the FR-062 closed refusal set.

## Behavior

This requirement is the aggregate. It establishes no obligation of its own
beyond composition: each obligation it previously stated in prose is now stated,
once and independently verifiably, by exactly one of its parts.

| Part | Obligation it owns |
| --- | --- |
| [FR-034](./FR-034-derive-e02-projection-identity.md) | The derived projection identity and its preimage, the single unchanged canonical-result reference, provenance of producer and adapter, target-binding equality with the selected context, and the semantic `bindingRefs` order. |
| [FR-035](./FR-035-map-source-states-to-e02.md) | The FR-062 source-to-E02 mapping, simultaneous finding preservation, adequacy-domain separation, the canonical retention boundary, oracle dependence preservation, and the closed refusal set. |

The two parts are separate requirements because their failures are separate
failures: a projection may carry a correct derived identity and still lose a
finding, and it may map every source row correctly and still bind the wrong
target context. Neither part's success substitutes for the other's.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-007-AC-1 | Every FR-062 source-state row maps to the exact E02 state and retains all simultaneous independent findings, which is exactly [FR-035](./FR-035-map-source-states-to-e02.md)-AC-1 and -AC-2. | Test (TC-007) |
| FR-007-AC-2 | The derived identity changes for each source, target-binding, contract, or adapter mutation and remains distinct from the canonical result identity, which is exactly [FR-034](./FR-034-derive-e02-projection-identity.md)-AC-1, with its preimage fixed by -AC-3 and its binding equality by -AC-4 and -AC-5. | Test (TC-007) |
| FR-007-AC-3 | E02 adequacy derives only from observation adequacy; protocol adequacy and sufficiency cannot cross into it, which is exactly [FR-035](./FR-035-map-source-states-to-e02.md)-AC-3, with the wider retention boundary held by -AC-4. | Test (TC-007) |
| FR-007-AC-4 | Invalid source, unknown versions/features, identity mismatch, unrepresented source state, missing context, or finding loss returns the stable FR-062 refusal and emits no result, which is exactly [FR-035](./FR-035-map-source-states-to-e02.md)-AC-6 and -AC-7 together with [FR-034](./FR-034-derive-e02-projection-identity.md)-AC-7. | Test (TC-007) |
| FR-007-AC-5 | `nativeArtifacts` retains exactly one unchanged canonical-result URI, digest, and media type, and provenance names both producer and adapter versions, which is exactly [FR-034](./FR-034-derive-e02-projection-identity.md)-AC-2. | Test (TC-007) |
| FR-007-AC-6 | Every `satisfied` or `violated` projection preserves the source decision-support identities and dependence relation and invents no independent oracle relation, which is exactly [FR-035](./FR-035-map-source-states-to-e02.md)-AC-5. | Test (TC-007) |

## Adopted vocabulary

This requirement had no adoption pass before this revision; FR-010's adoption
status table recorded it as not started. The nouns it uses are typed upstream
and are adopted by reference below. Each part carries the slice of this table it
uses; the table is retained here as the shared mapping the decomposition was
derived from, not as a second definition.

| Noun | Upstream object | Identity kind |
| --- | --- | --- |
| Canonical protocol result (source) | `FR-240` | exact opaque source identity consumed after strict validation |
| Derived portable projection result | `FR-249` | derived, keyed (canonical result identity and digest, target obligation and execution-binding identities, target contract version, adapter identity and version) |
| Projection refusal code | `FR-283` | value |
| Dependence relation | `FR-282` | value |
| Execution disposition | `FR-241` | value |
| Truth | `FR-242` | value |
| Activation | `FR-244` | value |
| Observation adequacy | `FR-245` | value |
| Protocol adequacy | `FR-246` | value |
| Assumptions | `FR-280` | value |
| Decision support | `FR-248` | value |
| Digest domain label | `FR-201` | value |

`FR-249` is the derived result this requirement emits, and its key is exactly
the derivation tuple [FR-034](./FR-034-derive-e02-projection-identity.md)
adopts: an identity distinct from the canonical protocol-result identity and
stable for that tuple alone. The tuple is adopted, not restated.

The E02 target vocabularies belong to `quire-verification` and reach this
repository only through FR-062's mapping table. No E02 member is spelled here.

The projection identity and the source-result digest both carry the
`quire.verification.jcs` member that
[FR-015](./FR-015-digest-domain-selection.md) selects from `FR-201`; the two
preimages differ and the domain does not. This repository mints no digest domain
label.

### Accepted rulings and remaining external gate

Quire Specification FS02 resolved the earlier source-domain gaps. FR-283 assigns
a recomputed source-result digest mismatch to `invalid-source-result`; FR-284
closes progress/closure; FR-285 closes participation; and FR-249 declares the
mapping total for selected source and target revisions. FR-034 and FR-035 adopt
those rulings exactly.

- **Canonical source-result identity kind.** `FR-240` declares no identity kind
  for the canonical source result and records the gap as open. The projection
  consumes that identity as an exact opaque value and declares no kind for it.
  Recorded by
  [FR-034](./FR-034-derive-e02-projection-identity.md). Owner:
  `quire-specification`.
- **qverify acceptance.** Issue #32 accepted the strict
  `quire-verification/e02-draft-2` schema/reader contract at `d2fc0d8`; issue
  #44 published dependency-compatible consumer revision
  `293e342c6f5e2c69738956ff5165f75761b0364a`, which Protocol pins exactly with
  its accepted Engineering Assurance edge.

## Dependencies

- [FR-034](./FR-034-derive-e02-projection-identity.md) and
  [FR-035](./FR-035-map-source-states-to-e02.md) are this requirement's parts.
- [FR-006](./FR-006-produce-canonical-protocol-results.md).
- [FR-015](./FR-015-digest-domain-selection.md) supplies the digest domain
  selection.
- `ix://agent-ix/quire-specification/FR-062`.
- `ix://agent-ix/quire-verification/FR-007` and `quire-verification/e02-draft-2`.
