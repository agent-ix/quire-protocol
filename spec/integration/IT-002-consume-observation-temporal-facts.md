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
surrounding-execution progress/closure, and temporal decision facts through
compensation, finite conformance, and canonical result production without
duplicating their producer semantics.

## Target Integration

The producers are accepted observation and temporal Rust interfaces. The
consumer is the protocol evaluator and canonical result producer.

## Preconditions

The observation-output consumer preserves decision-scope and surrounding
execution progress/closure, settlement basis, decision support, completeness,
and exact producer identities. The temporal profile is accepted.

## Inputs

Complete and open valid histories, a decisive witness, a decisive
counterexample, a non-decisive prefix, missing support, changed closure,
mis-correlated observation, duplicate delivery, and late contradiction.

## Test Procedure

1. Submit complete and open histories through the accepted typed interfaces.
   - IT-002-SC-01: producer facts retain their exact identities and authorities.
2. Assess compensation and finite conformance.
   - IT-002-SC-02: every outcome follows only the selected protocol and temporal premises.
3. Produce canonical results for decisive and non-decisive open prefixes.
   - IT-002-SC-03: settlement, pending state, support, decision-scope
     progress/closure, and surrounding-execution progress/closure remain distinct;
     independently change each progress/closure axis and require exact retention
     or typed refusal without cross-scope substitution.
4. Submit missing, changed, mis-correlated, duplicate, and contradictory facts.
   - IT-002-SC-04: each yields the exact refusal, violation, incompleteness, or linked supersession.

## Expected Results

The protocol library consumes producer facts without reinterpreting them. No
missing or contradictory fact improves truth, adequacy, recovery, or conformance.

## Metadata

- Priority: P0
- Target Integration: observation and temporal producers to protocol evaluator
- Automation: Automated after producer contracts are accepted

## Dependencies

F-owned FR-115/TC-145/IT-051 and the E-owned temporal profile must preserve the
complete canonical-result input shape.

## Notes

F's composed consumer correction remains unaccepted until its strict-reader tests
independently discriminate decision-scope and surrounding-execution
progress/closure; this test remains planned.

## Traceability

Verifies FR-003, FR-004, and FR-006.
