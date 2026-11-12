---
id: SR-2127064
title: "TL-386 PR #153: code-review (rust-review lane)"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-protocol; PR #153 head 3269b80; src/compensation/activate.rs, tests/tc_003_compensation.rs, tests/tc_154_settle_deadline_captures.rs, Cargo.lock"
review_set: subset
date: 2026-11-12
---

## Summary

Ticket: TL-386. Method: code-review with the rust-review lane folded in. PR agent-ix/quire-protocol#153 head 3269b80; scope is `git diff origin/main...HEAD`.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | ChoiceVisibilityEvidence is documented as "read from QSL's checked accessor", but its fields are pub and from_checked takes any ChoiceVisibility value, which callers can build as a struct literal. tc_154 builds the evidence directly, and nothing in the crate calls qsl_package::protocol_view::choice_visibility. So the doc makes a promise the type does not enforce. Binding already exposes every field as pub, so this follows existing practice and is graded low. Either reword the doc as "the shape of" the accessor result, or make the fields private and build the evidence only from the accessor. | src/compensation/activate.rs:12-35 |
| FND-002 | low | Cargo.lock also moves quire-contract-model from a156efd to 270410b. The only change between those revisions is an additive CatalogReadError::new constructor and its unit test. Neither this crate nor QSL at 465bf2e calls it, so this part of the lockfile change is unrelated churn. The QSL move from 4f69234 to 465bf2e is required, because qsl_package::protocol_view::ChoiceVisibility does not exist at 4f69234. | Cargo.lock:1418 |

## Verdict

PASS WITH FINDINGS (two low; nothing blocks). The new axis is compared with derived PartialEq over choice, owner and the ordered visible set, and pushes IdentityAxis::ChoiceVisibility in a fixed order before Trigger, which matches the enum's declaration order. The cloned evidence is owned (Box<[CheckedNodeId]>), so it does not borrow from the checked package. The rust-review lane found no unsafe code, no panics in library code, no new unbounded work and no Tokio use, and every new public item has documentation. tc_003 adds ChoiceVisibility to the per-axis mutation loop: the count goes from 9 to 10, and refusal.axes() must equal exactly [ChoiceVisibility]. It also checks three visible-set substitutions (extra member, empty set, other member). A focused run of `cargo test --test tc_003_compensation activation` passed 2 of 2 tests at 3269b80. No full gate was run because the lead runs it.

Units examined:

- `src/compensation/activate.rs` (examined)
- `tests/tc_003_compensation.rs` (examined)
- `tests/tc_154_settle_deadline_captures.rs` (examined)
- `Cargo.lock` (examined)
- `qsl-package protocol_view::choice_visibility at 465bf2e` (context_only)

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | fc4a824: "pub struct ChoiceVisibilityEvidence { choice: CheckedNodeId, owner: CheckedNodeId, visible: Box<[CheckedNodeId]>, } ... pub fn read(node: &CheckedSemanticNodeV2) -> Result<Self, ViewRefusal> { choice_visibility(node).map(\|checked\| Self::from_checked(&checked)) }" -- fields private with getters; the only constructors take QSL's accessor or its result type |
| FND-002 | still-open | fc4a824 does not touch Cargo.lock: quire-contract-model is still moved a156efd -> 270410b, and nothing in the fix round (CheckedSemanticNodeV2, CheckedPackageReadLimits, admit) needs 270410b. Low and harmless. Either restore a156efd for that one entry, or the lead records accepted-no-change. |
