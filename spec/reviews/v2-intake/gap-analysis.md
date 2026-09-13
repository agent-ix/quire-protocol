---
id: SR-037
title: "Gap analysis — Plan-001 on agent-b/it001-v2-intake"
type: SpecReview
analysis: gap-analysis
scope: "plan/Plan-001-finite-protocol-rust-slice/, spec/tests.md, src/, tests/, as they stood at 4a10925"
review_set: subset
relationships:
  - { target: "ix://agent-ix/quire-protocol/Plan-001", type: reviews }
  - { target: "ix://agent-ix/quire-protocol/TM-001", type: references }
  - { target: "ix://agent-ix/quire-protocol/IT-001", type: references }
---

## Summary

This audit covers Plan-001, TM-001 and the Rust source and test trees as they
stand on branch `agent-b/it001-v2-intake` at `2eb56c2`. The matrix's own numbers
re-derive exactly from the tree and its cross-repository status claims hold, but
no Plan-001 task is done, engine-measured binding is 24 of 322 rows, and one
trace tag names a criterion the owner has ruled unresolved.

## Verdict

**FAIL** — all 13 Plan-001 tasks are blocked or in progress, 240 declared rows
are unbacked, and a test carries the `FR-002-AC-1` tag that must not be carried.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | Plan-001 has 0 of 13 tasks done: Task-009, Task-012 and Task-013 are in progress and the other ten are blocked. Every one is P0. | Plan-001; Task-001 through Task-013 |
| FND-002 | high | `quire coverage` reports 24 of 322 rows backed and 240 unbacked rows. Whole requirements bind nothing: FR-003, FR-005, FR-007, FR-009, FR-010, FR-012 through FR-014, FR-016 through FR-027, FR-029 through FR-041, NFR-002 through NFR-004 and NFR-006 through NFR-008, and all 20 StR validation criteria. | `quire coverage` report; TM-001; spec/functional/; spec/non-functional/; spec/stakeholder/ |
| FND-003 | high | `tests/it001_native_handoff.rs:449` carries the `FR-002-AC-1` trace tag, which the owner has ruled unresolved; the tag mints one of the 24 backed rows, so the reported binding overstates by that row. | tests/it001_native_handoff.rs:449; FR-002-AC-1 |
| FND-004 | medium | 72 test symbols carry only plan-task identifiers — Task-009 (47), Task-008 (12), Task-012 (7), Task-002 (6) — which live outside the spec document root and therefore mint no matrix credit. The matrix records the Task-009 and Task-012 cases as deliberate; the Task-002 and Task-008 cases sit alongside criterion tags on the same symbols. | tests/it001_v2_intake.rs; tests/it001_native_handoff.rs; tests/it001_v2_handoff.rs; tests/tc_008_reproduction_record.rs; tests/tc_002_identities.rs |
| FND-005 | medium | Three symbols in `tests/tc_004_closure_axes.rs` bind the foreign identifiers `FR-061` and `FR-284`, which are `quire-specification` ids quoted in doc comments. They resolve to no local artifact and the engine reports them as untracked symbols. | tests/tc_004_closure_axes.rs:262; tests/tc_004_closure_axes.rs:305; tests/tc_004_closure_axes.rs:342 |
| FND-006 | high | The `functional-coverage` declaration expects a `Status` column and the table under `Functional Requirement Coverage` has `Coverage Status`, so status classification was skipped entirely; `status_lies: []` is therefore an absence of measurement, not an absence of lies. | spec/tests.md:198; `quire coverage` diagnostics |
| FND-007 | medium | `NFR-005` mints nothing: its heading is `Measurement and Evaluation` where the declaration selects `Acceptance Criteria`, so its rows are stranded. The matrix states the exclusion deliberately, but the engine cannot see the document at all. | spec/non-functional/NFR-005-retain-composed-acceptance-evidence.md; `quire coverage` diagnostics |
| FND-008 | high | Fourteen source modules remain two-line declared-empty placeholders for owned behaviour, so the FR-002 through FR-008 and NFR-002 behaviour their module docs reserve does not exist. | src/link/diagnostics.rs; src/compensation/{activate,algebra,purity,recovery}.rs; src/conformance/{batch,incremental}.rs; src/claims/{dispatch,premises}.rs; src/result/{global,lineage}.rs; src/bounds/inventory.rs; src/refusal/{catalog,cause}.rs |
| FND-009 | low | 209 of the 229 criteria whose `Verification` cell names a test case have no symbol carrying their tag; only 20 criteria plus 4 test-case rows are engine-bound. The matrix states this gap in its own words and the figure is recorded here as the measured size of it. | TM-001; spec/tests.md |

## Coverage

- Target: `plan/Plan-001-finite-protocol-rust-slice/`; spec root: `spec/`; matrix: `spec/tests.md` (TM-001); source/tests: `src/` and `tests/`; identity prefix: `ix://agent-ix/quire-protocol`. Branch `agent-b/it001-v2-intake` at `2eb56c2`.
- Reconciliation: `quire coverage` 0.32.0 (CLI 55a1c4e2, engine 0.46.0), executed against the repository root.
- Tasks done: 0 / 13. No checkbox drift: all 26 `plan.md` checkboxes are unchecked and no task is done.
- Rows backed by a tagged test: 24 / 322. `unbacked_rows`: 240. `status_lies`: 0, void under FND-006. `untracked_symbols`: 75. `no_symbol_rows`: 48, correctly excluded from the unbacked-row findings.
- Matrix arithmetic re-derived from the tree: 288 acceptance criteria (241 FR + 47 NFR), 229 test-backed, 12 integration-backed, 47 unbacked (37 Inspection + 10 Analysis); behavioural 177 = 162 + 2 + 13; object 64 = 37 + 6 + 21; non-functional 47 = 30 + 4 + 13; 32 owed integration scenarios (10 + 12 + 10). Every figure the matrix states matches.
- Cross-repository status claims verified: `quire-spec-language` main is `72507f8`, PR #75 is OPEN with head `bb17f74`, and main carries no `quire.compiled-protocol/2` producer; `quire-specification` issues 28 and 29 are open.
- Reverse-gap inventory: 30 Rust source files; 14 declared-empty placeholders; no untraced public behaviour beyond them.
- Execution evidence: none taken. The gate was not run in this audit; every finding is static.
- Semantic review: skipped — not requested.
