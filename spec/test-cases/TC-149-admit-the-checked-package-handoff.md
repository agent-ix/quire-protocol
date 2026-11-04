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

Write **R**'s handoff with `quire_spec_language::checked_handoff` and admit each of its three packages; submit each `reader` case and each `selection` case of the standard mutation corpus; change the selection identity with the offer unchanged; offer `quire.compiled-protocol/1` and `/2` bytes at the checked entry point; read with each read limit one below and at its exact usage; offer an unsafe bare number, a non-canonical rational and a signed-64 overflow; inspect the source for a reader, a refusal vocabulary or a numeric decoder. Tag the tests `#[trace("TC-149", "FR-043-AC-n")]`.

## Expected Results

Admitted with identities unchanged; each mutation refused with its manifest code and no linked view; selection refused before the bytes are read; retired formats refused; the reader's refusals and incomplete results carried unchanged, the incomplete read naming its limit and the exact-limit read admitted; no second reader in the source.
