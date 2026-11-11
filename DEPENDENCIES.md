# Dependency and rights record

This record states what the build consumes and under which license. Every
first-party dependency follows `branch = "main"` in `Cargo.toml`; `Cargo.lock`
holds the commit Cargo selected and is the one place a revision is recorded.
`NFR-006-fix-the-consumed-dependency-set` states the obligation this record
discharges.

## Consumed crates

| Crate | Source | Declared license | Role |
| --- | --- | --- | --- |
| `quire-spec-language` | `agent-ix/quire-spec-language`, `main`, feature `checked-handoff` | `AGPL-3.0-or-later` | Publishes `checked_handoff::Selection` (QSL FR-517), compared with an admitted package, and the in-process handoff writer the tests call. |
| `qsl-package` | `agent-ix/quire-spec-language`, `main` | `AGPL-3.0-or-later` | Publishes the bytes-only identity and equality file readers (QSL FR-531, FR-533), `protocol_view` and `causal_relation` (QSL FR-519), and `BarePackage` (QSL FR-531). |
| `qsl-eval` | `agent-ix/quire-spec-language`, `main` | `AGPL-3.0-or-later` | Owns the slot evaluation `CheckedPackageEvaluation` (QSL FR-390, FR-520, FR-533) and the temporal `EvidencePackage` (QSL FR-532). |
| `qsl-semantics` | `agent-ix/quire-spec-language`, `main` | `AGPL-3.0-or-later` | The channel overflow, loss selector and outstanding vocabulary the linked declarations retain (FR-051, FR-052); the wire graph that keys protocol nodes in tests. |
| `quire-eval`, `quire-exact`, `quire-semantic-value` | `agent-ix/quire-eval`, `agent-ix/quire-exact`, `agent-ix/quire-semantic-value`, `main` | `AGPL-3.0-or-later` | The evaluation outcome, meter and value types that the slot and evidence calls take and return. |
| `qsl-foundation` | `agent-ix/quire-spec-language`, `main` | `AGPL-3.0-or-later` | QSL's foundation crate. |
| `quire-contract-model` | `agent-ix/quire-contract-ir`, `main` | `MIT OR Apache-2.0` | Owns `read_checked_package`, the admitted `CheckedPackageV2` and the refusal, incomplete-read and read-limit types. The one reader of the checked-package format. |
| `quire-observation` | `agent-ix/quire-observation`, `main` | `AGPL-3.0-or-later` | Observation authority views used by the compensation kernel. |
| `quire-verification-contracts` | `agent-ix/quire-verification-contracts`, `main` | `AGPL-3.0-or-later` | E02 owner types. `build.rs` reads the commit Cargo selected from `Cargo.lock`. |
| `engineering-assurance` | `agent-ix/engineering-assurance`, `main`, `=0.7.1`, feature `claim-strength` only | `AGPL-3.0-or-later` | Owner of the claim-strength vocabulary (EA FR-022). This crate declares no strength enum of its own. |
| `sha2` | crates.io `=0.10.9` | `MIT OR Apache-2.0` | Carrier of the exact-byte SHA-256 of the FR-009 and FR-015 content digests. It supplies no schema, parser or authority. |
| `serde`, `serde_json`, `jsonschema` | crates.io | `MIT OR Apache-2.0` | Serialisation and closed-schema validation carriers. |
| `ix-trace-rs` | `agent-ix/ix-trace-rs`, `main` | `AGPL-3.0-or-later` | Test-only trace attributes binding tests to acceptance criteria. |
| `tempfile` | crates.io | `MIT OR Apache-2.0` | Test-only temporary directories for the handoff the tests write. |

The permissive `MIT OR Apache-2.0` grants may be consumed under this
repository's `AGPL-3.0-or-later` decision.

This crate adds no copied parser, encoder, formal semantics or reader. The
recipe inputs under `tests/fixtures/checked-handoff/` are copies of QSL's
`examples/checked-handoff/` files, which QSL does not publish through its
crate; they are test inputs only.
