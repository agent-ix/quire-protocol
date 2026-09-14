# Dependency and rights record

This record states what the build actually consumes. Every revision and license
below was read out of the pinned revision's own `Cargo.toml`, not out of a
release note, and every revision matches `Cargo.toml` and `Cargo.lock` in this
repository. `NFR-006-fix-the-consumed-dependency-set` states the obligation this
record discharges.

## Consumed revisions

| Crate | Pinned revision | Declared license | Role |
| --- | --- | --- | --- |
| `quire-spec-language` | `f1700a9264d6d3bcdd07e0f77b70f3dae9ed4c07` | `AGPL-3.0-only` | Supplies the accepted `quire.compiled-protocol/1` and `/2` readers, handoff records, owner-exported constants, the FR-051 checked temporal activation-guard selection, and the source-compatible FR-052 temporal owner authority. Release `v0.2.0` is its ancestor at `6eccb05`. |
| `agent-ix-baseline-producer` | `404288282402d60de007295ccbafa960532b955e` | `AGPL-3.0-only` | Transitive baseline producer reached through `quire-spec-language`; sourced from `agent-ix/filament-core-data`. |
| `quire-contract-ir` | `53cc03c639e2e26528132d34d96dc56449df78e8` | `MIT OR Apache-2.0` | Direct test type and the exact transitive contract-IR revision selected by `quire-spec-language` `f1700a9`; one identity type is present in the resolved graph. |
| `ix-trace-rs` | `2ce4ebf47f726b9d76388220545cd0abda8a5cfb` | `AGPL-3.0-or-later` | Test-only trace attributes binding tests to acceptance criteria. |

All four are compatible with this repository's `AGPL-3.0-or-later` decision:
three are AGPL, and the permissive `MIT OR Apache-2.0` grant may be consumed
under it.

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
  `quire-spec-language` `f1700a9` supplies both contracts and exports the path
  within its own exactly pinned crate source. The version 2 scenarios execute
  there under the default local gate; this repository vendors no copy.
