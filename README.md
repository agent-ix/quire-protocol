# quire-protocol

[![Discord](https://img.shields.io/badge/Discord-Join%20us-5865F2?logo=discord&logoColor=white)](https://discord.gg/k8DVhuYBR2)

quire-protocol is a Rust crate that checks recorded traffic against a compiled
protocol. A protocol describes roles that exchange messages over channels,
together with the steps that undo earlier work when something fails.

The crate takes a protocol compiled by the Quire Specification Language (QSL)
and a finite record of observations. It reports whether the traffic followed the
protocol and whether the undo path is available, using a closed vocabulary: a
missing observation is "incomplete", a conversation that may still advance is
"pending", and only an adverse observation is a "violation". Layers above it
use these results and do not read protocol files.

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="docs/images/hero-dark.svg">
  <img alt="A compiled protocol is admitted and linked into roles and channels. Observed traffic is then checked against it for conformance, and compensation is assessed alongside. The result is a conformance verdict and a compensation path that is available or unavailable." src="docs/images/hero-light.svg">
</picture>

> **Status: prerelease, private repository.** The pieces below exist and are
> tested. They are not yet joined into one end-to-end pipeline, the crate is not
> published (`publish = false`), and no qualification claim exists. Expect
> breaking changes.

## Features

- **Reads compiled protocols strictly.** It accepts QSL's
  `quire.compiled-protocol/1` and `/2` handoffs through the exact QSL readers.
  You choose the version by choosing the module. The version-1 and version-2
  readers refuse each other's files and do not translate them.
- **Selects independently.** The caller names the artifact, its dependency
  closure, profile and version-lock entries. A producer's own statement of what
  it emitted is evidence only.
- **Links.** Admission and linking are separate steps. Linking builds this
  crate's own finite linked representation of the protocol, which is separate
  from QSL's, and reports problems with locations.
- **Assesses compensation.** It works out from the reported facts whether
  compensation is active, registered, ordered correctly, retried within its
  bound and recovered. Attempts and effects stay distinct, and no business
  action is executed.
- **Checks conformance.** A bounded batch replay and an incremental mode give
  each obligation its own disposition. The incremental mode gives the same
  answer as the batch once the record settles.
- **Produces a canonical result.** The result record is
  `quire.protocol.result/v1-draft.1`, with a strict reader, and it projects
  into the `quire-verification/e02-draft-2` evidence format.

## Outcomes

Each obligation in a protocol ends in exactly one disposition. A violation
names the adverse observation behind it.

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="docs/images/outcomes-dark.svg">
  <img alt="Each obligation in a protocol ends in one of five dispositions: conforms, pending, violates, incomplete or unavailable. A violation is always backed by an adverse observation of one kind: unexpected, missing, causally misordered, duplicate effect or cross-workflow." src="docs/images/outcomes-light.svg">
</picture>

The whole run gets one aggregate status: conforms, pending, violates or
unavailable. The per-obligation results remain visible beneath it. Missing or
duplicate premises are reported as their own defects and cannot change an
obligation's result. Compensation availability is reported separately, as
either available or unavailable with the list of causes. Causes include no
activation fact, an ordering violation, a retry bound exceeded, and a recovery
decision not yet reported.

Operations return one of three outcomes: completed, refused with a typed
refusal, or resource-incomplete when a declared bound (counts, allocation or
deadline) stopped the work. Exhausting a bound is not reported as a refusal or a
violation.

## Admitting a protocol

Admission takes the four files of a checked-package handoff directory (QSL
FR-517), the evidence assembled from that directory, and read limits. It reads
`checked-package.json` with Contract IR's `read_checked_package`, compares the
admitted package with `selection.json`, and reads `identities.json` and
`equality.json` against it. The case below is from
[`tests/tc_149_admit_checked_handoff.rs`](tests/tc_149_admit_checked_handoff.rs):

```rust
let admitted = admit(
    &HandoffFiles {
        checked_package: &files["checked-package.json"],
        selection: &files["selection.json"],
        identities: &files["identities.json"],
        equality: &files["equality.json"],
    },
    &evidence,
    CheckedPackageReadLimits::bounded(),
)?;
assert_eq!(admitted.package().name(), "r");
```

A refusal carries the owner's value unchanged: the reader's refusal code, loci
and cause, the reader's incomplete result when a read limit is exhausted, QSL's
selection axis, or QSL's identity-file or equality-file refusal.

## Linking a protocol

`link(&package, &clause)` returns a `LinkedProtocol`: the clause's roles,
channels, queues, controls, compensations, required temporal clauses,
`terminal`, `scheduling` and activation rows as node ids from QSL's
`protocol_view`, and the causal edges exactly as QSL's `causal_relation`
returns them. It builds no edge. An absent node, a non-protocol node or a
`protocol_reference` carrier returns QSL's `ViewRefusal`. The consumer-owned
refusals, the window binding and the FR-002 criteria over recipe R are owed
to QSL-1740.

The compensation assessment over the checked-package evaluation and the
temporal evidence route return in the next stage of the repoint (FR-045 to
FR-047).

## Modules

| Module | What it holds |
| --- | --- |
| `checked_package` | Admission of the checked-package handoff (FR-043). |
| `link` | The linked protocol over QSL's read view and causal relation (FR-044). |
| `version_lock` | Version-lock resolution and the exact-byte delivery guard. |
| `compensation` | Activation, registration, ordering and retry facts. |
| `closure`, `outcome` | The five-axis closure and the closed operation and claim outcomes. |
| `bounds` | Count, allocation and deadline guards. |
| `digest` | Canonical JCS encoding and the exact-byte SHA-256 carrier. |
| `refusal` | The typed refusal envelope and its catalog. |
| `repro`, `delivery_index` | Reproduction record and the evidence index for one pinned delivery. |

The crate declares no Cargo feature.

## Out of scope

QSL owns native grammar, parsing and compiler diagnostics. Other repositories
own temporal-logic syntax and evaluation, observation collection and
correlation, running compensation or business actions, evidence storage,
verification-method catalogs and sufficiency decisions, and release policy. The
normative contracts live in quire-specification (private), and
the intended architecture is in
[`spec/assurance/AD-001-protocol-library-boundaries.md`](spec/assurance/AD-001-protocol-library-boundaries.md).
Start from [`spec/spec.md`](spec/spec.md) for the requirements.

## Quire ecosystem

`quire-protocol` is one of the backends. It reads checked output from the
language layer and works over observations. It sits above QSL and below the
orchestration tools.

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="docs/images/ecosystem-dark.svg">
  <img alt="The Quire repositories. The main chain runs from the standard, quire-specification, to the compiler, quire-spec-language, then quire-contract-ir, quire-contract-codegen and quire-contract-runtime. quire-spec-language also feeds quire-protocol, and quire-contract-ir feeds quire-analyze. quire-observation feeds quire-protocol and quire-mltl. Above the chain, quire-driver runs it, quire-integration tests it and quire-verification plans verification. Below it, shared libraries used throughout: quire-exact, quire-semantic-value, quire-walk, quire-canonical and quire-verification-contracts." src="docs/images/ecosystem-light.svg">
</picture>

It depends on [quire-spec-language](https://github.com/agent-ix/quire-spec-language)
(the compiled-protocol readers and their `handoff-writer` test producer),
[quire-observation](https://github.com/agent-ix/quire-observation) (qualified
observation values),
[quire-verification-contracts](https://github.com/agent-ix/quire-verification-contracts)
(the evidence format it projects into) and
[engineering-assurance](https://github.com/agent-ix/engineering-assurance) (the
claim-strength vocabulary), all pinned by Git revision in `Cargo.toml`.
[quire-contract-ir](https://github.com/agent-ix/quire-contract-ir) is a
dev-dependency used only by tests.

## Building and testing

All first-party code and tests are Rust. The local gate is four commands run in
order over one unchanged head, defined by
[`spec/non-functional/NFR-004-define-local-verification-gate.md`](spec/non-functional/NFR-004-define-local-verification-gate.md)
and listed in [`AGENTS.md`](AGENTS.md): `cargo fmt --check`, clippy with
warnings denied, tests and rustdoc. Tests bind to requirements with
`ix-trace-rs` attributes. No hosted CI runs from this repository.

## Contributing

Read [`CONTRIBUTING.md`](CONTRIBUTING.md) first. Every contribution needs a
signed [CLA](CLA.md). Contributor rules are in [`AGENTS.md`](AGENTS.md), and the
artifact and rights policy is in [`ARTIFACTS.md`](ARTIFACTS.md).

## License

AGPL-3.0-or-later. See [`LICENSE`](LICENSE).
