---
id: FR-008
title: "Return typed source-bound refusals"
type: FR
relationships:
  - target: ix://agent-ix/quire-specification/FR-047
    type: depends_on
  - target: ix://agent-ix/quire-protocol/US-001
    type: implements
  - target: ix://agent-ix/quire-protocol/US-002
    type: implements
  - target: ix://agent-ix/quire-protocol/FR-001
    type: depends_on
  - target: ix://agent-ix/quire-protocol/FR-007
    type: depends_on
---
# FR-008: Return typed source-bound refusals

## Description

When any public operation cannot return its declared success value, the library SHALL
return one typed refusal envelope whose domain, source-owned stable code,
selected diagnostic-catalog identity/revision/digest, originating stage and
prerequisites, affected identity, available or explicitly unavailable location,
cause chain, and partial-result policy are readable without parsing display text.

## Inputs

One admission, representation, compensation, conformance, claim, result-reader,
or projection failure produced under an exact selected local contract, or one
accepted typed upstream failure response with its declared source contract and
diagnostic-catalog selection.

## Outputs

One `ProtocolRefusal` with a domain from `admission`, `representation`,
`assessment`, `result`, or `projection`; the stable source-owned code; affected
identities and locations; selected diagnostic-catalog identity, revision, and
digest; originating stage and prerequisites; ordered typed causes; and an
explicit partial-output state.

## Behavior

The library SHALL validate the refusal code against the exact selected versioned
diagnostic catalog before interpreting it. It SHALL retain an unknown contract or
future code only behind local `unsupported-source-contract`, with the original
catalog selection and original value uninterpreted. It SHALL NOT derive a code,
stage, identity, location, equality, or retry decision from human-readable text.
It SHALL retain the originating source-recognition, identity/linking,
type/profile, input-binding, capability-selection, or resource-execution stage
and every available prerequisite. A wrapper cannot erase or replace that stage.
Unavailable source context remains explicitly unavailable; the library SHALL NOT
reconstruct a span by searching display names, paths, or messages.

Typed causes SHALL keep missing and conflicting definitions, wrong anchors and
missing observations, unknown syntax and prohibited recognized constructs,
unproved definedness and runtime logical violations, and resource-incomplete and
unsupported outcomes distinct. When a source response contains independent
causes or dispositions, the refusal SHALL retain all of them in their declared
semantic order; an empty diagnostic-detail budget cannot erase their existence
or establish success. A fatal admission, representation, result, or projection
refusal SHALL emit no success value. An assessment request SHALL retain no
result except the independent claim results expressly allowed by FR-005.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-008-AC-1 | Every public failure path returns the correct typed domain, selected diagnostic-catalog identity/revision/digest, stable source code, originating stage and prerequisites, affected identity, available or explicitly unavailable location, ordered causes, and partial-output state. | Test (TC-011) |
| FR-008-AC-2 | Changing display text, locale, or cause rendering cannot change a refusal identity, code, equality comparison, or retry decision. | Test (TC-011) |
| FR-008-AC-3 | An unknown source revision, catalog selection, or code becomes local `unsupported-source-contract`, retains the original catalog and value uninterpreted, and cannot be interpreted through a nearest known version. | Test (TC-011) |
| FR-008-AC-4 | Fatal admission, representation, result, and projection refusals emit no success value; only FR-005-authorized independent assessment results survive another claim's failure. | Test (TC-011) |
| FR-008-AC-5 | Missing/conflicting definitions, wrong-anchor/missing-observation, unknown-syntax/prohibited-construct, unproved-definedness/logical-violation, and resource-incomplete/unsupported cause pairs remain structurally distinct. | Test (TC-011) |
| FR-008-AC-6 | Changing the selected diagnostic-catalog identity/revision/digest or erasing/replacing the originating stage refuses the response. | Test (TC-011) |
| FR-008-AC-7 | Unavailable source context remains explicit and cannot become an invented location through repeated display names, paths, or message text. | Test (TC-011) |
| FR-008-AC-8 | Simultaneous independent invalid and missing-input dispositions both remain represented under a zero-detail control and cannot produce overall success. | Test (TC-011) |

## Dependencies

- [FR-001](./FR-001-admit-selected-protocol-profile.md) through [FR-007](./FR-007-project-results-to-e02.md).
- `ix://agent-ix/quire-specification/FR-047` owns the native typed-cause and diagnostic-catalog semantics.
- Each selected upstream contract owns its stable domain-specific refusal codes.
