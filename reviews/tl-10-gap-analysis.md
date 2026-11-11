---
id: SR-2127042
title: "TL-10 gap-analysis review"
type: SpecReview
analysis: gap-analysis
review_set: subset
scope: "agent-ix/quire-protocol; TL-10; PR #148 head 9a7c29c; src/dispatch.rs, src/disposition.rs, tests/tc_160..tc_163, tests/support/standin.rs; FR-150..FR-153"
date: 2026-11-11
---
## Summary

Ticket: TL-10. Planless gap analysis of PR #148 at head 9a7c29c, scoped to `git diff origin/main...HEAD`. Plan completion: not assessed. `quoin matrix` reports 27 of the 29 FR-150..FR-153 criteria tagged, one binder each. FR-151-AC-7 and FR-152-AC-6 are untagged, which matches spec/tests.md.

## Verdict

PASS WITH FINDINGS. Every tagged criterion has a living, traced test that can fail, and there are no stubs or coverage inflation. Two tagged criteria are backed more weakly than their text says (FR-151-AC-4, FR-151-AC-5). The selected backend is not retained when the request names none. Two code paths have no owning criterion (`Disposition::execution`, `DecisiveWithoutWitness`).

Units examined: FR-150-AC-1..AC-9, FR-151-AC-1..AC-7, FR-152-AC-1..AC-6, FR-153-AC-1..AC-7 (examined); src/dispatch.rs, src/disposition.rs (examined).

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | FR-151-AC-4 is tagged, but role internality comes only from the requester's `Premises.roles` value. `bind` checks that the value is present, not that it agrees with the checked package. The test inserts `external` into the request and reads the same value back, so it would pass if the package declared the role internal. This is the same QSL read-view gap as FR-151-AC-7, so AC-4 should be narrowed or moved under AC-7. | tests/tc_161_bind_communication_and_visibility_premises.rs:157-172 |
| FND-002 | medium | FR-151-AC-5 needs a stand-in that returns QSL's `Inconclusive` verdict with an `InconclusiveCause`. The test sets a local `inconclusive: bool` and carries no `qsl_replay::InconclusiveCause`, although that owner type exists. `BackendResult::inconclusive() -> bool` cannot carry a cause, so carrying the cause unchanged is never exercised. | tests/tc_161_bind_communication_and_visibility_premises.rs:174-197 |
| FND-003 | medium | FR-150 Outputs and FR-030 require a disposition bound to the backend it was selected against. When the request names no backend and FR-290 yields one candidate, `dispatch` sends to `candidate.id()`, but `Binding` keeps only `request.backend == None`. No disposition records which backend produced it. | src/dispatch.rs:340-380 |
| FND-004 | medium | The new public `Disposition::execution` maps `RequiresBound` to `ResourceIncomplete`. closure.rs defines that as 'a resource ceiling stopped the assessment', but requires-bound means unbounded domains that need a bound. QSL treats it as a negotiation disposition. No criterion or test covers the mapping. | src/disposition.rs:186-196 |
| FND-005 | low | The `DecisiveWithoutWitness` refusal has no owning criterion and is exercised only inside the FR-152-AC-4 test. It also applies to `local-projection`, although FR-152 ties the trace-position witness to `realizability`, and it accepts a record whose `trace_position` is `None`. | src/dispatch.rs:386-400 |

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | Round 1, 72743ff: / FR-151-AC-4 / Given a requester premise set stating `external` for a role, the backend call carries `external` for it as stated and no premise implying that the role is fully observable. / Test (TC-161) / |
| FND-002 | fixed | Round 1, 72743ff: / FR-151-AC-5 / Given a stand-in that returns the `Inconclusive` verdict with a `BoundReached` `InconclusiveCause` and one that returns a requires-bound outcome, each disposition carries the outcome and the cause unchanged and holds neither truth `satisfied` nor truth `violated`, and the `Inconclusive` disposition holds truth `unavailable`. / Test (TC-161) / |
| FND-003 | fixed | Round 1, 72743ff: / FR-150-AC-10 / Given a request naming no backend and one stand-in serving the claim kind, the disposition records that stand-in as the backend selected, and a disposition settled without a selected backend records none. / Test (TC-160) / |
| FND-004 | fixed | Round 1, 72743ff: Disposition::execution removed from src/disposition.rs at 72743ff; no RequiresBound-to-ResourceIncomplete mapping remains. |
| FND-005 | fixed | Round 1, 72743ff: / FR-152-AC-7 / Given a stand-in returning a decisive `realizability` result with no witness, and a second whose witness record has no trace position, each disposition is refused under FR-030, and a decisive `local-projection` result with no witness is carried unchanged. / Test (TC-162) / |
