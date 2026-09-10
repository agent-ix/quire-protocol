---
id: Task-001
title: "Admit the selected profile and establish the Rust crate"
type: Task
status: blocked
track: A
priority: P0
relationships:
  - { target: ix://agent-ix/quire-protocol/FR-001, type: references }
  - { target: ix://agent-ix/quire-protocol/TC-001, type: verifies }
---
# Task-001: Admit the Selected Profile and Establish the Rust Crate

## Scope

After the owner selects the outbound SPDX license and A publishes the successor
immutable native Quire candidate, establish the deny-by-default Rust library and
implement exact profile admission without a parser or locally authored formal profile.

## Subtasks

- [ ] Record the selected SPDX license and complete dependency compatibility review before adding `Cargo.toml` or source.
- [ ] Write TC-001 first with the accepted artifact and every one-axis identity, state, resealing, and feature mutation.
- [ ] Add the minimal Rust crate with forbidden unsafe code, denied warnings, pinned dependencies, and no automatic workflow trigger.
- [ ] Recompute all candidate and dependency identities before dependent reads and return the FR-008-compatible typed admission refusal.
- [ ] Bind every acceptance test to FR-001 and its upstream source obligations with bare `ix-trace-rs` declarations.

## Deliverables

- Reviewed license record, Rust crate boundary, selected candidate fixture, strict admission API, and TC-001.

## Notes

- Blocked by the owner's exact SPDX decision and A's successor to `1-draft.3` carrying integrated FR-061/FR-062 bytes.
- No source or dependency manifest may be added while either gate remains open.
