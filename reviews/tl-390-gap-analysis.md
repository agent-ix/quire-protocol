---
id: SR-2127060
title: "TL-390 PR #151 gap-analysis"
type: SpecReview
analysis: gap-analysis
review_set: subset
scope: "agent-ix/quire-protocol; TL-390; PR #151 head e239071; spec/functional/FR-100-settle-inclusive-deadline-branches-on-complete-progress.md, spec/tests.md, src/observation_link.rs, src/deadline.rs, src/declarations.rs, tests/tc_154_settle_deadline_captures.rs"
relationships:
  - { target: ix://agent-ix/quire-protocol/FR-100, type: reviews }
---
## Summary

Ticket: TL-390. Gap analysis scoped to the PR diff: the new FR-100-AC-23/24, the FR-100 Inputs text, the tests.md row, and the code and tests that back them. Plan completion: not assessed.

## Examined

- FR-100 Inputs (spec/functional/FR-100-settle-inclusive-deadline-branches-on-complete-progress.md)
- FR-100-AC-23 (spec/functional/FR-100-settle-inclusive-deadline-branches-on-complete-progress.md)
- FR-100-AC-24 (spec/functional/FR-100-settle-inclusive-deadline-branches-on-complete-progress.md)
- spec/tests.md FR-100 row (AC-1..AC-24, TC-154)
- TL-390 AC1 (anchor from the checked await accessor; caller path removed) and AC2 (link verified; unlinked returns typed unavailable)

## Verdict

**PASS with low findings.** TL-390 AC1 is met: `await_control` reads `await_anchor` and no constructor takes an anchor or node from a caller. AC2 is met: `Basis::verify` refuses an unresolved binding, a non-anchor node and another instance with typed `Unlinked` causes. AC-23 and AC-24 each have a `#[trace]`-tagged TC-154 test, and the tests.md row is extended to AC-24. Response observations (`WaitFact::observation`) still carry no record and are outside TL-390's scope. The subject-key defect is recorded once, under code-review SR-2127059 FND-001.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | FR-100 Inputs says a `compensation` anchor links to the observation at the forward body's entry, and AC-23 names `after = {kind, target}`. Every test uses an `event` anchor, and nothing reads `AwaitAnchorDeclaration::kind`, so the compensation case is untested. | spec/functional/FR-100-settle-inclusive-deadline-branches-on-complete-progress.md:51,119; tests/tc_154_settle_deadline_captures.rs:1026-1041 |
| FND-002 | low | FR-100-AC-23 joins two claims: the anchor is read from QSL's accessor, and an observation is admitted only when the mapping resolves to the target. The second overlaps AC-24. Keep AC-23 to the anchor source. | spec/functional/FR-100-settle-inclusive-deadline-branches-on-complete-progress.md:119 |

## Dispositions

Round 1, fix commit 4f0d3f4 (reviewed as `git diff e239071..4f0d3f4`).

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | accepted-no-change | 4f0d3f4 drops the unread `kind` (the anchor is now the target `CheckedNodeId`). Linking treats both kinds the same way through the target, and no AC claims behaviour specific to compensation. A compensation-anchor fixture would exercise QSL's accessor rather than this library. |
| FND-002 | fixed | 4f0d3f4: AC-23 now covers only the anchor source, and AC-24 carries the link admission and every refusal cause. |
