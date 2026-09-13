---
id: SR-056
title: "Gap analysis — Plan-001 after Task-013 generic resolution"
type: SpecReview
analysis: gap-analysis
scope: "plan/Plan-001-finite-protocol-rust-slice/, spec/tests.md, src/, tests/, with mandatory completion/coverage focus at dae1277"
review_set: subset
relationships:
  - { target: ix://agent-ix/quire-protocol/Plan-001, type: reviews }
  - { target: ix://agent-ix/quire-protocol/TM-001, type: references }
  - { target: ix://agent-ix/quire-protocol/Task-013, type: references }
---
# Gap analysis — Plan-001 after Task-013 generic resolution

## Summary

Plan-001 remains intentionally incomplete after the Task-013 generic
resolution increment. The resolver portion of TC-013 is implemented on B-owned
fixtures, but the scenario-evidence constructor and its positive test do not
satisfy the complete budget-population obligation. The concrete manifest,
producer pins and external scenario evidence also remain unavailable, so
Task-013 correctly stays in progress.

## Verdict

**FAIL** — Plan-001 has 0 of 13 tasks done and only 58 of 322 matrix rows are
backed. The delivered slice is useful and mergeable, but it is not a complete
protocol or production-qualification candidate.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | Plan-001 has 0 of 13 tasks done: 5 are in progress and 8 are blocked. Task-013's generic resolver is implemented, but its concrete manifest and external scenarios remain open. | Plan-001; Task-001..Task-013 |
| FND-002 | high | `quire coverage` reports 58 of 322 rows backed. TC-013 is complete with 13 tests, while the matrix continues to expose unbacked and unexecutable rows rather than converting planned test ownership into execution credit. | TM-001; TC-013 |
| FND-003 | high | Six intended version-lock owners publish no contract version. Because FR-012 makes that owner-authored value part of the entry key, the concrete current manifest cannot be authored locally and IT-002/IT-003 remain unexecutable. | FR-009; FR-012; IT-002; IT-003; Task-013 |
| FND-004 | medium | C-30's bidirectional resource-site inventory remains planned and must include the new resolver/evidence allocation, retention and iteration sites before TC-009/NFR-002 can be complete. | TC-009; NFR-002; Task-008 |
| FND-005 | high | TC-013's scenario-evidence test supplies 3 of the 20 declared `BudgetDimension` variants, and `ScenarioEvidenceRecord::new` accepts that subset while claiming a complete record. FR-017/NFR-005 require every declared dimension or an unexecuted disposition, so the implementation and test do not support the matrix's completed scenario-evidence claim. | `src/version_lock/evidence.rs:134-157,277-340`; `tests/tc_013_scenario_evidence.rs:68-74,105-136`; FR-017; NFR-005; TC-013; TM-001 |

## Coverage

- Reconciliation: Quire 0.31.0 (engine 0.46.0 at `ca7362d4`) over the
  repository's explicit specification and plan artifacts.
- Tasks done: 0 / 13 (5 in progress, 8 blocked).
- Rows backed by a tagged test: 58 / 322.
- Rust binding census: 211 / 211 / 211 candidates, tagged and bound.
- TC-013: 13 / 13 B-owned tests implemented and passing, but the
  scenario-evidence completeness oracle is insufficient (FND-005).
- External Task-013 integrations: IT-002 0 / 12 scenarios executable; IT-003
  0 / 10 scenarios executable.
- Semantic review: the previously retained Task-013 semantic table is preserved
  below as historical review context; this rerun did not opt into a new optional
  semantic pass. Mandatory completion and traceability checks found FND-005.

## Semantic review

| Intent | Test oracle | Code path | Result |
| --- | --- | --- | --- |
| FR-009 resolves only exact owner/version entries and exact delivered bytes | TC-013 independently mutates key presence, duplicate keys, pin fields, delivery selection and digest bytes, then compares typed outcomes and retained exact values | `EntryKey`, `ManifestEntry`, `VersionLockManifest` and `resolve` validate the full manifest and selected deliveries before returning producer bytes | aligned for B-owned fixtures; concrete producer keys/pins remain blocked |
| FR-009/FR-016 bound all resolution work by entries, bytes and deadline | Exact-limit, one-short, fresh-budget and manifest-validation regression controls compare the named incomplete dimension and observed usage | `ResolutionBudget`, `DeadlineGuard` and `resolve` charge the full manifest, delivery validation and digest units with checked accounting | aligned for the resolver; repository-wide C-30 inventory remains open |
| FR-017 records resolution, ownership, typed outcome, budgets and trace without becoming a store | TC-013 constructs full and nonempty partial records, mutates record identity and checks integration-wide refusal-axis collisions, but supplies only 3 of 20 budget dimensions | `ScenarioEvidenceRecord` retains values only in memory and `validate_refusal_axis_distinctness` compares typed identities within an integration, but the constructor checks only duplicate—not absent—dimensions | **not aligned** for complete budget retention (FND-005); actual IT-002/IT-003 records also remain unexecutable |
| NFR-006 prevents delivery material from authorizing the lock | TC-013 swaps same-path bytes, supplies unselected members and confirms no delivery-index or sidecar input exists | `resolve` accepts only the caller-selected manifest, required keys and deliveries; delivery bytes can verify but never populate or repair an entry | aligned for B-owned fixtures; dependency/right inspections remain outstanding |

The tests exercise public APIs and use independent inputs and typed expected
outcomes. Restoring the former unbudgeted manifest-validation path, changing
entry equality to include state, accepting a sidecar/index, substituting an
adjacent version, normalizing delivered bytes or collapsing outcomes makes a
named test fail. Inspection and Analysis obligations are not counted merely
because adjacent runtime behavior exists.

## Disposition

The concrete owner-version gap and external scenarios are already represented
by Task-013 and GitHub issue 49; C-30 remains represented by Task-008. The
resolver increment is useful, but the combined PR should not merge while
FND-005 remains open. Task-013, Plan-001 and the delivery epic must remain open.
