---
id: FR-027
title: "Report adverse observations without repair"
type: FR
relationships:
  - target: ix://agent-ix/quire-protocol/FR-004
    type: part_of
  - target: ix://agent-ix/quire-protocol/US-001
    type: implements
  - target: ix://agent-ix/quire-protocol/FR-011
    type: depends_on
  - target: ix://agent-ix/quire-specification/FR-059
    type: depends_on
  - target: ix://agent-ix/quire-specification/FR-210
    type: references
  - target: ix://agent-ix/quire-specification/FR-213
    type: references
  - target: ix://agent-ix/quire-specification/FR-220
    type: references
  - target: ix://agent-ix/quire-specification/FR-224
    type: references
  - target: ix://agent-ix/quire-specification/FR-226
    type: references
  - target: ix://agent-ix/quire-specification/FR-230
    type: references
  - target: ix://agent-ix/quire-specification/FR-260
    type: references
  - target: ix://agent-ix/quire-specification/FR-262
    type: references
---
# FR-027: Report adverse observations without repair

## Description

When an admitted observation is unexpected, missing, causally misordered, duplicated in effect or correlated to another workflow, the evaluator SHALL report it against the exact affected protocol obligation.

## Inputs

A linked finite protocol carrying its authored causal order, the admitted
observation fact set with each record's retained producer identities, and the
declared instance relationships the correlation service selected.

## Outputs

One adverse report per affected protocol obligation, naming that obligation's
exact identity and the adverse kind, alongside the unaltered admitted fact set
the report was derived from.

## Behavior

The evaluator SHALL NOT repair, reorder, deduplicate, re-correlate or discard an admitted observation in order to report it.
The evaluator SHALL name the affected obligation by its exact identity rather than by a display name, source path, span or message text.
The evaluator SHALL report a required absent valuation as incomplete or refused.
The evaluator SHALL NOT report a required absent valuation as false or as an implicit option-none.

Reporting is the whole obligation here. An adverse observation changes what the
assessment says and never what the admitted history contains.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-027-AC-1 | Unexpected, missing, causally misordered, duplicate-effect, and cross-workflow observations identify the exact affected obligation. | Test (TC-004) |
| FR-027-AC-2 | Every reported adverse observation is present and unaltered in the admitted fact set the evaluator assessed, and no adverse report removes, reorders, merges or re-correlates a record. | Test (TC-004) |
| FR-027-AC-3 | An adverse report names the affected obligation's exact identity, and changing only a display name, source path, span, message text, locale or rendering changes no report's subject. | Test (TC-004) |
| FR-027-AC-4 | A required absent valuation is reported as incomplete or refused and never as false or an implicit option-none. | Test (TC-004) |
| FR-027-AC-5 | Each adverse kind remains a distinct outcome; one adverse observation is never reclassified as another kind to make it reportable. | Analysis |

## Adopted vocabulary

Every subject this requirement reports about is upstream's. What this requirement
owns is the reporting discipline: attribute, never repair.

| Noun | Upstream object | Identity kind |
| --- | --- | --- |
| Protocol obligation | `FR-230-protocol-obligation` | derived, keyed (clause subject, semantic trigger-event identity) |
| Protocol subject kind discrimination | `FR-210-protocol-subject-vocabulary` | value |
| Authored causal (partial) order | `FR-220-causal-control-graph` | derived |
| Workflow instance | `FR-213-workflow-instance` | runtime subject under FR-287 |
| Delivery | `FR-224-delivery` | runtime subject under FR-287 |
| Business effect | `FR-226-business-effect` | derived, keyed (static declaration path, workflow instance identity, iteration identity) |
| Admitted observation | `FR-260-admitted-observation` | derived under FR-296 |
| Declared instance relationship | `FR-262-declared-instance-relationship` | value |

FR-027-AC-4 is adopted, not authored: `FR-260` records FR-110's rule that "A
required absent valuation SHALL be incomplete or refused, never false or an
implicit option-none."

### Accepted owner rulings

QSpec #28/#31/#40 supplies complete FR-287 workflow, delivery and effect keys,
FR-296 admitted-observation identity, and FR-262's closed `resolved`, `missing`,
`conflicting`, `ambiguous` relationship outcomes with ordered endpoint keys.
A repeated delivery of one send, a replay of one observation, a distinct
delivery and a duplicate business effect are therefore non-interchangeable.
Missing is incomplete; conflicting or ambiguous relationships refuse; unknown
producer/declaration input is an admission refusal. Protocol reports those
typed outcomes and never infers them from provider, trace, timestamp, payload,
path or arrival position.

## Dependencies

- [FR-004](./FR-004-assess-finite-global-conformance.md) is the parent this
  requirement is part of.
- [FR-011](./FR-011-consumer-identity-discipline.md) states the ordering
  obligation this requirement discharges by naming an owner.
- `ix://agent-ix/quire-specification/FR-059` establishes that the evaluator
  reports unexpected, missing, misordered, duplicated and wrongly correlated
  events without repairing them.
