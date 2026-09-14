---
id: FR-038
title: "Retain unknown source contracts uninterpreted"
type: FR
relationships:
  - target: ix://agent-ix/quire-protocol/FR-008
    type: part_of
  - target: ix://agent-ix/quire-protocol/FR-014
    type: depends_on
  - target: ix://agent-ix/quire-protocol/FR-036
    type: depends_on
  - target: ix://agent-ix/quire-protocol/FR-009
    type: depends_on
  - target: ix://agent-ix/quire-protocol/FR-013
    type: depends_on
  - target: ix://agent-ix/quire-specification/FR-270
    type: references
  - target: ix://agent-ix/quire-specification/FR-271
    type: references
  - target: ix://agent-ix/quire-specification/FR-273
    type: references
---
# FR-038: Retain unknown source contracts uninterpreted

## Description

When a source refusal is read, the library SHALL validate its source contract
against the exact FR-009-resolved contract before interpreting its typed code.
When the offered source contract, revision or code is unknown, the library SHALL retain the refusal under the local `unsupported-source-contract` disposition with the original contract and value uninterpreted.
The library SHALL NOT interpret an unknown revision or code through a nearest known version.
The library SHALL refuse a response whose selected source-contract identity or revision differs from the resolved one, and a response whose local originating stage has been erased or replaced.

## Inputs

One typed upstream failure response carrying a declared source contract and a
stable code; and the exact source contract this integration resolved through
[FR-009](./FR-009-lock-external-producer-revisions.md).

## Outputs

Either the retained typed source refusal whose code is a declared member of the
exact selected contract, or one refusal carrying the local
`unsupported-source-contract` disposition together with the original contract and code
value retained byte-exactly and uninterpreted.

## Behavior

The library SHALL order validation before interpretation, so no code is interpreted before its source contract has been checked.
The library SHALL copy the original source-contract identity, revision and code value byte-exactly into the retained disposition.
The library SHALL NOT normalize, alias, round or approximate an unknown revision or code onto a known one.
The library SHALL keep `unsupported-source-contract` distinguishable per refusal axis by the original code and cause it retains, so the disposition is not a catch-all identity.
The library SHALL NOT author a source-owner producer-stage member list, a source-owner code list, or a nearest-known-version table. This prohibition does not apply to FR-008's Protocol-owned local vocabulary.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-038-AC-1 | A source refusal code is matched only after the exact FR-009-resolved source contract is selected, and interpreting a code before that contract is checked fails the path. | Test (TC-011) |
| FR-038-AC-2 | An unknown source contract, revision or code becomes the local `unsupported-source-contract` disposition, retains the original contract and value uninterpreted and byte-exact, and is not interpreted through a nearest known version. | Test (TC-011) |
| FR-038-AC-3 | Changing the selected source-contract identity or revision refuses the response, and erasing or replacing the local originating stage refuses the response, including when a wrapper is the party that changed it. | Test (TC-011) |
| FR-038-AC-4 | Two distinct refusal axes both retained under `unsupported-source-contract` remain distinguishable by their retained original code and cause; two axes collapsing onto one retained value fail FR-014-AC-1 rather than passing as one supported disposition. | Test (TC-011) |
| FR-038-AC-5 | No local nearest-known-version mapping or source-owner producer-stage/code list is authored; supported source variants are matched exhaustively from their pinned Rust type. | Test (TC-011) |
| FR-038-AC-6 | Local Protocol causes require no external catalog; a scenario requiring an unresolved source contract is unexecutable, while known pinned source contracts and unknown-contract retention remain executable. | Test (TC-011, TC-013) |

## Adopted vocabulary

The `unsupported-source-contract` disposition is this repository's and
[FR-008](./FR-008-return-typed-refusals.md) owns it. The contract whose exactness
this requirement enforces is the producer's typed Rust interface selected
through FR-009. FR-270 through FR-273 may define a future adapter-specific
interpretation catalog, but their proposed catalog is not required by the
common envelope. [FR-013](./FR-013-offered-versus-selected.md) supplies the
offered-versus-selected discrimination this requirement applies.

FR-038-AC-4 is the injectivity obligation in this requirement's area. A retention
disposition is the natural place for a single shared identity to hide: every
unknown code could be folded into one `unsupported-source-contract` value that
then satisfies every negative scenario row. The criterion is stated over the
retained original code and cause, which the disposition does not determine, so it
can fail.

Source stages, loci and opaque payloads are retained exactly when the selected
contract supplies them. Protocol never promotes them into the local stage axis
or synthesizes a missing field.

## Dependencies

- [FR-008](./FR-008-return-typed-refusals.md) owns the envelope and the
  `unsupported-source-contract` disposition; this requirement is `part_of` it.
- [FR-009](./FR-009-lock-external-producer-revisions.md) resolves each source
  contract and owns the `unresolved` gate a dependent scenario inherits.
- [FR-013](./FR-013-offered-versus-selected.md) owns the offered-versus-selected
  distinction.
- [FR-014](./FR-014-typed-outcome-identity.md) owns the injectivity obligation
  FR-038-AC-4 keeps checkable.

## Status

Draft. Existing source errors are not yet all normalized through the common
envelope; Task-006 completes the coverage and TC-011.
