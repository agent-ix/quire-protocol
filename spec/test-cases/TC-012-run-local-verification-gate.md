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

Verify that the enumerated local gate commands execute over one unchanged head
in the single configuration lane this crate can express, that each denied-warning
channel is independently effective, and that an instruction naming an undeclared
feature lane is refused rather than skipped or satisfied by declaring a feature
with nothing behind it.

## Test Procedure

Over one unchanged head in a worktree with its own target directory, inspect
`Cargo.toml` for a `[features]` table and for `optional = true` dependencies, and
inspect every tracked Rust source for `cfg` attributes and `cfg!` macros. Confirm
`--no-default-features`, `--all-features` and the default selection resolve to the
same unit graph from Cargo metadata. With `CARGO_BUILD_JOBS=1` under
`flock /tmp/quire-heavy-check.lock`, run `cargo fmt --all -- --check`,
`cargo clippy --locked --all-targets -- -D warnings`, and `cargo test --locked`,
recording each exit status and the risk each command covers.

Then run the negative controls independently, restoring the head between each.
Introduce an unused import and run the lint command first with the crate `[lints]`
tables removed and then with `-D warnings` removed. Introduce dead code reachable
only from a test target. Invoke the lint command with
`--features quire-extraction`. Declare a `[features]` entry that no conditionally
compiled item references. Introduce one `cfg`-gated item and attempt promotion
with a single lane. Run a gate command with `CARGO_TARGET_DIR` pointing at a
directory shared with another worktree. Amend the head between two enumerated
commands and offer the combined result.

## Expected Results

All three enumerated commands execute and pass over the unchanged head, each
recorded with its covering risk, and the three feature selections are shown to be
the identical unit graph so exactly one lane is required. The unused import fails
the lint command under each denied-warning channel alone, and the test-target
dead code fails it because `--all-targets` is in force. The undeclared
`--features quire-extraction` invocation is reported as a gate definition defect;
it is neither skipped silently nor resolved by declaring the feature. The
feature entry with no conditionally compiled item behind it is refused. The
`cfg`-gated item is refused for promotion until a second lane covering the
opposing configuration is run. The shared `CARGO_TARGET_DIR` run is rejected as
evidence because it may be served stale from another worktree, and the result
spanning the amended head is rejected because no single unchanged head carries
it.
