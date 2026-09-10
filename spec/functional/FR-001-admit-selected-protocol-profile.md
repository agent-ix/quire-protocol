---
id: FR-001
title: "Admit one selected protocol profile"
type: FR
relationships:
  - target: ix://agent-ix/quire-protocol/US-001
    type: implements
  - target: ix://agent-ix/quire-specification/FR-050
    type: depends_on
---
# FR-001: Admit one selected protocol profile

## Description

When a protocol artifact is submitted, the library SHALL admit it only when its
artifact-contract version, contract version, source revision, selected rule
digests, dependency closure, profile identity, and required feature set exactly
match one promoted immutable native Quire protocol candidate and its accepted
compiled-artifact interface.

## Inputs

- Protocol artifact bytes plus the exact accepted compiled-artifact contract and
  declared contract versions.
- Exact candidate, source revision, selected-rule, dependency, and profile identities.
- Required feature set and strict-reader limits.

## Outputs

- One strictly admitted protocol artifact, or one typed refusal with the affected identity.

## Behavior

The library SHALL recompute every content digest before reading dependent fields.
It SHALL reject unknown artifact encodings or contracts and unknown, draft-only,
superseded, substituted, internally inconsistent, or feature-incomplete
candidates. It SHALL retain the accepted candidate, artifact-contract, producer,
and dependency identities without parsing native source or copying upstream
formal semantics into a second locally authored profile.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-001-AC-1 | Exact bytes under one accepted compiled-artifact contract, promoted candidate, producer revision, and complete dependency closure are admitted with every identity unchanged. | Test (TC-001) |
| FR-001-AC-2 | Mutating each artifact-contract, producer, candidate, version, revision, digest, dependency, profile, or required feature independently returns a typed refusal before semantic evaluation. | Test (TC-001) |
| FR-001-AC-3 | A draft, superseded, unknown, or internally resealed candidate cannot be selected through display-name equality or a locally authored substitute. | Test (TC-001) |

## Dependencies

- `ix://agent-ix/quire-specification/FR-050` through `FR-062` define the source semantics.
- The composed baseline must publish a new immutable candidate after integrated FR-061/FR-062 changes.
- The A-owned native compiler must publish an accepted versioned compiled-protocol
  artifact contract; prose profile definitions and native source are not that interface.
