# quire-protocol

[![Discord](https://img.shields.io/badge/Discord-Join%20us-5865F2?logo=discord&logoColor=white)](https://discord.gg/6qsdhSPE)

This is the Rust consumer and assessment boundary for the accepted A-owned
`quire.compiled-protocol/1` and `/2` contracts. It delegates byte admission to
the exact QSL readers, then owns a distinct identity-preserving linked
representation and the pure compensation, conformance, claim and result path.
Admission and linking are separate operations. The published root `/1`
`admit_and_link` remains a narrow compatibility facade; normal integrations
select versioned intake/model modules explicitly.

The pinned QSL dependency publishes default-running `/1` and `/2` handoffs and
their owner-defined selection records. A producer-emitted expectation is
evidence, not consumer authority: the caller independently selects the
artifact, dependency closure, profile and version-lock entries before Protocol
reads producer values. Task-009's `/1` compatibility corpus now runs from the
committed owner address with no environment selector.

`quire-protocol` is the private, first-party Rust implementation home for the
selected native Quire finite protocol slice. It will own strict protocol
representation, compensation assessment, finite global replay/conformance, the
canonical protocol result, and the source-bound projection of that result into
`quire-verification/e02-draft-2`.

## Status

Repository bootstrap and substantial subsystem kernels are complete. The crate
is still under development: those kernels are not yet one typed end-to-end
pipeline, and no qualification claim exists. The complete target architecture
is [AD-001](spec/assurance/AD-001-protocol-library-boundaries.md).

## Boundary

This repository owns:

- strict version-explicit consumption of the selected `/1` and `/2` artifacts;
- the exact linked representation binding `/2` requirements to QObs runtime values;
- finite causal protocol and compensation state;
- bounded batch replay and semantically equivalent settled incremental assessment;
- canonical `quire.protocol.result/v1-draft.1` production and strict reading;
- the reviewed total projection into `quire-verification/e02-draft-2`.

This repository does not own:

- native Quire grammar, source parsing, compiler linking, or compiler diagnostics;
- temporal logic syntax or evaluation;
- observation collection, correlation authority, or telemetry retention;
- business-action or compensation execution;
- generic producer execution, evidence storage, verification-method catalogues,
  sufficiency decisions, or human disposition;
- general projection, realizability, refinement, or composition algorithms;
- public release or publication policy.

The normative language and protocol contracts remain in
`agent-ix/quire-specification`. `quire-verification-contracts` owns the public
E02 wire types, strict reader, and RFC 8785 canonicalization this repository
projects into (PLAT-861; extracted from private `quire-verification`, which
still owns the planning and evidence-assessment strategy layer this repository
does not consume). Engineering Assurance owns reusable bounded producer
execution, typed transport and the claim-strength vocabulary (EA FR-022).
Quoin owns evidence. This repository consumes
those exact interfaces and does not copy their semantics into local
substitutes.

## Development controls

All first-party executable code and tests are Rust. Substantive changes follow
QUOIN `/specify` and self `/spec-review` before implementation. Tests use bare
compiler-checked `ix-trace-rs` bindings. The exact repository Rust review and
QUOIN semantic gap analysis are release gates. Local heavy Cargo work is
serialized with one build job while the shared workstation is loaded. Any
hosted workflow must expose only `workflow_dispatch`; no hosted CI is dispatched
from this repository bootstrap.

The local verification gate — which commands run, in which configurations, and
what risk each one detects — is defined by
[NFR-004](spec/non-functional/NFR-004-define-local-verification-gate.md) and
listed in [AGENTS.md](AGENTS.md). The crate has two Cargo-feature lanes: the
default lane, and the `test-support` lane, which compiles
`result::test_support` — validated-result constructors that dependents enable on
a `[dev-dependencies]` entry for their own test suites. The default build
exposes none of it. That is separate from the test-only conditional sites and
the Unix-only v2 handoff test selected on the admitted
`x86_64-unknown-linux-gnu` target. Every
compiling gate command uses the worktree-local `target-codex-backends`
directory.

See [the specification](spec/spec.md), [contributor instructions](AGENTS.md),
[license decision](LICENSE-DECISION.md), and [artifact and rights policy](ARTIFACTS.md).
