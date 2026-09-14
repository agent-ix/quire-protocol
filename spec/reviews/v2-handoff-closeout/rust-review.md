---
id: SR-063
title: "Rust review of the published version 2 handoff closeout"
type: SpecReview
analysis: code-review
scope: "quire-protocol issue 48 implementation, dependency pins, and version 2 handoff tests on task/48-published-v2-handoff"
review_set: subset
relationships:
  - { target: ix://agent-ix/quire-protocol/IT-001, type: reviews }
  - { target: ix://agent-ix/quire-protocol/Task-012, type: reviews }
---
## Summary

The issue #48 increment is fit for promotion after two review findings were
remediated. It changes no public implementation or reader path: the Rust change
imports the producer's already-published handoff constants into the integration
harness, strengthens its assertions, aligns exact dependency revisions, and
removes a stale production API-document reference to the retired provisional
test corpus.

## Verdict

**PASS** — no unresolved correctness, safety, idiom, test-oracle, resource,
dependency-identity, or repository-boundary finding remains in this ticket.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | A production API doc still named `MutatedSelection` in the deleted provisional test module. Fixed by removing that obsolete comparison while retaining the version 1 producer/refinement boundary. | `src/inventory/axis.rs`; deleted `tests/support/v2/mod.rs` |
| FND-002 | medium | SC-09 did not compare the exact and post-refusal reports' complete `Usage` values with the baseline. Fixed by asserting complete usage equality before consuming both admitted packages; the focused 15-test lane passes. | IT-001-SC-09; `tests/it001_v2_handoff.rs` |

## Review record

- Public API and trait seams: unchanged. `admit_and_link` and
  `admit_and_link_v2` remain separate caller-selected entry points; no detector,
  translation, fallback, parser, or wire mirror was added.
- Refusal oracle: all 28 producer-declared mutations must return the declared
  `Error::code`; full typed errors must differ across distinct mutation axes.
  Both ordered cross-version submissions return errors, differ from each other,
  and collide with none of those mutation outcomes.
- Bounds: every one of the 13 `Dimension::ALL` members is non-zero for the base
  handoff, admits at exact measured usage, refuses one-short as
  `Error::Incomplete` with the selected dimension and limit, and admits the
  unchanged input with identical usage under a fresh exact budget.
- Tests and panic surface: all added `expect`, `unwrap_err`, and `panic!` calls
  are assertion failures inside integration tests. No production refusal path
  gained a panic, side effect, blocking call, lock, unsafe block, or unchecked
  integer conversion.
- Dependency identity: `quire-spec-language` is fixed to `93c411b` and the
  direct test pin for `quire-contract-ir` is aligned to the same transitive
  `04eb6f8` revision. `cargo tree --locked -d` shows one Contract-IR package
  identity, not parallel revisions. The crate remains `publish = false`.
- Ownership: no `resources/native-v1/` or TL-* file, dependency, branch, or
  ticket was touched.

## Execution evidence

- `cargo fmt --all -- --check`: pass.
- `cargo clippy --locked --all-targets --target-dir
  target-codex-backends -- -D warnings`: pass.
- `cargo test --locked --target-dir target-codex-backends --
  --test-threads=1`: 153 passed, 0 failed, 23 ignored; 37 doc-tests passed.
- `cargo test --locked --test it001_v2_handoff --target-dir
  target-codex-backends -- --test-threads=1`: 15 passed, 0 failed, 0 ignored.
- Toolchain: `rustc 1.98.1`, `cargo 1.98.1`.

All commands ran locally with `CARGO_BUILD_JOBS=1` and the shared heavy-check
lock where compilation was involved. No hosted workflow was dispatched.
