---
id: SR-2127062
title: "TL-9 PR #152: gap-analysis"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-protocol; PR #152 head c492bdb; FR-200..FR-204 acceptance criteria vs tests/tc_200_dispatch_composition_and_refinement.rs and src/assume_guarantee.rs"
review_set: subset
date: 2026-11-12
---

## Summary

Ticket: TL-9. Method: gap-analysis. PR agent-ix/quire-protocol#152 head c492bdb; scope is `git diff origin/main...HEAD`.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | No finite global conformance claim is present; the test compares only the refinement disposition. Partly covered (coder-reported gap confirmed); blocked on code FND-004. | tests/tc_200_dispatch_composition_and_refinement.rs:108-130 |
| FND-002 | medium | Neither clause is exercised: no global conformance result or complete-global-conformance premise set appears; a refinement dispatch (labelled `_satisfied_beside` but answered Inconclusive) stands in. Partly covered at best. | tests/tc_200_dispatch_composition_and_refinement.rs:132-143 |
| FND-003 | high | Of the four named components only source identity is removed; 'backend' is exercised as an unknown backend (InvalidRequest, not a refusal), fragment is unrepresentable (code FND-002) and replaced by empty subjects, and assumption substitution is neither tested nor implemented (code FND-001). Substitution is tested for none. Coder's '3 of 4' overstates it. | tests/tc_200_dispatch_composition_and_refinement.rs:145-171 |
| FND-004 | low | The stand-in result has no slot representation, so 'returns that slot undischarged' is modelled as an empty discharge list with an Inconclusive label; the request has no supplied guarantee that would discharge a rely locally. Covers 'carries no success' only. | tests/tc_200_dispatch_composition_and_refinement.rs:285-294 |
| FND-005 | medium | The stand-in supplies no proved guarantee for a cycle member; it returns only Unsupported, so the AC's distinguishing condition (a proved member must not discharge the other member's premise) is never constructed. Duplicates FR-203-AC-1. | tests/tc_200_dispatch_composition_and_refinement.rs:617-633 |
| FND-006 | medium | No conformance claim beside the refused refinement; only the composition disposition is compared. Same root cause as code FND-004. | tests/tc_200_dispatch_composition_and_refinement.rs:749-781 |
| FND-007 | low | `premise-unproved` and `protocol-assumption-not-established` are fed through UnsupportedCause::unrecognised (the unknown-cause constructor) as Unsupported reports, so the recognised causes are not distinguished from the unknown one; per FR-489 `premise-unproved` arrives on an `inconclusive` result, not an unsupported one. | tests/tc_200_dispatch_composition_and_refinement.rs:355-385 |

## Verdict

PASS WITH FINDINGS (one high). Plan completion: not assessed. `quoin matrix` (0.29.4) shows all 32 FR-200..FR-204 criteria statically tagged by exactly one #[trace] test each (no run evidence recorded, as expected before the lead's gate); every binding names the AC it claims. Fully backed: FR-200-AC-1/2/6, FR-201-AC-1..4/6, FR-202-AC-1..3/5..7, FR-203-AC-1/2/5/6, FR-204-AC-1..4/6/7. Partly backed: FR-200-AC-3, FR-200-AC-4, FR-201-AC-5, FR-202-AC-4, FR-203-AC-3 (vacuous, see code FND-008), FR-203-AC-4, FR-204-AC-5. FR-200-AC-5 backs one of four components. No untraced production code beyond helpers.

Units examined:

- `FR-200-AC-1` (examined)
- `FR-200-AC-2` (examined)
- `FR-200-AC-3` (examined)
- `FR-200-AC-4` (examined)
- `FR-200-AC-5` (examined)
- `FR-200-AC-6` (examined)
- `FR-201-AC-1` (examined)
- `FR-201-AC-2` (examined)
- `FR-201-AC-3` (examined)
- `FR-201-AC-4` (examined)
- `FR-201-AC-5` (examined)
- `FR-201-AC-6` (examined)
- `FR-202-AC-1` (examined)
- `FR-202-AC-2` (examined)
- `FR-202-AC-3` (examined)
- `FR-202-AC-4` (examined)
- `FR-202-AC-5` (examined)
- `FR-202-AC-6` (examined)
- `FR-202-AC-7` (examined)
- `FR-203-AC-1` (examined)
- `FR-203-AC-2` (examined)
- `FR-203-AC-3` (examined)
- `FR-203-AC-4` (examined)
- `FR-203-AC-5` (examined)
- `FR-203-AC-6` (examined)
- `FR-204-AC-1` (examined)
- `FR-204-AC-2` (examined)
- `FR-204-AC-3` (examined)
- `FR-204-AC-4` (examined)
- `FR-204-AC-5` (examined)
- `FR-204-AC-6` (examined)
- `FR-204-AC-7` (examined)

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | 6eb955c: fn an_unsupported_composition_leaves_conformance_and_refinement_unchanged() ... vec![conformance(), Claims::AssumeGuarantee(request(&dynamic, COMPOSITION)), Claims::AssumeGuarantee(refinement)] ... assert_eq!(together[0], alone[0]); assert_eq!(together[2], alone[1]); |
| FND-002 | fixed | 6eb955c: fn a_conformance_result_neither_decides_nor_is_decided_by_a_composition() ... assert_eq!(beside[0], ClaimDisposition::GlobalConformance(CONFORMANT)); assert_eq!(guaranteed(beside[1].clone()), alone); |
| FND-003 | fixed | 6eb955c: broken_components: fragment removed, fragment substituted, assumption substituted, source substituted, backend substituted; each refused or invalid-request beside an unchanged intact claim; source removed -> NoSourceIdentity; assert_eq!(premises.selected, Some(BackendId::new("composer"))). Backend removal judged covered: the backend is FR-290's selection, not a caller-supplied component, so retaining the selected id is the testable half; substitution yields FR-290's invalid-request. |
| FND-004 | fixed | 6eb955c: asked.premises.relies = vec!["rely-1"]; ... let reading = outcome.label_reading().expect("a carried result"); assert_eq!(reading.truth, Some(Truth::Pending)); ... assert!(result.discharge.is_empty()); assert_eq!(premises.request.premises.relies, vec!["rely-1"]); |
| FND-005 | fixed | 6eb955c: let mut member = request(&dynamic, COMPOSITION); member.subjects = vec![cycle.cycles[0][0].clone()]; ... assert_eq!(member.label_reading().map(|r| r.truth), Some(Some(Truth::Satisfied))); ... assert!(matches!(disposition(cycle), Disposition::Unsupported(_))); |
| FND-006 | fixed | 6eb955c: vec![conformance(), Claims::AssumeGuarantee(request(&dynamic, REFINEMENT)), Claims::AssumeGuarantee(composition)] ... assert_eq!(beside[0], alone[0]); assert_eq!(beside[2], alone[1]); assert_eq!(refining.len(), 1); |
| FND-007 | still-open | 6eb955c now builds UnsupportedCause::RequestedCapability(CatalogCode::new("unsupported_projection", "premise-unproved")) and the same for protocol-assumption-not-established. That invents owner catalog codes in the test (repo CLAUDE.md: do not mirror owner vocabulary, not even in tests) and files both causes under the requested-capability kind, which is the mapping AC-4 forbids; per FR-489 `premise-unproved` arrives on an `inconclusive` result. Feed premise-unproved on an inconclusive result and the FR-148 cause as QSL spells it (or as unrecognised bytes) instead. Low. |
