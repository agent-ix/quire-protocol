---
id: SR-106
title: "Gap analysis of the Task-002 exact finite linker"
type: SpecReview
analysis: gap-analysis
scope: "Task-002 / quire-protocol#4, FR-002, TC-002, IT-001, Plan-001 and TM-001"
review_set: subset
relationships:
  - { target: ix://agent-ix/quire-protocol/Task-002, type: reviews }
  - { target: ix://agent-ix/quire-protocol/Plan-001, type: reviews }
  - { target: ix://agent-ix/quire-protocol/TM-001, type: references }
  - { target: ix://agent-ix/quire-protocol/FR-002, type: reviews }
  - { target: ix://agent-ix/quire-protocol/TC-002, type: reviews }
---
# SR-106: Gap Analysis of the Task-002 Exact Finite Linker

## Summary

Task-002's exact allocation is complete and semantically aligned. The versioned
facades, accepted static selection, QObs runtime binding, componentwise FR-288
identity, typed semantic and presentation projections, graph/FIFO values,
transactional registry and TC-002 evidence exist and agree. This subset review
does not give credit to later Protocol pipeline composition.

## Verdict

**PASS for Task-002 / quire-protocol#4.** No in-scope task, matrix, semantic,
trace, reverse-code, stub or validation gap remains. This is not a PASS for
Plan-001 or epic #14 as a whole.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-001 | high | Resolved: AD-001's separate retention of presentation metadata/source loci had no linked-result field or executable observation. `PresentationContext` and its mutation evidence now provide both. | AD-001; `PresentationContext`; FR-002-AC-4 | correct-requirement-no-evidence |
| FND-002 | high | Resolved: FR-002-AC-1/AC-3 evidence allowed incomplete static selections and incompatible reuse of several runtime occurrence identities. Complete ordered coverage and exhaustive reuse controls now kill those cases. | FR-002-AC-1/3; TC-002 | correct-requirement-no-evidence |
| FND-003 | medium | Resolved: FR-002-AC-4 mutated one temporal definition handle but not the retained clock alternative and parameter. Explicit closed-enum mutations now cover both; IT-001 still owns offered-binding admission. | FR-002-AC-4/5; IT-001 | correct-requirement-no-evidence |
| FND-004 | medium | Resolved: source/dependency/conditional inventories and test census trailed the implemented versioned layout. Inventories pass, TM-001 states 66 TC-002 tests and coverage measures 272/272/272. | TM-001; TC-012; NFR-006 | correct-requirement-no-evidence |
| FND-005 | low | The installed Quire configuration expects `Status` while the active TestMatrix contract requires `Coverage Status`; quire-cli#79 remains open and forbids a consumer-table workaround. No false structural PASS is claimed. | quire-cli#79; TM-001 | correct-requirement-no-evidence |

## Target Selection

- Target task: `plan/Plan-001-finite-protocol-rust-slice/tasks/Task-002-link-finite-representation.md`.
- Ticket/epic: quire-protocol#4 under #14, both read live during review.
- Architecture/specification: AD-001, FR-002, TC-002 and the existing whole-scope
  review set SR-089..SR-100; no second architecture cycle was introduced.
- Code/test surface: `src/intake.rs`, `src/model.rs`, `src/link/{mod,binding,
  definition,fifo,runtime_model}.rs`, exact QObs/QSL fixtures and every closed
  dependency/source/conditional inventory affected by the change.
- Baseline: `c26ae8ebe4f0ca8c70dc5b7cc1565ed50281a1ea`.

## Plan and Matrix Reconciliation

| Task-002 obligation | Executed evidence | Result |
| --- | --- | --- |
| Versioned public paths | `intake::{v1,v2}` and `model::{v1,v2}` compile and execute; root `/1` delegates and root `/2` is absent | Complete |
| Exact FR-288 definition | Source key, source-qualified declaration, formal owner/clause and complete selected language/profile/definition/dependency/model/type/temporal/binding closure mutate independently | Complete |
| Exact consistency content | Complete owner-typed declaration dependency closure plus accepted roles/slots; same key/different content refuses transactionally | Complete |
| Separate source context | Exact producer, source inventory, reachable declarations and selected model loci are retained and excluded from semantic equality | Complete |
| Runtime identities | Real constructor-guarded QObs subjects/relationships fill every selected slot; no slot-derived or universal-kind substitute exists | Complete |
| Channel/FIFO graph | Five exact channel premises, complete typed FIFO expression closure, generic owner value and compiler-validated topology are retained without invented edges | Complete |
| Refusal and replay | Every consumer-owned axis is typed/located as available; exact replay is idempotent; ambiguity/contradiction leaves the registry unchanged | Complete |
| Tracking and dependencies | Task-002, Plan-001, TM-001, README, dependency record, Cargo manifest/lock and issues #4/#14 identify the same delivered boundary | Complete pending the closing PR/issue transaction |

## Criterion and Semantic Review

- **FR-002-AC-1:** public and module tests retain distinct static definition,
  role and channel identity plus exact QObs workflow/role/participant/component/
  endpoint/message/send/receive/delivery/attempt/effect subjects. Cross-kind and
  incompatible reuse refuse; compatible participant reuse is not over-refused.
- **FR-002-AC-2:** four property tests enumerate bounded interleavings and prove
  per-branch order, admitted edges and joins; adjacent mutation tests kill a
  dropped join and reject alternatives as concurrent branches.
- **FR-002-AC-3:** static/runtime binding, internality, zero progress, identity,
  FR-288 registry and FR-262 replay/conflict/ambiguity paths assert exact typed
  results, coordinates/loci and absence of linked/committed state. Compiler-owned
  structural cases remain at the pinned QSL admission boundary.
- **FR-002-AC-4:** typed one-axis mutation covers every retained declaration,
  control, edge, correspondence, channel, FIFO, static-binding and temporal
  selection family. Presentation/build/locus and concrete runtime changes leave
  FR-288 unchanged while their separately owned outputs remain observable.
- **FR-002-AC-5:** the default-running IT-001 v2 handoff exercises every temporal
  profile/clock/table mutation against QSL's strict reader; the linker retains
  accepted temporal profile, alternative and parameter values componentwise.

The assertions were read semantically, not credited from tags alone. FR-002
AC-1 through AC-4 are directly bound to TC-002; AC-5 remains correctly allocated
to IT-001 rather than being relabelled as a local admission rule.

## Coverage

- `quire coverage --scope . --json`: 145/342 declared matrix rows backed.
- Rust binding census: 272 candidates, 272 tagged, 272 bound.
- TC-002: 66 default-running tests — 57 linker-module, seven cross-kind identity
  and two public v2 integration tests.
- FR-002-AC-1..AC-4: direct executable bindings; FR-002-AC-5: IT-001 allocation
  with default-running v2 mutation coverage.
- Repository-wide unrelated unbacked rows and the pre-existing TC-014 status lie
  remain visible. Neither is used to claim Task-002 complete.

## Reverse-Gap and Stub Audit

The reverse inventory covered both versioned facades; binding offer/accepted/
runtime types; every binding refusal; FR-288 identity and semantic/presentation
content; definition registry; graph/control/edge/channel/FIFO types; and the
QSL/QObs dependency pins. Each maps to FR-002, TC-002, IT-001 or the frozen `/1`
compatibility rule in AD-001. No unowned parser, evaluator, wire mirror, digest,
numeric approximation, runtime-identity constructor, fallback callback or I/O
seam was introduced.

Searches over the changed production surface find no `todo!`, `unimplemented!`,
`dbg!`, unsafe block, production panic/unwrap/expect, ignored TC-002 test or
placeholder success. All QSL semantic alternatives are matched explicitly; the
only broad test-module allow is documented on the shared IT-001 fixture import.

## Validation

The reviewed candidate passes format check, locked all-target Clippy with
warnings denied and the complete locked suite with 295 passed tests, using one
Cargo job, `/tmp/quire-heavy-check.lock`, and `target-codex-backends`. Coverage
and the Rust binding census match TM-001. All 214 specification/plan documents
are grammar-clean; structural validation reports only the exact live
quire-cli#79 header-contract defect described above.
