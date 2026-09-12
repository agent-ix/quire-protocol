---
id: SR-037
title: "Base review of the version-explicit /2 protocol intake"
type: SpecReview
analysis: base
scope: "IT-001 (SC-06..SC-09), FR-001, FR-002, NFR-002, TM-001, Task-009"
review_set: all
---
## Summary

The base checklist was applied to the `/2` version-explicit consumer scenarios
added to IT-001 (SC-06 through SC-09) and to the requirement and matrix
artifacts they lean on. Scenario identifiers follow `{PARENT}-SC-N`, are
sequential and unique, and each is referenced exactly once from its numbered
procedure step; the enumerated refusal axes satisfy the error-path and
constraint-boundary rules at the scenario level. The substantive gaps are ownership gaps: three of the four new scenarios
discharge obligations that no acceptance criterion states, IT-001 declares no
verification edge to the non-functional requirement two of them exercise, TM-001
carries no row for the lane, and Task-009 still describes only SC-01 through
SC-05.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-001 | high | No acceptance criterion states the version-explicit intake obligation. FR-001-AC-1 through FR-001-AC-4 are written against "one accepted compiled-artifact contract" in the singular; none requires two published contract versions to be consumed through separate entry points, and none forbids auto-detection, upgrade or downgrade between them. SC-06 therefore verifies a requirement that was never written. | IT-001-SC-06; FR-001-AC-1..AC-4 | missing-requirement |
| FND-002 | medium | No acceptance criterion owns the temporal selection axes. Neither FR-001 nor FR-002 mentions a temporal profile, clock alternative, clock parameter, or temporal binding set, and FR-002-AC-3's binding-mutation list does not include missing, surplus, duplicate, reordered, foreign-owner or out-of-range temporal bindings. FR-003, FR-004 and IT-002 own temporal *facts* after admission, not temporal *selection* during it, so SC-08 has no owning criterion. | IT-001-SC-08; FR-001; FR-002-AC-3; FR-003; FR-004; IT-002 | missing-requirement |
| FND-003 | medium | IT-001's `relationships` block declares `verifies` edges to FR-001 and FR-002 only, and its Traceability section repeats exactly those two. SC-09 exercises declared resource limits at the exact bound, one short, and on fresh retry, which is NFR-002's subject matter, and SC-07's digest-domain separation is likewise not attributable to either declared target. The links are valid but incomplete. | IT-001 frontmatter; IT-001-SC-07; IT-001-SC-09; NFR-002 | correct-requirement-no-evidence |
| FND-004 | medium | The six coverage rules fail at the matrix. TM-001 has no row, status or note for the `/2` lane: its Integration Coverage table lists IT-001 once against FR-001 and FR-002, and its Task-009 note describes the producer-fixture smoke lane only. A reader of TM-001 cannot tell that SC-06 through SC-09 exist or that they are unexecuted. | TM-001; IT-001-SC-06..SC-09 | correct-requirement-no-evidence |
| FND-005 | medium | `Task-009-integrate-native-compiler.md` scopes the `/2` work out of the plan: its acceptance list names SC-01 through SC-05 and freezes "the accepted compiler artifact contract and fixture revisions", with no task covering a second contract version, the temporal selection axes, or the exact/one-short limit probes. AGENTS.md requires QUOIN `/spec-to-plan` before implementation, and the B-owned loader and corpus in `tests/support/v2/mod.rs` were written ahead of that step. | Task-009; AGENTS.md; tests/support/v2/mod.rs | missing-requirement |

## Remediation

FND-001 and FND-002 require new acceptance criteria on FR-001 and FR-002, or an
explicit owner elsewhere, before SC-06 and SC-08 can be claimed as verification
of anything. FND-003 requires the `verifies` edge set on IT-001 to name NFR-002.
FND-004 requires a TM-001 row that records the `/2` lane as planned and
unexecuted. FND-005 requires a plan task for the `/2` consumer path; per the
repository's no-destructive-rewrite rule that is a new task file, not an edit to
Task-009's acceptance list.

None of these are satisfied by the tests already written. The B-owned loader and
scenario corpus assert only B-side obligations and make no claim about A's
behaviour, so they neither discharge nor obscure the gaps above.

## Reconciliation after the analysis reviews

Five of the seven analyses (SR-038 FND-001, SR-039 FND-001, SR-041 FND-001,
SR-042 FND-001, SR-044 FND-001) independently read IT-001-SC-09 as inverting
NFR-002's inclusive maximum: they took "one unit short" to describe the *input*,
in which case an input below an inclusive bound must admit, not refuse.

They were reading an ambiguous clause. The obligation is a *budget* probe, the
same "exact/one-short budget dimension" the producer contract uses: hold one
input constant, grant exactly the budget it consumes on a limit dimension
(admits), grant one unit less (refuses), then resubmit the same bytes under a
fresh budget at the exact value (admits, proving no budget is carried across a
refused attempt). SC-09 and the Inputs section were rewritten to say that
explicitly, and `LimitBound::OneShort => RefusesBeforeLinking` in
`tests/support/v2/mod.rs` is correct under that reading and was not changed.

SR-038 FND-001's second point survives the rewrite only in part: under the
budget reading, the one-short budget *is* the over-bound control. NFR-002's own
input-size boundary remains TC-009's obligation, not IT-001's.

The remaining high findings are not closed by this change and are carried on
`Task-012`: the two-valued admits/refuses oracle (SR-038 FND-003), the
divergence between IT-001's declared selection set and the corpus enumerations
(SR-039 FND-004, SR-040 FND-006, SR-042 FND-004, SR-043 FND-003), the
unfalsifiable "`admit_and_link` unchanged" clause (SR-044 FND-003), and the fact
that A's published factoring does not match the `v2::` names IT-001 assumes
(SR-040 FND-001).
