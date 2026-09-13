---
id: SR-054
title: "Gap analysis — Plan-001 after C-05 typed source retention"
type: SpecReview
analysis: gap-analysis
scope: "plan/Plan-001-finite-protocol-rust-slice/, spec/tests.md, src/, tests/, with semantic focus on C-05 at 0ab9d91"
review_set: subset
relationships:
  - { target: ix://agent-ix/quire-protocol/Plan-001, type: reviews }
  - { target: ix://agent-ix/quire-protocol/TM-001, type: references }
  - { target: ix://agent-ix/quire-protocol/Task-001, type: references }
---
# Gap analysis — Plan-001 after C-05 typed source retention

## Summary

Plan-001 remains intentionally incomplete after the focused C-05 increment.
The increment has a real TC-001 binding and its intent, test, and code agree,
but it does not remove the plan's owner-contract and downstream-task gaps.

## Verdict

**FAIL** — Plan-001 has no completed task and only 38 of 322 matrix rows are
backed; this is not a complete protocol or production-qualification candidate.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | Plan-001 has 0 of 13 tasks done: 5 are in progress and 8 are blocked. C-02/C-03 specifically cannot proceed while the owner catalog and cause contracts remain unresolved. | Plan-001; Task-001..Task-013; quire-specification FR-270, FR-272 |
| FND-002 | high | `quire coverage` reports 38 of 322 rows backed; only 8 of 14 Test Case rows have a bound Rust symbol. TC-003, TC-007, TC-010, TC-012, TC-013, and TC-014 remain unbacked. | TM-001; TC-003, TC-007, TC-010, TC-012, TC-013, TC-014 |
| FND-003 | high | Twelve reserved source modules remain intentional implementation stubs for compensation, conformance, claim premises/dispatch, typed causes, result/global/lineage, and the resource inventory. Their tasks are not marked done, so this is exposed unfinished work rather than false completion. | src/compensation/; src/conformance/; src/claims/premises.rs; src/claims/dispatch.rs; src/refusal/cause.rs; src/result/; src/bounds/inventory.rs |

## Coverage

- Reconciliation: `quire coverage` 0.31.0 (engine
  `ca7362d4dacecb96f01d74d1d971327118c25917`) over explicit scope
  `/home/peter/dev/quire-protocol`.
- Tasks done: 0 / 13 (5 in progress, 8 blocked).
- Rows backed by a tagged test: 38 / 322.
- Rust binding census: 198 / 198 / 198 bound/tagged/candidates.
- Untraced behaviors / stubs: 12 intentional reserved modules; no additional
  unowned behavior was introduced by C-05.
- Semantic review: ran for the C-05 requirement/test/code triple; the rest of
  Plan-001 was mechanically reconciled and retained as outstanding.

## Semantic review

| Intent | Test oracle | Code path | Result |
| --- | --- | --- | --- |
| FR-001 returns typed admission refusals rather than prose | `tc_001_mapping_retains_every_published_source_error` compares the complete owner value and stable owner code for every current top-level family and every current `Invalid`/`Unsupported` member | `classify_source` consumes and retains `Error`; `source_error` and `source_code` expose typed observations | aligned for the C-05 slice |
| Issue #34 keeps selection axes independently assertable | Existing IT-001 tests compare `DivergedSelection` and the exact A source error together | `admit_discriminating` refines only `Invalid::Selection` and preserves its source | aligned for the landed top-level inventory |
| FR-008/NFR-008 derive no refusal fact from rendering | The new regression fails on the old debug-string fallback and carries no FR-008 trace credit | The wildcard arm stores `Error` directly and performs no formatting | aligned as a prerequisite only; complete envelope remains blocked |

The test exercises the public mapper directly, uses A's `Error::code` as the
owner oracle, and is non-tautological: restoring the prior debug-string fallback
or dropping either newly published error family makes the exact-value assertion
or compilation fail. The change declares no source catalog, cause taxonomy,
wire schema, parser, or new requirement.

## Disposition

The three findings are program-level work already represented by open tasks and
owner blocks. None is hidden or marked complete, and none can be remediated
inside C-05 without violating the owner-before-declaring boundary. The focused
increment is therefore mergeable as partial Plan-001 work, but the Plan-001
verdict remains FAIL and no task/epic acceptance box may be checked from it.
