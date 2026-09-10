---
id: IT-003
title: "Publish typed assurance and E02 results"
type: IT
relationships:
  - target: ix://agent-ix/quire-protocol/FR-006
    type: verifies
  - target: ix://agent-ix/quire-protocol/FR-007
    type: verifies
  - target: ix://agent-ix/quire-protocol/NFR-003
    type: verifies
---
# IT-003: Publish typed assurance and E02 results

## Objective

Verify canonical protocol results and derived E02 results through the accepted
Engineering Assurance transport and Quoin evidence boundary without a local
runner, transcript parser, evidence store, or sufficiency decision.

## Target Integration

The protocol library produces canonical and E02 result values. Engineering
Assurance supplies bounded execution context and typed transport;
quire-verification supplies the E02 strict reader; Quoin retains evidence.

## Preconditions

Engineering Assurance issue 34 has an accepted Rust request/result interface,
the exact E02 revision is accepted, Quoin's evidence interface is pinned, and
the Quoin containment contract from `agent-ix/quire-research#62` is accepted.

## Inputs

Completed, refused, unsupported, exhausted, failed, pending, inadequate,
missing-participant, disagreeing-oracle, malformed-response, and finding-loss cases.

## Test Procedure

1. Submit exact typed producer requests through Engineering Assurance.
   - IT-003-SC-01: request, run, budget, executable, environment, and dependency identities survive transport.
2. Return canonical results through the typed result boundary.
   - IT-003-SC-02: non-success states remain distinct and no transcript text determines a verdict.
3. Project accepted canonical results to E02 and invoke its strict reader.
   - IT-003-SC-03: every mapped finding and exact canonical source reference survives.
4. Retain the evidence through Quoin.
   - IT-003-SC-04: provenance and limitations remain exact, Quoin stays external
     behind the accepted structured interface, and no local store or Quoin runtime
     dependency is created.
5. Exercise malformed, non-completed, and finding-loss paths.
   - IT-003-SC-05: each refuses or remains incomplete without an E02 success or sufficiency decision.

## Expected Results

Typed producer context, canonical result meaning, E02 findings, and evidence
provenance remain separately owned and exactly linked across the whole boundary.

## Metadata

- Priority: P0
- Target Integration: protocol library, Engineering Assurance, E02, and Quoin
- Automation: Automated after Engineering Assurance issue 34 is accepted

## Dependencies

Engineering Assurance issue 34, quire-verification FR-007/E02, Quoin evidence,
and `agent-ix/quire-research#62` containment interfaces must be accepted at exact
revisions.

## Notes

The absent Engineering Assurance producer blocks execution but does not authorize
a repository-local substitute.

## Traceability

Verifies FR-006, FR-007, and NFR-003.
