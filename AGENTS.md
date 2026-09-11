# Contributor instructions

Read `spec/spec.md`, `LICENSE-DECISION.md`, and `ARTIFACTS.md` before changing
this repository.

- First-party executable code, generators, validators, test assertions, fixture
  audits, and command-line tooling must be Rust. Do not add JavaScript, TypeScript,
  Python, or shell semantic logic.
- Run QUOIN `/specify`, then self `/spec-review`, and resolve its findings before
  implementing a substantive behavior or interface change. Use QUOIN
  `/spec-to-plan` before implementation begins.
- Treat `agent-ix/quire-specification` as the formal-clause source. Consume one
  promoted immutable profile and exact rule digests; do not rewrite protocol or
  temporal semantics locally.
- Reuse the accepted Engineering Assurance Rust producer boundary. Do not add a
  local subprocess runner, stdout verdict parser, evidence store, verification
  catalogue, or human-disposition path.
- Bind automated acceptance tests with bare `ix_trace_rs::trace` attributes.
- Before promotion, run `/home/peter/dev/agent-skills/rust-review/SKILL.md` and
  QUOIN semantic `/gap-analysis`, then remediate every in-scope finding.
- Run local checks only. Set `CARGO_BUILD_JOBS=1` and use
  `flock /tmp/quire-heavy-check.lock` for heavy Cargo work. Do not dispatch hosted CI.
- The local gate is defined by [NFR-004](spec/non-functional/NFR-004-define-local-verification-gate.md).
  It is exactly these three commands, in this order, over one unchanged head:

  ```console
  CARGO_BUILD_JOBS=1 flock /tmp/quire-heavy-check.lock cargo fmt --all -- --check
  CARGO_BUILD_JOBS=1 flock /tmp/quire-heavy-check.lock cargo clippy --locked --all-targets -- -D warnings
  CARGO_BUILD_JOBS=1 flock /tmp/quire-heavy-check.lock cargo test --locked
  ```

  This crate is **single-lane**. It declares no `[features]` table and contains no
  `cfg` attribute, so `--no-default-features` and `--all-features` resolve to a
  byte-identical unit graph; running both covers no extra configuration. Do not
  pass `--features quire-extraction` here — that feature belongs to
  `quire-spec-language`, and this package does not declare it. If you are told to
  run a second feature lane, that instruction is a gate defect to report, not a
  lane to satisfy, and it is never fixed by declaring a feature with no `cfg`'d
  code behind it. Introducing any `cfg` attribute, `cfg!` macro, `optional = true`
  dependency, or `[features]` entry makes a second lane mandatory under
  NFR-004-AC-7.
- Run each gate command with the worktree's own target directory. A
  `CARGO_TARGET_DIR` shared across worktrees can serve a stale result and is not
  accepted as gate evidence.
- Every GitHub Actions workflow must use `workflow_dispatch` as its only trigger.
- License new implementation source and tests as `AGPL-3.0-or-later`; preserve
  all imported grants and complete dependency compatibility review before use.
- Preserve source, fixture, generated-artifact, and dependency rights and provenance.
