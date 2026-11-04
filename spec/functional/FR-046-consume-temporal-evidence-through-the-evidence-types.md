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
`qsl_eval::temporal::evidence::EvidenceRequest`, call the temporal-evidence entry
that evaluates from a `CheckedPackageV2` alone and read the `EvidenceResult`, in
place of the retired `native_temporal::v2::ValidatedRequest`,
`native_temporal::v2::ValidatedResult`, `native_temporal::v2::EvidenceRef` and
`native_temporal::v2::ActivationState` and their four producer contracts.

## Inputs

The admitted `CheckedPackageV2` (FR-043) through QSL's evidence entry that
evaluates from the package alone, a `CheckedNodeId` naming a temporal clause, an
`EvidenceRequest` built from the observation evidence (QSL FR-516), and a
`Meter`. No source package, emission or admitted-read value of QSL's emit side is
an input.

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
- The library shall keep using QSL's public `temporal::` surface (`Closure`,
  `Completeness`, `Execution`, `Evidence`, `Position`, `OrderKey`) until QSL
  retires it (QSL FR-521 row 5), and shall fix its own use of
  `Position.instance_valuations` for the QSL revision it consumes.
- The producer-contract registry shall hold none of the four contract strings
  `quire.native-temporal-request/v1`, `/v2`, `quire.native-temporal-result/v1`
  and `/v2`; the request and result are in-memory values with no document and no
  contract identifier.
- The result record shall be the same for two evidence results that QSL reports
  as equal.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-046-AC-1 | Over **R**'s four temporal clauses, the assessment builds an `EvidenceRequest` from observation evidence and returns the `EvidenceResult`'s activation state, truth, settlement and decision support in its result record, equal to the values QSL's evidence call reports. | Test (TC-152) |
| FR-046-AC-2 | The producer-contract registry holds none of the four contract strings, and a test that reads the registry fails when one is present. | Test (TC-152) |
| FR-046-AC-3 | No source file names `native_temporal`, `native_temporal::v2::ValidatedRequest`, `native_temporal::v2::ValidatedResult`, `native_temporal::v2::EvidenceRef` or `native_temporal::v2::ActivationState`; the evidence types' own `EvidenceRef` and `ActivationState` are not these names. | Inspection (TC-152) |
| FR-046-AC-4 | A call refusal with each cause of QSL FR-516's cause table returns the library's typed refusal naming that cause, with no charge and the caller's ledgers unchanged; no cause is renamed or merged. | Test (TC-152) |
| FR-046-AC-5 | A `PackageMismatch` of each variant that QSL's evidence entry returns is carried as a typed construction refusal and no assessment is attempted. | Test (TC-152) |
| FR-046-AC-6 | The IT-002 scenarios (QSL FR-517's recipe **R**) pass on the evidence route, and an unavailable or withdrawn anchor settles as a missing non-value and never as a Boolean. | Test (TC-152) |
| FR-046-AC-7 | The library builds against the retained `temporal::` surface and reads `Position.instance_valuations` in the form of the consumed QSL revision; a build with the surface removed fails, and the position values the assessment passes equal the ones the evaluation reads. | Test (TC-152) |

## Dependencies

- QSL FR-516, FR-515 (the evidence package), FR-521 rows 3, 4 and 5, FR-044. QSL provides the entry that evaluates from a `CheckedPackageV2` alone; this requirement depends on it.
- [FR-003](FR-003-assess-compensation-state.md) and
  [FR-004](FR-004-assess-finite-global-conformance.md) state the assessment
  contracts; this requirement changes only their temporal evidence source.
