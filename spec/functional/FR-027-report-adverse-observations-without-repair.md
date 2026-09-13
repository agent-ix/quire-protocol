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
| Workflow instance | `FR-213-workflow-instance` | authored; key components not fixed upstream |
| Delivery | `FR-224-delivery` | not established upstream; see below |
| Business effect | `FR-226-business-effect` | derived, keyed (static declaration path, workflow instance identity, iteration identity) |
| Admitted observation | `FR-260-admitted-observation` | not established upstream; see below |
| Declared instance relationship | `FR-262-declared-instance-relationship` | value |

FR-027-AC-4 is adopted, not authored: `FR-260` records FR-110's rule that "A
required absent valuation SHALL be incomplete or refused, never false or an
implicit option-none."

### Where the upstream vocabulary does not yet reach

`FR-011`'s ordering obligation applies to this requirement directly: an object
whose collections admit `missing`, `surplus`, `duplicate` or `reordered` as
defects must declare a unique key and a total order over that key, or name the
owner that has not declared one. FR-027-AC-1 admits four of those four defects
over collections this repository does not own, so the owner is named here and no
key is supplied. The owner is `agent-ix/quire-specification` in every case below.

- **Admitted observation has no key and no order.** `FR-260` states that "the
  prose does not fix the key components" and that no key order follows, because
  the components are not fixed. Neither *missing* nor *duplicated* is decidable
  over a collection with no key.
- **Delivery has neither.** `FR-224` records under its open gap that "Delivery
  has neither a uniqueness key nor a total order in the prose, while three
  requirements each depend on one", and that "Every candidate component a reader
  might reach for — trace identifier, timestamp, payload, arrival position — is
  either explicitly excluded or explicitly denied causal force. Nothing is
  admitted in its place."
- **"Duplicated" has two readings and they need different comparisons.**
  `FR-224`'s open gap asks whether "duplicated" in FR-059 "means a second
  delivery of one send, which FR-054-AC-1 explicitly admits, or a second business
  effect, which FR-051-AC-2 refuses. The two readings require different
  comparisons." FR-004-AC-2 and FR-027-AC-1 both spell the local criterion
  `duplicate-effect`, which selects the second reading at the criterion; the
  upstream sentence the criterion discharges is unresolved between them, and
  this requirement does not resolve it on upstream's behalf.
- **A duplicate business effect is undetectable under the declared key.**
  `FR-226` states of its key that it "is constant across two effect claims at one
  static node within one workflow instance, so two duplicate effects compare
  equal under it and the duplicate the evaluator is required to detect is
  undetectable". FR-027-AC-1 requires the duplicate-effect case to name its
  affected obligation; the discriminator that case needs does not exist upstream.
- **A duplicate effect's disposition is contradicted upstream.** `FR-226` quotes
  both readings and merges neither: FR-051-AC-2 — "Relabeling a duplicate
  delivery as a second business effect refuses the binding or yields the authored
  duplicate-effect violation" — against FR-058-AC-2 — "A timed-out attempt,
  failed attempt and duplicate effect remain three distinct outcomes." A refusal
  of the binding and a retained distinct outcome are different results for the
  same observation. FR-027-AC-5 requires the kinds to stay distinct and states
  nothing about which of the two dispositions applies.
- **Cross-workflow correlation rests on an unfixed endpoint key.** `FR-262`
  states that "**The prose does not state how the endpoints themselves are
  keyed**", because it does not state a workflow instance's key
  (`FR-213-workflow-instance`, whose open gap opens "No requirement states what
  constitutes a workflow instance's identity; it states only what cannot stand in
  for it"). FR-027-AC-1's cross-workflow case therefore has no equality test for
  "another workflow".
- **The non-success correlation outcome vocabulary is named twice and not
  identically.** `FR-262` records FR-111's Description naming "unknown,
  conflicting, and ambiguous relationships" against FR-111-AC-2's "Missing,
  conflicting, and ambiguous required relationships", and states that "Whether
  *unknown* and *missing* name one outcome or two is not stated."

## Dependencies

- [FR-004](./FR-004-assess-finite-global-conformance.md) is the parent this
  requirement is part of.
- [FR-011](./FR-011-consumer-identity-discipline.md) states the ordering
  obligation this requirement discharges by naming an owner.
- `ix://agent-ix/quire-specification/FR-059` establishes that the evaluator
  reports unexpected, missing, misordered, duplicated and wrongly correlated
  events without repairing them.
