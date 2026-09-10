# quire-protocol

`quire-protocol` is the private, first-party Rust implementation home for the
selected native Quire finite protocol slice. It will own strict protocol
representation, compensation assessment, finite global replay/conformance, the
canonical protocol result, and the source-bound projection of that result into
`quire-verification/e02-draft-2`.

## Status

Repository bootstrap and specification are in progress. There is no library
implementation, released artifact, or qualification claim yet. Implementation
is gated on an accepted composed Quire profile whose immutable candidate pins
the reviewed protocol/result rule bytes, completion of this repository's QUOIN
specification review and implementation plan, and an exact license decision.

## Boundary

This repository owns:

- strict consumption of one selected, versioned protocol representation;
- finite causal protocol and compensation state;
- bounded batch replay and semantically equivalent settled incremental assessment;
- canonical `quire.protocol.result/v1-draft.1` production and strict reading;
- the reviewed total projection into `quire-verification/e02-draft-2`.

This repository does not own:

- native Quire grammar, parsing, linking, or compiler diagnostics;
- temporal logic syntax or evaluation;
- observation collection, correlation authority, or telemetry retention;
- business-action or compensation execution;
- generic producer execution, evidence storage, verification-method catalogues,
  sufficiency decisions, or human disposition;
- general projection, realizability, refinement, or composition algorithms;
- public release or publication policy.

The normative language and protocol contracts remain in
`agent-ix/quire-specification`. `quire-verification` owns E02. Engineering
Assurance owns reusable bounded producer execution and typed transport. Quoin
owns evidence. This repository consumes those exact interfaces and does not copy
their semantics into local substitutes.

## Development controls

All first-party executable code and tests are Rust. Substantive changes follow
QUOIN `/specify` and self `/spec-review` before implementation. Tests use bare
compiler-checked `ix-trace-rs` bindings. The exact repository Rust review and
QUOIN semantic gap analysis are release gates. Local heavy Cargo work is
serialized with one build job while the shared workstation is loaded. Any
hosted workflow must expose only `workflow_dispatch`; no hosted CI is dispatched
from this repository bootstrap.

See [the specification](spec/spec.md), [contributor instructions](AGENTS.md),
[license decision](LICENSE-DECISION.md), and [artifact and rights policy](ARTIFACTS.md).

