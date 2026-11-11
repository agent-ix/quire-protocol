# quire-protocol

[![Discord](https://img.shields.io/badge/Discord-Join%20us-5865F2?logo=discord&logoColor=white)](https://discord.gg/k8DVhuYBR2)
[![Quire](https://img.shields.io/badge/Quire-Specification%20Language-1F6FEB)](https://github.com/agent-ix/quire-spec-language)

quire-protocol is a Rust crate that consumes the checked package the Quire
Specification Language (QSL) produces for a protocol. A protocol describes
roles that exchange messages over channels, together with the steps that undo
earlier work when something fails.

The crate admits a checked-package handoff, links one protocol clause into its
roles, channels, queues and controls, evaluates protocol slots and temporal
evidence through QSL's evaluation entries, and keeps the compensation state
machine. Every refusal carries the value of the component that produced it, and
resource bounds are declared up front.

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="docs/images/hero-dark.svg">
  <img alt="A checked-package handoff of four files is admitted into one checked package. The package is then linked into a protocol with its roles and channels, its slots are evaluated to a slot outcome, and its temporal obligations are assessed to an evidence record." src="docs/images/hero-light.svg">
</picture>

## Getting started

quire-protocol is not published to a registry. Add it as a git dependency:

```toml
[dependencies]
quire-protocol = { git = "https://github.com/agent-ix/quire-protocol", branch = "main" }
```

To build from a clone, install the Rust toolchain named in `rust-toolchain.toml`
and run `cargo build`.

## Usage

`admit` reads the four files of a checked-package handoff directory:
`checked-package.json`, `selection.json`, `identities.json` and `equality.json`.
It reads the package with Contract IR's reader, compares it with the selection,
and reads the identity and equality files against it. `link` then builds the
linked protocol for one protocol clause of the admitted package.

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
    &mut join_budget(),
)?;
let linked = link(admitted.package(), &clause)?;
```

`evidence` is the evidence assembled from the handoff's files, `join_budget()`
is the caller's native-join allowance for the pass, and `clause` is the node id
of a protocol clause in the package.

## Outcomes

A slot evaluation returns a `SlotOutcome` that carries QSL's value unchanged.
Admission returns a `Refusal` with one variant per file or check that failed.
In addition to the package and sidecar results shown below,
`Refusal::CatalogUnavailable` carries Contract IR's `CatalogReadError` unchanged
when the reader cannot read its operation catalog. That result concerns the
reader's catalog, not the offered package, and stops admission before sidecar
checks.

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="docs/images/outcomes-dark.svg">
  <img alt="A slot evaluation ends in one of eight outcomes: holds, does not hold, not Boolean, unavailable, undefined, refused, incomplete or a family result. Admission of a handoff fails with one of six refusals: reader, incomplete, selection file, selection, identities or equality, each carrying its owner's value unchanged." src="docs/images/outcomes-light.svg">
</picture>

## Reference

| Module | What it holds |
| --- | --- |
| `checked_package` | Admission of the checked-package handoff. |
| `link` | The linked protocol over QSL's read view and causal relation. |
| `slot` | The outcome of evaluating a protocol slot. |
| `evidence` | Temporal evidence read from the checked package's own nodes. |
| `compensation` | Activation, registration, ordering and retry facts. |
| `closure`, `outcome` | The closure axes and the closed operation and claim outcomes. |
| `bounds` | Count, allocation and deadline guards. |
| `digest` | The exact-byte SHA-256 carrier. |
| `refusal` | The typed refusal envelope and its catalog. |
| `repro`, `delivery_index` | The reproduction record and the evidence index for one pinned delivery. |

The crate declares no Cargo feature.

## Quire ecosystem

quire-protocol sits beside the compiler. It reads QSL's checked output and
works over observations.

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="docs/images/ecosystem-dark.svg">
  <img alt="The Quire repositories. The main chain runs from the standard, quire-specification, to the compiler, quire-spec-language, then quire-contract-ir, quire-contract-codegen and quire-contract-runtime. quire-spec-language also feeds quire-protocol, and quire-contract-ir feeds quire-analyze. quire-observation feeds quire-protocol and quire-mltl. Above the chain, quire-driver runs it, quire-integration tests it and quire-verification plans verification. Below it, shared libraries used throughout: quire-eval, quire-exact, quire-semantic-value, quire-walk, quire-canonical and quire-verification-contracts." src="docs/images/ecosystem-light.svg">
</picture>

It depends on [quire-spec-language](https://github.com/agent-ix/quire-spec-language),
[quire-eval](https://github.com/agent-ix/quire-eval),
[quire-observation](https://github.com/agent-ix/quire-observation),
[quire-verification-contracts](https://github.com/agent-ix/quire-verification-contracts)
and [engineering-assurance](https://github.com/agent-ix/engineering-assurance).
[quire-contract-ir](https://github.com/agent-ix/quire-contract-ir) supplies the
checked-package reader. The requirements are in [`spec/spec.md`](spec/spec.md).

## Building and testing

```
cargo fmt --all -- --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked
RUSTDOCFLAGS="-D warnings" cargo doc --locked --no-deps
```

## Contributing

Read [`CONTRIBUTING.md`](CONTRIBUTING.md) first. Every contribution needs a
signed [CLA](CLA.md). The artifact and rights policy is in
[`ARTIFACTS.md`](ARTIFACTS.md).

## License

AGPL-3.0-or-later. See [`LICENSE`](LICENSE).
