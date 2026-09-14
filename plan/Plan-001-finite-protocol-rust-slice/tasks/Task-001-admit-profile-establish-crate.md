---
id: Task-001
title: "Admit the selected profile and establish the Rust crate"
type: Task
status: done
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
- [x] Complete dependency compatibility review before adding `Cargo.toml` or source.
- [x] Write TC-001 first with the accepted artifact and every one-axis contract,
  producer, identity, state, resealing, feature, unsafe-number,
  non-canonical-number, boundary, invalid-rational, and digest-domain mutation.
- [x] Add the minimal Rust crate with forbidden unsafe code, denied warnings, pinned dependencies, and no automatic workflow trigger.
- [x] Recompute all accepted-baseline and dependency identities before dependent reads and return the FR-008-compatible typed admission refusal.
- [x] Bind every acceptance test to FR-001 and its upstream source obligations with bare `ix-trace-rs` declarations.

## Deliverables

- Reviewed license record, Rust crate boundary, accepted-baseline fixture, strict admission API, and TC-001.

## Notes

- The former external blockers are delivered. Quire Specification PR #15
  accepted the coherent baseline; quire-spec-language `v0.2.0` and the exact
  pinned post-release revision `7931a0021612d26132ea8bbef3b0cd19b27248e0`
  publish the versioned readers, lossless numeric profile and default-running
  version 2 handoff.
- Task-001 closes TC-001 and FR-001-AC-1 through FR-001-AC-4. Task-009 retains
  FR-001-AC-5 and IT-001 SC-06a's complete two-revision version 1 comparison;
  that independent integration remainder does not block the strict admission
  and finite-link prerequisite used by Task-002.
- C-05 now retains every A-owned admission error as the exact published `Error`
  value and delegates its stable code to `Error::code`; `Producer` and `/2`
  errors no longer fall through to a debug string. TC-001 verifies that
  boundary through the current default-running owner corpus.

## Completion Evidence

- `DEPENDENCIES.md`, `Cargo.toml` and `Cargo.lock` agree on the exact direct and
  transitive revisions and compatible licenses; the crate remains
  `publish = false`, forbids unsafe code and denies Clippy warnings.
- The default-running version 2 handoff suite executes 17 tests. It verifies
  the closed owner inventory, exact admission/linking, all 28 owner-declared
  mutations, caller-selected baseline identity/revision/digest substitutions,
  exact numeric round-trip and malformed/boundary refusals, both explicit
  cross-version refusal directions, and all 13 resource dimensions.
- Bare `#[trace(...)]` attributes bind TC-001 to FR-001-AC-1 through
  FR-001-AC-4. `quire coverage --scope .` reports FR-001 at 4/5 and leaves only
  FR-001-AC-5 to Task-009's two-revision comparison.
