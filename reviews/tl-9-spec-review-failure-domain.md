---
id: SR-2127037
title: "TL-9 FR-200..FR-204: failure domain"
type: SpecReview
analysis: failure-domain
scope: "agent-ix/quire-protocol; PR #146 head dea0428; spec/functional/FR-200..FR-204, spec/test-cases/TC-200..TC-204, spec/tests.md"
review_set: subset
date: 2026-11-11
---

## Summary

Ticket: TL-9. Method: spec-review/failure-domain. Spec-only PR; scope is `git diff origin/main...HEAD`.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | FR-204 carries a result 'when every retained assumption identity is among the request's'. A result that retains no assumption identities, or omits the field, satisfies that vacuously and is carried as if established under the request, although QSpec FR-177-AC-1 requires a proof record to name its assumptions and FR-030-AC-4 says absence compares only to absence. State the outcome for an absent retained-assumption set (refuse, or carry as absent without success) and add a criterion for it. | spec/functional/FR-204-refuse-a-refinement-result-under-stronger-assumptions.md:35 |

## Verdict

PASS WITH FINDINGS. Unknown cause names, one-half counterexamples, empty discharge records, unknown subjects and fixed-point outcomes other than unsupported each have an explicit outcome and criterion.

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

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | fef63ea (9fb5cea): Given a stand-in result that omits its retained assumption identities, and another that retains an empty set while the request holds assumptions, the first is refused with `identity-mismatch` naming the absent field and the second is carried as a strict subset. |
