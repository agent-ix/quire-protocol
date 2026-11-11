---
id: SR-2127047
title: "TL-6 PR #149 spec delta: scope boundary"
type: SpecReview
analysis: scope-boundary
review_set: subset
scope: "agent-ix/quire-protocol; TL-6; PR #149 head ecdac92; spec/functional/FR-100-settle-inclusive-deadline-branches-on-complete-progress.md, spec/tests.md (spec delta only), src/closure.rs (FR-242 adoption)"
relationships:
  - { target: ix://agent-ix/quire-protocol/FR-100, type: reviews }
---
## Summary

Ticket: TL-6. Whether the delta's use of QSpec FR-241/FR-242/FR-061 vocabulary stays within what the owner states. Owner text read from quire-specification origin/main: FR-241, FR-242, FR-061, FR-049-AC-9, TC-174.

## Examined

- FR-100-AC-7 (spec/functional/FR-100-settle-inclusive-deadline-branches-on-complete-progress.md)
- FR-100 (spec/functional/FR-100-settle-inclusive-deadline-branches-on-complete-progress.md)
- FR-100 (spec/functional/FR-100-settle-inclusive-deadline-branches-on-complete-progress.md)

## Verdict

**FAIL.** 'No branch selected' for a false constraint is the owner's rule (FR-049-AC-9: timeout 'never substitutes for a false constraint'; TC-174). 'Truth `unavailable`' for a valueless constraint fits FR-242 (a decision-support fact is missing). Truth `unavailable` for a false constraint with closed progress and complete input is not stated by the owner, and the 'selected branch carries no truth' rationale is not what FR-242 says. Adopting `TruthDisposition` locally follows the FR-241 precedent and is acceptable: no Rust crate publishes FR-242 (checked the resolved dependency sources), and the labels are exact.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | For a false constraint with progress closed and completeness complete, AC-7 assigns truth `unavailable`. QSpec FR-242 defines `unavailable` as non-completed execution, inactive/unknown activation, or a missing required decision-support fact; none holds when the false constraint was observed and every input is complete. The owner (FR-049-AC-9, TC-174) states only that timeout does not substitute; it assigns no truth. The local spec is defining owner vocabulary semantics. Ask QSpec for the truth of a closed, complete wait whose only response is false (or ticket it there) rather than settling it here; the valueless half of AC-7 is consistent with FR-242. | spec/functional/FR-100-settle-inclusive-deadline-branches-on-complete-progress.md:67 |
| FND-002 | medium | The Outputs now say 'A selected branch carries no truth disposition, because an absent truth compares only to absence (FR-242)', reversing the previous Outputs ('the selected branch ... with its FR-242 truth disposition'). FR-242's absence clause only forbids defaulting absence to `unavailable`; it gives no reason a selection lacks a truth. QSpec FR-061 makes truth a required axis a strict reader refuses when omitted. If a branch settlement is not an FR-061 result (it is a control decision, not an obligation truth), say that, and drop the FR-242 rationale. | spec/functional/FR-100-settle-inclusive-deadline-branches-on-complete-progress.md:43 |
| FND-003 | low | src/closure.rs adopts the FR-242 vocabulary 'the way AssessmentExecution adopts FR-241', but spec.md §10.1, which registers the owner vocabularies this library uses, lists FR-241 and not FR-242. Record FR-242 there so the adoption has a spec statement. | spec/spec.md:157-166 |

## New findings (disposition pass 1)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-004 | medium | FR-100 records two questions for the quire-specification owner (the truth of a closed, complete wait whose only response is false; and whether a selected branch is an FR-061 result that carries a truth). No Linear ticket carries them to the owner (searched). This repo's CLAUDE.md treats an owner gap as 'stop and ask', and a paragraph without a ticket leaves the question unasked. File one ticket in the quire-specification owner's team covering both questions and cite it from the section. The local behaviour (no truth assigned) is conservative and fine meanwhile. | spec/functional/FR-100-settle-inclusive-deadline-branches-on-complete-progress.md:72-75 |

## Dispositions

Round 1, fix commit 183d4f1.

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | 183d4f1 |
| FND-002 | fixed | 183d4f1 |
| FND-003 | fixed | 183d4f1 |

Round 2, fix commit fb68003.

| FND | outcome | reason |
| --- | --- | --- |
| FND-004 | still-open | TL-389 now carries both questions and FR-100 cites it, but it is filed in the TL team (Temporal & Protocol, project Temporal Language Implementation), the consumer's own queue. quire-specification owner work lives in STD (for example STD-450, fixed by quire-specification PR 607). Move TL-389 to the STD team, or file it there and link it from TL-389. No branch change is needed. |

Round 3, fix commit 89325f1.

| FND | outcome | reason |
| --- | --- | --- |
| FND-004 | fixed | 89325f1 |
