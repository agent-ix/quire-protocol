---
id: SR-2127033
title: "TL-9 FR-200..FR-204: integrity"
type: SpecReview
analysis: integrity
scope: "agent-ix/quire-protocol; PR #146 head dea0428; spec/functional/FR-200..FR-204, spec/test-cases/TC-200..TC-204, spec/tests.md"
review_set: subset
date: 2026-11-11
---

## Summary

Ticket: TL-9. Method: spec-review/integrity. Spec-only PR; scope is `git diff origin/main...HEAD`.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | FR-202 Behavior says the library sets FR-241 `refused` on no composition or refinement disposition 'because that value states that the evaluator refused the request', while AC-6 says a stand-in that states `refused` is carried as stated. QSL does refuse composition items (`invalid_model_binding`/`malformed-declaration` for an unprofiled cycle, FR-491-AC-4, which FR-203-AC-2 carries; `invalid_runtime_input`/`invalid-value` for a bad discharge record, FR-530-AC-7) - exactly the FR-241 `refused` case. The Behavior bullet and its rationale contradict both AC-6 and FR-241's meaning; state which execution value a QSL refusal of the claim carries. | spec/functional/FR-202-carry-composition-and-refinement-results-unchanged.md:54 |
| FND-002 | low | FR-200-AC-2 settles a no-backend composition claim `unsupported` with a warning but names no cause, while FR-057 says a claim no QSL engine serves carries `unsupported-requested-capability`, and parallel PR #145's FR-153-AC-1 names that cause and FR-241 execution `unsupported` for the same pattern. Name the cause (or say why none is carried) so the two kinds do not diverge. | spec/functional/FR-200-dispatch-composition-claims-to-qsl.md:52 |
| FND-003 | low | The PR adds 5 FR rows and 5 TC rows but leaves the Overview counts ('49 FRs', '24 test cases', 348 criteria) and the Test Execution Summary (Functional rows 40, Test case rows 24, owed criteria 344) unchanged; #144 on main updates that summary, so a rebase must recompute it rather than keep either side. | spec/tests.md:219 |

## Verdict

PASS WITH FINDINGS. Each FR is atomic to one obligation family, ids are inside ADR-010's FR-200..FR-249 block, every AC is bound to exactly one TC with matching counts (6/6/7/6/6), and the TC rows' bound-criteria counts agree with the FRs.

Units examined:

- `FR-200` (examined)
- `FR-200-AC-1` (examined)
- `FR-200-AC-2` (examined)
- `FR-200-AC-3` (examined)
- `FR-200-AC-4` (examined)
- `FR-200-AC-5` (examined)
- `FR-200-AC-6` (examined)
- `FR-201` (examined)
- `FR-201-AC-1` (examined)
- `FR-201-AC-2` (examined)
- `FR-201-AC-3` (examined)
- `FR-201-AC-4` (examined)
- `FR-201-AC-5` (examined)
- `FR-201-AC-6` (examined)
- `FR-202` (examined)
- `FR-202-AC-1` (examined)
- `FR-202-AC-2` (examined)
- `FR-202-AC-3` (examined)
- `FR-202-AC-4` (examined)
- `FR-202-AC-5` (examined)
- `FR-202-AC-6` (examined)
- `FR-202-AC-7` (examined)
- `FR-203` (examined)
- `FR-203-AC-1` (examined)
- `FR-203-AC-2` (examined)
- `FR-203-AC-3` (examined)
- `FR-203-AC-4` (examined)
- `FR-203-AC-5` (examined)
- `FR-203-AC-6` (examined)
- `FR-204` (examined)
- `FR-204-AC-1` (examined)
- `FR-204-AC-2` (examined)
- `FR-204-AC-3` (examined)
- `FR-204-AC-4` (examined)
- `FR-204-AC-5` (examined)
- `FR-204-AC-6` (examined)
- `spec/tests.md` (examined)

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | fef63ea (9fb5cea): Over every stand-in outcome in this table, no composition or refinement disposition carries execution `refused` unless the stand-in returned a refusal of the item; a stand-in `malformed-declaration` refusal is carried unchanged, and a stand-in refinement result of any label receives no derived axis value. |
| FND-002 | fixed | fef63ea (9fb5cea): Given a `composition` claim with no accepted FR-292 backend, the disposition is `unsupported` carrying the cause `unsupported-requested-capability` (the name FR-057 carries) with a warning naming the required capability, even when every premise in the request is supplied by a matching component guarantee, and the settlement is not a hold, blocker or refusal error. |
| FND-003 | still-open | deferred by the code lead until PR #145 (TL-10) merges: rebase onto main and recompute spec/tests.md Overview, Behavioural totals (now 307 criteria after FR-204-AC-7; the totals line still reads 306) and Test Execution Summary in one pass; not fixed at fef63ea |
