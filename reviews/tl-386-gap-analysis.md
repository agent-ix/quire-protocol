---
id: SR-2127065
title: "TL-386 PR #153: gap-analysis"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-protocol; PR #153 head 3269b80; FR-019-AC-2, src/compensation/activate.rs, tests/tc_003_compensation.rs"
review_set: subset
date: 2026-11-12
---

## Summary

Ticket: TL-386. Method: gap-analysis, scoped to the criterion this PR changed (FR-019-AC-2). Plan completion: not assessed.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | FR-019-AC-2 names three parts of the evidence: the choice, the owner and the visible set. No test substitutes the choice node, because choice_evidence always uses node("choice"). The mutation loop substitutes only the owner, and the extra loop substitutes only the visible set. A from_checked that dropped or misread the choice field would pass every test. Add a case that changes only the choice node and asserts refusal on [ChoiceVisibility]. | tests/tc_003_compensation.rs:84-91 |
| FND-002 | medium | FR-019-AC-2 says the evidence is "read from QSL's checked accessor", and spec/tests.md marks FR-019 as Implemented. No code or test calls qsl_package::protocol_view::choice_visibility. The test builds a ChoiceVisibility struct literal and passes it to from_checked, so nothing checks that the evidence comes from a checked choice node. Either exercise the accessor on a checked choice node in TC-003, or drop the accessor clause from the AC. | tests/tc_003_compensation.rs:83-91 |

## Verdict

PASS WITH FINDINGS (two medium). Substituting the owner and the visible set (extra member, empty set, other member) is tested, and each refuses with exactly [ChoiceVisibility]. The test carries the trace tag `#[trace("TC-003", "FR-019-AC-2", "FR-019-AC-5")]`. The production comparison in mismatches() covers all three fields. No production code was added without a requirement. Reordering the visible set is not tested; AC-2 does not require order sensitivity, so this is not counted as a finding.

Units examined:

- `FR-019-AC-2` (examined)
- `FR-019` (context_only)
- `src/compensation/activate.rs` (examined)
- `tests/tc_003_compensation.rs` (examined)

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | fc4a824: "evidence_of(&node(\"other\"), real.owner(), real.visible()), evidence_of(real.choice(), &node(\"other\"), real.visible()), evidence_of(real.choice(), real.owner(), &extra), ..." -- choice, owner and visible set are each substituted alone and must refuse with exactly [ChoiceVisibility] |
| FND-002 | fixed | fc4a824: "let choice = admitted.package().graph().nodes.iter().find(\|node\| control_operation(node) == Ok(ControlOperation::Choice)).expect(\"recipe R includes a choice control\"); ChoiceVisibilityEvidence::read(choice).expect(\"accessor reads the choice\")" -- the selected binding's evidence is read through qsl_package::protocol_view::choice_visibility on recipe R's admitted checked package |
