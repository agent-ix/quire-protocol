---
id: FR-046
title: "Consume temporal evidence through the in-memory evidence types"
type: FR
relationships:
  - { target: ix://agent-ix/quire-protocol/US-001, type: implements }
  - { target: ix://agent-ix/quire-protocol/FR-003, type: references }
  - { target: ix://agent-ix/quire-protocol/FR-004, type: references }
  - { target: ix://agent-ix/quire-protocol/FR-044, type: depends_on }
  - { target: ix://agent-ix/quire-spec-language/FR-516, type: depends_on }
  - { target: ix://agent-ix/quire-spec-language/FR-521, type: references }
---
# FR-046: Consume temporal evidence through the in-memory evidence types

## Description

When the library assesses a temporal obligation, it SHALL build a
`qsl_eval::temporal::evidence::EvidenceRequest`, call
`EvidencePackage::evaluate_temporal_evidence` and read the
`EvidenceResult`, in place of the retired
`native_temporal::v2::{ValidatedRequest, ValidatedResult}`, `EvidenceRef` and
`ActivationState` and their four producer contracts.

## Inputs

An `EvidencePackage` built from the admitted package (QSL FR-516), a
`CheckedNodeId` naming a temporal clause, an `EvidenceRequest` built from the
observation evidence, and a `Meter`.

## Outputs

An `EvidenceResult` carrying the activation state, truth, settlement and decision
support, joined to the library's result record; or the call's typed refusal.

## Behavior

- The assessment, result record, conformance kernel and the IT-001 and IT-002
  procedures shall use the evidence types and no retired type.
- The library shall carry a call refusal (`CallFailure::Admission` with its cause
  name) into its own typed refusal unchanged and shall reinterpret none.
- The library shall carry a construction refusal (`PackageMismatch`) as a typed
  refusal before any call and charge nothing for it.
- The producer-contract registry shall hold none of the four contract strings
  `quire.native-temporal-request/v1`, `/v2`, `quire.native-temporal-result/v1`
  and `/v2`; the request and result are in-memory values with no document and no
  contract identifier.
- The result record shall give a result built from an `EvidencePackage` made from
  an emission and one made from the read-back package the same record.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-046-AC-1 | Over **R**'s four temporal clauses, the assessment builds an `EvidenceRequest` from observation evidence and returns the `EvidenceResult`'s activation state, truth, settlement and decision support in its result record, equal for the two package routes. | Test (TC-152) |
| FR-046-AC-2 | The producer-contract registry holds none of the four contract strings, and a test that reads the registry fails when one is present. | Test (TC-152) |
| FR-046-AC-3 | No source file names `native_temporal`, `ValidatedRequest`, `ValidatedResult`, the retired `EvidenceRef` or the retired `ActivationState`. | Inspection (TC-152) |
| FR-046-AC-4 | A call refusal with each cause of QSL FR-516's cause table returns the library's typed refusal naming that cause, with no charge and the caller's ledgers unchanged; no cause is renamed or merged. | Test (TC-152) |
| FR-046-AC-5 | A `PackageMismatch` of each variant returns a typed construction refusal and no assessment is attempted. | Test (TC-152) |
| FR-046-AC-6 | The IT-002 scenarios (QSL FR-517's recipe **R**) pass on the evidence route, and an unavailable or withdrawn anchor settles as a missing non-value and never as a Boolean. | Test (TC-152) |

## Dependencies

- QSL FR-516, FR-515 (`EvidencePackage`), FR-521 rows 3 and 4, FR-044.
- [FR-003](FR-003-assess-compensation-state.md) and
  [FR-004](FR-004-assess-finite-global-conformance.md) state the assessment
  contracts; this requirement changes only their temporal evidence source.
