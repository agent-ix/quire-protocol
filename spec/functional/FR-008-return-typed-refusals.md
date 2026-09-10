---
id: FR-008
title: "Return typed source-bound refusals"
type: FR
relationships:
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
affected identity, location, cause chain, and partial-result policy are readable
without parsing display text.

## Inputs

One admission, representation, compensation, conformance, claim, result-reader,
or projection failure produced under an exact selected contract.

## Outputs

One `ProtocolRefusal` with a domain from `admission`, `representation`,
`assessment`, `result`, or `projection`; the stable source-owned code; affected
identities and locations; ordered typed causes; and an explicit partial-output state.

## Behavior

The library SHALL validate the refusal code against the selected domain contract
and retain an unknown future code only as `unsupported-source-contract` with its
uninterpreted original value. It SHALL NOT derive a code, identity, equality, or
retry decision from human-readable text. A fatal admission, representation,
result, or projection refusal SHALL emit no success value. An assessment request
may retain only the independent claim results expressly allowed by FR-005.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-008-AC-1 | Every public failure path returns the correct typed domain, stable source code, affected identity/location, ordered causes, and partial-output state. | Test (TC-011) |
| FR-008-AC-2 | Changing display text, locale, or cause rendering cannot change a refusal identity, code, equality comparison, or retry decision. | Test (TC-011) |
| FR-008-AC-3 | An unknown source revision or code becomes `unsupported-source-contract` and cannot be interpreted through a nearest known version. | Test (TC-011) |
| FR-008-AC-4 | Fatal admission, representation, result, and projection refusals emit no success value; only FR-005-authorized independent assessment results survive another claim's failure. | Test (TC-011) |

## Dependencies

- [FR-001](./FR-001-admit-selected-protocol-profile.md) through [FR-007](./FR-007-project-results-to-e02.md).
- Each selected upstream contract owns its stable domain-specific refusal codes.
