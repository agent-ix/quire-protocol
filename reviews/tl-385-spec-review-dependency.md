---
id: SR-2127017
title: "TL-385 ADR-010 protocol/QSL split: dependency"
type: SpecReview
analysis: dependency
review_set: subset
scope: "agent-ix/quire-protocol; TL-385; PR #141 head 2e751a81781f6b5ddc96ac578289db0372908f1c; spec/adr/ADR-010-consume-qsl-protocol-analysis-own-assessment-and-dispatch.md; spec/functional/FR-301-admit-protocol-refinement-dispatch.md; spec/assurance/AD-001-protocol-library-boundaries.md"
relationships:
  - { target: ix://agent-ix/quire-protocol/ADR-010, type: reviews }
  - { target: ix://agent-ix/quire-protocol/FR-301, type: reviews }
---
## Summary

Ticket: TL-385. Dependency review of the relationship edges PR #141 adds
(ADR-010's frontmatter, AD-001's new ADR-010 reference) and of FR-301's
dependencies after the amendment.

## Assurance Context

AP-001 is proposed, not active; direct analysis, no enforced run claimed.

## Verdict

**PASS WITH FINDINGS.** All local targets (AD-001, ADR-007, FR-002, FR-004,
FR-005, FR-029, FR-030, FR-301, FR-302) exist; QSL ADR-017, ADR-020 and ADR-027
exist on QSL main. Enablement (QSL engines, gaps G1..G7) is cleanly separated from
feature work (TL-6/9/10/11). Two low edge defects.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | ADR-010 declares `depends_on` QSL ADR-017, but the body never cites ADR-017; the boundary is grounded only in ADR-020 and ADR-027 | spec/adr/ADR-010-consume-qsl-protocol-analysis-own-assessment-and-dispatch.md:25-26 |
| FND-002 | low | FR-301 now dispatches to QSL's refinement backend but has no relationship edge to that owner (QSL ADR-020 or FR-142..FR-149) or to ADR-010; its only owner edges remain QSpec FR-176/FR-177, and ADR-010 is linked only in body prose | spec/functional/FR-301-admit-protocol-refinement-dispatch.md:1-28 |

## New findings (disposition pass 1)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-003 | low | FR-301's rewritten Inputs now take the checked-package references of FR-043 and FR-044 as the dispatched subjects, but FR-301's relationships carry no edge to FR-043 or FR-044 | spec/functional/FR-301-admit-protocol-refinement-dispatch.md:1-40, 54-60 |

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | ec54159 cites QSL ADR-017 in Decision 1 as the protocol and frame boundaries the library reads |
| FND-002 | fixed | ec54159 adds FR-301 edges to ADR-010 (references) and QSL ADR-020, FR-142, FR-144, FR-147 (depends_on); all exist on QSL main |
