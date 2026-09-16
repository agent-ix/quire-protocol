---
id: SR-125
title: "Dependency review of the QObs C00 compatibility successor"
type: SpecReview
analysis: dependency
scope: "IT-002, FR-009 and NFR-006 against accepted quire-observation 9240063"
review_set: subset
relationships:
  - { target: ix://agent-ix/quire-protocol/IT-002, type: reviews }
  - { target: ix://agent-ix/quire-protocol/FR-009, type: reviews }
  - { target: ix://agent-ix/quire-protocol/NFR-006, type: reviews }
---
## Summary

The dependency chain is acyclic and externally enabled. Accepted QObs PR #27 merge
`924006300f45b38483be1cbdf99b68f899b7d368` publishes all five owner contract
constants and schema byte arrays; Protocol needs only an exact pin, lock/rights
reconciliation, exhaustive inventory extension and tests.

## Classification

| Item | Class | State and evidence |
| --- | --- | --- |
| QObs C00 contract constants and schema bytes | External enablement | Accepted at exact PR #27 merge `9240063`; no local schema is needed. |
| Exact Cargo, lockfile and rights pin | Consumer enablement | Must advance together before the interface inventory can compile. |
| Five contract descriptors and schema digest checks | Consumer feature | Extends Protocol's closed accepted-interface inventory without parsing owner bytes. |
| Repair/query support status | Consumer boundary | Remains typed unsupported until a separate adapter requirement and implementation exist. |

## Dependency order

1. Pin accepted QObs merge `9240063` and reconcile `Cargo.lock` and `DEPENDENCIES.md`.
2. Import the five owner modules/constants and extend the exhaustive inventory.
3. Bind each exported schema byte array to the independently recorded digest.
4. Assert distinct unsupported repair/query capabilities and run SC-08 plus the
   repository gate.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No dependency blocker remains; repair and query are deliberately excluded from the supported-contract DAG rather than treated as missing producer deliverables. | IT-002 Owner boundaries; FR-009 Entry status; NFR-006 Status |

## Disposition

Implement in the order above. A missing exported constant or schema byte array
is a pin mismatch to report upstream, not permission to add a local literal
contract or copied schema.
