---
id: FR-047
title: "End the retired compiler surfaces and the integrity lanes"
type: FR
relationships:
  - { target: ix://agent-ix/quire-protocol/US-001, type: implements }
  - { target: ix://agent-ix/quire-protocol/FR-043, type: depends_on }
  - { target: ix://agent-ix/quire-protocol/FR-046, type: depends_on }
  - { target: ix://agent-ix/quire-spec-language/FR-511, type: references }
  - { target: ix://agent-ix/quire-spec-language/FR-521, type: depends_on }
---
# FR-047: End the retired compiler surfaces and the integrity lanes

## Description

When the library reads the checked package, it SHALL stop using the retired
compiler surfaces that QSL's requirements list. If the owner rules for removal,
it SHALL also remove the IT-001 checksum and seal lanes with `ArtifactRef.digest`
and `revision`. The planner ruled for removal (QSL-1549), and the lanes are
removed.

## Behavior

- The library shall import `quire_spec_language::number` until the repoint
  (FR-043 to FR-046) is merged and shall not name it after (QSL FR-511's end
  condition).
- The library shall name none of the retired format identities
  `quire.compiled-protocol/1` and `/2`, `quire.protocol.v2-mutations/1`, and shall
  have no `handoff-writer` Cargo feature (QSL FR-521 rows 2, 14 and 15).
- The IT-001 recipe shall be the ported recipe **R** in
  `quire_spec_language::checked_handoff` (QSL FR-517); the IT-001 and IT-002
  procedures shall be redone against it.
- **Removal (planner ruling, QSL-1549).** The IT-001 checksum and seal lanes,
  `ArtifactRef.digest` and `revision`, and `qsl_foundation::ByteDigest` on
  admitted packages are removed: an admitted package is identified by its
  canonical bytes, and no integrity digest or seal of the checked package is
  verified or carried, the precedent being the removal of source-integrity
  digests where trust is not the feature. The protocol-result and version-lock
  content digests of FR-009 and FR-015 are other items and are not affected.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-047-AC-1 | After the repoint, no source file or manifest names `quire_spec_language::number`; before it, the import exists and a build with it removed fails. | Test (TC-153) |
| FR-047-AC-2 | The crate's `Cargo.toml` has no `handoff-writer` feature, and no production source or public API names `quire.compiled-protocol/1`, `quire.compiled-protocol/2` or `quire.protocol.v2-mutations/1`; the refusal fixtures of FR-043-AC-5, which offer retired-format bytes, are the one named exception. | Inspection (TC-153) |
| FR-047-AC-3 | The IT-001 and IT-002 tests run against **R** through the checked-package route and every scenario of the retired recipe has a counterpart row or a stated reason it ends. | Test (TC-153) |
| FR-047-AC-4 | No checksum or seal lane, no `ArtifactRef.digest` or `revision` and no integrity digest of an admitted checked package exists in the crate, and an admitted package is identified by its canonical bytes; the result and version-lock content digests of FR-009 and FR-015 remain. | Inspection (TC-153) |

## Dependencies

- QSL FR-511, FR-517, FR-521 rows 2, 9, 13, 14 and 15; FR-043 to FR-046.
