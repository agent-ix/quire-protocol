---
id: FR-028
title: "Keep progress and closure axes independent"
type: FR
relationships:
  - target: ix://agent-ix/quire-protocol/FR-004
    type: part_of
  - target: ix://agent-ix/quire-protocol/US-001
    type: implements
  - target: ix://agent-ix/quire-specification/FR-059
    type: depends_on
  - target: ix://agent-ix/quire-specification/FR-240
    type: references
  - target: ix://agent-ix/quire-specification/FR-241
    type: references
  - target: ix://agent-ix/quire-specification/FR-242
    type: references
  - target: ix://agent-ix/quire-specification/FR-243
    type: references
  - target: ix://agent-ix/quire-specification/FR-245
    type: references
  - target: ix://agent-ix/quire-specification/FR-246
    type: references
  - target: ix://agent-ix/quire-specification/FR-247
    type: references
  - target: ix://agent-ix/quire-specification/FR-258
    type: references
  - target: ix://agent-ix/quire-specification/FR-268
    type: references
---
# FR-028: Keep progress and closure axes independent

## Description

When any progress or closure fact changes, the evaluator SHALL retain decision-scope progress, decision-scope closure, surrounding-execution progress, surrounding-execution closure and complete-global-conformance closure as five independently readable facts.

## Inputs

The independently supplied decision-scope and surrounding-execution
progress/closure facts with their exact scope, authority and boundary
identities, the complete-global-conformance closure record with its exact
required execution, branch and workflow closure-premise identities, the
assessment-execution disposition, the input-completeness fact, and the
per-obligation truth dispositions.

## Outputs

A global result carrying each of the five axes separately, plus the exact cause
where complete global success is withheld.

## Behavior

The evaluator SHALL withhold complete global success unless the complete-global-conformance closure state is `closed` and every exact required execution, branch and workflow closure premise is present.
The evaluator SHALL NOT substitute a closed decision scope, a closed surrounding execution, a completed assessment execution, a settled per-obligation truth or a completeness fact for any closure axis.
The evaluator SHALL retain `not-required` explicitly, with no global premise set, for a claim that is not global conformance.
The evaluator SHALL leave an independently settled per-obligation truth unchanged by any state of the global closure record.

Closing one axis is evidence about that axis and about nothing else. This is the
one obligation in the FR-004 group that is about what the evaluator refuses to
conclude rather than about what it computes.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-028-AC-1 | Decision-scope progress/closure, surrounding-execution progress/closure and complete-global-conformance closure remain independent; changing or closing one cannot change or close another. | Test (TC-004) |
| FR-028-AC-2 | A refused, unsupported, resource-incomplete or failed assessment execution cannot become complete global success, and an open or incomplete closure axis cannot either. | Test (TC-004) |
| FR-028-AC-3 | Closing a decision scope neither closes the surrounding execution nor establishes complete global conformance. | Test (TC-004) |
| FR-028-AC-4 | A claim that is not global conformance carries `not-required` explicitly with no global premise set; a global premise supplied under `not-required`, a global claim marked `not-required`, and another progress/closure axis relabelled `not-required` are each refused. | Test (TC-004) |
| FR-028-AC-5 | Omitting, duplicating, cross-wiring or scope-mismatching a required global closure premise withholds complete global success without changing an independently settled per-obligation truth. | Test (TC-004) |
| FR-028-AC-6 | An absent progress, closure or completeness fact compares only to absence and is never defaulted to open, closed, complete or `not-required`. | Test (TC-004) |

## Adopted vocabulary

The five-axis independence rule is upstream's, stated by FR-059 and FR-061 and
typed by `FR-247`. This requirement adopts it and enumerates no axis member.

| Noun | Upstream object | Identity kind |
| --- | --- | --- |
| Complete-global-conformance closure state | `FR-247` | value |
| Decision scope and surrounding execution | `FR-258` | value, keyed (scope role, scope identity) |
| Assessment-execution disposition | `FR-241` | value |
| Truth disposition | `FR-242` | value |
| Settlement basis | `FR-243` | value |
| Observation adequacy state | `FR-245` | value |
| Protocol adequacy state | `FR-246` | value |
| Progress and late-data authority | `FR-268` | value, keyed (progress/assertion definition identity, revision, byte digest) |
| Canonical protocol result | `FR-240` | derived under FR-286 |

`FR-247` carries the whole of this requirement's positive rule and this
requirement restates none of it: its five members are adopted by reference, its
AC-3 is adopted as FR-028-AC-5, and its AC-4 as FR-028-AC-4. `FR-241` supplies
the reason FR-028-AC-2 covers `unsupported`, `failed` and `resource-incomplete`:
those are its members, and under FR-061 no state other than `completed` carries a
successful truth value.

`FR-268`'s key is why a closure fact cannot be re-derived locally: a change to
the progress/assertion definition identity, revision or byte digest is a
different authority, so this repository consumes the axis with its authority
retained and never recomputes it.

### The four scope axes are now declared

An earlier revision of this requirement recorded that four of the five axes had
no member list anywhere, and that `FR-028-AC-1`'s independent-variation
obligation was therefore unexecutable because "the value sets it would be
changed across exist for the fifth axis only".

`FR-284` closes that gap. Each of decision-scope progress, decision-scope
closure, surrounding-execution progress and surrounding-execution closure
carries exactly one of `open` or `closed`. The pair is one vocabulary serving
both scopes and progress; this requirement selects it and mints no spelling.

`FR-028-AC-1` is executable. The product over the four axes and the fifth
(`FR-247`, five members) is 2 x 2 x 2 x 2 x 5 = 80 states, each axis
independently variable, and `TC-004` enumerates all of them.

The labels an earlier revision of this repository had invented for those axes —
`not-started`, `advanced`, `stalled` and `unobserved` — are members of no
adopted vocabulary. They are withdrawn and are not reintroduced here.

### Accepted owner rulings

Quire Specification #29/#40 closes the previously open axes: FR-240 derives the
canonical result identity and refuses same identity/different bytes; FR-243
defines the five settlement bases; FR-284 defines independent `open`/`closed`
progress and closure; FR-285 defines participation; FR-289 maps half-open QObs
windows to inclusive native-temporal boundaries; and FR-295 defines
`complete`/`incomplete`/`contradicted` input completeness. Protocol imports
these types componentwise and never substitutes one axis for another.

## Dependencies

- [FR-004](./FR-004-assess-finite-global-conformance.md) is the parent this
  requirement is part of.
- `ix://agent-ix/quire-specification/FR-059` establishes the global closure
  record, its independence from decision-scope closure, surrounding-execution
  closure, assessment execution and completeness, and the rule that complete
  global success requires `closed` plus every exact required premise.
- `IT-002` scenarios SC-03 and SC-04 verify this requirement at the integration
  boundary, where the complete-global-conformance closure record is the
  Protocol-authored fact that no scoped closure may substitute for.
