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

The repository SHALL define its local verification gate as an enumerated set of
Cargo invocations, each bound to the exact build configuration it covers and to
the stated risk it detects, and SHALL run exactly one configuration lane while
the crate declares no Cargo feature and contains no conditionally compiled item.

## Scope

Applies to every local pre-promotion check run over this repository: formatting,
lints, and tests, in every configuration lane the crate can express. Applies to
the written gate instructions in `AGENTS.md` and `README.md` as much as to the
commands themselves.

Does not apply to the shared Rust language-boundary checker, the exact Rust
review, or QUOIN semantic gap analysis, which remain owned by
[NFR-003](NFR-003-preserve-rust-assurance-boundary.md). Does not apply to hosted
workflow triggers or to whether a hosted run may serve as local evidence, both of
which remain owned by [NFR-003-AC-1](NFR-003-preserve-rust-assurance-boundary.md).

While the repository remains specification-only under `spec.md` section 14, the
`cargo test` command detects little, because every acceptance test is planned and
the one present test file is ignored pending A's producer output. The command is
nonetheless part of the enumerated gate from the outset, so that its risk
coverage grows with the suite rather than being introduced later as a new gate
step. The `fmt` and `clippy` commands are fully effective today.

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

This crate declares no `[features]` table and contains no `cfg` attribute, so
`--no-default-features` and `--all-features` resolve to the identical unit
graph. Running both covers no additional configuration; it only doubles build
time and creates the appearance of coverage. Declaring a feature to make
`--features quire-extraction` stop erroring would be worse still: a feature with
no gated code behind it reports a green second lane while protecting nothing,
which is an untrue assurance claim under
[NFR-003](NFR-003-preserve-rust-assurance-boundary.md).

The honest definition is therefore a single lane now, plus a stated trigger that
obligates the second lane at the moment the crate acquires something for it to
protect.

## Measurement and Evaluation

| Metric | Target | Threshold | Method |
| --- | --- | --- | --- |
| Enumerated gate commands executed before promotion | Every command in the Acceptance Criteria table | 100% executed, 0 skipped | Test (TC-012) |
| Configuration lanes required, while no feature and no `cfg` item exists | 1 | Exactly 1 | Inspection (TC-012) |
| Cargo build jobs during heavy local checks | 1 | 1 | Test (TC-012) |
| Denied-warning channels active on the lint command | 2 (crate `[lints]` table and `-D warnings`) | Both present | Test (TC-012) |
| Declared Cargo features with no conditionally compiled item behind them | 0 | 0 | Test (TC-012) |
| Gate runs served from a build directory shared with another worktree | 0 | 0 | Test (TC-012) |

## Verification

Static inspection of `Cargo.toml` for a `[features]` table and for
`optional = true` dependencies; static inspection of tracked Rust sources for
`cfg` attributes and `cfg!` macros; execution of each enumerated gate command
over one unchanged head with its recorded configuration; and a negative control
for each acceptance criterion in which the guarded condition is independently
introduced and the gate is required to refuse it for its own stated reason.

Verification consumes the crate's own Cargo metadata and the exit status of the
enumerated commands. It SHALL NOT introduce a local lint runner, a stdout verdict
parser, or a shell wrapper; the gate is a documented set of Cargo invocations,
and any future automation of it is Rust under
[NFR-003](NFR-003-preserve-rust-assurance-boundary.md).

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| NFR-004-AC-1 | The local gate is exactly `cargo fmt --all -- --check`, `cargo clippy --locked --all-targets -- -D warnings`, and `cargo test --locked`, run in that order over one unchanged head, and every one of the three is executed rather than assumed. | Test (TC-012) |
| NFR-004-AC-2 | Each enumerated command is recorded with the risk it detects: formatting drift for `fmt`; lint violations and, because `--all-targets` covers tests, fixtures and examples as well as the library, dead code and unused imports outside the library target for `clippy`; behavioural regression against the bound acceptance obligations for `test`. A command with no recorded risk is not part of the gate. | Inspection (TC-012) |
| NFR-004-AC-3 | The gate runs serially with `CARGO_BUILD_JOBS=1` and under `flock /tmp/quire-heavy-check.lock`, so that concurrent sessions on the shared workstation cannot interleave heavy Cargo work. | Test (TC-012) |
| NFR-004-AC-4 | Warnings are denied on two independent channels — the crate's `[lints.rust]` and `[lints.clippy]` tables and the `-D warnings` command-line argument — and removing either channel alone still causes an introduced warning to fail the gate. | Test (TC-012) |
| NFR-004-AC-5 | While `Cargo.toml` declares no `[features]` table and no `optional = true` dependency, the gate requires exactly one configuration lane, and `--no-default-features`, `--all-features` and the default selection are asserted to resolve to the identical unit graph. | Test (TC-012) |
| NFR-004-AC-6 | A `--features <name>` argument naming a feature this crate does not declare is a gate definition defect, not a lane to be satisfied; the gate refuses such an instruction rather than skipping it silently or declaring the named feature. | Test (TC-012) |
| NFR-004-AC-7 | Introducing any `cfg` attribute, `cfg!` macro, `optional = true` dependency, or `[features]` entry obligates a second gate lane that builds, lints and tests the opposing configuration before that change may be promoted, and the absence of that second lane fails the gate. | Test (TC-012) |
| NFR-004-AC-8 | A declared Cargo feature that no conditionally compiled item references is refused, so no lane can report a passing configuration it does not actually select. | Test (TC-012) |
| NFR-004-AC-9 | Every gate command builds into a target directory owned by the running worktree, and a gate run served from a `CARGO_TARGET_DIR` shared with another worktree is not accepted as evidence. | Test (TC-012) |
| NFR-004-AC-10 | Every gate result is bound to one unchanged head, and a result aggregated from separate working trees or from a head that moved between commands is not accepted. Whether a hosted run may stand in for a local run is owned by [NFR-003-AC-1](NFR-003-preserve-rust-assurance-boundary.md) and is not restated here. | Test (TC-012) |

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
