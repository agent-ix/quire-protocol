---
id: TC-016
title: "Compose complete scenario evidence without storing it"
type: TC
relationships:
  - target: ix://agent-ix/quire-protocol/FR-017
    type: verifies
  - target: ix://agent-ix/quire-protocol/NFR-005
    type: verifies
---
# TC-016: Compose Complete Scenario Evidence Without Storing It

## Description

Verify final FR-017/NFR-005 composition from the actual shared resolution,
outcome and resource types, including cross-record refusal-axis distinctness and
the passed-never-stored boundary.

## Preconditions

Task-013's resolver capability/state, Task-006's operation and claim outcomes,
and Task-008's final accountant records are available. Fixtures are
Protocol-owned values and require no external handoff to execute.

## Test Data

Complete and deterministic partial resolution states; every operation and claim
outcome; every Protocol and external-owner budget dimension; compiler-checked
trace bindings; and one-axis omissions, duplicates, substitutions, cross-wires
and refusal-interior collisions.

## Test Procedure

1. Compose one record for every outcome from a complete resolution.
   - TC-016-SC-01: all owner/version/revision, outcome, budget and trace values
     are retained componentwise.
2. Compose a resolution-failure record from each permutation of a manifest with
   multiple bad required keys.
   - TC-016-SC-02: every permutation carries the same deterministic partial
     state and first causing key.
3. Mutate one owner, contract, revision, outcome, accountant owner/version,
   dimension, grant, use or trace component at a time.
   - TC-016-SC-03: each mutation refuses composition and names the axis.
4. Compare record sets with two scenarios on one refusal axis and with two
   distinct axes sharing one retained interior.
   - TC-016-SC-04: same-axis sharing admits; cross-axis collision refuses the
     integration set.
5. Inspect and exercise the transient value boundary.
   - TC-016-SC-05: Protocol exposes no storage/read-back/index/score/disposition
     operation, and a failed handoff leaves no local record.

## Expected Results

Every complete record is attributable and every incomplete record identifies
the exact missing or conflicting component. Evidence is a transient typed value,
not a Protocol-owned store.

## Metadata

- Priority: P0
- Type: Property
- Automation: Automated; Protocol-owned fixtures only

## Traceability

Verifies FR-017-AC-1 through FR-017-AC-8 and NFR-005-AC-1 through
NFR-005-AC-6. IT-001 through IT-003 supply the concrete integration records;
Task-011 exercises the accepted Quoin handoff.
