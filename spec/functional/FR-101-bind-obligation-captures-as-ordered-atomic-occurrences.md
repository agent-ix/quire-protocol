---
id: FR-101
title: "Bind obligation captures as ordered atomic occurrences"
type: FR
relationships:
  - target: ix://agent-ix/quire-protocol/ADR-010
    type: references
  - target: ix://agent-ix/quire-protocol/US-001
    type: implements
  - target: ix://agent-ix/quire-protocol/FR-019
    type: extends
  - target: ix://agent-ix/quire-protocol/FR-045
    type: depends_on
  - target: ix://agent-ix/quire-protocol/FR-057
    type: references
  - target: ix://agent-ix/quire-specification/FR-172
    type: references
---
# FR-101: Bind obligation captures as ordered atomic occurrences

Closes V1-PROT-019 (immutable captures) for the assessment of an observed run. Extends FR-019, whose AC-2 refuses a substituted capture identity but does not state ordering, duplicates or atomicity. This requirement binds the captures of an obligation activation, supplied through the FR-479 binding and FR-045. It does not cover QSL's `Unsupported::Capture`, which is a capture of the protocol clause itself; FR-057 carries that cause unchanged as an `unsupported` disposition, in exploration and in assessment of an observed history alike. An obligation-activation capture is never reported under that cause.

## Description

When the evaluator records the captures of an activated obligation, the evaluator SHALL bind the values the owner supplies in authored order as immutable occurrences, each with its own identity, or refuse the activation with no capture recorded.

## Inputs

The activation of FR-019 with its owner-issued FR-479 binding, and for each authored capture the value or the unavailable/undefined outcome that QSL's checked-package evaluation (FR-045) returns at the inclusive activation boundary. The evaluator evaluates no capture expression itself. Each supply names the authored capture it answers, so a missing supply is attributed to its own capture.

## Outputs

An ordered capture list on the activation, or a typed refusal naming the undefined capture.

## Behavior

The evaluator SHALL keep duplicate captures as separate occurrences with one value each. The evaluator SHALL NOT change a bound value after activation. When any one capture is undefined, unavailable or without a supply, the evaluator SHALL refuse the whole activation.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-101-AC-1 | Supplied values are bound in authored order whatever order they are supplied in, and a later admitted fact changing a value's source leaves every bound value unchanged. | Test (TC-154) |
| FR-101-AC-2 | Two identical captures yield two occurrences with distinct identities. | Test (TC-154) |
| FR-101-AC-3 | One undefined capture among three refuses the activation, binds no capture, and names the undefined one. | Test (TC-154) |
| FR-101-AC-4 | The bound value is the one supplied for the inclusive activation boundary; a value supplied for any other boundary is refused. | Test (TC-154) |
| FR-101-AC-5 | Given a QSL-supplied capture value that differs from what the authored expression would yield if computed locally, the bound value equals the supplied one. | Test (TC-154) |

## Dependencies

- FR-019 supplies the activation and its identity axes; FR-045 supplies the evaluation outcome that this requirement binds without recomputing.
- FR-057 carries the `protocol capture` unsupported cause; QSpec FR-172 states the capture semantics.
