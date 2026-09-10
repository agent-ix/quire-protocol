---
id: TC-010
title: "Preserve Rust and shared assurance containment"
type: TC
relationships:
  - target: ix://agent-ix/quire-protocol/NFR-003
    type: verifies
---
# TC-010: Preserve Rust and shared assurance containment

## Description

Verify the first-party language, traceability, shared-assurance, unsafe-code, and
manual-workflow boundaries over the complete repository.

## Test Procedure

Inventory executable source and workflow triggers, inspect Cargo metadata and
dependencies, run Quire coverage, search for prohibited local runner/evidence
surfaces, and mutate one bare trace binding and each workflow trigger guard.

## Expected Results

All scoped executable logic and assertions are Rust, all acceptance tests carry
bare compiler-checked bindings, unsafe Rust is forbidden, no duplicate assurance
surface exists, and every hosted workflow is manual-dispatch only.

