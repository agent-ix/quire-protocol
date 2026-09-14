---
id: Task-012
title: "Consume the version-explicit compiled-protocol /2 artifact"
type: Task
status: done
track: B
priority: P0
relationships:
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

**Delivered and pinned.** A's `/2` is published
and promoted. `quire-spec-language` release `v0.2.0` is at `6eccb05`, and PR #92
exports its handoff path, member names, and mutation format on `main` at
`93c411b287ccf4789879ac4ef3094fc921dada58`. `Cargo.toml` and `Cargo.lock`
pin that exact revision. The direct test dependency on `quire-contract-ir` is
also aligned to A's `04eb6f8` transitive pin, so one Rust identity type is
present in the resolved graph.
The complete surface B consumes is:

- `protocol_artifact::v2::{Expected, ExpectedDefinition, ExpectedTemporal,
  AdmittedPackage, read, encode_candidate}`
- `protocol_artifact::v2::wire::{Package, TemporalBinding, ClockConfiguration}`
- `protocol_artifact::native::{TemporalSelection, AdmissionV2, admit_v2}`
- `temporal::{evaluate_v2, evaluate_with_progress_v2, mapping_support_v2}`
- `protocol_artifact::handoff::{SelectionV2, MutationManifest, ...}` — the
  interchange record shapes for `expected-v2.json` and
  `mutations/manifest.json`, decoded rather than mirrored
- `protocol_artifact::handoff::{PUBLISHED_HANDOFF, PUBLISHED_OFFER_FILE,
  PUBLISHED_ARTIFACT_REFERENCE_FILE, PUBLISHED_SELECTION_FILE,
  PUBLISHED_MUTATION_MANIFEST_FILE, PUBLISHED_CHECKSUMS_FILE,
  MUTATION_MANIFEST_FORMAT}` — the owner-exported address, member names, and
  manifest format
- `protocol_artifact::Error::code` — the stable refusal identity per axis
- the existing shared `protocol_artifact::{Limits, Report, Invalid, Unsupported}`

`/1` stays strict and byte-compatible; `/1` and `/2` mutually refuse.

The handoff bundle A committed under `artifacts/compiled-protocol-v2/` is 109
files plus `SHA256SUMS`, carrying 28 mutation cases over 21 axes in format
`quire.protocol.v2-mutations/1`.

## Subtasks

### B-owned, available now

- [x] Author the B-owned pre-delivery `/2` evidence loader and scenario index;
  this provisional corpus was retired after A published the authoritative
  handoff and manifest.
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
  the exact public names of the `/2` boundary. The reader/corpus was delivered
  at `04254a3`; the complete exported handoff surface is at `93c411b`.
- [x] Pin `93c411b` in `Cargo.toml` and `Cargo.lock`; it is A's `origin/main`,
  not a branch commit.
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

- [x] Pin `quire-spec-language` `93c411b`, import A's exported handoff constants,
  remove `QUIRE_PROTOCOL_A_V2_HANDOFF`, `A_OWED_API`, and the six ignored-test
  gates, and execute the handoff on every default local test run.
- [x] Exercise both cross-version submissions through the public `/1` and `/2`
  entry points, without auto-detection, upgrade, or downgrade.
- [x] Exercise SC-09 against every `protocol_artifact::Dimension::ALL` member:
  measure the unchanged base input, admit at exact usage, refuse at one-short
  with the matching dimension, and admit the same bytes under a fresh exact
  budget.
- [x] Retire the superseded pre-delivery corpus. `tests/support/v2/mod.rs` and
  `tests/it001_v2_intake.rs` hold B's own scenario inventory, planned before A
  published one, whose `ExpectedRefusal` identities were to be mapped
  injectively onto A's typed refusals at the pin step. A now publishes its own
  28-case manifest, so that mapping supersedes the corpus rather than annotating
  it. Reconcile TM-001, IT-001, the current functional references, the fixture
  README, and the plan log; preserve the historical `spec/reviews/v2-intake/`
  reviews as the record of the state they examined.
- [x] Run the Rust local gate plus self `/rust-review` and `/gap-analysis`, fix
  all findings, merge the one-ticket PR administratively, and update issue #48,
  parent tracking, this task, and the plan log immediately.

## Deliverables

- `admit_and_link_v2`, the pinned A revision, B's rederived `/2` selection
  fixture, the executed scenario corpus, and per-axis refusal evidence.

## Boundary

Admission and linking alone establish no PT02 result, F observation adequacy,
replay agreement, or campaign completion. Completing this task executes the
version 2 portions of IT-001 SC-06 through SC-09. Task-009 continues to own the
broader version 1 SC-01 through SC-05 regression lane.
