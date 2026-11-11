---
id: SR-2127014
title: "TL-385 ADR-010 protocol/QSL split: integrity"
type: SpecReview
analysis: integrity
review_set: subset
scope: "agent-ix/quire-protocol; TL-385; PR #141 head 2e751a81781f6b5ddc96ac578289db0372908f1c; spec/adr/ADR-010-consume-qsl-protocol-analysis-own-assessment-and-dispatch.md; spec/spec.md"
relationships:
  - { target: ix://agent-ix/quire-protocol/ADR-010, type: reviews }
---
## Summary

Ticket: TL-385. Integrity (completeness, consistency, atomicity) review of the
ADR-010 id-block reservation and the spec.md amendments in PR #141.

## Assurance Context

AP-001 is proposed, not active; direct analysis, no enforced run claimed.

## Verdict

**PASS WITH FINDINGS.** The four blocks FR-050..099, 100..149, 150..199 and
200..249 are disjoint, cover exactly the 050-249 range TL-385 reserves, and do not
collide with existing local ids (FR-001..FR-047, FR-301, FR-302). The note that
FR-050..054 citations are QSpec ids is correct (FR-001 and ADR-007 cite
`ix://agent-ix/quire-specification/FR-050`). One medium inconsistency: spec.md's
identification policy still forbids the gaps the blocks create.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | spec.md section 6 says each namespace is allocated sequentially with no gaps, but ADR-010 Decision 5 reserves FR-050..FR-249 blocks (leaving FR-048/FR-049 unallocated) without amending that rule; the "highest allocated" line this PR edits still reads FR-042 and TC-017 although FR-047, FR-301, FR-302 and TC-147..TC-153 exist | spec/spec.md:104-110; spec/adr/ADR-010-consume-qsl-protocol-analysis-own-assessment-and-dispatch.md:143-157 |
