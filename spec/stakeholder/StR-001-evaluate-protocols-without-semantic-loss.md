---
id: StR-001
title: "Evaluate native protocols without semantic loss"
type: StR
relationships:
  - target: ix://agent-ix/quire-protocol/FR-001
    type: satisfied_by
  - target: ix://agent-ix/quire-protocol/FR-004
    type: satisfied_by
  - target: ix://agent-ix/quire-protocol/FR-006
    type: satisfied_by
  - target: ix://agent-ix/quire-protocol/FR-008
    type: satisfied_by
---
# StR-001: Evaluate native protocols without semantic loss

## Stakeholder Need

Protocol verification owners SHALL receive one deterministic evaluation of the
selected native Quire protocol that preserves identity, compensation,
incompleteness, uncertainty, and result provenance.

## Rationale

Distributed workflows admit retries, duplicate delivery, partial recovery,
external participants, and incomplete observations. Collapsing those facts into
a Boolean or executing a different profile can falsely report conformance and
make later qualification irreproducible.

## Validation Criteria

| ID | Criteria | Validation |
| --- | --- | --- |
| StR-001-VC-1 | The implementation accepts only an immutable promoted source profile and preserves every identity and premise required by that profile. | Demonstration (IT-001) |
| StR-001-VC-2 | Conforming, violating, pending, incomplete, unsupported, failed, and resource-exhausted assessments remain independently visible. | Demonstration (IT-003) |
| StR-001-VC-3 | The canonical result and any E02 projection can be recomputed from their exact retained inputs without semantic loss. | Test (TC-006) |

## Stakeholders

The primary stakeholder is the protocol verification owner. Native compiler,
observation, temporal, Engineering Assurance, Quoin, and quire-verification
maintainers are interface owners and affected consumers.

## Context and Assumptions

The first slice is a finite profile. Its accepted source candidate, exact
dependency closure, producer interfaces, and license must exist before
implementation promotion.

## Stakeholder Constraints (Contextual)

The hot evaluation path and all qualification-facing executable logic must be
Rust. The implementation must reuse shared assurance services and avoid a local
runner or evidence framework.

## Dependencies

The need depends on the composed native Quire baseline and the independent E02
contract. It drives local admission, conformance, and canonical result behavior.

## Priority and Risk (Informative)

Priority is P0. A false conformance or erased incomplete state can authorize an
invalid production claim, so the impact is critical.

## Notes (Informative)

Advanced projection, realizability, refinement, and composition remain separate
future claims and do not gate finite replay.

## Traceability

FR-001, FR-004, FR-006, and FR-008 satisfy this stakeholder need; TM-001 records
planned evidence.
