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

After the composed review accepts and freezes A's coherent native Quire baseline
and [quire-spec-language#40](https://github.com/agent-ix/quire-spec-language/issues/40)
publishes an accepted versioned compiled-protocol artifact contract, establish the
`AGPL-3.0-or-later`, deny-by-default
Rust library and implement exact profile admission without a parser or locally
authored formal profile. Admit native signed-64 and exact-rational values only
through the contract's lossless numeric wire profile.

## Subtasks

- [x] Apply the selected `AGPL-3.0-or-later` license and record its compatibility policy.
- [ ] Complete dependency compatibility review before adding `Cargo.toml` or source.
- [ ] Write TC-001 first with the accepted artifact and every one-axis contract,
  producer, identity, state, resealing, feature, unsafe-number,
  non-canonical-number, boundary, invalid-rational, and digest-domain mutation.
- [ ] Add the minimal Rust crate with forbidden unsafe code, denied warnings, pinned dependencies, and no automatic workflow trigger.
- [ ] Recompute all accepted-baseline and dependency identities before dependent reads and return the FR-008-compatible typed admission refusal.
- [ ] Bind every acceptance test to FR-001 and its upstream source obligations with bare `ix-trace-rs` declarations.

## Deliverables

- Reviewed license record, Rust crate boundary, accepted-baseline fixture, strict admission API, and TC-001.

## Notes

- Blocked by acceptance and exact freeze of the coherent composed baseline, which
  must carry the integrated FR-061/FR-062 semantics, and by the A-owned
  versioned compiled-protocol artifact contract and numeric wire profile tracked by
  [quire-spec-language#40](https://github.com/agent-ix/quire-spec-language/issues/40).
- No source or dependency manifest may be added while either source gate remains open.
- C-05 now retains every A-owned admission error as the exact published `Error`
  value and delegates its stable code to `Error::code`; `Producer` and `/2`
  errors no longer fall through to a debug string. This is partial admission
  evidence only and does not complete TC-001 or change this task's status.
