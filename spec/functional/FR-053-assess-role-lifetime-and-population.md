---
id: FR-053
title: "Assess an observed history against role lifetime and population bound"
type: FR
relationships:
  - { target: ix://agent-ix/quire-protocol/US-001, type: implements }
  - { target: ix://agent-ix/quire-protocol/ADR-010, type: depends_on }
  - { target: ix://agent-ix/quire-protocol/FR-004, type: references }
  - { target: ix://agent-ix/quire-protocol/FR-027, type: references }
  - { target: ix://agent-ix/quire-protocol/FR-028, type: references }
  - { target: ix://agent-ix/quire-protocol/FR-041, type: references }
  - { target: ix://agent-ix/quire-specification/FR-170, type: references }
  - { target: ix://agent-ix/quire-specification/FR-171, type: references }
  - { target: ix://agent-ix/quire-spec-language/ADR-027, type: references }
  - { target: ix://agent-ix/quire-protocol/FR-050, type: depends_on }
---
# FR-053: Assess an observed history against role lifetime and population bound

## Description

When an admitted observation history is assessed against a linked protocol, the evaluator SHALL judge each observed role instance against its role's authored lifetime and population bound. The evaluator SHALL report each observed violation as a typed adverse observation.

## Inputs

A linked protocol with its role declarations (FR-050), and an admitted observation fact set.

## Outputs

Per role: the observed instance identities, whether each instance's observed activity lies inside its authored lifetime, and whether the number of simultaneously live instances stayed within the authored population bound; plus the typed adverse observations.

## Behavior

- The evaluator shall treat an instance identity as the identity the observation names.
- The evaluator shall mint none.
- The evaluator shall report activity by an instance before its spawn or after its retirement as a lifetime violation naming the instance and the observation.
- The evaluator shall report a live-instance count above the authored bound as a population violation naming the role, the bound and the observed count.
- The evaluator shall report.
- The evaluator shall not repair, a violation (FR-027)
- The evaluator shall give the same result under every permitted presentation order (FR-026).
- The evaluator shall keep the progress and closure axes independent of this assessment (FR-028).
- This requirement states no step semantics. QSL owns the exploration semantics (QSL ADR-027); this library assesses one admitted observation history (ADR-010 Decision 2).

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-053-AC-1 | Over a history whose instances all act inside their lifetimes and whose live count never exceeds the bound, the role assessment reports no violation. | Test (TC-051) |
| FR-053-AC-2 | Over histories with activity before spawn, activity after retire, and a retire that precedes its spawn, the assessment reports a lifetime violation per case naming the instance and observation. | Test (TC-051) |
| FR-053-AC-3 | Over a history reaching exactly the bound and one reaching the bound plus one live instance, the assessment reports no violation for the first and a population violation naming role, bound and count for the second. | Test (TC-051) |
| FR-053-AC-4 | Over a role with no authored population bound, the assessment reports no population violation and reports the bound as absent; it does not infer a bound from the history. | Test (TC-051) |
| FR-053-AC-5 | Permuting the presentation order of the history within the permitted orders changes no role assessment value. | Test (TC-051) |

## Dependencies

- [ADR-010](../adr/ADR-010-consume-qsl-protocol-analysis-own-assessment-and-dispatch.md) fixes what this library owns for dynamic roles, bounded queues, joins and loops, and what QSL owns.
- QSL ADR-027 states the step semantics this requirement does not restate; QSpec FR-170 and FR-171 state the thresholds and overflow rules the criteria cite.
