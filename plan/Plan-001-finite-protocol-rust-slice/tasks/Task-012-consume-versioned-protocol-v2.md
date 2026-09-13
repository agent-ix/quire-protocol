---
id: Task-012
title: "Consume the version-explicit compiled-protocol /2 artifact"
type: Task
status: in_progress
track: B
priority: P0
relationships:
  - { target: ix://agent-ix/quire-protocol/Task-009, type: depends_on }
  - { target: ix://agent-ix/quire-protocol/FR-001, type: references }
  - { target: ix://agent-ix/quire-protocol/FR-002, type: references }
  - { target: ix://agent-ix/quire-protocol/NFR-002, type: references }
  - { target: ix://agent-ix/quire-protocol/IT-001, type: verifies }
---
# Task-012: Consume the Version-Explicit Compiled-Protocol /2 Artifact

## Scope

Add a second, version-explicit consumer entry point for the `/2`
compiled-protocol contract, verifying IT-001 SC-06 through SC-09. Task-009 owns
`/1` and SC-01 through SC-05; this task changes neither. `/1` and `/2` are
separate contracts: no auto-detection, upgrade, downgrade, repair, or
substitution between them, and adding `/2` must leave `admit_and_link`
byte-for-byte unchanged in behaviour.

## Dependency state

**Delivered.** A's `/2` is published and promoted. `quire-spec-language`
`04254a31071cc2d4c97e5c0707e1414141e55d89` is that repository's `main` after
PR #75 merged, released as `v0.1.0`, and it is what `Cargo.toml` and
`Cargo.lock` pin. The surface B builds against:

- `protocol_artifact::v2::{Expected, ExpectedDefinition, ExpectedTemporal,
  AdmittedPackage, read, encode_candidate}`
- `protocol_artifact::v2::wire::{Package, TemporalBinding, ClockConfiguration}`
- `protocol_artifact::native::{TemporalSelection, AdmissionV2, admit_v2}`
- `temporal::{evaluate_v2, evaluate_with_progress_v2, mapping_support_v2}`
- `protocol_artifact::handoff::{SelectionV2, MutationManifest, ...}` — the
  interchange record shapes for `expected-v2.json` and
  `mutations/manifest.json`, decoded rather than mirrored
- `protocol_artifact::Error::code` — the stable refusal identity per axis
- the existing shared `protocol_artifact::{Limits, Report, Invalid, Unsupported}`

`/1` stays strict and byte-compatible; `/1` and `/2` mutually refuse.

The handoff bundle A committed under `artifacts/compiled-protocol-v2/` is 109
files plus `SHA256SUMS`, carrying 28 mutation cases over 21 axes in format
`quire.protocol.v2-mutations/1`.

**What is still owed** is addressing, not content: A publishes no path constant
naming that directory inside its own crate source, so B can only reach it
through the out-of-band `QUIRE_PROTOCOL_A_V2_HANDOFF` variable and the
submitting tests stay `#[ignore]`d under the declared gate. Stated once in
`tests/support/v2_handoff/mod.rs::A_OWED_API`. Vendoring the bundle here is
refused: it would mirror what its owner publishes.

## Subtasks

### B-owned, available now

- [x] Author the B-owned `/2` evidence loader and absence reporting
  (`tests/support/v2/mod.rs`), with no A type named and the offered bytes carried
  through as opaque bytes.
- [x] Enumerate the scenario corpus SC-06 through SC-09 owe: one valid
  admission, both cross-version submissions, every changed-selection axis, every
  ordered digest-domain substitution, every recanonicalized producer input,
  every temporal-binding mutation, and per-limit exact/one-short/fresh-budget
  probes.
- [x] Make the blocked state visible to the declared gate rather than hiding it
  behind `#[ignore]`, and keep every B-side self-test off IT-001's trace tags.
- [x] Reconcile the corpus enumerations with IT-001's prose: `SelectionAxis`
  now carries the configuration and result axes, and the wire/version, seal,
  feature, referenced-binding, numeric-representation and numeric-domain axes
  IT-001 declares for both lanes have `/2` rows as `OfferedAxis`.
- [x] Give each refusal scenario an expected refusal identity rather than a
  two-valued admits/refuses oracle. `ExpectedRefusal` carries a distinct B-owned
  identity per refusing row; mapping it onto A's typed refusal enum is part of
  the pin step and must stay injective.
- [x] Make SC-06's "`admit_and_link` unchanged" clause falsifiable. It is now
  IT-001-SC-06a: re-run SC-01..SC-05 on the same `/1` fixture across the
  revision that adds `/2` and compare outcomes.
- [x] Address temporal bindings by A's declared unique declaration-index key
  under its declared total order, so SC-08's "duplicate" and "reordered" are
  defined against the key rather than file position. The loader refuses a
  repeated key and an out-of-order index; a sparse ascending set is legitimate.
- [x] Record that SC-07's recanonicalization probe is non-vacuous: A supplies a
  pretty-printed JSON re-encoding of the original model bytes, byte-distinct
  from the as-delivered form, and crosses digest domains over raw bytes.

### Discharged at the pin

- [x] Obtain A's immutable revision, raw `/2` bytes and external artifact
  reference, independent selection recipe, original unrecanonicalized input
  corpus, per-axis mutation fixtures with documented refusal expectations, and
  the exact public names of the `/2` boundary. All in hand at `04254a3`.
- [x] Pin that revision in `Cargo.toml` and `Cargo.lock`. `04254a3` is A's
  `origin/main`, not a branch commit.
- [x] Add `quire_protocol::admit_and_link_v2` as a distinct public entry point
  over A's `/2` reader, leaving `admit_and_link` untouched.
- [x] Rederive B's `/2` selection from the original input corpus without reading
  the offered bytes. B decodes A's independent selection sidecar through A's own
  published `handoff::SelectionV2` and checks each original input against the
  identity the sidecar selects for it before any admission runs.
- [x] Run the owed scenarios and report the public API, the pinned commit, and
  the exact refusal outcome per axis. All 28 declared cases refuse with the
  identity A declares, scored through `Error::code`; the unmutated offer admits
  under the same expectation as the non-vacuity control; and the 21-axis
  inventory is frozen at the pin so the corpus cannot shrink unnoticed.

### Remaining

- [ ] Reach the handoff under the declared gate. Until A publishes a path
  constant for its own handoff directory, the 6 submitting tests are
  `#[ignore]`d and SC-06..SC-09 are reported unexecuted rather than passing.
  The 7 tests that exercise B's integrity mechanism build synthetic handoffs
  under `CARGO_TARGET_TMPDIR` and do run on every gate.
- [ ] Settle the pre-delivery corpus. `tests/support/v2/mod.rs` and
  `tests/it001_v2_intake.rs` hold B's own scenario inventory, planned before A
  published one, whose `ExpectedRefusal` identities were to be mapped
  injectively onto A's typed refusals at the pin step. A now publishes its own
  28-case manifest, so that mapping supersedes the corpus rather than annotating
  it. Retiring it touches TM-001, IT-001 Notes and the `spec/reviews/v2-intake`
  findings, so it is specified before it is deleted.

## Deliverables

- `admit_and_link_v2`, the pinned A revision, B's rederived `/2` selection
  fixture, the executed scenario corpus, and per-axis refusal evidence.

## Boundary

Admission and linking alone establish no PT02 result, F observation adequacy,
replay agreement, or campaign completion. Completing this task closes IT-001
SC-06 through SC-09 and nothing else.
