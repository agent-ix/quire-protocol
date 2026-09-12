---
type: master-requirements
name: quire-protocol
org: agent-ix
component_type: rust-library
implementation_language: rust
tags:
  - native-quire
  - protocol
  - conformance
  - compensation
  - rust
depends_on: []
relationships:
  - target: ix://agent-ix/quire-specification/FR-050
    type: depends_on
  - target: ix://agent-ix/quire-specification/FR-061
    type: depends_on
  - target: ix://agent-ix/quire-specification/FR-062
    type: depends_on
  - target: ix://agent-ix/quire-verification/FR-007
    type: depends_on
standards_alignment:
  - iso-iec-ieee-29148
  - iso-iec-ieee-29119
---
# Master Requirements Specification
## Quire Protocol Rust Library

## 1. Purpose

This specification governs the first-party Rust library that consumes one exact
accepted and frozen native Quire protocol profile and implements its bounded
representation, compensation, finite conformance, canonical result, and E02
projection contracts.
The authored formal meaning remains in `agent-ix/quire-specification`; this
repository owns its strict implementation and evidence-backed conformance.

## 2. Scope

### 2.1 In Scope

- Strict admission of one accepted, immutable selected protocol profile and dependency closure.
- Typed finite protocol representation and identity preservation.
- Activation, compensation registration, commit, retry, and recovery assessment.
- Bounded finite global replay and settled incremental conformance.
- Independent claim dispositions without fallback promotion.
- Canonical protocol result production, RFC 8785 identity, and strict reading.
- Total source-bound projection into the selected E02 contract.
- Rust-only local verification and shared Engineering Assurance integration.

### 2.2 Out of Scope

- Native Quire grammar, parsing, linking, compiler diagnostics, and source editing.
- Temporal syntax/evaluation and observation collection/correlation authority.
- Business-operation or compensation execution.
- Generic producer execution, evidence persistence, method catalogues, sufficiency,
  qualification disposition, or public release policy.
- General projection, realizability, refinement, or composition algorithms.
- An external choreography library before E03 admission.

## 3. System Overview

### 3.1 System Description

The library accepts exact versioned artifacts from the native compiler and
observation/temporal producers, validates their identities and selected contracts,
assesses the admitted finite protocol without performing business actions, emits
one canonical rich result, and may derive one narrower portable E02 result.

### 3.2 Intended Users

- Native Quire compiler and runtime integrators.
- Protocol and compensation verification owners.
- Engineering Assurance and quire-verification consumers.

## 4. Requirements Architecture

Stakeholder intent is refined through user stories into atomic local
implementation requirements. Each functional and non-functional requirement is
bound to an authored test case in [TM-001](tests.md). Integration tests keep the
compiler, temporal/observation, Engineering Assurance, Quoin, and E02 ownership
seams independently observable.

## 5. Requirement Classes

The repository uses ISO-style StR, US, FR, NFR, IT, TC, TestMatrix,
AssuranceProfile, and ArchitectureDescription artifacts. External `ix://`
relationships identify normative owners without copying their clauses.

## 6. Requirement Identification

Local immutable identifiers begin at `StR-001`, `US-001`, `FR-001`, `NFR-001`,
`IT-001`, `TC-001`, `TM-001`, `AP-001`, and `AD-001`.

## 7. Requirement Quality Policy

Functional requirements are atomic, testable, source-bound, and fail closed at
unknown revisions or missing identities. No local requirement may weaken or
reinterpret an accepted upstream formal clause.

## 8. State and Execution Model

### 8.1 State Semantics

Protocol, workflow, role, message, delivery, attempt, effect, obligation,
compensation, progress, closure, truth, adequacy, and claim states remain distinct
typed values under the selected upstream profile.

### 8.2 Transition Semantics

Only authored causal, channel, activation, commit, retry, and recovery relations
authorize transitions. Input order, timestamps, display names, and successful
transport do not create protocol facts.

### 8.3 Determinism Guarantees

Equivalent complete input identities produce byte-identical canonical results.
Changing any semantic input or dependency identity changes the affected result
identity or returns a typed refusal.

## 9. Events and Signals

### 9.1 Event Model

The library consumes typed external observations and never manufactures business
effects from transport events.

### 9.2 Event Guarantees

Ordering and delivery guarantees come only from selected protocol/channel
premises. Missing or conflicting premises remain explicit.

## 10. Error and Failure Model

### 10.1 Error Classification

Failures distinguish invalid or unaccepted input, identity mismatch, unsupported
capability/version, resource exhaustion, incomplete evidence, producer failure,
semantic violation, and projection finding loss.

### 10.2 Failure Handling Guarantees

Failures return typed data, preserve independent results, and cannot become
successful truth, adequacy, conformance, or sufficiency.

## 11. Traceability

[TM-001](tests.md) records requirement-to-test status. Rust tests will carry bare
compiler-checked `ix-trace-rs` bindings to the exact local and upstream obligations.

## 12. Verification Strategy

Verification combines example, property, metamorphic, differential, fuzz,
mutation, integration, bounded benchmark, strict-reader, and replay evidence as
recommended by the accepted catalogue. Shared producer execution and retained
evidence remain in Engineering Assurance and Quoin.

## 13. Change Management

Substantive changes run QUOIN `/specify` and self `/spec-review` before code,
then update the implementation plan and matrix. Promotion requires the exact Rust
review and semantic QUOIN `/gap-analysis` over one unchanged head.

## 14. Lifecycle Status

The repository is specification-only. Every test is planned and every
implementation requirement remains unimplemented until the accepted source
baseline, license, specification review, and implementation plan gates close.

## 15. Governance Notes

All hosted workflows remain manual dispatch. Local heavy Cargo activity uses one
job and the shared workstation lock. This repository does not implement another
agent's owned producer or evidence surface.

## 16. References

Every external producer this specification consumes, and the artifact that
defines its contract. An entry here is a reference, not a pin: FR-009's
version-lock manifest carries the pinned revision and the resolution status, and
its entry-status table records why each is currently `unresolved`.

- `agent-ix/quire-specification` — the formal-clause source. FR-050 through
  FR-062 and NFR-020/NFR-021 define the semantics this repository consumes;
  FR-200 through FR-283 are the domain object layer it references for every
  protocol, temporal and observation noun. Open questions this repository raised
  against that layer are issues 28 and 29.
- `agent-ix/quire-spec-language` — the A-owned native compiler and its
  compiled-protocol artifact contracts. `quire.compiled-protocol/1` is the
  current pin; `/2` is issue 40. This is the only native compiler reader in the
  admitted path, and this repository adds no second one.
- `agent-ix/quire-observation` — the F-owned observation producer interface,
  carrying the scope facts and global closure record IT-002 consumes.
- `agent-ix/quire-verification` FR-007 and `quire-verification/e02-draft-2` —
  the E02 strict reader and its finding vocabulary.
- `agent-ix/engineering-assurance` issue 34 — the producer-execution boundary.
- `agent-ix/quoin` and `agent-ix/quire-research` issue 62 — the evidence
  retention interface and its containment contract.
- `agent-ix/quire-research` PT01, PT02, E03, and setup issue 33.

The complete-global-conformance closure record is **not** an external reference.
It is B-authored, landed in `quire-specification` through PR 22 from
`agent-b/global-conformance-closure`, and takes no version-lock entry because no
external owner can assert anything about it.
