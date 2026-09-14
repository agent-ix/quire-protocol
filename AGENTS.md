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
- **Find the owner before you declare a type.** This repository is a consumer in
  a multi-repository ecosystem, and most types it handles are owned elsewhere.
  Before adding any type, enum member, key, wire string or file schema, look up
  its owner in the spec — the artifacts and their `ix://` references exist to
  answer exactly that. If an owner publishes it, import it. If the spec shows no
  owner, stop and ask; do not supply it locally and do not record the divergence
  as though recording it settled anything. See `CLAUDE.md`.
- **Never mirror another repository's type as a placeholder**, including while
  that repository's work is still in progress. Waiting is correct; a local
  stand-in is not, because every consumer of the stand-in has to be rewritten
  once the real type lands.
- **`tests/` is held to this identically.** Test-support code is where local
  shadow vocabularies actually accumulate in this repository, and a shadow enum
  under `tests/support/` makes assertions that cannot fail — it compares the
  local labels to each other instead of to the owner's values.
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
  CARGO_BUILD_JOBS=1 flock /tmp/quire-heavy-check.lock cargo clippy --locked --all-targets --target-dir target-codex-backends -- -D warnings
  CARGO_BUILD_JOBS=1 flock /tmp/quire-heavy-check.lock cargo test --locked --target-dir target-codex-backends
  ```

  The formatting command detects formatting drift. The Clippy command detects
  lint violations plus dead code and unused imports in library, test, fixture and
  example targets. The test command detects behavioural regressions against the
  bound acceptance obligations. A command without a named risk is not part of
  the gate.

  This crate has one **Cargo-feature lane**. It declares no `[features]` table,
  optional dependency or `cfg(feature = ...)` condition, so
  `--no-default-features`, `--all-features` and the default selection resolve to
  the same unit graph; running all three covers no extra Cargo-feature
  configuration. Separately, the current Unix host selects three tracked
  `#[cfg(test)]` sites through `clippy --all-targets` and `test`, and one tracked
  `#[cfg(unix)]` integration test through the admitted
  `x86_64-unknown-linux-gnu` target. This is not a claim about another target.
  Do not
  pass `--features quire-extraction` here — that feature belongs to
  `quire-spec-language`, and this package does not declare it. If you are told to
  run a second feature lane, that instruction is a gate defect to report, not a
  lane to satisfy, and it is never fixed by declaring a feature with no `cfg`'d
  code behind it. Introducing a `cfg(feature = ...)` condition, an
  `optional = true` dependency, or a `[features]` entry makes opposing Cargo
  feature lanes mandatory under NFR-004-AC-7. Any other new condition must name
  the command or admitted target that selects it.
- Run each gate command with the worktree's own target directory. A
  literal `target-codex-backends` directory is required for compiling commands;
  a shared `CARGO_TARGET_DIR` or another worktree's path can serve a stale result
  and is not accepted as gate evidence.
- Every GitHub Actions workflow must use `workflow_dispatch` as its only trigger.
- License new implementation source and tests as `AGPL-3.0-or-later`; preserve
  all imported grants and complete dependency compatibility review before use.
- Preserve source, fixture, generated-artifact, and dependency rights and provenance.
