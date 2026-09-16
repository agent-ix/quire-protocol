---
id: SR-122
title: "Scope review of the QObs C00 compatibility successor"
type: SpecReview
analysis: scope-boundary
scope: "IT-002 owner boundary for the QObs C00 compatibility successor"
review_set: subset
relationships:
  - { target: ix://agent-ix/quire-protocol/IT-002, type: reviews }
  - { target: ix://agent-ix/quire-protocol/FR-009, type: reviews }
---
## Summary

The successor keeps the producer/consumer boundary intact. QObs owns contract
labels, schema bytes, strict readers, repair and query semantics; Protocol owns
only its exact accepted-interface selection, resolution metadata and typed
statement that two unadapted consumer capabilities are unsupported.

## Boundary allocation

| Responsibility | Owner | Treatment |
| --- | --- | --- |
| Partial, activation, bundle v1/v2 and lineage contract/schema | QObs | Guaranteed by exact Rust dependency and checked against owner bytes. |
| Contract revision selection and request-bound resolution | Protocol | Core consumer responsibility under FR-009. |
| Repair-plan derivation and coordination semantics | QObs | Out of scope; no Protocol adapter or mirrored operation. |
| Aggregate-query filtering/counting/sum semantics | QObs | Out of scope; no Protocol adapter or mirrored operation. |
| Capability support declaration | Protocol | Closed typed consumer status; unsupported entries mint no owner contract key. |

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No boundary leak remains: schema identity is checked over QObs-exported bytes, while repair/query are neither invoked nor represented as producer contracts. | IT-002 Target Integration; IT-002 Owner boundaries; IT-002-SC-08 |

## Disposition

Proceed without importing repair/query operations or declaring any replacement
wire vocabulary. A later supported adapter requires its own specification and
consumer tests.
