---
id: AP-001
title: Protocol implementation assurance profile
type: AssuranceProfile
status: proposed
owner: agent-ix-protocol-owner
profile_version: 0.2
profile_kind: general
scope: one exact quire-protocol Rust candidate implementing the selected finite native Quire protocol profile
impact_assessments:
  - id: impact-false-conformance
    scenario: a refused, unsupported, resource-incomplete or failed protocol assessment is reported as complete conformance
    severity: critical
    verifiability:
      class: cheap-conclusive
      stochastic_dependency: none
    detect_before_harm:
      expected: true
      control_ref: ix://agent-ix/quire-protocol/FR-004
  - id: impact-compensation-confusion
    scenario: a delivery or attempt is treated as a successful effect or recovery and activates an invalid compensation result
    severity: critical
    verifiability:
      class: cheap-conclusive
      stochastic_dependency: none
    detect_before_harm:
      expected: true
      control_ref: ix://agent-ix/quire-protocol/FR-003
  - id: impact-result-loss
    scenario: projection or strict reading erases uncertainty, support lineage, adequacy separation, or an independent finding
    severity: material
    verifiability:
      class: cheap-conclusive
      stochastic_dependency: none
    detect_before_harm:
      expected: true
      control_ref: ix://agent-ix/quire-protocol/FR-007
  - id: impact-identity-substitution
    scenario: changed source, dependency, observation, profile, result, or adapter bytes retain an accepted identity
    severity: critical
    verifiability:
      class: cheap-conclusive
      stochastic_dependency: none
    detect_before_harm:
      expected: true
      control_ref: ix://agent-ix/quire-protocol/NFR-001
  - id: impact-assurance-duplication
    scenario: a local runner, transcript verdict parser, evidence store, or foreign-language qualification path bypasses shared assurance controls
    severity: material
    verifiability:
      class: cheap-conclusive
      stochastic_dependency: none
    detect_before_harm:
      expected: true
      control_ref: ix://agent-ix/quire-protocol/NFR-003
review_policy:
  mode: require
  operations: [spec-review, code-review, gap-analysis]
relationships:
  - target: ix://agent-ix/quire-protocol/StR-001
    type: governs
  - target: ix://agent-ix/quire-protocol/FR-001
    type: governs
  - target: ix://agent-ix/quire-protocol/FR-003
    type: governs
  - target: ix://agent-ix/quire-protocol/FR-004
    type: governs
  - target: ix://agent-ix/quire-protocol/FR-006
    type: governs
  - target: ix://agent-ix/quire-protocol/FR-007
    type: governs
  - target: ix://agent-ix/quire-protocol/FR-008
    type: governs
  - target: ix://agent-ix/quire-protocol/NFR-001
    type: governs
  - target: ix://agent-ix/quire-protocol/NFR-002
    type: governs
  - target: ix://agent-ix/quire-protocol/NFR-003
    type: governs
---
# Protocol implementation assurance profile

## Decision Boundary

This profile governs one exact private Rust candidate, selected upstream
protocol/profile and dependency closure, retained corpus, compiler/observation/
temporal/E02/Engineering Assurance/Quoin interface revisions, toolchain, feature
set, budgets, and execution environment. It authorizes no later revision,
advanced protocol claim, public release, or foreign runtime.

## Impact Scenarios

The impact records cover false conformance, compensation identity confusion,
loss during result projection, identity substitution, and bypass of shared
assurance. Each is expected to be detected before promotion with a named local
control and exact adverse tests.

## Evidence Policy

Acceptance uses catalogue-selected example, property, metamorphic, differential,
fuzz, mutation, integration, bounded benchmark, strict-reader, and replay
evidence where applicable. Every automated test uses bare `ix-trace-rs`. Producer
execution uses Engineering Assurance; Quoin retains evidence. The exact Rust
review and semantic gap analysis run over one unchanged head. A missing producer,
interface, budget, corpus class, test, review, license decision, or owner
disposition remains an owned gap and prevents a production qualification claim.

## Exceptions

No exception is accepted. A future exception must name its owner, rationale,
expiry, affected impact scenario, and exact decision before it can change this profile.
