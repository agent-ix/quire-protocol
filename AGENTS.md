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
- Every GitHub Actions workflow must use `workflow_dispatch` as its only trigger.
- Do not add implementation code until an exact outbound SPDX license is accepted.
- Preserve source, fixture, generated-artifact, and dependency rights and provenance.

