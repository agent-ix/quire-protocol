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

When a refusal is read, the library SHALL validate its code against the exact selected versioned diagnostic catalog before interpreting that code.
When the offered source contract, catalog selection or code is unknown, the library SHALL retain the refusal under the local `unsupported-source-contract` disposition with the original selection and the original value uninterpreted.
The library SHALL NOT interpret an unknown revision, catalog selection or code through a nearest known version.
The library SHALL refuse a response whose selected diagnostic-catalog identity, revision or digest differs from the selected one, and a response whose originating stage has been erased or replaced.

## Inputs

One typed upstream failure response carrying a declared source contract, a
diagnostic-catalog selection and a stable code; and the exact catalog selection
this integration resolved through
[FR-009](./FR-009-lock-external-producer-revisions.md).

## Outputs

Either the interpreted refusal whose code validated against the exact selected
catalog, or one refusal carrying the local `unsupported-source-contract`
disposition together with the original catalog selection and the original code
value retained byte-exactly and uninterpreted.

## Behavior

The library SHALL order validation before interpretation, so no code is interpreted before its catalog selection has been checked.
The library SHALL copy the original catalog selection and the original code value byte-exactly into the retained disposition.
The library SHALL NOT normalize, alias, round or approximate an unknown revision or code onto a known one.
The library SHALL keep `unsupported-source-contract` distinguishable per refusal axis by the original code and cause it retains, so the disposition is not a catch-all identity.
The library SHALL NOT author a local producer-stage member list, a local code list, or a nearest-known-version table.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-038-AC-1 | A refusal code is validated against the exact selected versioned diagnostic catalog before it is interpreted, and interpreting a code before its catalog selection is checked fails the path. | Test (TC-011) |
| FR-038-AC-2 | An unknown source revision, catalog selection or code becomes the local `unsupported-source-contract` disposition, retains the original catalog selection and original value uninterpreted and byte-exact, and is not interpreted through a nearest known version. | Test (TC-011) |
| FR-038-AC-3 | Changing the selected diagnostic-catalog identity, revision or digest refuses the response, and erasing or replacing the originating stage refuses the response, including when a wrapper is the party that changed it. | Test (TC-011) |
| FR-038-AC-4 | Two distinct refusal axes both retained under `unsupported-source-contract` remain distinguishable by their retained original code and cause; two axes collapsing onto one retained value fail FR-014-AC-1 rather than passing as one supported disposition. | Test (TC-011) |
| FR-038-AC-5 | No local nearest-known-version mapping, producer-stage member list or diagnostic-code list is authored by this requirement, and each such dependency cites the upstream gap instead. | Inspection |
| FR-038-AC-6 | Every criterion above that requires an exact selected catalog is reported unexecutable while the FR-009 diagnostic-catalog entry's status is `unresolved`, rather than passing or failing. | Inspection |

## Adopted vocabulary

The `unsupported-source-contract` disposition is this repository's and
[FR-008](./FR-008-return-typed-refusals.md) owns it. The catalog selection whose
exactness this requirement enforces is upstream's
`ix://agent-ix/quire-specification/FR-270`, keyed on interpretation identity,
revision and byte digest; the code is `FR-271`'s; the originating stage is
`FR-273`'s. [FR-013](./FR-013-offered-versus-selected.md) supplies the
offered-versus-selected discrimination this requirement applies to a catalog
selection.

FR-038-AC-4 is the injectivity obligation in this requirement's area. A retention
disposition is the natural place for a single shared identity to hide: every
unknown code could be folded into one `unsupported-source-contract` value that
then satisfies every negative scenario row. The criterion is stated over the
retained original code and cause, which the disposition does not determine, so it
can fail.

### Where the upstream vocabulary does not yet reach

**The selected catalog is not adopted, and this requirement inherits that gate.**
`FR-270` states of the only catalog so far offered: "**Proposed, not adopted or
implemented.**" and calls it source material, not an adopted selection. FR-009's
manifest carries the diagnostic catalog as an `unresolved` entry, and `spec.md`
section 16 records that the reference names no adopted artifact yet. Every
criterion above that requires an *exact selected* catalog therefore selects
something no adopted artifact supplies, and FR-038-AC-6 states the consequence:
the scenario is unexecutable, not passing.

- **Originating stage** — `FR-273` records a contradiction rather than an
  absence: "Two sources enumerate the same axis and do not agree on its members."
  FR-038-AC-3 requires erasure or replacement of the stage to refuse; it rests on
  an axis whose membership is contested upstream, and it establishes no local
  membership. **Gap, owner `agent-ix/quire-specification` (`FR-273`, `FR-047`):**
  the stage axis has no agreed member list, so no criterion here can enumerate
  the stages a replacement would be detected against, only that the retained
  value must be the one that arrived.
- **Diagnostic code** — `FR-271` records that "no requirement states which
  convention belongs to which namespace", and places no literal code in a
  selected catalog. This requirement therefore states validation *against the
  selected catalog* without asserting which literals that catalog contains.

## Dependencies

- [FR-008](./FR-008-return-typed-refusals.md) owns the envelope and the
  `unsupported-source-contract` disposition; this requirement is `part_of` it.
- [FR-009](./FR-009-lock-external-producer-revisions.md) resolves the catalog
  selection and owns the `unresolved` gate this requirement inherits.
- [FR-013](./FR-013-offered-versus-selected.md) owns the offered-versus-selected
  distinction.
- [FR-014](./FR-014-typed-outcome-identity.md) owns the injectivity obligation
  FR-038-AC-4 keeps checkable.

## Status

Draft. No execution is claimed. The diagnostic-catalog version-lock entry is
`unresolved` and A's `/2` contract is unpinned, so every scenario behind the Test
criteria above is unexecutable today.
