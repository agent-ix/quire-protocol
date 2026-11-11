---
id: SR-2127013
title: "TL-385 ADR-010 protocol/QSL split: base"
type: SpecReview
analysis: base
review_set: subset
scope: "agent-ix/quire-protocol; TL-385; PR #141 head 2e751a81781f6b5ddc96ac578289db0372908f1c; spec/adr/ADR-010-consume-qsl-protocol-analysis-own-assessment-and-dispatch.md; spec/functional/FR-301-admit-protocol-refinement-dispatch.md; spec/test-cases/TC-147-admit-protocol-refinement-dispatch.md; spec/assurance/AD-001-protocol-library-boundaries.md; spec/spec.md"
relationships:
  - { target: ix://agent-ix/quire-protocol/ADR-010, type: reviews }
  - { target: ix://agent-ix/quire-protocol/FR-301, type: reviews }
  - { target: ix://agent-ix/quire-protocol/TC-147, type: reviews }
  - { target: ix://agent-ix/quire-protocol/AD-001, type: reviews }
---
## Summary

Ticket: TL-385. Base checklist review of quire-protocol PR #141, diff
origin/main...HEAD only (five spec files). `quire validate --scope .` exits 0 on
each of the five changed files, including FR-301, AD-001 and TC-147. The
repository-wide `spec/**/*.md` run reports the same 11 structural failures on
this head as on origin/main (FR-012..FR-018 value-object records, FR-016 cells,
spec/tests.md column header); none is in a changed file.

## Assurance Context

AP-001 (spec/assurance/AP-001-protocol-implementation.md) declares
`review_policy.mode: require` for spec-review but is `status: proposed`, so it is
not an active applicable profile; no enforced ix-flow review run is claimed.
These are direct analyses: base, integrity, scope-boundary, ears-conformance and
dependency, selected because the PR adds a boundary ADR, edits an FR statement and
AC, amends cross-artifact text and adds relationship edges.

## Verdict

**PASS WITH FINDINGS.** ADR-010's claims about QSL were checked against
quire-spec-language origin/main 4ffe6b7 and hold (see the scope-boundary
review). FR-id blocks FR-050..FR-249 match the ticket's reservation. The FR-301,
AD-001 item 7, spec.md scope bullet and TC-147 amendments agree with each other on
the dispatch-and-carry boundary. Two medium gaps remain: TC-147 does not verify
the new "carried unchanged" obligation, and spec/tests.md still says FR-176/FR-177
are implemented here under quire-protocol#81.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | FR-301-AC-3 now requires the QSL backend's result to be carried unchanged and forbids a local product construction, but TC-147's procedure and expected results contain no step that compares the carried result, counterexample or certificate with the backend's output, and the "builds no product construction" clause has no failing test (Verification: Test) | spec/functional/FR-301-admit-protocol-refinement-dispatch.md:89; spec/test-cases/TC-147-admit-protocol-refinement-dispatch.md:20-36 |
| FND-002 | medium | spec/tests.md FR-301 and TC-147 rows still say refinement awaits `agent-ix/quire-protocol#81` landing FR-176/FR-177 here, contradicting amended FR-301 ("QSL implements them"; this repository owns only dispatch and carrying) | spec/tests.md:181; spec/tests.md:272 |
| FND-003 | low | ADR-010 Decision 4 and Consequences name "spec.md section 5" and "AD-001 view 7"; the amended text is in spec.md section 2 (Scope, out-of-scope list) and AD-001's Decisions list item 7, not a view | spec/adr/ADR-010-consume-qsl-protocol-analysis-own-assessment-and-dispatch.md:136-139, 180 |
