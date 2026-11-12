---
id: SR-2127061
title: "TL-9 PR #152: code-review (rust-review lane)"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-protocol; PR #152 head c492bdb; src/assume_guarantee.rs, src/dispatch.rs, src/lib.rs, tests/support/assume_guarantee.rs, tests/support/standin.rs, tests/tc_200_dispatch_composition_and_refinement.rs"
review_set: subset
date: 2026-11-12
---

## Summary

Ticket: TL-9. Method: code-review. PR agent-ix/quire-protocol#152 head c492bdb; scope is `git diff origin/main...HEAD`.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | assume_guarantee::bind takes no Declarations and checks nothing but a non-empty subject list. Unlike dispatch::bind (which refuses UndeclaredChannel/SubstitutedChannelPremise/UndeclaredRole), subjects, assumption identities and source identities are never checked against the checked package (FR-043/FR-044), and carry_refusal returns before any assumption comparison for a composition. A substituted or undeclared assumption identity on a composition request is forwarded and carried, so the 'substituting assumption identities refuses' half of FR-200-AC-5 is not implemented, not merely untested. | src/assume_guarantee.rs:242-252 |
| FND-002 | medium | AssumeGuaranteeRequest has no admitted-fragment field, though FR-200 Inputs name 'its admitted fragment' and Outputs require the disposition to retain 'its exact admitted fragment'. The fragment can be neither retained nor removed, so FR-200-AC-5's fragment case is unrepresentable; the test substitutes 'no subjects' for it. | src/assume_guarantee.rs:110-130 |
| FND-003 | medium | Decisiveness is read from a separate backend-stated is_decisive() bool, not from the label's FR-489 basis. A `proved` result (basis decisive-witness via read_composition_label) whose is_decisive() is false and whose request has no source identity is carried instead of refused NoSourceIdentity. The protocol path derives this from result.basis().is_decisive(); derive it from the label reading here too, or drop the duplicate accessor. | src/assume_guarantee.rs:144-151 |
| FND-004 | medium | ProviderBoundary is now generic over the claim K, so protocol claims and assume-guarantee claims need two boundary instances, each with its own QSL Registry. FR-200 says it 'introduces no separate dispatch path' and FR-290 selects from one registry; two registries can disagree, and neither dispatch_claims accepts a global-conformance or protocol claim beside a composition, so FR-029 cross-kind independence (FR-200-AC-3/AC-4, FR-204-AC-5) cannot be exercised through the library. One boundary with an enum claim (or one dispatch_claims over a combined ClaimRequest) closes this. | src/dispatch.rs:258-275 |
| FND-005 | medium | The FR-360/FR-489 label reading is a free function the caller must remember to call; dispatch returns the raw R and no disposition carries the reading. Nothing ties the composition table to composition results (a caller can read a refinement result with read_composition_label), and a backend's stated execution that contradicts its label (e.g. `unsupported` stated with `completed`, FR-360-AC-4) is silently replaced, not refused or surfaced. AC-5 says 'the disposition ... reads it as'. | src/assume_guarantee.rs:185-235 |
| FND-006 | medium | A result whose retained_premises() is None (it omits all four lists) is carried, while a refinement result that omits its assumptions is refused (AbsentAssumptions, 'absence compares only to absence'). Dropping every list therefore passes where dropping one is refused. Either refuse absence or state in FR-201 why lists may be omitted. | src/assume_guarantee.rs:310-315 |
| FND-007 | low | FR-204 Outputs require the refusal to name 'each extra assumption identity'; IdentityMismatch::Assumption carries only the first found (find). Same for Subject in FR-201. Return the full list of extras. | src/assume_guarantee.rs:323-328 |
| FND-008 | medium | The library does not enforce FR-203's 'no guarantee of the cycle available as an assumption, in the request or in the retained disposition'. The test only passes because cyclic() replaces the default assumptions {Head, Tail}, which are exactly the cycle's members, with {New}; with the default fixture the cycle guarantees would be forwarded and retained as assumptions unchecked. The assertion is vacuous with respect to library behaviour. | tests/tc_200_dispatch_composition_and_refinement.rs:530-538 |
| FND-009 | low | A new local enum IdentityMismatch (and DispositionRefusal::NoSubjects / RetainedPremiseList) shares the name of FR-008's closed `identity-mismatch` ProtocolCause (src/refusal/cause.rs) without being or mapping to it, and NoSubjects has no spec cause at all. Follows the pre-existing DispositionRefusal pattern, so low, but the specs say 'with the cause identity-mismatch' from FR-008's vocabulary. | src/assume_guarantee.rs:332-343 |
| FND-010 | low | assume_guarantee::dispatch duplicates dispatch::dispatch's bind -> select -> provider_mut -> settle_claim -> carry_refusal sequence line for line; only bind and carry_refusal differ. A shared generic driver taking those two as parameters removes the copy (and would make FND-004's single path easier). | src/assume_guarantee.rs:256-289 |

## Verdict

PASS WITH FINDINGS (one high). The ProviderBoundary generalisation is clean and the protocol path is behaviour-preserving (select() is an exact extraction). Label reading matches FR-360/FR-489 as now written in FR-202 and quire-specification main (failed takes the stated failed/resource-incomplete, `incomplete` derives nothing). Using qsl_replay::ItemLabel, which still has an Incomplete member, is correct: it is the owner's type and the code reads it into nothing as FR-202 requires; not a defect. Results are carried unchanged; no local discharge, ordering or graph construction. rust-review lane: no unsafe, no panics in library code, no new unbounded allocation beyond request-sized clones; Box<ProtocolOutcome> error in select() is reasonable. Blocking: FND-001 (composition binding validates no identity against the checked package).

Units examined:

- `src/assume_guarantee.rs` (examined)
- `src/dispatch.rs (ProviderBoundary<K,R>, select, DispositionRefusal)` (examined)
- `src/lib.rs` (examined)
- `tests/support/assume_guarantee.rs` (examined)
- `tests/support/standin.rs` (examined)
- `tests/tc_200_dispatch_composition_and_refinement.rs` (examined)
- `FR-200` (examined)
- `FR-201` (examined)
- `FR-202` (examined)
- `FR-203` (examined)
- `FR-204` (examined)

## New findings (disposition pass 1)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-011 | medium | bind() checks every subject, assumption and source against one Declarations, i.e. one checked package. FR-200 Inputs name 'the checked-package references (FR-043, FR-044) naming the component protocols'; if components live in separate checked packages, every other component's subject is refused UndeclaredSubject. Tests only use roles of one package as subjects. Confirm components share one package, or bind each subject against its own package's declarations. | src/assume_guarantee.rs:246-292 |
| FND-012 | low | One result type R must now implement both BackendResult and AssumeGuaranteeResult<T>, so a protocol-only caller must implement the assume-guarantee trait (tests/support/standin.rs grew 35 lines to do so), and dispatch::dispatch carries an unused T: Clone parameter. A per-kind result enum, or a Claim-indexed result, would keep each caller to its own trait. | src/dispatch.rs:468-481 |

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | 6eb955c: pub fn bind<T: Clone>(declarations: &Declarations, request: &AssumeGuaranteeRequest<T>) ... if let Some(node) = request.assumptions.iter().find(|node| !declarations.declares(node)) { return Err(DispositionRefusal::UndeclaredAssumption(node.clone())); } (fragment, subjects and sources likewise; 6eb955c src/assume_guarantee.rs) |
| FND-002 | fixed | 6eb955c: /// The admitted fragment the claim ranges over.     pub fragment: Fragment, ... if request.fragment.channels.is_empty() && request.fragment.roles.is_empty() { return Err(DispositionRefusal::NoFragment); } |
| FND-003 | fixed | 6eb955c: const fn is_decisive(label: ItemLabel) -> bool { matches!(label, ItemLabel::Proved | ItemLabel::Refuted) } ... if is_decisive(result.label()) && request.sources.is_empty() { return Some(DispositionRefusal::NoSourceIdentity); } |
| FND-004 | fixed | 6eb955c: /// The claim a registered provider is asked about: one boundary, registry and dispatch path serve every claim kind (FR-200). pub enum Claim<A, C, I, T> { Protocol(Binding<A, C, I>), AssumeGuarantee(AssumeGuaranteeBinding<T>) }  -- and ClaimRequest::AssumeGuarantee in dispatch::dispatch_claims |
| FND-005 | fixed | 6eb955c: pub fn label_reading(&self) -> Option<LabelReading> { ... match premises.request.kind { Composition => read_composition_label(result.label(), result.stated_execution()), Refinement => read_refinement_label(result.label()) } }  plus DispositionRefusal::ContradictoryExecution(result.label()) when the stated execution disagrees with the reading |
| FND-006 | fixed | 6eb955c: None if request.kind == AssumeGuaranteeKind::Composition => { return Some(DispositionRefusal::IdentityMismatch(IdentityMismatch::AbsentPremiseLists)); } |
| FND-007 | fixed | 6eb955c: let stronger = extra(retained, &request.assumptions); if !stronger.is_empty() { return Some(DispositionRefusal::IdentityMismatch(IdentityMismatch::Assumptions(stronger))); }  (Subjects(Vec<CheckedNodeId>) likewise) |
| FND-008 | fixed | 6eb955c: if let Some(node) = cycle_member(&request.cycles, &request.assumptions) { return Err(DispositionRefusal::CycleGuaranteeAsAssumption(node)); }  and the same check on the result's retained assumptions; FR-203-AC-3 test now offers a cycle member in the request and in the result and asserts both refusals |
| FND-009 | fixed | 6eb955c: pub const fn cause(&self) -> Option<ProtocolCauseKind> { match self { Self::IdentityMismatch(_) | Self::RetainedPremiseList(_) | Self::SubstitutedChannelPremise(_) => Some(ProtocolCauseKind::IdentityMismatch), _ => None } } |
| FND-010 | still-open | Not addressed in 6eb955c: assume_guarantee::dispatch still repeats dispatch::dispatch's bind -> select -> provider_mut -> settle_claim -> carry_refusal sequence. Low and behaviour-neutral now that both share one boundary, registry and select(); the lead may accept it without change. |
