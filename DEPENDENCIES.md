# Dependency and rights record

This record states what the build actually consumes. Every revision and license
below was read out of the pinned revision's own `Cargo.toml`, not out of a
release note, and every revision matches `Cargo.toml` and `Cargo.lock` in this
repository. `NFR-006-fix-the-consumed-dependency-set` states the obligation this
record discharges.

## Consumed revisions

| Crate | Pinned revision | Declared license | Role |
| --- | --- | --- | --- |
| `quire-spec-language`, `qsl-attrs`, `qsl-cst`, `qsl-forms`, `qsl-package`, `qsl-semantics`, `quire-exact` | `51245876e0d406aeed3047c608ad478b53b19205` | `AGPL-3.0-or-later` | Supplies the accepted `quire.compiled-protocol/1`, `/2`, and strict `/3` control-to-temporal mapping readers, the `native_protocol_handoff`/`native_protocol_v2_handoff` producer examples the IT-001 lanes run for complete handoffs, owner-exported constants, the FR-051 checked temporal activation-guard selection, the FR-052 temporal owner authority, FR-053 opaque native-temporal v2 trigger identity and exact borrowed temporal-axis/completeness evidence views, and the shared exact bounded `state::evaluate_v2` compensation-expression evaluator, including immutable authored-initializer resolution and exact compensation forward-effect admission for capture sources. The `/1` handoff retains distinct Service/Provider model authorities. Release `v0.2.0` is its ancestor at `6eccb05`. |
| `qsl-foundation` | `51245876e0d406aeed3047c608ad478b53b19205` | `AGPL-3.0-or-later` | A's foundation crate (ADR-011 layer F), from the same repository and revision as `quire-spec-language`. Owns `ByteDigest`, `Source` and `SourceIdentity`, which A's reader returns; the two must resolve one commit for those types to unify. |
| `quire-observation` | `2bdeb833a330bfa777c19eb4c28c423f856f3ba6` | `AGPL-3.0-or-later` | Direct observation authority dependency used by the current compensation, conformance and result kernels. The contracts it consumes were accepted at owner PR #27 merge `9240063` (`QOBS_REVISION`, informational); this pin publishes the partial-fact, activation-scope-authority, authority-bundle v1/v2 and bounded-lineage contracts/schema bytes Task-020 recognizes. Protocol imports owner constants and bytes rather than mirroring their wire vocabulary; repair and aggregate-query remain explicit unsupported consumer capabilities. |
| `quire-verification-contracts` | `da8d7272051b1cf02768ec459539551414fd6c21` | `AGPL-3.0-or-later` | Direct E02 owner dependency (PLAT-861). This exact accepted revision supplies the generated `quire-verification/e02-draft-2` types, strict reader and RFC 8785/JCS operations used by Task-007 without a local target-contract mirror. Extracted from private `quire-verification`, which previously supplied this interface directly and is no longer in this build's dependency graph. |
| `engineering-assurance` | `712f4ff76c2b1e01d26d10efd97a0eef0235defa` | `AGPL-3.0-or-later` | Direct owner of the claim-strength vocabulary (EA FR-022, TL-206). Consumed with default features off and only the serde-only `claim-strength` feature, which adds no dependency beyond `serde`. Protocol imports `ClaimStrength` and declares no strength enum of its own. |
| `sha2` | `0.10.9` | `MIT OR Apache-2.0` | Direct digest carrier used to stream bounded producer-value request identities without allocating a caller-sized preimage. It supplies no schema, parser, semantics, or producer authority. |
| `agent-ix-baseline-producer` | `a6fe662542770e6acbec4917cfe1772b349ee414` | `AGPL-3.0-only` | Test-only FCD static-producer owner that admits the shared QObs fixture bytes; sourced from `agent-ix/filament-core-data`. |
| `agent-ix-extraction-frontend`, `agent-ix-semantic-ir` | `1572ba4ba8fad0e94e9a0b19a569f82a5181b488` | `AGPL-3.0-or-later` | Transitive crates from `agent-ix/filament-core-data`, reached through `quire-spec-language` `5124587`. B does not call them. |
| `quire-canonical` | `0143a2b76be9a51941e95ee2613f2d5608d756fe` | `AGPL-3.0-or-later` | Transitive, reached through `quire-spec-language` `5124587`. |
| `quire-rs` | `2823a93bd3797c970c1d389afae9b893dea6f232` | `AGPL-3.0-or-later` | Transitive, reached through `quire-spec-language` `5124587`. |
| `quire-verification-contracts` (second revision) | `2e06cf5b959bc325a0231329a0e3a223ea2c2606` | `AGPL-3.0-or-later` | Transitive, reached through the `quire-contract-model` `dc03aa4` that `quire-spec-language` `5124587` pins. B's own E02 types come from the direct `da8d727` row above. |
| `quire-contract-ir` | `dc03aa4ecfbce9958efe6d766497d44c44e1200a` | `MIT OR Apache-2.0` | Direct test type and the exact transitive contract-IR revision selected by `quire-spec-language` `5124587`; one identity type is present in the resolved graph. |
| `ix-trace-rs` | `2ce4ebf47f726b9d76388220545cd0abda8a5cfb` | `AGPL-3.0-or-later` | Test-only trace attributes binding tests to acceptance criteria. |

All twenty crates in the table (thirteen rows; two rows name several crates)
are compatible with this repository's `AGPL-3.0-or-later` decision: eighteen
are AGPL (seventeen `AGPL-3.0-or-later`, one `AGPL-3.0-only`), and the two permissive `MIT OR Apache-2.0` grants may be
consumed under it. `publish =
false` on the QObs crate is the required delivery policy, not a compatibility
or availability failure for an exact git dependency.

This crate adds no copied parser, encoder, formal semantics, or fixture corpus.

## Correction: this record previously named a superseded revision

An earlier revision of this file recorded `quire-spec-language` at
`84975dd687d38d9b24eae84b7356e3f24cdc7b42`, described as "A's merged four-unit
native producer handoff, PR #58", while `Cargo.toml` and `Cargo.lock` both
pinned `5690c2b`. The two were not alternatives: `git merge-base --is-ancestor`
confirms `84975dd` is an ancestor of `5690c2b` ("Record numeric producer handoff
review", 2026-09-11), so this record named a superseded commit and the build
consumed a later one. The build was correct and the prose was stale. It is
corrected here rather than rewritten silently, because a rights record that has
ever misstated the consumed revision should say so.

`quire-contract-ir` was pinned in `Cargo.toml` but absent from this record
entirely. That is the closure gap `NFR-006` names: `FR-009` pins the producers
and nothing pinned this repository's own consumed set.

`quire-observation` was also a direct `Cargo.toml`/lockfile dependency while
absent from this record. The row above now records the accepted FR-287/FR-262
revision the build actually consumes.

## Correction: the record trailed the version 2 pin, and omitted a transitive

This record named `5690c2b` while `Cargo.toml` and `Cargo.lock` pinned
`04254a3`, A's promoted `main` carrying the `/2` contract. It also omitted
`agent-ix-baseline-producer`, which `quire-spec-language` `04254a3` pins from
`agent-ix/filament-core-data` and which the lockfile therefore resolves into
this build. Both are corrected above: the same closure gap `NFR-006` names,
recurring on the transitive edge rather than the direct one.

## What this record does not establish

- It asserts no immutability of any pinned revision. `FR-009` takes an
  owner-supplied assertion for that; `AD-001` and `NFR-003` forbid this
  repository from reaching a network or a version-control system to check it.
- It does not turn the producer's handoff into consumer-owned bytes.
  `quire-spec-language` `5124587` supplies both contracts and the
  `native_protocol_handoff`/`native_protocol_v2_handoff` producer examples. The
  version 1 and version 2 scenarios run those examples from the exactly pinned
  crate source into the build's target directory under the default local gate;
  this repository vendors no copy.
