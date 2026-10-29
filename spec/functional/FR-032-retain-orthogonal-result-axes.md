---
id: FR-032
title: "Retain orthogonal result axes"
type: FR
relationships:
  - target: ix://agent-ix/quire-protocol/FR-006
    type: part_of
  - target: ix://agent-ix/quire-protocol/FR-004
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
  - target: ix://agent-ix/quire-specification/FR-280
    type: references
  - target: ix://agent-ix/quire-specification/FR-281
    type: references
---
# FR-032: Retain orthogonal result axes

## Description

When a canonical protocol result is emitted, the result producer SHALL retain every axis `FR-240` lists as an independently readable fact.
When a canonical protocol result is read, the strict reader SHALL refuse every axis combination the FR-061 result-combination table forbids.
When a global-conformance claim is reported, the result producer SHALL carry one complete-global-conformance closure state from `FR-247` plus the exact required execution, branch and workflow closure-premise identities.

## Inputs

Assessment dispositions, decision-scope progress and closure, surrounding-execution
progress and closure, the complete-global-conformance closure record when global
conformance is requested, truth and settlement facts, activation, participation,
assumptions, completeness, observation adequacy, protocol adequacy, and the
requested claim with its retained strength.

## Outputs

One result whose axes are each separately readable, or one typed strict-reader
refusal naming the forbidden combination or the omitted axis.

## Behavior

Each axis in the **Adopted vocabulary** table below is retained separately. This
requirement restates none of their member lists; it requires them to stay
distinct and to be independently mutable.

No axis is populated from another. FR-061 owns the prohibition and this
requirement adopts it: no completeness, assessment-execution or other
progress/closure fact substitutes for any closure axis here. A strict reader
refuses an omitted required axis, a cross-scope substitution, a global premise
carried under `not-required`, and a global claim without its
complete-global-conformance closure record.

The two adequacy domains are disjoint. Under FR-061 neither adequacy domain is
used as evidence for the other. Observation adequacy and
protocol adequacy therefore remain two separately readable facts and neither
establishes the other.

A truth may settle while the subject remains open only with completed assessment
execution, exact complete decision support, and a decisive witness or
counterexample whose truth is preserved under every admitted continuation. A
missing fact inside the decision-support set makes that truth unavailable; a
missing fact outside it remains visible in completeness and does not falsify the
settled truth.

For a global-conformance claim, complete global success requires the `closed`
state together with the complete exact required premise set. No other closure
axis, completed assessment, settled truth, closed surrounding execution or
completeness fact substitutes for that record or for one of its premises. For a
claim that is not global conformance the record is explicitly `not-required` and
carries no global premise set; a supplied global premise, or another axis
relabelled `not-required`, is refused.

Each axis fact carries its own exact scope, authority and boundary identities.
Substituting one axis's scope or authority identity for another's refuses the
record.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-032-AC-1 | Every valid FR-061 result-combination row is accepted and every one-axis forbidden combination is refused. | Test (TC-006) |
| FR-032-AC-2 | Settled truth on an open subject requires exact complete decisive support whose truth is preserved by every admitted continuation. | Test (TC-006) |
| FR-032-AC-3 | Assumption, observation-adequacy, protocol-adequacy, execution, truth, activation, participation, decision-scope progress, decision-scope closure, surrounding-execution progress, surrounding-execution closure, complete-global-conformance closure, settlement, claim, and completeness mutations remain independently observable. | Test (TC-006) |
| FR-032-AC-4 | Neither adequacy domain establishes the other, and each remains separately readable after the other is mutated. | Test (TC-006) |
| FR-032-AC-5 | Independently omitting, duplicating, cross-wiring, changing scope on, or changing the identity of every required execution, branch, and workflow global-closure premise refuses the record or prevents complete global success while leaving an independently settled per-obligation truth unchanged. | Test (TC-006) |
| FR-032-AC-6 | A non-global claim carries `not-required` explicitly with no global premise set, and a supplied global premise or another axis relabelled `not-required` is refused. | Test (TC-006) |
| FR-032-AC-7 | Each retained axis fact's exact scope, authority and boundary identities are strictly matched, and substituting one axis's identity for another's refuses the record. | Test (TC-006) |

## Adopted vocabulary

Every axis this requirement keeps separate is an upstream object. This
requirement does not restate their members.

| Axis | Upstream object | Identity kind |
| --- | --- | --- |
| Execution disposition | `FR-241` | value |
| Truth | `FR-242` | value |
| Settlement basis | `FR-243` | value |
| Activation | `FR-244` | value |
| Observation adequacy | `FR-245` | value |
| Protocol adequacy | `FR-246` | value |
| Complete-global-conformance closure | `FR-247` | value |
| Assumptions | `FR-280` | value |
| Claim strength | `FR-281` | value |
| Result fact set | `FR-240` | derived under FR-286 |

`FR-247`'s five members — `closed`, `open`, `incomplete`, `contradicted`,
`not-required` — are adopted by reference, including the `not-required` rule that
a claim which is not global conformance carries no global premise set. An earlier
local enumeration of the same five was replaced by this reference rather than
reconciled with it.

### Accepted owner rulings

QSpec #29/#40 fixes the complete axis set: FR-284's four independent
`open`/`closed` progress/closure values, FR-295 completeness, FR-285
participation, FR-243's five settlement bases, FR-290 claim kinds, FR-281 claim
strength/cardinality and FR-286 result identity. The implementation consumes
those values through one shared internal domain module and never substitutes a
target E02 spelling or a local alias.

## Dependencies

- [FR-006](./FR-006-produce-canonical-protocol-results.md) is the parent this
  requirement is part of.
- [FR-004](./FR-004-assess-finite-global-conformance.md) supplies the
  global-conformance assessment whose closure record is carried here.
- `ix://agent-ix/quire-specification/FR-061` states the axis set, the
  no-substitution rule and the result-combination table.
