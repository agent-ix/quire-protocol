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
`IT-003`, `TC-015`, `ADR-009`, `TM-001`, `AP-001`, and `AD-001`.

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

Failure classification is not this repository's vocabulary and is not restated
here. Three owners publish it, and each is used at its own boundary:

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
requirements, 7 user stories, 9 architecture decisions, 3 integration tests, 15
test cases, and 294 acceptance criteria across the functional and non-functional
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
`NFR-005` denominates its metrics on owed scenarios for the same reason. `TC-015`
is the highest authored test case and no matrix row cites a higher identifier.

Rust tests carry bare compiler-checked `ix-trace-rs` bindings to the exact
local and upstream obligations: a census over this head reports 198 tagged
tests and 198 bare `#[trace(...)]` markers, none path-qualified. The future
tense this sentence carried through an earlier revision was superseded by that
run. What does not yet exist is the enforcement — nothing today fails when a
test is added untagged or a marker is path-qualified. That obligation is
`NFR-003-AC-5`'s, its sole declared verification is `TC-010`, and `TC-010` is
owned by `Task-011`, which is `blocked`.

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

The repository is partially implemented. Fifteen integration-test binaries plus
the in-crate unit-test binary list 244 regular tests on this candidate: 221 run
in the default gate and 23 generated version 1 fixture tests remain explicitly
ignored. The same gate lists and runs 37 doc-tests. `tc_012_local_gate` contributes
ten default-running tests that bind all ten NFR-004 criteria and independently
exercise thirteen stable gate-defect identities. The version 2 handoff's 17
tests execute by default from the pinned dependency source; its superseded
24-test pre-delivery corpus was retired rather than retained as a second
protocol vocabulary. Historical smaller totals described earlier
implementation checkpoints and are not the current inventory.

What remains unimplemented is the larger part. `admit_and_link_v2` is now
written against A's published `quire.compiled-protocol/2` reader at the pinned
revision `7931a00`. The default gate reaches A's committed bundle through the
owner-exported `PUBLISHED_HANDOFF`: the base admits, all 28 published mutations
return their declared typed identities, both cross-version submissions refuse,
and every declared resource dimension admits at exact usage, refuses one-short,
and admits again under a fresh exact budget. The broader version 1 SC-01 through
SC-05 regression lane remains generated and ignored. Canonical result ownership
and its exact Contract-IR mapping landed on main in PR #51; the wider
refusal-catalog, cause, projection, observation and external qualification work
remains open. Forty-seven acceptance criteria reach no test case at all.
`TM-001` is the authority on which criteria
are backed, unbacked, blocked or planned; this section states only that the
repository is no longer specification-only.

## 15. Governance Notes

All hosted workflows remain manual dispatch. Local heavy Cargo activity uses one
job and the shared workstation lock. This repository does not implement another
agent's owned producer or evidence surface.

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
  compiled-protocol artifact contracts. The pin is `7931a00`, A's promoted
  FR-052 compatibility correction after release `v0.2.0`, which supplies both
  `quire.compiled-protocol/1` and `/2` plus the exported handoff address; issues
  40 and 78 owned that delivery and have landed. This is the only native
  compiler reader in the admitted path, and this repository adds no second one.
- `agent-ix/quire-observation` — the F-owned observation producer interface,
  carrying the scope facts and global closure record IT-002 consumes.
- `agent-ix/quire-verification` FR-007 and `quire-verification/e02-draft-2` —
  the E02 strict reader and its finding vocabulary.
- `agent-ix/engineering-assurance` issue 34 — the producer-execution boundary.
- The selected diagnostic catalog, whose identity, revision and byte digest
  FR-008 binds. `quire-specification` FR-270 records that the only catalog
  offered is a proposed native diagnostic interpretation that describes itself as
  not adopted or implemented, so this reference names no adopted artifact yet.
- `agent-ix/quoin` and `agent-ix/quire-research` issue 62 — the evidence
  retention interface and its containment contract.
- `agent-ix/quire-research` PT01, PT02, E03, and setup issue 33.

The complete-global-conformance closure record is **not** an external reference.
It is B-authored, landed in `quire-specification` through PR 22 from
`agent-b/global-conformance-closure`, and takes no version-lock entry because no
external owner can assert anything about it.
