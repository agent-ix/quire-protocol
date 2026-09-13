---
id: SR-041
title: "Evidence review of the version-explicit /2 consumer intake"
type: SpecReview
analysis: evidence
scope: "IT-001 SC-06..SC-09, FR-001, FR-002, NFR-002, NFR-004, TM-001, Task-009, tests/it001_v2_intake.rs, tests/support/v2/mod.rs, as they stood at 3d2f789"
review_set: all
relationships:
  - { target: ix://agent-ix/quire-protocol/IT-001, type: reviews }
  - { target: ix://agent-ix/quire-protocol/TM-001, type: reviews }
---
## Summary

`quoin advise` placed all 83 obligations against the catalogue with two
mismatches and no inconclusive result, but it advises nothing about SC-06
through SC-09: IT-001 scenarios are not obligations, so the version 2 lane's
verification method is inherited from FR-001, FR-002 and NFR-002 rather than
chosen. Read against those inherited methods, the authored version 2 evidence
inverts NFR-002's inclusive-boundary rule, mints matrix credit from a test that
is never executed, and proves corpus completeness against its own enumerations
instead of against IT-001's declared mutation set.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-001 | high | SC-09 and `LimitBound::OneShort` require a refusal one unit short of an inclusive maximum, which NFR-002 says must admit, and neither the scenario nor `resource_scenarios` carries the smallest-excess-unit probe that is the only discriminating case; NFR-002-AC-1 is advised `property-based-testing (universal)`, i.e. a below/at/over property, not a two-point example pair. | IT-001-SC-09; NFR-002-AC-1; tests/support/v2/mod.rs | wrong-requirement |
| FND-002 | high | The `#[ignore]` lane test carries `#[trace("IT-001", ...)]`, so `quire coverage` counts it among the 88 bound backing symbols for IT-001 while `cargo test --locked` — the only enumerated gate command under NFR-004-AC-1 — never runs it and it panics by construction: the deliberate failure is honest, the trace tag is not. | tests/it001_v2_intake.rs; NFR-004-AC-1; TM-001 | correct-requirement-no-evidence |
| FND-003 | medium | The 14 executing tests assert the corpus covers `SelectionAxis::ALL`, `DigestDomain::ALL` and `TemporalBindingMutation::ALL` — the code's own enums, not IT-001's text — so they are self-referential completeness claims whose method is inspection of B's corpus against the spec, yet their `IT-001` tags credit them to FR-001 and FR-002 criteria they exercise nothing of. | tests/it001_v2_intake.rs; FR-001; FR-002 | correct-requirement-no-evidence |
| FND-004 | medium | IT-001 Inputs binds both lanes to one-axis mutations of offered wire/version, seal, feature, referenced binding, numeric representation and numeric domain plus the `i64` and exact-rational endpoints, and `SelectionAxis` declares none of the six; FR-001-AC-4 names IT-001 as its evidence and is advised `golden-approval-testing (serialization)`, for which the version 2 corpus has no byte-for-byte round-trip scenario at all. | IT-001; FR-001-AC-4; tests/support/v2/mod.rs | correct-requirement-no-evidence |
| FND-005 | medium | TM-001 carries one IT-001 integration row for two contracts the spec says may not stand in for each other, so a completed version 1 lane will flip that row while SC-06..SC-09 stay unexecuted; no NFR-002 criterion or TM-001 cell routes SC-09 evidence to TC-009 despite the new IT-001→NFR-002 `verifies` edge. | TM-001; IT-001; NFR-002 | missing-requirement |
| FND-006 | medium | The version 2 lane is selected at run time by `QUIRE_PROTOCOL_A_V2_FIXTURE` plus `#[ignore]`, and NFR-004's inventory inspects only `[features]` and `cfg`, so an env-selected lane is invisible to AC-5, AC-7 and AC-8 while producing exactly the green-lane-that-selects-nothing the requirement exists to forbid; the advised `combinatorial-tway (configuration-matrix)` for AC-5/AC-7 should cover runtime lane selectors, or NFR-004's scope should name them. | NFR-004-AC-5; NFR-004-AC-7; tests/it001_v2_intake.rs | wrong-requirement |
| FND-007 | low | `quire coverage` reports three unmatched `SC-06`/`SC-09` doc-comment tags from this file, and `Scenario.criterion` values such as `"IT-001-SC-06"` are checked by nothing, so the lane's criterion-level bookkeeping is prose that no tool reconciles against TM-001. | tests/it001_v2_intake.rs; tests/support/v2/mod.rs; TM-001 | correct-requirement-no-evidence |
| FND-008 | low | The two advisor mismatches both sit in scope: NFR-004-AC-2 and NFR-004-M-2 are authored `Inspection` against a recommended `static-quality` analysis, and quire separately reports that AC-2's `Inspection (TC-012)` mints no source symbol; the authored choice is defensible but must be confirmed in the cell rather than left standing by default. | NFR-004-AC-2; NFR-004-M-2 | correct-requirement-no-evidence |
| FND-009 | low | Absence reporting is the one obligation this lane discharges today, and two of its five report paths — `Absence::EnvUnset` and `Absence::UnreadableManifest` — have no test, so a malformed evidence index could be reported as an unrelated absence without any case failing. | tests/it001_v2_intake.rs; tests/support/v2/mod.rs | correct-requirement-no-evidence |

## Method recommendations for the version 2 obligations

`quoin advise` returned 83 obligations, 2 mismatched, 0 uncatalogued, 0
inconclusive. None of them is an IT-001 scenario, so the recommendations below
are the inherited obligation methods applied to SC-06 through SC-09, with the
judgement half labelled as judgement.

| Obligation | Advised method | version 2 scenario | Authored evidence |
| --- | --- | --- | --- |
| FR-001-AC-1 | `bdd-spec-by-example`, `unit-testing` | SC-06 valid admission | matches once A's boundary is pinned |
| FR-001-AC-2 | `bdd-spec-by-example`, `unit-testing` | SC-06 changed-selection axes, SC-07, SC-08 | axis set narrower than IT-001 Inputs (FND-004) |
| FR-001-AC-4 | `golden-approval-testing (serialization)` | none | no version 2 numeric round-trip scenario (FND-004) |
| FR-002-AC-1 | `bdd-spec-by-example`, `deterministic-simulation` | SC-06 admitted-package identities | no criterion-level binding (FND-003) |
| NFR-002-AC-1 | `property-based-testing (universal)` | SC-09 | two-point and inverted (FND-001) |
| NFR-004-AC-5, AC-7 | `combinatorial-tway (configuration-matrix)` | the lane's own selection mechanism | env-selected lane uninventoried (FND-006) |

Judgement, not catalogue: the 14 executing tests are best classed as inspection
of B's corpus against IT-001's text. They are not integration evidence, and
`Test` is the defaulted label this analysis exists to refuse.

## Is the deliberately failing ignored test acceptable?

As a marker, yes; as a traced symbol, no. A lane that panics with what A still
owes is the honest alternative to a vacuous pass, and it belongs in the tree.
The defect is that `#[trace("IT-001", "Task-009")]` makes `quire coverage`
treat it as a backing symbol indistinguishable from an executed one, while
`cargo test --locked` never runs it — coverage the gate cannot contradict.
Remove the `IT-001` tag from that one test, leaving the corpus and loader tests
traced, and record the lane's blocked state in TM-001 where a human reads it.

## TM-001 and the version 2 lane

TM-001 needs a row. IT-001's own text makes version 1 and version 2 two integrations that
may not substitute for each other, and the matrix currently records one row for
both, so the version 2 lane has no cell that can say `Blocked`. Add a second
integration row for the version 2 contract at `🚧 Blocked (quire-spec-language#40)`,
name it in NFR-002's evidence column alongside TC-009 so SC-09 has somewhere to
land, and fix the `status-column-matches-nothing` diagnostic first: the
Functional table's `Coverage Status` header means quire skips status
classification entirely, so any status a new row carries is unchecked today.
