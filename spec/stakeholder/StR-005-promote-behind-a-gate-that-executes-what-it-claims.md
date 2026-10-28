---
id: StR-005
title: "Promote behind a gate that executes what it claims"
type: StR
relationships:
  - target: ix://agent-ix/quire-protocol/NFR-003
    type: satisfied_by
  - target: ix://agent-ix/quire-protocol/NFR-004
    type: satisfied_by
---
# StR-005: Promote behind a gate that executes what it claims

## Stakeholder Need

The promotion owner SHALL read the repository's gate as an enumerated set of commands, each naming the configuration it covers and the risk it detects.

The promotion owner SHALL reject any gate step that reports success over a configuration the crate cannot express.

The shared assurance platform owners SHALL remain the only implementers of producer execution and retained evidence for this repository.

## Rationale

The promotion decision for this library is taken on the strength of its gate.
A gate step that cannot execute, or that executes over a configuration the crate
does not have, converts a green result into an untrue assurance claim — and an
untrue green is worse than a missing check, because the missing check is visible.
The owner needs the written gate and the executed gate to be the same object, so
that reading the instructions tells them what was actually protected.

The platform half of the need has the same shape one level out. If this
repository grows its own producer runner, its own evidence store, or its own
assurance surface in a second language, then the shared platform's evidence and
this repository's evidence stop being comparable, and the owner reviewing both is
reading two models that look alike and are not. Reuse here is not an efficiency
preference; it is what makes one repository's evidence legible to the party who
assures all of them.

## Validation Criteria

| ID | Criteria | Validation |
| --- | --- | --- |
| StR-005-VC-1 | Every gate step named in the repository's own instructions executes against the crate as it stands, and every step the instructions name is present in the enumerated gate. | Demonstration (TC-012) |
| StR-005-VC-2 | No gate step claims coverage of a configuration the crate cannot express, and the count of configuration lanes run equals the count of distinct unit graphs the crate has. | Inspection (TC-012) |
| StR-005-VC-3 | Every first-party production and qualification path in the repository is implemented in one language, with no transcript scraping and no second evidence model. | Inspection (TC-010) |
| StR-005-VC-4 | Producer execution and retained evidence are reached only through the shared platform owners' typed interfaces, and no local boundary stores, indexes, or answers questions from retained evidence. | Analysis (TC-010) |

## Stakeholders

The primary stakeholder is the promotion owner for this repository, recorded in
AD-001 as `agent-ix-protocol-owner`, who decides whether an implementation may be
promoted. Engineering Assurance is an affected party as the owner of producer
execution and of the shared language-boundary and review gates; Quoin is an
affected party as the owner of retained evidence. Reviewers reading this
repository's gate alongside other repositories' gates are affected consumers.

## Context and Assumptions

The repository is specification-only at this need's anchoring, so several gate
steps detect little today. It is assumed that a step's risk coverage is expected
to grow with the suite, and that this is stated rather than left for a reader to
infer. It is assumed the shared platform publishes the typed interfaces this need
requires; that publication is an external obligation, not an assumption of fact.

## Stakeholder Constraints (Contextual)

Build and qualification workflows remain manual dispatch. No hosted status
stands as evidence for the local gate. The existing CLA administration workflow
retains its separate triggers, and heavy local activity shares one workstation
lock.

## Dependencies

The need depends on the shared assurance platform publishing a typed producer
boundary and a typed evidence interface. It drives the language and delegation
boundary and the definition of the local gate and its configuration lanes.

## Priority and Risk (Informative)

Priority is P0 at promotion and P1 before it. The risk if unmet is a promotion
authorized by a green result that covered nothing, which is unrecoverable once
downstream claims rest on it.

## Notes (Informative)

Nothing here asserts that the shared platform's producer or evidence interfaces
are published and callable from this repository today; SR-047 FND-001 records
that they are not, so VC-4 is currently unexecutable end to end.

## Traceability

NFR-003 and NFR-004 satisfy this stakeholder need. US-007 refines it; TM-001
records planned evidence.
