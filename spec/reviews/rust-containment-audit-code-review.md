---
id: SR-024
title: "Code review — C-32 TC-010 Rust containment audit"
type: SpecReview
analysis: code-review
scope: "tests/tc_010_rust_boundary.rs, spec/test-cases/TC-010-preserve-rust-boundary.md, spec/tests.md"
review_set: subset
---

## Summary

Reviewed chunk C-32, which implements TC-010 as a standing Rust audit that fails
the local gate when a non-Rust executable path enters the repository. The audit
itself is Rust with no process spawn and audits its own source; four defects were
found during review and live-fire testing and are fixed on the same head, and one
outstanding spec-level gap is recorded — the shared language-boundary checker
NFR-003 requires does not exist yet.

## Verdict

**CONDITIONAL** — no high findings remain on this head; FND-005 is an unmet
NFR-003 clause owned by another repository and is recorded, not remediated here.

## Assurance Context

- **Profile**: AP-001 "Protocol implementation assurance profile"
  (`spec/assurance/AP-001-protocol-implementation.md`, `profile_version: 0.2`,
  `status: proposed`).
- **Evaluated revision**: branch `agent-b/c32-rust-containment-audit`, based on
  `agent-b/c00-lib-facade` @ `48db039`, itself based on `origin/main` @
  `681e25a`.
- **Context examined**: AP-001's impact assessments, NFR-003 and its six
  acceptance criteria, TC-010, TM-001, `AGENTS.md`, and the Cargo lint
  configuration.
- **Relation to the profile**: this change adds no protocol behaviour and
  therefore bears on none of AP-001's impact assessments directly. It is a
  control on the containment boundary those assessments are evaluated within.
- **Unavailable context**: the shared Rust-native language-boundary checker
  (`agent-ix/quire-research#64`) is unpublished, so its structured result — the
  evidence NFR-003's Verification section names — could not be consumed or
  cited. No architecture-evaluation or producer-reliance evidence was selected
  for this change.
- **Active exceptions**: none. The audit's own allowlist carries five entries,
  all one finding (the wasm32-gated wasm-bindgen bridge), each with a written
  reason and a ticket.

## Findings

| ID      | Severity | Summary                                                                                                  | Refs                                  | Escape Cause                        |
| ------- | -------- | -------------------------------------------------------------------------------------------------------- | ------------------------------------- | ----------------------------------- |
| FND-001 | high     | `.sh` was absent from the script-extension set; shell scripts were caught only by the unclassified fallback | tests/tc_010_rust_boundary.rs:210      | implementation-bug-despite-evidence |
| FND-002 | medium   | Walker followed symbolic links, so a link could point the census at unaudited bytes or cycle the walk      | tests/tc_010_rust_boundary.rs:437      | correct-requirement-no-evidence     |
| FND-003 | medium   | Every dot-file was skipped as inert, admitting `.envrc` and similar shell configuration unexamined         | tests/tc_010_rust_boundary.rs:508      | correct-requirement-no-evidence     |
| FND-004 | low      | A filter key nested under a workflow trigger (`branches:` under `push:`) was reported as a trigger         | tests/tc_010_rust_boundary.rs:661      | implementation-bug-despite-evidence |
| FND-005 | medium   | NFR-003 requires the shared checker from quire-research#64 and forbids a local replacement; it is unpublished | spec/non-functional/NFR-003-preserve-rust-assurance-boundary.md:60 | missing-requirement                 |
| FND-006 | low      | TM-001's Test Execution Summary totals are now stale against the TC-010 row, by the collision rule         | spec/tests.md:79                       | correct-requirement-no-evidence     |

### Disposition

- **FND-001, FND-002, FND-003, FND-004**: fixed on this head, each with a test
  that fires the rule. FND-001 and FND-004 were found by planting real
  violations in the working tree and observing the gate; FND-002 and FND-003
  were found by reading the walker against the rename/extensionless bypass
  family NFR-003-AC-3 names.
- **FND-005**: recorded in the test's module documentation, in TC-010's
  Metadata section, and in the TM-001 rows. TC-010's status is `⚠️ Partial`
  rather than passing so the unmet clause is not claimed as evidence. Remedy is
  owned by `agent-ix/quire-research#64`, not by this repository.
- **FND-006**: intentional. COLLISION-4 reserves the summary totals for one
  bookkeeping pass per queue wave; a note in TM-001's overview records that the
  totals lag deliberately.

### Checks that produced no finding

- **Rust idiom**: closed `Rule` enum matched exhaustively with no catch-all;
  `BTreeSet` and a sorted census for deterministic output; `&str`/`&[u8]`
  parameters with owned returns; `Display` implemented rather than an inherent
  formatting method.
- **Panic surface**: every `unwrap`/`expect`/`panic!` is in a test binary and
  names the path and error. The `.gitignore` parser deliberately panics on a
  pattern form the walker cannot honour, rather than silently hiding files.
- **Tautology and stub detection**: no assertion is over a value the test set
  and never read back. Every rule has a firing test whose failure mode is real —
  deleting a rule turns its firing test red. The `fires` helper fails when a
  rule produces nothing, so a rule that greps fine and binds to nothing cannot
  pass.
- **Guarded assertions**: no assertion sits inside an `if let` without a failing
  `else`.
- **Seam compliance**: the audit reads the real tree; the one synthetic surface
  (`inspect_file` over in-memory bytes) is the pure classifier, and the walker
  that feeds it is separately proved against a planted on-disk tree.
- **Determinism**: no clock-based assertion, no `HashMap` iteration order, no
  fixed port; the temp directory is per-process and per-nanosecond and is
  removed.
- **Integrity**: no `#[allow]` added, no lint weakened, no `#[ignore]`, no new
  dependency (COLLISION-7 reserves `Cargo.toml` for one chunk per wave).
- **Trace bindings**: all 18 tests carry bare `#[trace(...)]` attributes naming
  the criterion each verifies; the audit enforces this on itself.

### Gates run on this head

| Gate                                                      | Result                                                      |
| --------------------------------------------------------- | ----------------------------------------------------------- |
| `cargo fmt --all -- --check`                              | clean, no diff                                              |
| `cargo clippy -j 1 --all-targets -- -D warnings`          | clean, 0 warnings                                           |
| `cargo test -j 1 -- --test-threads=1`                     | 18 passed, 0 failed, 9 ignored (IT-001 fixture lane), 0 unit, 0 doc |

The `--features quire-extraction` lane named in the standing gate was **not
run**: `Cargo.toml` declares no `[features]` table, so the lane does not exist.
It is recorded here as unrun with its reason rather than reported as passed.
