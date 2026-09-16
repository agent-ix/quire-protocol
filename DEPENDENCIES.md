# Dependency and rights record

This record states what the build actually consumes. Every revision and license
below was read out of the pinned revision's own `Cargo.toml`, not out of a
release note, and every revision matches `Cargo.toml` and `Cargo.lock` in this
repository. `NFR-006-fix-the-consumed-dependency-set` states the obligation this
record discharges.

## Consumed revisions

| Crate | Pinned revision | Declared license | Role |
| --- | --- | --- | --- |
| `quire-spec-language` | `0c2f13e261c0ffeaf89f0638931b573ec038e99b` | `AGPL-3.0-or-later` | Supplies the accepted `quire.compiled-protocol/1`, `/2`, and strict `/3` control-to-temporal mapping readers, committed owner-addressed handoffs, owner-exported constants, the FR-051 checked temporal activation-guard selection, the FR-052 temporal owner authority, FR-053 opaque native-temporal v2 trigger identity, and the shared exact bounded `state::evaluate_v2` compensation-expression evaluator, including immutable authored-initializer resolution and exact compensation forward-effect admission for capture sources. The `/1` handoff retains distinct Service/Provider model authorities. Release `v0.2.0` is its ancestor at `6eccb05`. |
| `quire-observation` | `581d98f1ac9f1467cc5355785d67b242f8d5d150` | `AGPL-3.0-or-later` | Direct observation authority dependency used by the current compensation, conformance and result kernels. This accepted C00 revision additionally publishes the partial-fact, activation-scope-authority, authority-bundle v1/v2 and bounded-lineage contracts/schema bytes Task-020 recognizes. Protocol imports owner constants and bytes rather than mirroring their wire vocabulary; repair and aggregate-query remain explicit unsupported consumer capabilities. |
| `agent-ix-baseline-producer` | `404288282402d60de007295ccbafa960532b955e` | `AGPL-3.0-only` | Transitive baseline producer reached through `quire-spec-language`; sourced from `agent-ix/filament-core-data`. |
| `quire-contract-ir` | `53cc03c639e2e26528132d34d96dc56449df78e8` | `MIT OR Apache-2.0` | Direct test type and the exact transitive contract-IR revision selected by `quire-spec-language` `0c2f13e`; one identity type is present in the resolved graph. |
| `ix-trace-rs` | `2ce4ebf47f726b9d76388220545cd0abda8a5cfb` | `AGPL-3.0-or-later` | Test-only trace attributes binding tests to acceptance criteria. |

All five are compatible with this repository's `AGPL-3.0-or-later` decision:
four are AGPL, and the permissive `MIT OR Apache-2.0` grant may be consumed
under it. `publish = false` on the QObs crate is the required delivery policy,
not a compatibility or availability failure for an exact git dependency.

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
absent from this record. The row above records the revision the build actually
consumes; it does not prematurely claim the later accepted FR-287/FR-262 pin.

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
- It does not turn the producer's committed handoff into consumer-owned bytes.
  `quire-spec-language` `6c8c401` supplies both contracts and exports both paths
  within its own exactly pinned crate source. The version 1 and version 2 scenarios execute
  there under the default local gate; this repository vendors no copy.
