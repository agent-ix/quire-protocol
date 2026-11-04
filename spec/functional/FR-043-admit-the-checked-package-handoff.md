---
id: FR-043
title: "Admit the checked-package handoff with the Contract IR reader"
type: FR
relationships:
  - { target: ix://agent-ix/quire-protocol/US-001, type: implements }
  - { target: ix://agent-ix/quire-protocol/FR-001, type: references }
  - { target: ix://agent-ix/quire-protocol/FR-008, type: references }
  - { target: ix://agent-ix/quire-spec-language/FR-517, type: depends_on }
  - { target: ix://agent-ix/quire-spec-language/FR-518, type: depends_on }
  - { target: ix://agent-ix/quire-spec-language/FR-521, type: references }
---
# FR-043: Admit the checked-package handoff with the Contract IR reader

## Description

When a protocol is submitted as a checked-package handoff, the library SHALL read
it with `quire_contract_model::read_checked_package`, check the result against
the handoff's selection, and expose no admitted package unless both succeed. The
library SHALL NOT contain a second reader, a forked reader or a locally declared
refusal vocabulary for the checked package.

## Inputs

- The handoff directory that `quire_spec_language::checked_handoff` writes
  (QSL FR-517): `checked-package.json`, `sources/<label>`, `models/<n>.json`,
  `inventory.json`, `selection.json` and, for a mutation corpus, `mutations/`.
- The evidence assembled from the handoff's own files, as QSL FR-517 states.
- The selection (`selection.json`, `quire.checked-handoff.selection/1`), whose
  identities were derived from the source before any bytes were emitted.

## Outputs

One admitted `CheckedPackageV2`, or one typed refusal carrying Contract IR's
`CheckedPackageRefusalCode` or QSL FR-518's table code, unchanged.

## Behavior

- The library shall read `checked-package.json` through
  `read_checked_package` and no other function.
- The library shall compare the offered identities with the selection before it
  reads a field that depends on them (`Selection::check`, QSL FR-517).
- The library shall return the refusal code the reader or the selection check
  returned and shall add no code of its own for a checked-package defect.
- The library shall admit the format `quire.checked-package/v2` through its one
  explicit entry point and shall not infer a format or version from the offered
  bytes.
- The library shall read integers and exact rationals only in the Contract IR
  (QSpec FR-322) encoding the reader decodes, and shall re-implement no numeric
  decoding.
- The library shall expose no linked representation, view or assessment input
  before admission has succeeded.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-043-AC-1 | Over the three packages of **R** (QSL FR-517: event position, fixed sample, timestamped event), the handoff is admitted with every selection identity unchanged and the admitted package is the value `read_checked_package` returned. | Test (TC-149) |
| FR-043-AC-2 | Each case of the handoff's standard mutation corpus (`quire.checked-handoff.mutations/1`, QSL FR-518) returns the refusal code its manifest row names, carried unchanged, and no linked representation is exposed. | Test (TC-149) |
| FR-043-AC-3 | Changing the caller's selection identity while the offered bytes stay unchanged returns a selection refusal and no admitted package; the offered bytes are not read before the comparison. | Test (TC-149) |
| FR-043-AC-4 | No source file of the crate parses checked-package bytes or declares a refusal code or format string for the checked package: the only call that reads them is `read_checked_package`, and the refusal vocabulary is imported. | Inspection (TC-149) |
| FR-043-AC-5 | Bytes of `quire.compiled-protocol/1` or `/2` offered at the checked-package entry point return a typed refusal, and checked-package bytes offered at a retired entry point return a typed refusal; no version is detected, upgraded or repaired. | Test (TC-149) |
| FR-043-AC-6 | A bare JSON number outside the safe-integer interval, a non-canonical rational and a signed-64 overflow in a package field each return the reader's refusal unchanged, and the library holds no numeric decoder. | Test (TC-149) |

## Dependencies

- QSL FR-517 and FR-518 (the handoff, selection and mutation corpus), QSL FR-521
  (the repoint gate), Contract IR's `read_checked_package`.
- [FR-001](FR-001-admit-selected-protocol-profile.md) states the admission
  contract for the retired `quire.compiled-protocol/2` wire; this requirement is
  its checked-package successor.
