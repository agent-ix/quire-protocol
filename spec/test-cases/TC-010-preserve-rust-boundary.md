---
id: TC-010
title: "Preserve Rust and shared assurance containment"
type: TC
relationships:
  - target: ix://agent-ix/quire-protocol/NFR-003
    type: verifies
---
# TC-010: Preserve Rust and shared assurance containment

## Metadata

- **Execution**: standing. TC-010 runs in the local gate of every change, not
  only at the qualification gate. A containment audit that runs once at the end
  reports a violation months after the commit that introduced it.
- **Evidence for IT-001-SC-05**: the inventory portion of this test case supplies
  the "no second parser, formal frontend, or foreign runtime participates"
  evidence for IT-001-SC-05. That evidence needs no producer fixture and may
  close independently of IT-001-SC-01..SC-04.
- **Current implementation**: `tests/tc_010_rust_boundary.rs`, a repository-local
  Rust audit. The shared Rust-native language-boundary checker required by
  NFR-003's Verification section (`agent-ix/quire-research#64`) is not published,
  so the shared-checker clause of NFR-003-AC-3 is **not** satisfied by this
  implementation and remains outstanding.

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

Enumerate the Rust-wrapper bypass explicitly: a first-party Rust file whose body
invokes a non-Rust interpreter or script — a `std::process::Command` spawn of a
shell, an interpreter, or a generated script template — fails NFR-003-AC-3
exactly as the script itself would. A Rust wrapper around non-Rust logic is not
remediation.

## Expected Results

All scoped executable logic and assertions are Rust, all acceptance tests carry
bare compiler-checked bindings, unsafe Rust is forbidden, no duplicate assurance
surface or in-process Quoin implementation exists, and every hosted workflow is
manual-dispatch only. Each language-boundary fixture mutation is rejected by the
accepted shared checker or the unsafe-code compiler gate for its intended reason;
no local scanner supplies the verdict.
