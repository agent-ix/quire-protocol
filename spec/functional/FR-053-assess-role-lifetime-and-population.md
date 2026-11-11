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

A linked protocol with its role declarations (FR-050), and an admitted observation fact set, given to the assessment as the library's `RoleFact` values: each names the observation (its position in the history's own order and its identity), the checked role node, the workflow instance the role instance belongs to, the role instance identity the observation names and one of spawn, act, retire, workflow end or scope end. A static role is never spawned: only its activity is a fact.

## Outputs

Per role: the observed instance identities, whether each instance's observed activity lies inside its authored lifetime, and whether the number of simultaneously live instances stayed within the authored population bound; plus the typed adverse observations.

## Behavior

- The evaluator shall treat an instance identity as the identity the observation names.
- The evaluator shall judge each role once for each workflow instance the history names: the authored `max` bounds the live instances of one workflow instance, and a workflow end ends only that workflow instance's role instances.
- The evaluator shall mint none.
- The evaluator shall take the end of a `workflow` lifetime at the observation that ends the workflow instance holding the role, the end of a `scope` lifetime at the observation of the step that QSL ADR-027 RR-4 ends it by (no running thread can reach a node the role owns, the effect of a compensation template it owns, or an open registration of such a template), and the end of an `until(p)` lifetime at the observed `retire` of the instance.
- The evaluator shall not evaluate `p`.
- The evaluator shall report activity by an instance before its spawn or after its retirement as a lifetime violation naming the instance and the observation.
- The evaluator shall report a live-instance count above the authored bound as a population violation naming the role, the bound and the observed count.
- The evaluator shall report each violation without repairing it (FR-027).
- The evaluator shall refuse a history that names a spawn, retire or end fact for a static role, which QSL never spawns, and return no assessment.
- The evaluator shall refuse a history that names a node that is not a declared role, naming the node, and return no assessment.
- The evaluator shall give the same result under every permitted presentation order (FR-026).
- The evaluator shall keep the progress and closure axes independent of this assessment (FR-028).
- This requirement states no step semantics. QSL owns the exploration semantics (QSL ADR-027); this library assesses one admitted observation history (ADR-010 Decision 2).

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-053-AC-1 | Over a history whose instances all act inside their lifetimes and whose live count never exceeds the bound, the role assessment reports no violation. | Test (TC-051) |
| FR-053-AC-2 | Over histories with activity before spawn, activity after retire, and a retire that precedes its spawn, the assessment reports a lifetime violation per case naming the instance and observation. | Test (TC-051) |
| FR-053-AC-3 | Over a history reaching exactly the bound and one reaching the bound plus one live instance, the assessment reports no violation for the first and a population violation naming role, bound and count for the second. | Test (TC-051) |
| FR-053-AC-4 | Over a static role, the assessment reports the bound as absent, reports no population violation and no lifetime violation for its activity, and does not infer a bound from the history. | Test (TC-051) |
| FR-053-AC-5 | Exchanging the positions of observations the authored causal order leaves concurrent (the spawns and the acts of different instances) changes no role assessment value, while moving an instance's act before its own spawn does. | Test (TC-051) |
| FR-053-AC-6 | Over one role per lifetime kind, activity after the observation that ends the workflow, after the observed end of a `scope` lifetime and after the observed `retire` of an `until(p)` instance is reported as a lifetime violation for `workflow`, `scope` and `until(p)` respectively, and activity before those observations is not. | Test (TC-051) |
| FR-053-AC-7 | Over a `scope` role that owns a compensation template, a compensation attempt observed while a registration of that template is open, or while a running branch can still reach its effect, is not reported as a lifetime violation. | Test (TC-051) |
| FR-053-AC-8 | Over an `until(p)` role and a history in which `p` is observed to hold and the instance acts before its observed `retire`, the assessment reports no lifetime violation and does not evaluate `p`. | Test (TC-051) |
| FR-053-AC-9 | A fact naming a node that is not a declared role refuses the history with the node named, and no role assessment is returned. | Test (TC-051) |
| FR-053-AC-10 | A spawn, retire or end fact for a static role refuses the history with the role named, and no role assessment is returned. | Test (TC-051) |
| FR-053-AC-11 | Over two workflow instances each holding as many live instances of a role as its `max`, the assessment reports no population violation for either; one more live instance in one workflow instance reports a population violation naming that workflow instance; and the end of one workflow instance ends none of the other's role instances. | Test (TC-051) |

## Dependencies

- [ADR-010](../adr/ADR-010-consume-qsl-protocol-analysis-own-assessment-and-dispatch.md) fixes what this library owns for dynamic roles, bounded queues, joins and loops, and what QSL owns.
- QSL ADR-027 states the step semantics this requirement does not restate; QSpec FR-170 and FR-171 state the thresholds and overflow rules the criteria cite.
