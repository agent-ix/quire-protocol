---
id: SR-020
title: "Scope review of compiled numeric-domain admission"
type: SpecReview
analysis: scope-boundary
scope: "FR-001, IT-001 and AD-001 at fde0300"
review_set: all
relationships:
  - { target: ix://agent-ix/quire-protocol/FR-001, type: reviews }
---
## Summary

Reviewed ownership of semantic numbers, wire encoding, digest domains and
strict reading. A's compiler owns the artifact schema and encoding; B owns exact
admission and refuses any value it cannot preserve.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No boundary leak found; the requirement constrains what B may accept without defining native syntax, selecting D's canonicalization or creating a second compiler schema. | FR-001-AC-4; AD-001 |

## Responsibility Allocation

| Boundary | Owner | Contract status |
| --- | --- | --- |
| Native integer/rational meaning | quire-specification and native compiler | Guaranteed through accepted source/profile contract |
| Compiled artifact schema and numeric encoding | quire-spec-language#40 | Required external guaranteed contract; currently unavailable |
| Model/configuration artifact digest | Producer owner | Consumed as exact typed identity; never recomputed in B's JCS domain |
| Exact artifact admission and refusal | quire-protocol FR-001 | B-owned core behavior |
| Process execution and evidence retention | Engineering Assurance / Quoin | Outside this pure reader |

No parser, source frontend, generic runner, evidence store or foreign runtime
enters the boundary.
