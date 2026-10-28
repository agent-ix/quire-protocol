---
id: NFR-004
title: "Define the local verification gate and its configuration lanes"
type: NFR
quality_attribute: maintainability
relationships:
  - target: ix://agent-ix/quire-protocol/US-007
    type: derives_from
  - target: ix://agent-ix/quire-protocol/NFR-003
    type: constrains
  - target: ix://agent-ix/quire-protocol/FR-001
    type: constrains
  - target: ix://agent-ix/quire-protocol/FR-002
    type: constrains
  - target: ix://agent-ix/quire-protocol/FR-003
    type: constrains
  - target: ix://agent-ix/quire-protocol/FR-004
    type: constrains
  - target: ix://agent-ix/quire-protocol/FR-005
    type: constrains
  - target: ix://agent-ix/quire-protocol/FR-006
    type: constrains
  - target: ix://agent-ix/quire-protocol/FR-007
    type: constrains
  - target: ix://agent-ix/quire-protocol/FR-008
    type: constrains
---
# NFR-004: Define the local verification gate and its configuration lanes

## Statement

The repository SHALL execute its enumerated local verification gate over one
unchanged worktree state, using one Cargo-feature lane while the manifest and
tracked Rust select no Cargo feature and accounting separately for every
test-only or target-platform conditional compilation site.

## Scope

Applies to every local pre-promotion check run over this repository: formatting,
lints, and tests, in every configuration lane the crate can express. A Cargo
feature selection, a Cargo target kind such as `test`, and a target-platform
predicate such as `unix` are three different axes and are never treated as one
another. Applies to the written gate instructions in `AGENTS.md` and `README.md`
as much as to the commands themselves.

The admitted local target for one evidence run is the exact `rustc -vV` host
triple recorded with that run. This ticket qualifies
`x86_64-unknown-linux-gnu`; it makes no portability claim for a target whose
conditions did not execute.

Does not apply to the exact Rust review, or QUOIN semantic gap analysis, which remain owned by
[NFR-003](NFR-003-preserve-rust-assurance-boundary.md). Does not apply to hosted
workflow triggers or to whether a hosted run may serve as local evidence, both of
which remain owned by [NFR-003-AC-1](NFR-003-preserve-rust-assurance-boundary.md).

On the current admitted local Unix target, twenty tracked test-kind sites (thirteen
`#[cfg(test)]` and seven `cfg_attr(not(test), ...)` attributes) are selected by
`cargo clippy --all-targets` and `cargo test`, one tracked
`cfg(any(test, feature = "test-support"))` site is selected by the `test-support`
lane, and one tracked `#[cfg(unix)]` integration test is selected by that target. This establishes no
Windows or other-target support. A future condition that is false in every
declared gate execution is unexecuted until an opposing target lane is added or
the target is explicitly excluded from the supported scope.

## Rationale

A gate whose written rule cannot be executed is worse than no gate. The standing
directive to check "both the minimal and `--features quire-extraction` lanes" was
inherited from `agent-ix/quire-spec-language`, where `quire-extraction =
["dep:quire-rs"]` gates roughly forty `cfg` sites: an entire `pub mod
quire_source`, a `cfg`'d field on a `#[serde(deny_unknown_fields)]` wire struct
whose acceptance behaviour therefore differs per lane, and a whole test file
under `#![cfg(feature = "quire-extraction")]`. In that crate a second lane
catches real defects: items that compile in only one configuration, `cfg`-
divergent wire and runtime behaviour, and unused-import or dead-code drift in
whichever lane the author did not build.

While a crate declares no `[features]` table, no optional dependency and no
`cfg(feature = ...)` condition, `--no-default-features`, `--all-features` and
the default feature selection resolve to the identical unit graph. Running all
three covers no additional Cargo-feature configuration; it only multiplies
build time and creates the appearance of coverage. This crate now declares one
feature, `test-support`, which compiles `result::test_support` — validated-result
constructors for dependents' test suites — behind one
`cfg(any(test, feature = "test-support"))` site, so AC-7 obligates its opposing
lane: the gate runs clippy, test and doc in both the default and the
`test-support` selection. The existing `cfg(test)` and
`cfg(unix)` sites do not change that fact: one varies target kind and the other
varies target platform. Declaring a feature to make
`--features quire-extraction` stop erroring would be worse still: a feature with
no gated code behind it reports a green second lane while protecting nothing,
which is an untrue assurance claim under
[NFR-003](NFR-003-preserve-rust-assurance-boundary.md).

The honest definition is therefore two lanes now: the default lane and the
`test-support` lane that the crate's one declared feature obligates. The same
trigger applies to any later feature: the opposing lane is obligated at the moment
the crate acquires something for it to protect.

## Measurement and Evaluation

| Metric | Target | Threshold | Method |
| --- | --- | --- | --- |
| Enumerated gate commands executed before promotion | Every command in the Acceptance Criteria table | 100% executed, 0 skipped | Test (TC-012) |
| Cargo-feature lanes required while no feature, optional dependency or `cfg(feature = ...)` site exists | 1 | Exactly 1 | Test (TC-012) |
| Cargo build jobs during heavy local checks | 1 | 1 | Test (TC-012) |
| Denied-warning channels active on the lint command | 2 (crate `[lints]` table and `-D warnings`) | Both present | Test (TC-012) |
| Declared Cargo features with no conditionally compiled item behind them | 0 | 0 | Test (TC-012) |
| Tracked conditional-compilation sites selected by no gate command or admitted target | None | 0; 22 of 22 selected today | Test (TC-012) |
| Gate runs served from a build directory shared with another worktree | 0 | 0 | Test (TC-012) |

## Verification

Static inspection of `Cargo.toml` for a `[features]` table and `optional = true`
dependencies; a closed inventory of tracked Rust `cfg` attributes and `cfg!`
macros with the command or admitted target that selects each; execution of each
enumerated gate command over one unchanged worktree state; and a negative
control for each acceptance criterion in which the guarded condition is
independently introduced and the gate policy is required to refuse it for its
own stated reason.

Verification consumes the crate's own Cargo metadata and the exit status of the
enumerated commands. It SHALL NOT introduce a local lint runner, a stdout verdict
parser, or a shell wrapper; the gate is a documented set of Cargo invocations,
and any future automation of it is Rust under
[NFR-003](NFR-003-preserve-rust-assurance-boundary.md).

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| NFR-004-AC-1 | The local gate is exactly seven commands: `cargo fmt --all -- --check`, `cargo clippy --locked --all-targets --target-dir target-codex-backends -- -D warnings`, `cargo test --locked --target-dir target-codex-backends`, `env RUSTDOCFLAGS="-D warnings" cargo doc --locked --no-deps --target-dir target-codex-backends`, then the opposing `test-support` lane: `cargo clippy --locked --all-targets --features test-support --target-dir target-codex-backends -- -D warnings`, `cargo test --locked --features test-support --target-dir target-codex-backends`, and `env RUSTDOCFLAGS="-D warnings" cargo doc --locked --no-deps --features test-support --target-dir target-codex-backends`, run in that order over one unchanged worktree state, and every command is executed rather than assumed. | Test (TC-012) |
| NFR-004-AC-2 | Each enumerated command is recorded with the risk it detects: formatting drift for `fmt`; lint violations and, because `--all-targets` covers tests, fixtures and examples as well as the library, dead code and unused imports outside the library target for `clippy`; behavioural regression against the bound acceptance obligations for `test`; undocumented public API (`missing_docs`) — already caught by `clippy`, since `missing_docs` is a plain rustc lint, not a rustdoc-only one — and broken intra-doc links (`rustdoc::broken_intra_doc_links`) for `doc`, which is rustdoc-only and is the actual reason `doc` belongs in the gate: neither `clippy` nor `cargo check` builds rustdoc output, so neither selects it. A command with no recorded risk is not part of the gate. | Inspection (TC-012) |
| NFR-004-AC-3 | The gate runs serially with `CARGO_BUILD_JOBS=1` and under `flock /tmp/quire-heavy-check.lock`, so that concurrent sessions on the shared workstation cannot interleave heavy Cargo work. | Test (TC-012) |
| NFR-004-AC-4 | Warnings are denied on two independent channels — the crate's `[lints.rust]` and `[lints.clippy]` tables and the `-D warnings` command-line argument — and removing either channel alone still causes an introduced warning to fail the gate. | Test (TC-012) |
| NFR-004-AC-5 | While `Cargo.toml` declares no `[features]` table, no `optional = true` dependency and no tracked `cfg(feature = ...)` condition, the gate requires exactly one Cargo-feature lane, and `--no-default-features`, `--all-features` and the default selection resolve to the identical unit graph. | Test (TC-012) |
| NFR-004-AC-6 | A `--features <name>` argument naming a feature this crate does not declare is a gate definition defect, not a lane to be satisfied; the gate refuses such an instruction rather than skipping it silently or declaring the named feature. | Test (TC-012) |
| NFR-004-AC-7 | Introducing a `[features]` entry, an `optional = true` dependency or a tracked `cfg(feature = ...)` condition obligates opposing Cargo-feature lanes that build, lint and test each selectable configuration before promotion, and a missing opposing lane refuses promotion. | Test (TC-012) |
| NFR-004-AC-8 | Every tracked conditional-compilation site names how the gate selects it: `cfg(test)` is selected by `clippy --all-targets` and `test`; a target-platform predicate is selected by a recorded admitted target or an opposing target lane; and every other condition has an explicit selecting command. A condition selected by no declared execution, or a declared Cargo feature referenced by no condition, refuses promotion. | Test (TC-012) |
| NFR-004-AC-9 | Every compiling gate command uses the running worktree's literal `target-codex-backends` directory, and a result served through a shared `CARGO_TARGET_DIR` or another worktree's target path is not accepted as evidence. | Test (TC-012) |
| NFR-004-AC-10 | Every gate result is bound to one unchanged commit and clean tracked worktree state, and a result aggregated from separate worktrees, a moving head or different tracked bytes is not accepted. Whether a hosted run may stand in for a local run is owned by [NFR-003-AC-1](NFR-003-preserve-rust-assurance-boundary.md) and is not restated here. | Test (TC-012) |

## Dependencies

- **Upstream**: [NFR-003](NFR-003-preserve-rust-assurance-boundary.md) owns the
  Rust-only and shared-assurance boundary that this gate must not duplicate.
- **Upstream**: `agent-ix/quire-spec-language` owns the `quire-extraction`
  feature and the two-lane gate that feature justifies there. That repository's
  lane rule is scoped to its own optional Quire consumer and is not inherited
  here.
- **Downstream**: [TM-001](../tests.md) records this requirement's coverage
  status.
- **Downstream**: repository issue 31 owns the gap this requirement closes.
