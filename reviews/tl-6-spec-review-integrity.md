---
id: SR-2127019
title: "TL-6 delta FR-100..FR-102: integrity"
type: SpecReview
analysis: integrity
review_set: subset
scope: "agent-ix/quire-protocol; TL-6; PR #144 head 52c68cb9d9117964c734fb32d9bd8e3768942866; spec/functional/FR-100-settle-inclusive-deadline-branches-on-complete-progress.md; spec/functional/FR-101-bind-obligation-captures-as-ordered-atomic-occurrences.md; spec/functional/FR-102-carry-unsupported-protocol-constructs-as-typed-dispositions.md; spec/test-cases/TC-154-settle-deadline-captures-and-unsupported-constructs.md; spec/tests.md"
relationships:
  - { target: ix://agent-ix/quire-protocol/FR-100, type: reviews }
  - { target: ix://agent-ix/quire-protocol/FR-101, type: reviews }
  - { target: ix://agent-ix/quire-protocol/FR-102, type: reviews }
  - { target: ix://agent-ix/quire-protocol/TC-154, type: reviews }
---
## Summary

Ticket: TL-6. Integrity analysis of FR-100..FR-102 and TC-154 against each other and against FR-002, FR-019, FR-023, FR-028 and ADR-010.

## Assurance Context

AP-001 (spec/assurance/AP-001-protocol-implementation.md) declares `review_policy.mode: require` but is `status: proposed`, so it is not an active profile and no enforced ix-flow run is claimed. Direct analyses run: base, integrity, ears-conformance, scope-boundary, dependency, failure-domain.

## Examined

- FR-100 (spec/functional/FR-100-settle-inclusive-deadline-branches-on-complete-progress.md)
- FR-100-AC-1 (spec/functional/FR-100-settle-inclusive-deadline-branches-on-complete-progress.md)
- FR-100-AC-2 (spec/functional/FR-100-settle-inclusive-deadline-branches-on-complete-progress.md)
- FR-100-AC-3 (spec/functional/FR-100-settle-inclusive-deadline-branches-on-complete-progress.md)
- FR-100-AC-4 (spec/functional/FR-100-settle-inclusive-deadline-branches-on-complete-progress.md)
- FR-100-AC-5 (spec/functional/FR-100-settle-inclusive-deadline-branches-on-complete-progress.md)
- FR-101 (spec/functional/FR-101-bind-obligation-captures-as-ordered-atomic-occurrences.md)
- FR-101-AC-1 (spec/functional/FR-101-bind-obligation-captures-as-ordered-atomic-occurrences.md)
- FR-101-AC-2 (spec/functional/FR-101-bind-obligation-captures-as-ordered-atomic-occurrences.md)
- FR-101-AC-3 (spec/functional/FR-101-bind-obligation-captures-as-ordered-atomic-occurrences.md)
- FR-101-AC-4 (spec/functional/FR-101-bind-obligation-captures-as-ordered-atomic-occurrences.md)
- FR-102 (spec/functional/FR-102-carry-unsupported-protocol-constructs-as-typed-dispositions.md)
- FR-102-AC-1 (spec/functional/FR-102-carry-unsupported-protocol-constructs-as-typed-dispositions.md)
- FR-102-AC-2 (spec/functional/FR-102-carry-unsupported-protocol-constructs-as-typed-dispositions.md)
- FR-102-AC-3 (spec/functional/FR-102-carry-unsupported-protocol-constructs-as-typed-dispositions.md)
- FR-102-AC-4 (spec/functional/FR-102-carry-unsupported-protocol-constructs-as-typed-dispositions.md)
- TC-154 (spec/test-cases/TC-154-settle-deadline-captures-and-unsupported-constructs.md)

## Verdict

PASS WITH FINDINGS. No AC contradicts FR-019, FR-023, FR-028 or FR-002 outright. FR-101 and FR-102 disagree about captures without saying why, and the three Dependencies sections were copied rather than written.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | FR-102 lists 'protocol capture' (QSL Unsupported::Capture) as an unsupported construct carried as a non-success disposition, while FR-101 in the same PR requires the evaluator to bind ordered immutable captures. Neither FR states that FR-102 applies only to claims dispatched to QSL's protocol-system exploration and FR-101 to the assessment of an observed activation, so a reader can conclude that every protocol with a capture is unsupported. State the path each FR governs. | spec/functional/FR-102-carry-unsupported-protocol-constructs-as-typed-dispositions.md:29 |
| FND-002 | low | All three FRs carry the same Dependencies line 'FR-002, FR-019, FR-028 as named in the relationships', but FR-101's relationships name FR-019, FR-024 and QSpec FR-172 (not FR-002 or FR-028), FR-100's omit FR-002 and add FR-023, and FR-102's omit FR-019. Write each Dependencies list from its own relationships. | spec/functional/FR-101-bind-obligation-captures-as-ordered-atomic-occurrences.md:48 |

## Dispositions

Round 1, fix commit 2d40a60.

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | still-open | FR-102 is gone, but the capture tension moved rather than closed. FR-101 now says the `protocol capture` construct 'is a different path' (exploration) carried by FR-057, yet FR-057 Behavior (PR #143 head 00836cf, FR-057) applies the same `Capture` disposition 'when the assessment of an observed history meets a construct for which QSL would report an Unsupported cause', which is exactly the path FR-101 governs. Either FR-101 separates the two by construct (QSL `Unsupported::Capture` is 'a capture of the protocol clause'; FR-101 binds obligation activation captures through FR-300/FR-045), or FR-057 excludes obligation captures. The two PRs must agree. |
| FND-002 | fixed | 2d40a60: - FR-019 supplies the activation and its identity axes; FR-045 supplies the evaluation outcome that this requirement binds without recomputing. |
| FND-001 | fixed | ddde4c0 (round 2): Closes V1-PROT-019 (immutable captures) for the assessment of an observed run. Extends FR-019, whose AC-2 refuses a substituted capture identity but does not state ordering, duplicates or atomicity. This requirement binds the captures of an obligation activation, supplied through the FR-300 binding and FR-045. It does not cover QSL's `Unsupported::Capture`, which is a capture of the protocol clause itself; FR-057 carries that cause unchanged as an `unsupported` disposition, in exploration and in assessment of an observed history alike. An obligation-activation capture is never reported under t |
