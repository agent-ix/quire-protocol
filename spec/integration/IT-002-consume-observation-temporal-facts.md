---
id: IT-002
title: "Consume observation and temporal facts"
type: IT
relationships:
  - target: ix://agent-ix/quire-protocol/FR-003
    type: verifies
  - target: ix://agent-ix/quire-protocol/FR-004
    type: verifies
  - target: ix://agent-ix/quire-protocol/FR-006
    type: verifies
---
# IT-002: Consume observation and temporal facts

## Objective

Verify the F/E-owned observation, decision-scope progress/closure,
surrounding-execution progress/closure, and temporal decision facts plus the
B-owned complete-global-conformance closure record through compensation, finite
conformance, and canonical result production without duplicating producer
semantics.

## Target Integration

The producers are accepted observation and temporal Rust interfaces. The
consumer is the protocol evaluator and canonical result producer.

## Preconditions

The observation-output consumer preserves decision-scope and surrounding
execution progress/closure, settlement basis, decision support, completeness,
and exact producer identities. PT02 adds an independently typed
complete-global-conformance closure record with exact execution, branch, and
workflow premises. The temporal profile is accepted.

## Inputs

Complete and open valid histories, a decisive witness, a decisive
counterexample, a non-decisive prefix, missing support, each
complete-global-conformance closure state and premise set, changed closure,
mis-correlated observation, duplicate delivery, and late contradiction.

## Test Procedure

1. Submit complete and open histories through the accepted typed interfaces.
   - IT-002-SC-01: producer facts retain their exact identities and authorities.
2. Assess compensation and finite conformance.
   - IT-002-SC-02: every outcome follows only the selected protocol and temporal premises.
3. Produce canonical results for decisive and non-decisive open prefixes.
   - IT-002-SC-03: settlement, pending state, support, decision-scope
     progress/closure, surrounding-execution progress/closure, and
     complete-global-conformance closure remain distinct; independently change
     each progress/closure axis and require exact retention or typed refusal
     without cross-scope substitution.
4. Submit missing, changed, mis-correlated, duplicate, and contradictory facts.
   - IT-002-SC-04: independently omit, duplicate, cross-wire, scope-mutate, or
     identity-mutate each required global execution, branch, and workflow
     premise; each adverse input yields the exact refusal, violation,
     incompleteness, or linked supersession without changing independent truth.

## Expected Results

The protocol library consumes producer facts without reinterpreting them. No
missing or contradictory fact improves truth, adequacy, recovery, or
conformance, and no scoped closure substitutes for complete-global-conformance
closure.

## Metadata

- Priority: P0
- Target Integration: observation and temporal producers to protocol evaluator
- Automation: Automated after producer contracts are accepted

## Dependencies

F-owned FR-115/TC-145/IT-051 and the E-owned temporal profile must preserve the
complete canonical-result input shape.

## Notes

F's composed consumer correction remains unaccepted until its strict-reader
tests independently discriminate decision-scope and surrounding-execution
progress/closure plus the complete-global-conformance closure record and
premises; this test remains planned.

## Traceability

Verifies FR-003, FR-004, and FR-006.
