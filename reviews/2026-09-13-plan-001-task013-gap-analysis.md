---
id: SR-056
title: "Gap analysis — Plan-001 after Task-013 generic resolution"
type: SpecReview
analysis: gap-analysis
scope: "plan/Plan-001-finite-protocol-rust-slice/, spec/tests.md, src/, tests/, with semantic focus on Task-013 at adfaa0f"
review_set: subset
relationships:
  - { target: ix://agent-ix/quire-protocol/Plan-001, type: reviews }
  - { target: ix://agent-ix/quire-protocol/TM-001, type: references }
  - { target: ix://agent-ix/quire-protocol/Task-013, type: references }
---
# Gap analysis — Plan-001 after Task-013 generic resolution

## Summary

Plan-001 remains intentionally incomplete after the Task-013 generic
resolution increment. TC-013 is fully implemented on B-owned fixtures and its
intent, tests and code agree. The concrete manifest, producer pins and external
scenario evidence remain unavailable, so Task-013 correctly stays in progress.

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

## Coverage

- Reconciliation: Quire 0.31.0 (engine 0.46.0 at `ca7362d4`) over the
  repository's explicit specification and plan artifacts.
- Tasks done: 0 / 13 (5 in progress, 8 blocked).
- Rows backed by a tagged test: 58 / 322.
- Rust binding census: 211 / 211 / 211 candidates, tagged and bound.
- TC-013: 13 / 13 B-owned tests implemented and passing.
- External Task-013 integrations: IT-002 0 / 12 scenarios executable; IT-003
  0 / 10 scenarios executable.
- Semantic review: performed for the Task-013 requirement/test/code slice; the
  rest of Plan-001 was mechanically reconciled and remains outstanding.

## Semantic review

| Intent | Test oracle | Code path | Result |
| --- | --- | --- | --- |
| FR-009 resolves only exact owner/version entries and exact delivered bytes | TC-013 independently mutates key presence, duplicate keys, pin fields, delivery selection and digest bytes, then compares typed outcomes and retained exact values | `EntryKey`, `ManifestEntry`, `VersionLockManifest` and `resolve` validate the full manifest and selected deliveries before returning producer bytes | aligned for B-owned fixtures; concrete producer keys/pins remain blocked |
| FR-009/FR-016 bound all resolution work by entries, bytes and deadline | Exact-limit, one-short, fresh-budget and manifest-validation regression controls compare the named incomplete dimension and observed usage | `ResolutionBudget`, `DeadlineGuard` and `resolve` charge the full manifest, delivery validation and digest units with checked accounting | aligned for the resolver; repository-wide C-30 inventory remains open |
| FR-017 records resolution, ownership, typed outcome, budgets and trace without becoming a store | TC-013 constructs full and nonempty partial records, mutates record identity and checks integration-wide refusal-axis collisions | `ScenarioEvidenceRecord` retains values only in memory and `validate_refusal_axis_distinctness` compares typed identities within an integration | aligned for the value model; actual IT-002/IT-003 records remain unexecutable |
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
generic increment can merge without concealing those gaps. Task-013, Plan-001
and the delivery epic must remain open.
