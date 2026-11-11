---
id: SR-2127032
title: "TL-9 FR-200..FR-204 composition and refinement dispatch: spec review"
type: SpecReview
analysis: base
scope: "agent-ix/quire-protocol; PR #146 head dea0428; spec/functional/FR-200..FR-204, spec/test-cases/TC-200..TC-204, spec/tests.md"
review_set: subset
date: 2026-11-11
---

## Summary

Ticket: TL-9. Method: spec-review/base. Spec-only PR; scope is `git diff origin/main...HEAD`.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | FR-202 says the library carries the FR-241 execution and FR-242 truth values 'exactly as the backend's result states them', and AC-5/AC-6 feed stand-ins that state FR-241 values. QSL's composition and refinement results state no FR-241/FR-242 values: the planner emits FR-360-style labels (proved/inconclusive/unsupported/incomplete, qsl-replay/src/planner/settle.rs decide()) and refinement emits refines/violates/undecided causes. The owner mapping for composition is QSpec FR-489 Outputs ('an FR-360 result') read through FR-360's label-to-FR-241/242/243 table, which FR-202 never cites; for refinement (FR-177) no owner mapping exists and should be named as an open owner question, not hidden behind 'as stated'. As written, the disposition's required axes have no source and AC-5/AC-6 test a shape the real backend never produces. | spec/functional/FR-202-carry-composition-and-refinement-results-unchanged.md:53 |

## Verdict

FAIL. The five FRs follow ADR-010's dispatch-and-carry split, cite real QSL wire names (`unsupported-requested-capability`, `premise-unproved`, `protocol-assumption-not-established`, `malformed-declaration` all exist on QSL main), and every criterion is a runnable Test against a stand-in for the QSL call boundary; none is a source scan. Author's flagged points: (1) `identity-mismatch` is a member of FR-008's closed ProtocolCause set and FR-030's accepted ruling already maps target-context mismatch to it, so its use in FR-201/FR-204 is consistent and adds no parallel vocabulary; (2) on QSL main, decide() lets a non-deferred `fixed-point` slot labelled `unsupported` decide the composition item `unsupported` with that slot's cause, an `inconclusive` fixed-point slot yields `inconclusive`/`premise-unproved` (FR-530-AC-5), and an unprofiled cycle refuses `invalid_model_binding`/`malformed-declaration` (FR-491-AC-4) - FR-203's three branches match; the exact cause on an unserved fixed-point item is the generic V-8 `unsupported-requested-capability`, untested for that item because no producer exists (G6); (3) exact identity inclusion in FR-204 is correct, entailment stays with FR-177; (4) QSpec FR-177 indeed has no FR-241/242 mapping, but composition does (FR-489 -> FR-360), see FND-001. Consistent with merged FR-057/FR-058/FR-301-AC-6. spec/tests.md: the PR's delta (+5 rows, +31 criteria, 81+31=112, 275+31=306) is arithmetically right, but the tree actually totals 308 criteria over the 55 rows because the merged 275 baseline was already 2 short (for example FR-043's row lists AC-1..AC-8 while FR-043 has 9) - pre-existing, not this PR's defect. `quire validate` fails only on spec/tests.md's pre-existing `Coverage Status` column header (same on the merge base). The branch now conflicts with main in spec/tests.md after #144 merged; #145 also edits the same tables.

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
- `TC-200` (examined)
- `TC-201` (examined)
- `TC-202` (examined)
- `TC-203` (examined)
- `TC-204` (examined)
- `spec/tests.md` (examined)

## New findings (disposition pass 1)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-002 | low | FR-202-AC-5 reads `proved` and `inconclusive` 'as the FR-360 rows', but FR-360's `proved` row allows basis `closed-scope` or `decisive-witness`; the composition-specific choice is QSpec FR-489's Settlement table (`proved` -> `decisive-witness`, `inconclusive` -> `unsettled`). FR-489's table also lists `incomplete` as an FR-360 label with basis `unavailable` although FR-360 has no such member. Read the basis through FR-489's Settlement table, and record the FR-489/FR-360 `incomplete` disagreement beside the refinement-mapping open question for the quire-specification owner. | spec/functional/FR-202-carry-composition-and-refinement-results-unchanged.md:61 |

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | fef63ea (9fb5cea): Given a stand-in composition result labelled `unsupported` that also carries certificate-shaped bytes, the disposition carries the label, reads it as execution `unsupported`, truth `unavailable` and basis `unavailable` through the FR-360 table, and derives no truth from the bytes; given the labels `proved`, `inconclusive` and `incomplete`, the first two read as the FR-360 rows and `incomplete`, absent from that table, is carried unchanged with no derived axis value. |
| FND-002 | fixed | round 2, eea438d: - The library shall read the settlement basis of `ix://agent-ix/quire-specification/FR-243` from the Settlement table of `ix://agent-ix/quire-specification/FR-489` (`proved` to `decisive-witness`, `inconclusive` to `unsettled`, `unsupported` and `incomplete` to `unavailable`), because FR-360 leaves `proved` open between two bases. |
