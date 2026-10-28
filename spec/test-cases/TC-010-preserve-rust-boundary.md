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
dependencies, run Quire coverage, and search for prohibited local
runner/evidence surfaces. Add an unauthorized unsafe Rust block, a
Quoin implementation/runtime dependency, and local
runner/parser/store/catalogue/disposition symbols. Mutate one bare trace binding
and the build/qualification workflow trigger constraints. Verify that all build and qualification workflows retain
only `workflow_dispatch` and that hosted status is absent from the local
gate evidence.

## Expected Results

All scoped executable logic and assertions are Rust, all acceptance tests carry
bare compiler-checked bindings, unsafe Rust is forbidden, no duplicate assurance
surface or in-process Quoin implementation exists. Build and qualification
workflows are manual-dispatch only; the existing CLA administration workflow
retains its separate triggers. No hosted status is local gate evidence. Each
mutation is rejected for its intended reason.

## Traceability
This test case additionally backs criteria on the object requirements FR-017, which cite it as their verification. Those criteria are exercised by the same procedure above; a change here that stops exercising them is a change to their coverage, not only to this test case's.
