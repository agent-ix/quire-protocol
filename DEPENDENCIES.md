# Dependency and rights record

This record states what the build actually consumes. Every revision and license
below was read out of the pinned revision's own `Cargo.toml`, not out of a
release note, and every revision matches `Cargo.toml` and `Cargo.lock` in this
repository. `NFR-006-fix-the-consumed-dependency-set` states the obligation this
record discharges.

## Consumed revisions

| Crate | Pinned revision | Declared license | Role |
| --- | --- | --- | --- |
| `quire-spec-language` | `04254a31071cc2d4c97e5c0707e1414141e55d89` | `AGPL-3.0-only` | Supplies the accepted `quire.compiled-protocol/1` and `/2` wire contracts, their public Rust readers, and the published `protocol_artifact::handoff` record shapes. |
| `agent-ix-baseline-producer` | `f79e5a641b3eac702642282ad6cc84dd48d280dc` | `AGPL-3.0-only` | Transitive baseline producer reached through `quire-spec-language`; sourced from `agent-ix/filament-core-data`. |
| `quire-contract-ir` | `690bde7f2dc58662cf9ff0595c2c0e3b17107c6f` | `MIT OR Apache-2.0` | Transitive contract-IR types reached through `quire-spec-language`. |
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
- It does not assert that a version 2 scenario executed. `quire-spec-language`
  `04254a3` supplies both the `/1` and `/2` contracts and `admit_and_link_v2` is
  written against `/2`, but A's handoff bundle is not retained in this
  repository, so the scenarios that submit to A are reported unexecuted under
  the declared gate rather than passing. `spec/tests.md` records that state.
