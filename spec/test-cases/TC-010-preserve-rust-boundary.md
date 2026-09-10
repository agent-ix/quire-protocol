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
dependencies, run Quire coverage, consume the accepted structured result from
the shared Rust-native language-boundary checker, and search for prohibited local
runner/evidence surfaces. Against the shared checker's maintained fixture
interface, independently add an unapproved tracked source, executable shebang,
manifest command, inline workflow command, generated executable template,
runtime dependency, extensionless executable, renamed non-Rust path, and Rust
wrapper around non-Rust semantic logic. Add an unauthorized unsafe Rust block, a
Quoin implementation/runtime dependency, and local
runner/parser/store/catalogue/disposition symbols. Mutate one bare trace binding
and each workflow trigger guard.

## Expected Results

All scoped executable logic and assertions are Rust, all acceptance tests carry
bare compiler-checked bindings, unsafe Rust is forbidden, no duplicate assurance
surface or in-process Quoin implementation exists, and every hosted workflow is
manual-dispatch only. Each language-boundary fixture mutation is rejected by the
accepted shared checker or the unsafe-code compiler gate for its intended reason;
no local scanner supplies the verdict.
