---
id: FR-006
title: "Produce canonical protocol results"
type: FR
relationships:
  - target: ix://agent-ix/quire-protocol/US-002
    type: implements
  - target: ix://agent-ix/quire-protocol/FR-004
    type: depends_on
  - target: ix://agent-ix/quire-protocol/FR-005
    type: depends_on
  - target: ix://agent-ix/quire-specification/FR-061
    type: depends_on
  - target: ix://agent-ix/quire-specification/FR-240
    type: references
  - target: ix://agent-ix/quire-specification/FR-241
    type: references
  - target: ix://agent-ix/quire-specification/FR-242
    type: references
  - target: ix://agent-ix/quire-specification/FR-243
    type: references
  - target: ix://agent-ix/quire-specification/FR-244
    type: references
  - target: ix://agent-ix/quire-specification/FR-245
    type: references
  - target: ix://agent-ix/quire-specification/FR-246
    type: references
  - target: ix://agent-ix/quire-specification/FR-247
    type: references
  - target: ix://agent-ix/quire-specification/FR-248
    type: references
  - target: ix://agent-ix/quire-specification/FR-280
    type: references
  - target: ix://agent-ix/quire-specification/FR-281
    type: references
---
# FR-006: Produce canonical protocol results

## Description

When an assessment disposition is available, the result producer SHALL emit one
strict-reader-valid `quire.protocol.result/v1-draft.1` artifact that preserves
every orthogonal fact and exact identity required by source requirement FR-061.

## Inputs

One exact request, assessment dispositions, decision-scope progress/closure,
surrounding-execution progress/closure, the complete-global-conformance closure
record when global conformance is requested, completeness facts, assumptions,
decision support, adequacy, provenance, limitations, and applicable prior result.

## Outputs

One canonical protocol result plus its non-self-referential RFC 8785
`sha256-jcs` artifact reference, or one typed construction/strict-reader refusal.

## Behavior

The producer SHALL keep execution, decision-scope progress, decision-scope
closure, surrounding-execution progress, surrounding-execution closure,
complete-global-conformance closure, truth, settlement basis, activation,
participation, assumptions, completeness, observation adequacy, protocol
adequacy, claim, and provenance separate. For a global-conformance claim, the
complete-global-conformance closure record SHALL carry one state from
`FR-247-complete-global-conformance-closure-state` plus the exact required
execution, branch, and workflow closure-premise identities. No other closure axis, completed
assessment, settled truth, or completeness fact can substitute for that record
or one of its premises.
The strict reader SHALL enforce the complete FR-061 result-combination table,
recompute identities, and reject missing, cyclic, self-referential, or falsely
independent support.
When later input contradicts a retained premise, the producer SHALL emit a linked
superseding or invalidating result while preserving every byte and identity of
the earlier result.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-006-AC-1 | Every valid FR-061 result-combination row is accepted and every one-axis forbidden combination is refused. | Test (TC-006) |
| FR-006-AC-2 | Settled truth on an open subject requires exact complete decisive support whose truth is preserved by every admitted continuation. | Test (TC-006) |
| FR-006-AC-3 | Assumption, observation-adequacy, protocol-adequacy, execution, truth, activation, participation, decision-scope progress, decision-scope closure, surrounding-execution progress, surrounding-execution closure, complete-global-conformance closure, settlement, claim, and completeness mutations remain independently observable. | Test (TC-006) |
| FR-006-AC-4 | Every source, profile, model, protocol, request, binding, input, backend, decision-scope progress/closure, surrounding-execution progress/closure, complete-global-conformance closure, settlement, support, producer, and prior-result identity is recomputed or strictly matched. | Test (TC-006) |
| FR-006-AC-5 | Support lineage rejects missing predecessors, cycles, self-reference, and invented independence while retaining shared and derived relations. | Test (TC-006) |
| FR-006-AC-6 | A later contradiction emits a new linked result and cannot rewrite or reseal the earlier artifact. | Test (TC-006) |
| FR-006-AC-7 | Independently omitting, duplicating, cross-wiring, changing scope on, or changing the identity of every required execution, branch, and workflow global-closure premise refuses the record or prevents complete global success while leaving an independently settled per-obligation truth unchanged. | Test (TC-006) |

## Adopted vocabulary

Every axis this requirement keeps separate is an upstream enumeration. This
requirement does not restate their members; it requires them to stay distinct.

| Axis | Upstream object |
| --- | --- |
| Execution disposition | `FR-241-assessment-execution-disposition` |
| Truth | `FR-242-truth-disposition` |
| Settlement basis | `FR-243-settlement-basis` |
| Activation | `FR-244-activation-state` |
| Observation adequacy | `FR-245-observation-adequacy-state` |
| Protocol adequacy | `FR-246-protocol-adequacy-state` |
| Complete-global-conformance closure | `FR-247-complete-global-conformance-closure-state` |
| Decision support | `FR-248-decision-support-record` |
| Assumptions | `FR-280-assumption-state` |
| Claim strength | `FR-281-claim-strength` |

`FR-247`'s five members — `closed`, `open`, `incomplete`, `contradicted`,
`not-required` — match this requirement's previous local enumeration exactly,
including the `not-required` rule that a claim which is not global conformance
carries no global premise set. The local enumeration is therefore replaced by the
reference rather than reconciled.

### Where the upstream vocabulary does not yet reach

- **Canonical result identity** — `FR-240-canonical-protocol-result` records that
  "no requirement states whether the identity is authored by the producer or
  derived from named components", that no key order is established, and that
  "no requirement states whether two results bearing the same result identity and
  different canonical bytes are admissible".
- **The four progress/closure axes are unenumerated upstream.** FR-240 records:
  "no requirement declares a closed vocabulary. No member list is invented here."
  This requirement and IT-002-SC-03 both require decision-scope and
  surrounding-execution progress and closure to remain distinct and to be
  independently mutated. That obligation stands; the member lists it would be
  mutated across do not exist upstream.
- **Settlement basis membership is not stated uniformly upstream.**
  `FR-243-settlement-basis` carries a section headed "Recorded divergence:
  membership is not stated uniformly", opening: "Four requirements state this
  vocabulary and one of them lists only four members." Its Open gap is explicit
  about which: "FR-091 lists four members where FR-061, FR-094-AC-7 and FR-115
  list five. No requirement states whether FR-091's evaluator may emit
  `unavailable`. Both lists are quoted above; this artifact does not merge them."
  This requirement cites the axis, not a membership.
- **Participation is unenumerated at the source** (FR-240).

## Dependencies

- [FR-004](./FR-004-assess-finite-global-conformance.md).
- [FR-005](./FR-005-dispatch-independent-protocol-claims.md).
- `ix://agent-ix/quire-specification/FR-061`.
