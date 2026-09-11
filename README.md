# quire-protocol

This is the Rust consumer boundary for the accepted A-owned
`quire.compiled-protocol/1` contract. It does not parse native Quire source or
create another protocol representation. Call:

```rust
let report = quire_protocol::admit_and_link(bytes, &expected, limits);
```

`expected` is a `quire_protocol::Expected` built from independently selected
artifact, contract, baseline, producer, source, dependency, and admitted-model
inputs. It must be selected before inspecting `bytes`.

To generate the actual four-unit fixture from A's pinned source revision, run:

```console
cd /path/to/quire-spec-language
git checkout 84975dd687d38d9b24eae84b7356e3f24cdc7b42
CARGO_PROFILE_RELEASE_STRIP=symbols CARGO_BUILD_JOBS=1 \
  cargo run --locked --offline --release --example native_protocol_handoff -- \
  /tmp/quire-native-handoff
```

The producer writes `compiled-protocol.json`, original predicate/state/temporal/
workflow sources, `model-source.json`, and selected dependency bytes. Its
`expected.json` is an inspection aid only; it does not authorize a consumer
selection.

`quire-protocol` is the private, first-party Rust implementation home for the
selected native Quire finite protocol slice. It will own strict protocol
representation, compensation assessment, finite global replay/conformance, the
canonical protocol result, and the source-bound projection of that result into
`quire-verification/e02-draft-2`.

## Status

Repository bootstrap, QUOIN specification review, and implementation planning
are complete. There is no library implementation, released artifact, or
qualification claim yet. Implementation uses `AGPL-3.0-or-later` and is gated
on an accepted composed Quire baseline that freezes the reviewed
protocol/result rule bytes and on the native compiler's accepted
versioned compiled-protocol artifact contract.

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
