---
id: FR-036
title: "Carry the complete typed refusal envelope"
type: FR
relationships:
  - target: ix://agent-ix/quire-protocol/FR-008
    type: part_of
  - target: ix://agent-ix/quire-protocol/FR-014
    type: depends_on
  - target: ix://agent-ix/quire-protocol/FR-009
    type: references
  - target: ix://agent-ix/quire-specification/FR-047
    type: depends_on
  - target: ix://agent-ix/quire-specification/FR-270
    type: references
  - target: ix://agent-ix/quire-specification/FR-271
    type: references
  - target: ix://agent-ix/quire-specification/FR-272
    type: references
  - target: ix://agent-ix/quire-specification/FR-273
    type: references
  - target: ix://agent-ix/quire-specification/FR-274
    type: references
---
# FR-036: Carry the complete typed refusal envelope

## Description

When a public operation reaches a typed refusal, the library SHALL return exactly one typed refusal envelope carrying every field declared in Outputs below.
The library SHALL record on that envelope the refusal axis under which the refusal was raised.
The library SHALL report an envelope carrying no refusal axis as an unassigned axis rather than folding it into a catch-all identity.
The library SHALL treat an absent declared envelope field as a failure of this requirement rather than defaulting it.

## Inputs

One resolution, admission, representation, compensation, conformance, claim,
result-reader, projection or evidence-handoff refusal produced under an exact selected local contract, or one
accepted typed upstream failure response with its declared source contract and
its diagnostic-catalog selection.

## Outputs

One `ProtocolRefusal` carrying exactly these fields:

| Field | Owner | Note |
| --- | --- | --- |
| Domain | This requirement's parent [FR-008](./FR-008-return-typed-refusals.md) | One of `resolution`, `admission`, `representation`, `assessment`, `result`, `projection`, `evidence`. |
| Originating stage and every available prerequisite | Parent FR-008 | One exact local `ProtocolStage`; producer stages remain inside source causes. |
| Affected identity | The failing operation's own subject | Retained, never reconstructed. |
| Location, available or explicitly unavailable | Failing operation or selected source contract | Absence is a value, not a gap to fill. |
| Ordered typed causes | Parent FR-008 or selected source contract | Each cause is a local closed cause, retained source cause, or unsupported-source-contract; order and multiplicity are FR-039's obligation. |
| Resource usage | [NFR-002](../non-functional/NFR-002-enforce-resource-bounds.md) | Exact Protocol usage plus separate owner-versioned external usage. |
| Partial-output state | This requirement's parent FR-008 | Explicit on every envelope. |
| Refusal axis | [FR-014](./FR-014-typed-outcome-identity.md) | The identity component that makes injectivity observable. |

## Behavior

The library SHALL populate every field above on every public refusal path, or fail this requirement.
The library SHALL record the refusal axis as the `FR-014` identity component it is, so that the pairwise injectivity check of FR-014-AC-1 is evaluable from the envelope alone.
The library SHALL NOT substitute a two-valued admits/refuses disposition for the recorded axis, because such a disposition is satisfied by one shared error code across every negative row.
The library SHALL refuse an envelope whose domain is absent or is a sixth value outside the five closed members.

Field *content* rules live in the sibling requirements: text-independence in
[FR-037](./FR-037-derive-no-refusal-fact-from-text.md), unknown-contract
retention in [FR-038](./FR-038-retain-unknown-source-contracts.md), cause order
and multiplicity in [FR-039](./FR-039-preserve-typed-causes-distinctly.md), and
suppression of a co-emitted success value in
[FR-040](./FR-040-emit-no-success-beside-fatal-refusal.md). This requirement owns
presence and cardinality only.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-036-AC-1 | Every public failure path returns exactly one envelope carrying the typed domain, actual local originating stage and every available prerequisite, affected identity, an available or explicitly unavailable location, ordered typed causes, resource usage, and partial-output state; a retained source cause additionally carries its exact source contract and stable code; an absent required field fails the path rather than being defaulted. | Test (TC-011) |
| FR-036-AC-2 | Every returned envelope records the refusal axis under which it was raised, and an envelope recording no axis is reported as an unassigned axis and satisfies no negative scenario row. | Test (TC-011) |
| FR-036-AC-3 | The recorded axis and the retained interior are together sufficient to evaluate FR-014-AC-1's pairwise comparison of distinct axes without reading any display text, and a corpus whose refusing rows record only a two-valued admits/refuses disposition fails this criterion. | Test (TC-011) |
| FR-036-AC-4 | An envelope whose domain is absent, outside the seven declared values, or inconsistent with its closed stage-to-domain mapping is refused. | Test (TC-011) |
| FR-036-AC-5 | The partial-output state is explicit on every envelope and is inferred from neither an empty output nor the absence of a success value. | Test (TC-011) |
| FR-036-AC-6 | Every local cause belongs to `quire.protocol/refusal/1`; every retained source cause names the exact FR-009-resolved source contract, and an unknown contract is retained as `unsupported-source-contract` rather than interpreted. | Test (TC-011) |

## Adopted vocabulary

The envelope, local stage, local cause vocabulary and unsupported disposition
are this repository's and FR-008 owns them. A retained source cause adopts the
exact selected source contract rather than a repository-wide inferred catalog.
FR-270 through FR-274 remain informative until an adapter explicitly selects an
accepted catalog. The refusal identity this requirement makes observable
is [FR-014](./FR-014-typed-outcome-identity.md)'s: identity kind *value*,
authority *consumer*, key (`integration`, `refusal_axis`, `outcome_kind`).

FR-014 records why that key cannot itself be the injectivity oracle:
`refusal_axis` is a key component, so distinctness by key is true by
construction. This requirement therefore obliges the envelope to *carry* the axis
and the producer-supplied interior, so that FR-014-AC-1 can compare interiors
across a pair of distinct axes and fail a shared one. A criterion that cannot
fail is not asserted here.

Producer stages and loci, when supplied, are opaque fields inside the retained
source cause. They never replace the required local originating stage. Missing
source context is represented as unavailable, not invented from display data.

## Dependencies

- [FR-008](./FR-008-return-typed-refusals.md) owns the envelope, its seven-member
  domain and its partial-output state; this requirement is `part_of` it.
- [FR-014](./FR-014-typed-outcome-identity.md) owns the refusal identity, its key
  and its injectivity obligation.
- [FR-009](./FR-009-lock-external-producer-revisions.md) owns the version-lock
  entry that gates the catalog selection, and reports a scenario requiring an
  `unresolved` entry as unexecutable.
- `ix://agent-ix/quire-specification/FR-047` and FR-270 through FR-274 inform
  source adapters but do not block local refusal execution.

## Status

Draft. Existing public paths only partially use the common envelope; Task-006
completes this requirement and TC-011.
