---
id: TC-153
title: "End the retired compiler surfaces and the integrity lanes"
type: TC
relationships:
  - target: ix://agent-ix/quire-protocol/FR-047
    type: verifies
---
# TC-153: End the retired compiler surfaces and the integrity lanes

## Description

Verify the end conditions of the retired surfaces; the digest and seal row follows the planner ruling for removal (QSL-1549).

## Test Procedure

Build with and without the `number` import before and after the repoint; inspect `Cargo.toml` and the source for the retired features and format identities; run IT-001 and IT-002 against **R** and list each retired scenario's counterpart; inspect for checksum, seal, `ArtifactRef.digest` and `revision` (the planner ruled for removal). Tag the tests `#[trace("TC-153", "FR-047-AC-n")]`.

## Expected Results

The import exists until the repoint and not after; no retired feature or identity; every scenario has a counterpart or a stated end; no integrity lane or digest in the crate.
