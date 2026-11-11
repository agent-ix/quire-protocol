---
id: SR-2127021
title: "TL-6 delta FR-100..FR-102: scope-boundary"
type: SpecReview
analysis: scope-boundary
review_set: subset
scope: "agent-ix/quire-protocol; TL-6; PR #144 head 52c68cb9d9117964c734fb32d9bd8e3768942866; spec/functional/FR-100-settle-inclusive-deadline-branches-on-complete-progress.md; spec/functional/FR-101-bind-obligation-captures-as-ordered-atomic-occurrences.md; spec/functional/FR-102-carry-unsupported-protocol-constructs-as-typed-dispositions.md; spec/test-cases/TC-154-settle-deadline-captures-and-unsupported-constructs.md; spec/tests.md"
relationships:
  - { target: ix://agent-ix/quire-protocol/FR-100, type: reviews }
  - { target: ix://agent-ix/quire-protocol/FR-101, type: reviews }
  - { target: ix://agent-ix/quire-protocol/FR-102, type: reviews }
  - { target: ix://agent-ix/quire-protocol/TC-154, type: reviews }
---
## Summary

Ticket: TL-6. Scope-boundary analysis against ADR-010 Decisions 1-3, AD-001 and CLAUDE.md's owner rule, checked against QSL origin/main 4ffe6b7e1 (qsl-eval/src/simulation/protocol.rs `Unsupported`, 14 variants) and QSpec origin/main 76efa51f (FR-049-AC-9, FR-172, FR-232).

## Assurance Context

AP-001 (spec/assurance/AP-001-protocol-implementation.md) declares `review_policy.mode: require` but is `status: proposed`, so it is not an active profile and no enforced ix-flow run is claimed. Direct analyses run: base, integrity, ears-conformance, scope-boundary, dependency, failure-domain.

## Examined

- FR-100 (spec/functional/FR-100-settle-inclusive-deadline-branches-on-complete-progress.md)
- FR-101 (spec/functional/FR-101-bind-obligation-captures-as-ordered-atomic-occurrences.md)
- FR-102 (spec/functional/FR-102-carry-unsupported-protocol-constructs-as-typed-dispositions.md)
- FR-101-AC-3 (spec/functional/FR-101-bind-obligation-captures-as-ordered-atomic-occurrences.md)
- FR-102-AC-1 (spec/functional/FR-102-carry-unsupported-protocol-constructs-as-typed-dispositions.md)

## Verdict

PASS WITH FINDINGS. All three FRs sit on the Protocol side of ADR-010 (assessment and carrying), and the G7 construct names match QSL's `Unsupported` variants. But FR-100 and FR-102 mint local vocabulary that has no owner, and FR-101 does not say who evaluates captures.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | FR-101's Inputs are 'its authored capture expressions, and the inclusive activation snapshot from the admitted history', which reads as Protocol evaluating capture expressions itself. FR-019 takes immutable captures from the owner-issued FR-300 binding (QSL v2 views) and FR-045 evaluates captures through QSL's checked-package slot evaluation; ADR-010 says the library keeps no copy of QSL's step semantics. Name the evaluation source and make FR-101 an obligation to retain and validate those captures (order, occurrence identity, atomic refusal). | spec/functional/FR-101-bind-obligation-captures-as-ordered-atomic-occurrences.md:23-27 |
| FND-002 | medium | FR-102 fixes a closed list of four constructs, each 'mapped to its own disposition', instead of carrying QSL's `Unsupported` cause unchanged under FR-241 `unsupported` (ADR-010 Decision 2 'Carrying', Decision 3). That mints four local disposition values with no owner, and the seven other `Unsupported` variants not assigned to TL-11 (Control, WeakMemory, AccessFootprint, UndecidedInitialGuard, OverNotAnObject, UnboundAttemptBinder, NodelessBinder) get no disposition at all. Require the cause to be carried unchanged for every variant, and keep the four G7 constructs as TC-154 fixtures. | spec/functional/FR-102-carry-unsupported-protocol-constructs-as-typed-dispositions.md:29-33 |
| FND-003 | medium | FR-100's Outputs mint a four-member settlement vocabulary ('response taken, timeout taken, pending, or incomplete') that no QSpec or local artifact owns, and that mixes branch selection with the FR-242 truth axis (`pending`) and an execution or completeness value (`incomplete`). Under the repository's owner rule, name the owner of each value or adopt existing ones (FR-242, FR-241, FR-295) plus the selected branch identity. | spec/functional/FR-100-settle-inclusive-deadline-branches-on-complete-progress.md:33 |

## Dispositions

Round 1, fix commit 2d40a60.

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | 2d40a60: The activation of FR-019 with its owner-issued FR-300 binding, and for each authored capture the value or the unavailable/undefined outcome that QSL's checked-package evaluation (FR-045) returns at the inclusive activation boundary. The evaluator evaluates no capture expression itself. |
| FND-002 | fixed | 2d40a60: Closes V1-PROT-019 (immutable captures) for the assessment of an observed run. Extends FR-019, whose AC-2 refuses a substituted capture identity but does not state ordering, duplicates or atomicity. The `protocol capture` construct that QSL's exploration does not step is a different path: it is carried unchanged as an `unsupported` disposition by FR-057 and is not covered here. |
| FND-003 | fixed | 2d40a60: For each waiting instance, keyed by that instance and not by the static branch, the selected branch (response or timeout) with its FR-242 truth disposition and FR-241 execution disposition, or no selection. |
