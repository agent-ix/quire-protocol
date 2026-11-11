---
id: SR-2127035
title: "TL-9 FR-200..FR-204: scope boundary"
type: SpecReview
analysis: scope-boundary
scope: "agent-ix/quire-protocol; PR #146 head dea0428; spec/functional/FR-200..FR-204, spec/test-cases/TC-200..TC-204, spec/tests.md"
review_set: subset
date: 2026-11-11
---

## Summary

Ticket: TL-9. Method: spec-review/scope-boundary. Spec-only PR; scope is `git diff origin/main...HEAD`.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | FR-201 refuses a disposition whose discharge record 'names a slot that is in none of the four lists'. QSL's discharge record (QSL FR-530, qsl-replay/src/planner/compose.rs DischargeEntry::to_wire) has one entry per derived slot, and slots include roles outside the four premise lists - `component-guarantee` and, on an accepted cycle, `fixed-point` (SlotRole in planner/declaration.rs; composition_tests.rs a_cycle_is_discharged_only_by_a_proved_fixed_point_result asserts discharges[FIXED_POINT].relation == 'fixed-point'). Every real composition result would therefore be refused `identity-mismatch`, including FR-203's proved-cycle case. Slots are also addressed by index derived by QSL from the FR-491 declaration, so deciding slot-to-list membership requires restating QSL's slot derivation, which ADR-010 and FR-201's own closing paragraph leave to QSL. Bind the record to the request's subjects and premise entries QSL echoes, not to a local slot-membership rule. | spec/functional/FR-201-bind-composition-premises-by-kind.md:45 |

## Verdict

FAIL. FR-200, FR-202, FR-203 and FR-204 stay on the library side of ADR-010 (dispatch, binding, carrying, identity inclusion) and defer discharge, fixed point and assumption entailment to QSL/QSpec; FR-203 correctly computes no fixed point (G6). FR-201-AC-3 crosses the boundary.

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
| FND-001 | fixed | fef63ea (9fb5cea): Given a stand-in discharge record whose entries carry the roles QSL names, including `component-guarantee` and `fixed-point` entries, the disposition carries every entry byte-equal, refuses none of them, and the library derives no slot, role or premise list of its own. |
