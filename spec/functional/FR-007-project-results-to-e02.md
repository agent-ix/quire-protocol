---
id: FR-007
title: "Project canonical results to E02"
type: FR
relationships:
  - target: ix://agent-ix/quire-protocol/US-002
    type: implements
  - target: ix://agent-ix/quire-protocol/FR-006
    type: depends_on
  - target: ix://agent-ix/quire-specification/FR-062
    type: depends_on
  - target: ix://agent-ix/quire-verification/FR-007
    type: depends_on
---
# FR-007: Project canonical results to E02

## Description

When a canonical protocol result and exact portable evidence context are
submitted, the adapter SHALL emit one source-bound
`quire-verification/e02-draft-2` `TechniqueResult` according to the total FR-062
mapping or one stable typed refusal with no output result.

## Inputs

One strict-reader-accepted canonical result, target obligation, plan/task/run,
profile, binding, dependency closure, execution scope, adapter identity/version,
and exact E02 contract revision.

## Outputs

One derived E02 `TechniqueResult`, or one refusal from the FR-062 closed refusal set.

## Behavior

The adapter SHALL preserve every mapped execution, logical, activation,
participation, assumption, observation-adequacy, oracle-dependence, and provenance
finding. It SHALL retain exactly one canonical-result artifact reference, compute
a distinct RFC 8785 projection identity over the complete FR-062 preimage, and
keep protocol adequacy, closure, settlement, claim, and completeness in the
canonical source. It SHALL NOT infer evidence sufficiency or oracle independence.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-007-AC-1 | Every FR-062 source-state row maps to the exact E02 state and retains all simultaneous independent findings. | Test (TC-007) |
| FR-007-AC-2 | The derived identity changes for each source, target-binding, contract, or adapter mutation and remains distinct from the canonical result identity. | Test (TC-007) |
| FR-007-AC-3 | E02 adequacy derives only from observation adequacy; protocol adequacy and sufficiency cannot cross into it. | Test (TC-007) |
| FR-007-AC-4 | Invalid source, unknown versions/features, identity mismatch, unrepresented source state, missing context, or finding loss returns the stable FR-062 refusal and emits no result. | Test (TC-007) |
| FR-007-AC-5 | `nativeArtifacts` retains exactly one unchanged canonical-result URI, digest, and media type, and provenance names both producer and adapter versions. | Test (TC-007) |

## Dependencies

- [FR-006](./FR-006-produce-canonical-protocol-results.md).
- `ix://agent-ix/quire-specification/FR-062`.
- `ix://agent-ix/quire-verification/FR-007` and `quire-verification/e02-draft-2`.

