---
id: SR-2127022
title: "TL-6 delta FR-100..FR-102: dependency"
type: SpecReview
analysis: dependency
review_set: subset
scope: "agent-ix/quire-protocol; TL-6; PR #144 head 52c68cb9d9117964c734fb32d9bd8e3768942866; spec/functional/FR-100-settle-inclusive-deadline-branches-on-complete-progress.md; spec/functional/FR-101-bind-obligation-captures-as-ordered-atomic-occurrences.md; spec/functional/FR-102-carry-unsupported-protocol-constructs-as-typed-dispositions.md; spec/test-cases/TC-154-settle-deadline-captures-and-unsupported-constructs.md; spec/tests.md"
relationships:
  - { target: ix://agent-ix/quire-protocol/FR-100, type: reviews }
  - { target: ix://agent-ix/quire-protocol/FR-101, type: reviews }
  - { target: ix://agent-ix/quire-protocol/FR-102, type: reviews }
  - { target: ix://agent-ix/quire-protocol/TC-154, type: reviews }
---
## Summary

Ticket: TL-6. Dependency analysis of the new relationships edges (ADR-010, US-001, FR-019, FR-023, FR-024, FR-028, FR-002, QSpec FR-049, QSpec FR-172) and TC-154's verifies edges.

## Assurance Context

AP-001 (spec/assurance/AP-001-protocol-implementation.md) declares `review_policy.mode: require` but is `status: proposed`, so it is not an active profile and no enforced ix-flow run is claimed. Direct analyses run: base, integrity, ears-conformance, scope-boundary, dependency, failure-domain.

## Examined

- FR-100 (spec/functional/FR-100-settle-inclusive-deadline-branches-on-complete-progress.md)
- FR-101 (spec/functional/FR-101-bind-obligation-captures-as-ordered-atomic-occurrences.md)
- FR-102 (spec/functional/FR-102-carry-unsupported-protocol-constructs-as-typed-dispositions.md)
- TC-154 (spec/test-cases/TC-154-settle-deadline-captures-and-unsupported-constructs.md)

## Verdict

PASS WITH FINDINGS. Every target resolves; the QSpec FR-049 and FR-172 edges point at the clauses that actually own the rules (FR-049-AC-9, FR-172 AC-3/AC-5). One edge does not belong.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | FR-100 says it 'Extends FR-028, FR-019 and FR-023' and references FR-023, but FR-023 is attempt and effect distinctness under one compensation registration; nothing in FR-100 depends on it. Drop the edge, or replace it with FR-033 (late-contradiction lineage), which FR-100-AC-4 does depend on. | spec/functional/FR-100-settle-inclusive-deadline-branches-on-complete-progress.md:14, 21 |

## New findings (disposition pass 1)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-002 | low | FR-101 and TC-154 hand G7 coverage to FR-057 in prose only; FR-057 exists only on PR #143, so if #144 merges first main carries a dangling id and the TL-6 G7 claim rests on unmerged work. Acceptable only with the merge order #143 then #144; once FR-057 is on main, add a frontmatter `references` edge FR-101 -> FR-057 so quire resolves it. | spec/functional/FR-101-bind-obligation-captures-as-ordered-atomic-occurrences.md:19, 50 |

## Dispositions

Round 1, fix commit 2d40a60.

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | 2d40a60: Closes V1-PROT-019 (inclusive timeout branches). Extends FR-028; no existing requirement states how a deadline branch is selected. The settlement vocabulary is not new: the selected branch is carried with the FR-242 truth disposition and FR-241 execution disposition the owner already defines. |
| FND-002 | still-open | deferred to post-#143 by the coordinator: FR-101 still names FR-057 in prose only (FR-057 exists only on PR #143); add the frontmatter `references` edge FR-101 -> FR-057 once #143 is on main. |
