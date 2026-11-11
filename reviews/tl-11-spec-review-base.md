---
id: SR-2127018
title: "Base checklist review of TL-11 FR-050..FR-059 and TC-050..TC-055"
type: SpecReview
analysis: base
scope: "agent-ix/quire-protocol; spec/functional/FR-050..FR-059, spec/test-cases/TC-050..TC-055, spec/tests.md (PR #143, head 00836cf)"
review_set: subset
date: 2026-11-11
---

## Summary

Ticket: TL-11. Base checklist over the ten new FRs, their 39 acceptance criteria, TC-050..TC-055 and the spec/tests.md rows. IDs are well formed and inside ADR-010's TL-11 block (FR-050..FR-099), so they collide with none of the FR-100..FR-249 blocks; every AC has a TC row; ACs are concrete and mostly falsifiable. Two criteria require a source-text scan of the crate (checklist antipattern, high), the delivery-premise obligation of FR-054 has no criterion, and the tests.md totals sentence is now false.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-001 | high | Second clause 'no function of the crate generates a role instance identifier at link time' can only be checked by scanning crate source; the base checklist forbids test cases that scan source text. The first clause (linking twice yields equal representations) plus FR-050-AC-4 already constrain behaviour. Remove the second clause or restate it as an observable property of the linked representation (no instance-id field exists in it). | FR-050-AC-3; FR-050; TC-050; spec/functional/FR-050-link-role-declarations-without-minting-identity.md:43 | wrong-requirement |
| FND-002 | high | Inspection that 'no function of the crate checks an invariant, ranks a variant or constructs a termination argument; the crate's analysis surface names no such operation' is a source-text scan (TC-054: 'Inspect the crate's analysis surface'), which the base checklist forbids. FR-059-AC-1..AC-3 already pin the observable outcome (undischarged, unsupported, no proved status). Remove the criterion. | FR-059-AC-4; FR-059; TC-054; spec/functional/FR-059-keep-loop-invariant-and-variant-proof-outside-the-library.md:46 | wrong-requirement |
| FND-003 | medium | Behavior requires judging delivery order 'against the authored ordering key and the delivery premise', but no AC covers the delivery premise (duplication or loss permitted or forbidden by the premise) or an `unordered` channel; FR-054-AC-4 covers only the `fifo` key case. Add a criterion for a delivery-premise violation and for an `unordered` channel, or drop the premise from the obligation. | FR-054; FR-054-AC-4; TC-052; spec/functional/FR-054-assess-queue-capacity-and-overflow-outcome.md:37 | missing-requirement |
| FND-004 | low | spec/tests.md keeps 'Behavioural totals across these 40 rows: 229 acceptance criteria. ... 35 are planned ...' after adding 10 rows and 39 planned criteria, so the sentence is now false (50 rows, 268 criteria, 74 planned). The pre-existing functional_coverage header mismatch (quire exit 3, identical on main) is acceptable to leave to a separate fix, but a false count should be updated or deleted in this PR. | FR-050; spec/tests.md:209 | wrong-requirement |

## Verdict

Findings above must be fixed or dispositioned before merge. Units examined:

- `FR-050` (examined)
- `FR-051` (examined)
- `FR-052` (examined)
- `FR-053` (examined)
- `FR-054` (examined)
- `FR-055` (examined)
- `FR-056` (examined)
- `FR-057` (examined)
- `FR-058` (examined)
- `FR-059` (examined)
- `FR-050-AC-1` (examined)
- `FR-050-AC-2` (examined)
- `FR-050-AC-3` (examined)
- `FR-050-AC-4` (examined)
- `FR-051-AC-1` (examined)
- `FR-051-AC-2` (examined)
- `FR-051-AC-3` (examined)
- `FR-052-AC-1` (examined)
- `FR-052-AC-2` (examined)
- `FR-052-AC-3` (examined)
- `FR-053-AC-1` (examined)
- `FR-053-AC-2` (examined)
- `FR-053-AC-3` (examined)
- `FR-053-AC-4` (examined)
- `FR-053-AC-5` (examined)
- `FR-054-AC-1` (examined)
- `FR-054-AC-2` (examined)
- `FR-054-AC-3` (examined)
- `FR-054-AC-4` (examined)
- `FR-054-AC-5` (examined)
- `FR-055-AC-1` (examined)
- `FR-055-AC-2` (examined)
- `FR-055-AC-3` (examined)
- `FR-055-AC-4` (examined)
- `FR-055-AC-5` (examined)
- `FR-056-AC-1` (examined)
- `FR-056-AC-2` (examined)
- `FR-056-AC-3` (examined)
- `FR-057-AC-1` (examined)
- `FR-057-AC-2` (examined)
- `FR-057-AC-3` (examined)
- `FR-057-AC-4` (examined)
- `FR-058-AC-1` (examined)
- `FR-058-AC-2` (examined)
- `FR-058-AC-3` (examined)
- `FR-059-AC-1` (examined)
- `FR-059-AC-2` (examined)
- `FR-059-AC-3` (examined)
- `FR-059-AC-4` (examined)
