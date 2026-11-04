---
id: FR-001
title: "Admit one selected protocol profile"
type: FR
relationships:
  - target: ix://agent-ix/quire-protocol/ADR-002
    type: references
  - target: ix://agent-ix/quire-protocol/ADR-006
    type: references
  - target: ix://agent-ix/quire-protocol/ADR-007
    type: references
  - target: ix://agent-ix/quire-protocol/US-001
    type: implements
  - target: ix://agent-ix/quire-specification/FR-050
    type: depends_on
---
# FR-001: Admit one selected protocol profile

## Description

When a protocol artifact is submitted, the library SHALL admit it only when its
artifact-contract version, contract version, source revision, selected rule
identities/revisions, dependency closure, profile identity, and required feature set exactly
match one immutable protocol profile frozen by an accepted coherent native Quire
baseline and its accepted compiled-artifact interface.

## Inputs

- Protocol artifact bytes plus the exact accepted compiled-artifact contract and
  declared contract versions.
- Exact accepted source-baseline, source revision, selected-rule, dependency,
  and profile identities.
- The compiled-artifact contract's versioned numeric wire profile.
- Required feature set and strict-reader limits.

## Outputs

- One strictly admitted protocol artifact, or one typed refusal with the affected identity.

## Behavior

The library SHALL match each offered artifact identity against its independently
selected reference before reading dependent fields. Its reader SHALL NOT require
or synthesize an ArtifactRef content digest.
It SHALL reject unknown artifact encodings or contracts and unknown, draft-only,
unaccepted, superseded, substituted, internally inconsistent, or feature-incomplete
baselines. It SHALL retain the accepted baseline, artifact-contract, producer,
and dependency identities without parsing native source or copying upstream
formal semantics into a second locally authored profile.

For every native integer or exact rational semantic value, the accepted
compiled-artifact contract SHALL declare a typed encoding whose decode is
injective over the admitted source domain. A bare JSON number is admissible only
for an integer from `-9007199254740991` through `9007199254740991`; exact
integers outside that interval and every rational SHALL use a contract-defined
canonical non-number representation. The strict reader SHALL reject floats,
negative zero, a non-canonical signed decimal form, an out-of-range signed-64
integer, a zero or negative rational denominator, a non-reduced rational, or a
numeric representation whose value changes after RFC 8785 serialization and
decoding. It SHALL NOT reinterpret a producer-owned model/configuration digest
through this artifact's numeric or JCS domain.

Where the native compiler publishes more than one compiled-artifact contract
version, the library SHALL admit each published version only through that
version's own explicit entry point.
The library SHALL NOT infer a contract version from the offered bytes.
The library SHALL NOT upgrade, downgrade, repair, or substitute one version's
artifact for another's.
When bytes are offered at an entry point other than their own contract
version's, the library SHALL return a typed refusal.
When a later contract version is published, the library SHALL leave every
already-published version's admission behaviour unchanged.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-001-AC-1 | Exact bytes under one accepted compiled-artifact contract, accepted frozen baseline, producer revision, and complete dependency closure are admitted with every identity unchanged. | Test (TC-001) |
| FR-001-AC-2 | Mutating each artifact-contract, producer, accepted source-baseline, version, revision, dependency, profile, or required feature independently returns a typed refusal before semantic evaluation. | Test (TC-001) |
| FR-001-AC-3 | A pre-acceptance working draft, superseded or unknown baseline cannot be selected through display-name equality or a locally authored substitute. | Test (TC-001) |
| FR-001-AC-4 | Every declared numeric minimum and maximum, the signed-64 extrema in fields whose source domains admit them, and every normalized exact-rational boundary round-trip without loss under the selected numeric wire profile; an unsafe bare JSON number, non-canonical integer/rational form, field-domain or signed-64 overflow refuses before semantic evaluation. | Test (TC-001, TC-006) |
| FR-001-AC-5 | Each published compiled-artifact contract version is admitted only through its own explicit entry point: bytes of one version offered at another version's entry point return a typed refusal with no version detection, upgrade, downgrade, or repair, and the admission behaviour of an already-published version is unchanged by the addition of a later one. | Test (IT-001) |

## Checked-package successor

[FR-043](FR-043-admit-the-checked-package-handoff.md) states the admission contract for the checked-package handoff and replaces this requirement for the retired `quire.compiled-protocol/2` wire once the repoint merges. FR-001-AC-1 to AC-3 and AC-5 are restated as FR-043-AC-1, AC-3, AC-2 and AC-5, and the numeric decoding of AC-4 belongs to the Contract IR reader (FR-043-AC-6). Until the repoint merges, this requirement stays in force for the retired wire.

## Dependencies

- `ix://agent-ix/quire-specification/FR-050` through `FR-062` define the source semantics.
- The composed specification must accept one coherent baseline and freeze its exact
  profile, selected-rule, dependency, and source identities before this consumer runs.
  Ordinary prototype edits and Git history do not themselves mint selectable profiles.
- The A-owned native compiler must publish an accepted versioned compiled-protocol
  artifact contract; prose profile definitions and native source are not that interface.
