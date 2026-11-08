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
  - target: ix://agent-ix/quire-protocol/FR-042
    type: contains
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

- Native Quire grammar, parsing, compiler-internal/source linking, compiler
  diagnostics, and source editing. Protocol's typed artifact-to-runtime graph
  linking is in scope.
- Temporal syntax/evaluation and observation collection/correlation authority.
- Business-operation or compensation execution.
- Generic producer execution, evidence persistence, method catalogues, sufficiency,
  qualification disposition, or public release policy.
- General projection, realizability, or composition algorithms. Refinement is
  in scope as the explicit-state product construction
  [FR-301](functional/FR-301-admit-protocol-refinement-dispatch.md) dispatches
  over the accepted upstream assume-guarantee and refinement contracts.
- An external choreography library before E03 admission.

## 3. System Overview

### 3.1 System Description

The library accepts exact versioned artifacts from the native compiler and
observation/temporal producers, validates their identities and selected contracts,
assesses the admitted finite protocol without performing business actions, emits
one canonical result per requested claim in request order, and may derive one
narrower portable E02 result from each validated source result.

### 3.2 Intended Users

- Native Quire compiler and runtime integrators.
- Protocol and compensation verification owners.
- Engineering Assurance and quire-verification consumers.

## 4. Requirements Architecture

Stakeholder intent is refined through user stories into atomic local
implementation requirements. Each functional and non-functional requirement is
bound in [TM-001](tests.md) to the verification that discharges it: an authored
test case where one exists, an integration scenario where the obligation is a
seam, and a declared inspection or analysis where neither applies. TM-001 names
the requirements that reach no test case rather than implying one. Integration
tests keep the compiler, temporal/observation, Engineering Assurance, Quoin, and
E02 ownership seams independently observable.

## 5. Requirement Classes

The repository uses ISO-style StR, US, FR, NFR, IT, TC, TestMatrix, ADR,
AssuranceProfile, and ArchitectureDescription artifacts. External `ix://`
relationships identify normative owners without copying their clauses.

## 6. Requirement Identification

Local immutable identifiers begin at `StR-001`, `US-001`, `FR-001`, `NFR-001`,
`IT-001`, `TC-001`, `ADR-001`, `TM-001`, `AP-001`, and `AD-001`.
Each namespace is allocated sequentially with no gaps and no reuse; the highest
allocated identifier is currently `StR-005`, `US-007`, `FR-042`, `NFR-008`,
`IT-003`, `TC-017`, `ADR-009`, `TM-001`, `AP-001`, and `AD-001`.

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

Failure classification is partitioned by owner and is never inferred across a
boundary. Three external or semantic owners publish values used at their own
boundaries:

- Whether a requested evaluator ran:
  `ix://agent-ix/quire-specification/FR-241`, five members.
- Why a projection to E02 was refused:
  `ix://agent-ix/quire-specification/FR-283`, whose spellings are kebab-case
  and "distinct from the snake_case native producer codes of FR-271".
- Why an offered artifact was not admitted: the producer's own
  `protocol_artifact::Error`, with its `Invalid` and `Unsupported` payloads,
  carried unmodified.

[FR-014](functional/FR-014-typed-outcome-identity.md) states the identity of an
outcome this library returns; [FR-008](functional/FR-008-return-typed-refusals.md)
owns the envelope that carries one.

FR-008 additionally owns the Protocol envelope, its local originating-stage
vocabulary and its closed local causes. A source-owner cause is retained under
its exact owner contract and catalog; an unknown source catalog produces
`unsupported-source-contract` rather than a locally interpreted cause.

An earlier revision of this section listed eight failure classes in prose —
"invalid or unaccepted input, identity mismatch, unsupported capability/version,
resource exhaustion, incomplete evidence, producer failure, semantic violation,
and projection finding loss". That was a third rendering of FR-241's and
FR-283's members in spellings neither publishes.

### 10.2 Failure Handling Guarantees

Failures return typed data, preserve independent results, and cannot become
successful truth, adequacy, conformance, or sufficiency.

## 11. Traceability

[TM-001](tests.md) records requirement-to-test status over the repository as it
stands: 42 functional requirements, 8 non-functional requirements, 5 stakeholder
requirements, 7 user stories, 9 architecture decisions, 3 integration tests, 17
test cases, and 305 acceptance criteria across the functional and non-functional
sets.

The functional set is three layers rather than one list: 9 aggregate or
behavioural parents (`FR-001`..`FR-009`), 9 object requirements
(`FR-010`..`FR-018`), and 24 atomic behavioural parts (`FR-019`..`FR-042`). A
parent states each of its obligations once, through exactly one part, and
reaches its parts by `contains`; a parent row and its part rows are therefore two
views of one obligation and are not separate work.

TM-001 separates five states and merges none of them: test-backed, backed only
by an integration scenario, unbacked — verified by declared inspection or
analysis with no test case at all — unexecutable, meaning a backing exists
and cannot be run today, which is neither passing nor failing — and
engine-bound, meaning a compiler-checked trace symbol actually resolves to the
criterion. Test-backed and engine-bound are different words: the first says a
criterion has somewhere to be tested, the second says something tests it. An
earlier revision of this paragraph said four states and omitted engine-bound,
while TM-001 defined it and reported it as the coverage figure. `FR-009-AC-3`,
`NFR-007-AC-8` and `NFR-008-AC-8` each require an unexecutable obligation to be
counted as a zero inside its denominator rather than omitted from it, and
`NFR-005` denominates its metrics on owed scenarios for the same reason. `TC-017`
is the highest authored test case and no matrix row cites a higher identifier.

Rust tests carry bare compiler-checked `ix-trace-rs` bindings to the exact
local and upstream obligations: the Task-009 checkpoint reports 272 Rust
evidence candidates, 266 tagged and 266 bound. Six candidates remain untagged;
the final enforcement obligation is `NFR-003-AC-5`'s, its sole declared
verification is `TC-010`, and `TC-010` remains owned by blocked Task-011.

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

The repository is partially implemented. Strict `/1` and `/2` admission,
version-1 linking, standalone compensation, batch/incremental conformance,
independent claim disposition, canonical result reading/production, the complete
borrowed Contract-IR result view, refusal-envelope components, reproduction
records and an inventoried resource accountant all exist on `main`.

Those implementations are not yet one production pipeline. The linker has no
version-2 linked graph, downstream modules accept independently constructed
models, canonical result assembly accepts caller-built duplicates of upstream
states, and E02 projection and scenario handoff do not exist on `main`.
Version-lock resolution now exists as the Task-013 mechanism. Task-009 has
completed the concrete QSL `/1` and `/2` intake handoff; Tasks 010–011 retain
the other producer integrations. The distinct consumer-owned delivery index is
complete under Task-019/#66. Task status and executable coverage are stated
only by TM-001 and Plan-001; module presence does not close an aggregate
requirement.

The current accepted compiler pin is QSL `2736b62`; the compiled-protocol `/1`,
`/2`, and `/3` handoffs are owner-addressed and the `/1` Service/Provider
authorities are distinct. The direct QObs pin is accepted C00
runtime/authority revision `924006300f45b38483be1cbdf99b68f899b7d368`,
from merged owner PR #27. Task-020's owner-acceptance prerequisite is satisfied.
Protocol recognizes the owner's partial, activation, authority-bundle v1/v2
and lineage interfaces without copying their schemas; QObs repair and
aggregate-query remain explicit typed unsupported consumer capabilities until
Protocol owns adapters for them.
E02's draft-2 schema/reader contract is accepted at qverify revision `d2fc0d8`
and its exact dependency-compatible consumer revision is
`293e342c6f5e2c69738956ff5165f75761b0364a`, with Engineering Assurance
`9237ce50ce19c6d6c937705bfde4b64adcc9d16b`. Quoin has a Rust evidence
implementation but no Protocol scenario-record intake. That remaining gate is
not permission to substitute a local contract.

## 15. Governance Notes

Build and qualification workflows remain manual dispatch. No hosted status is
evidence for the local gate.
The existing CLA administration workflow retains its separate triggers. Local
heavy Cargo activity uses one job and the shared workstation lock. This
repository does not implement another agent's owned producer or evidence surface.

## 16. References

Every external producer this specification consumes, and the artifact that
defines its contract. An entry here is a reference, not a pin: FR-009's
version-lock manifest carries the pinned revision and the resolution status, and
its entry-status table records why each is currently `unresolved`.

- `agent-ix/quire-specification` — the formal-clause source. FR-047, FR-050
  through FR-062, and NFR-020/NFR-021 define the semantics this repository
  consumes — FR-047 owns the typed located cause vocabulary FR-008 and FR-036
  adopt, and it sits outside the FR-050..FR-062 block;
  FR-200 through FR-283 are the domain object layer it references for every
  protocol, temporal and observation noun. Open questions this repository raised
  against that layer are issues 28 and 29.
- `agent-ix/quire-spec-language` — the A-owned native compiler and its
  compiled-protocol artifact contracts. The pin is `6c8c401`, A's promoted
  FR-052 compatibility correction, FR-051 checked activation-guard selection,
  committed owner-addressed `/1` and `/2` handoffs, and distinct `/1`
  Service/Provider authorities after release `v0.2.0`; issues 40, 78, 98, 112
  and 114 owned that delivery and have landed. This is the only native compiler
  reader in the admitted path, and this repository adds no second one.
- `agent-ix/quire-observation` — the F-owned observation producer interface and
  authority contracts. Accepted runtime references, partial/activation facts,
  revisioned authority bundles and bounded lineage are consumed locally at
  accepted PR #27 merge `9240063`.
  Complete-global-conformance closure remains Protocol-owned and is not a QObs
  artifact; repair and aggregate-query are not Protocol adapters at this pin.
- `agent-ix/quire-verification` FR-007 and `quire-verification/e02-draft-2` —
  the accepted E02 strict reader, generated types, JCS implementation and
  finding vocabulary at exact compatible consumer revision
  `293e342c6f5e2c69738956ff5165f75761b0364a`.
- `agent-ix/engineering-assurance` issue 34 — the delivered typed Rust producer-
  execution request/result boundary, consumed at its exact accepted git pin.
- The selected diagnostic catalog, whose identity, revision and byte digest
  FR-008 binds. `quire-specification` FR-270 records that the only catalog
  offered is a proposed native diagnostic interpretation that describes itself as
  not adopted or implemented, so this reference names no adopted artifact yet.
- `agent-ix/quoin` — the external Rust evidence-retention owner. Its existing
  experiment and operational records do not accept FR-017; Task-011 waits for a
  structured Protocol scenario-record operation rather than coercing either.
- `agent-ix/quire-contract-ir` issue 71 — the downstream consumer of Protocol's
  complete borrowed result-map view for native/TL correspondence and joining.

The complete-global-conformance closure record is **not** an external reference.
It is Protocol-owned and takes no version-lock entry because no external owner
can assert anything about it.
