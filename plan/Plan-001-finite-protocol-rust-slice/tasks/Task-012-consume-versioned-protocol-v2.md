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

A's `/2` is **implemented and under final integration**, not absent. The
published surface B builds against is:

- `protocol_artifact::v2::{Expected, ExpectedDefinition, ExpectedTemporal,
  AdmittedPackage, read, encode_candidate}`
- `protocol_artifact::v2::wire::{Package, TemporalBinding, ClockConfiguration}`
- `protocol_artifact::native::{TemporalSelection, AdmissionV2, admit_v2}`
- `temporal::{evaluate_v2, evaluate_with_progress_v2, mapping_support_v2}`
- the existing shared `protocol_artifact::{Limits, Report, Error, Invalid,
  Unsupported}`

`/1` stays strict and byte-compatible; `/1` and `/2` mutually refuse. A's
implementation already carries a declared unique declaration-index key and total
order, typed per-axis mutation refusals, independent `Expected` construction,
raw-byte digest-domain crossing tests, and a non-vacuous pretty-JSON
recanonicalization mutation over original model bytes.

The surface is not yet on `quire-spec-language` `origin/main` or any remote
branch, so B has not verified it directly. B builds against these exact names
now and keeps execution dependency-gated.

**B pins no intermediate commit.** A is integrating the evaluator, occurrence
schema, real `/2` handoff directory, and final review, and will supply one final
immutable revision together with the raw `/2` bytes and reference, the sidecar
recipe, the original input corpus, and the mutation-fixture paths. B pins only
that revision.

The previously provisional `/1` numeric and handoff commits — reachable only
from `agent-a/handoff-numeric-domains`, including the current pin
`5690c2b29c155ec72e21f5a7bb00fe1146b6f71a` — are already included in A's
coherent integration branch. B does **not** wait on a separate PR for that
commit; it repins to A's final revision.

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

### Blocked on A's delivery

- [ ] Obtain A's immutable revision, raw `/2` bytes and external artifact
  reference, independent selection recipe, original unrecanonicalized input
  corpus, per-axis mutation fixtures with documented refusal expectations, and
  the exact public names of the `/2` boundary.
- [ ] Pin that revision in `Cargo.toml` and `Cargo.lock`. Do not pin a branch
  commit; require an ancestor of A's `origin/main`.
- [ ] Add `quire_protocol::admit_and_link_v2` as a distinct public entry point
  over A's `/2` reader, leaving `admit_and_link` untouched.
- [ ] Rederive B's `/2` selection from the original input corpus without reading
  the offered bytes, and record it as a B-owned fixture with provenance.
- [ ] Run the owed scenarios and report the public API, the pinned commit, and
  the exact refusal outcome per axis.

## Deliverables

- `admit_and_link_v2`, the pinned A revision, B's rederived `/2` selection
  fixture, the executed scenario corpus, and per-axis refusal evidence.

## Boundary

Admission and linking alone establish no PT02 result, F observation adequacy,
replay agreement, or campaign completion. Completing this task closes IT-001
SC-06 through SC-09 and nothing else.
