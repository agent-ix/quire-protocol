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

After the composed review accepts A's proposed immutable native Quire candidate
`1-draft.4`, establish the `AGPL-3.0-or-later`, deny-by-default Rust library and
implement exact profile admission without a parser or locally authored formal profile.

## Subtasks

- [ ] Apply the selected `AGPL-3.0-or-later` license and complete dependency compatibility review before adding `Cargo.toml` or source.
- [ ] Write TC-001 first with the accepted artifact and every one-axis identity, state, resealing, and feature mutation.
- [ ] Add the minimal Rust crate with forbidden unsafe code, denied warnings, pinned dependencies, and no automatic workflow trigger.
- [ ] Recompute all candidate and dependency identities before dependent reads and return the FR-008-compatible typed admission refusal.
- [ ] Bind every acceptance test to FR-001 and its upstream source obligations with bare `ix-trace-rs` declarations.

## Deliverables

- Reviewed license record, Rust crate boundary, selected candidate fixture, strict admission API, and TC-001.

## Notes

- Blocked only by acceptance of proposed candidate `1-draft.4`, which carries the
  integrated FR-061/FR-062 bytes but is not yet adopted.
- No source or dependency manifest may be added while either gate remains open.
