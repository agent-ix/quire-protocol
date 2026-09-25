---
id: TC-012
title: "Run the local verification gate and refuse an undefined lane"
type: TC
relationships:
  - target: ix://agent-ix/quire-protocol/NFR-004
    type: verifies
---
# TC-012: Run the local verification gate and refuse an undefined lane

## Description

Verify that the enumerated local gate commands execute over one unchanged clean
worktree state in each Cargo-feature lane this crate currently expresses — the
default lane and the opposing `test-support` lane —
that every tracked conditional-compilation site is selected by a named command
or admitted target, and that an undeclared, phantom or unexecuted configuration
is refused rather than skipped or approximated.

## Test Procedure

Over one unchanged commit and clean tracked state in a worktree using the
literal `target-codex-backends` directory, inspect `Cargo.toml` for a
`[features]` table and `optional = true` dependencies, and inventory every
tracked Rust `cfg` attribute and `cfg!` macro. Confirm that the one declared
feature, `test-support`, is referenced by a tracked `cfg(feature = ...)`
condition and is selected by its own clippy, test and doc commands beside the
default lane's, and that a crate with no declared feature would run exactly one
lane. Confirm that the current `cfg(test)` sites are selected by `clippy --all-targets` and `test`, and
that the current `cfg(unix)` test is selected on the admitted local Unix target;
claim no unexecuted target. With `CARGO_BUILD_JOBS=1` under
`flock /tmp/quire-heavy-check.lock`, run `cargo fmt --all -- --check`,
`cargo clippy --locked --all-targets --target-dir target-codex-backends -- -D
warnings`, and `cargo test --locked --target-dir target-codex-backends`,
recording each exit status, commit, tracked state and covering risk.

Then exercise independent Rust policy fixtures for every negative control.
Remove each denied-warning channel in turn; omit `--all-targets`; request
`--features quire-extraction`; add a feature with no `cfg(feature = ...)` site;
add a real feature-selected site without the opposing feature lane; add a target
condition false on every admitted target; point a compiling command at a shared
target directory; change the commit or tracked-state digest between commands;
and reorder, omit or add an enumerated command. The fixtures invoke no subprocess
and parse no command output; the outer local gate supplies the real command exit
statuses for the unchanged candidate.

## Expected Results

All enumerated commands execute and pass over one unchanged clean worktree
state, each recorded with its covering risk, and the default and `test-support`
lanes are both built, linted, tested and documented.
All seven current conditional sites are selected without misclassifying target
kind or platform as a Cargo feature. Each negative policy fixture returns its
own refusal: missing warning channel, missing all-target coverage, undeclared or
phantom feature, missing opposing feature lane, unexecuted target condition,
shared target directory, changed head/state, or wrong command set/order. No
fixture makes an unsupported configuration pass by default and no subprocess
or rendered-output parser is introduced.
