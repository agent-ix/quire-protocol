---
id: TC-149
title: "Admit the checked-package handoff"
type: TC
relationships:
  - target: ix://agent-ix/quire-protocol/FR-043
    type: verifies
---
# TC-149: Admit the checked-package handoff

## Description

Verify admission through the Contract IR reader and the handoff selection.

## Test Procedure

Write **R**'s handoff with `quire_spec_language::checked_handoff` and admit each of its three packages; submit each `reader` case and each `selection` case of the standard mutation corpus; change the selection identity with the offer unchanged; offer `quire.compiled-protocol/1` and `/2` bytes at the checked entry point; read with each limit that has nonzero measured usage one below and at its exact usage (a limit whose usage is 0 has no representable one-below); offer an unsafe bare number, a non-canonical rational and a signed-64 overflow; inspect the source for a reader, a refusal vocabulary or a numeric decoder. Tag the tests `#[trace("TC-149", "FR-043-AC-n")]`.

For FR-043-AC-9, inspect the production dispatch match: its
`CatalogUnavailable` arm carries Contract IR's imported `CatalogReadError`
unchanged and returns before any selection or sidecar check. Check that neither
a package refusal, an incomplete read nor a panic replaces the owner's result.
This is inspection of the adapter boundary, not runtime credit for corrupting
the production catalog; no consumer test manufactures the opaque owner error.

## Expected Results

Admitted with identities unchanged; each mutation refused with its manifest code and no linked view; selection checked only over a reader-admitted package; retired formats refused; the reader's refusals and incomplete results carried unchanged, the incomplete read naming its limit and the exact-limit read admitted; no second reader in the source. The inspected catalog-unavailable branch returns the owner's read error unchanged, exposes no admitted handoff and performs no later selection or sidecar check.
