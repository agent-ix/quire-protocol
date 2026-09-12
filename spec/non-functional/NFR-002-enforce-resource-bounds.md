---
id: NFR-002
title: "Enforce protocol and caller resource bounds"
type: NFR
quality_attribute: performance_efficiency
relationships:
  - target: ix://agent-ix/quire-protocol/US-006
    type: derives_from
  - target: ix://agent-ix/quire-specification/NFR-020
    type: depends_on
  - target: ix://agent-ix/quire-protocol/FR-001
    type: constrains
  - target: ix://agent-ix/quire-protocol/FR-002
    type: constrains
  - target: ix://agent-ix/quire-protocol/FR-003
    type: constrains
  - target: ix://agent-ix/quire-protocol/FR-004
    type: constrains
  - target: ix://agent-ix/quire-protocol/FR-005
    type: constrains
  - target: ix://agent-ix/quire-protocol/FR-006
    type: constrains
  - target: ix://agent-ix/quire-protocol/FR-007
    type: constrains
  - target: ix://agent-ix/quire-protocol/FR-008
    type: constrains
---
# NFR-002: Enforce protocol and caller resource bounds

## Statement

The library SHALL enforce every authored protocol limit and every positive
caller-supplied input-byte, definition, node, edge, role, message, branch,
iteration, active-instance, observation, lineage, result-item, derived-artifact,
output-byte, operation, accounted-allocation, and elapsed-time limit selected
for the exact request according to the boundary rules in this requirement.

## Scope

Applies to strict reading, linking, compensation assessment, finite replay,
result construction, support-lineage validation, and E02 projection.

Count, byte, operation, and accounted-allocation maxima are inclusive. The
library checks the requested increment before allocating, decoding, inserting,
or starting the next unit of work and refuses that unit if it would exceed its
maximum. Elapsed time uses a monotonic deadline checked before and after each
declared indivisible unit. The library starts no later unit after the deadline;
it returns resource-incomplete with the observed elapsed time and any indivisible-
unit overshoot. Engineering Assurance owns whole-process containment and external
resource enforcement.

This constraint propagates through containment. Where a requirement this
non-functional requirement constrains is an aggregating parent, the constraint
binds every requirement that parent `contains`, to the same extent and without
a separate edge per child. The `constrains` edges in the frontmatter name the
parents; enumerating each child instead would drift the moment a parent gains
one, which is the failure this repository has already paid for elsewhere.

## Rationale

Finite syntax alone does not prevent adversarial size or combinatorial cost.
Protocol authors supply semantic maxima and the request owner supplies execution
budgets; the implementation must enforce both without inventing a universal
threshold.

## Measurement and Evaluation

| Metric | Target | Threshold | Method |
| --- | --- | --- | --- |
| Inclusive authored and caller-supplied count/byte/operation/accounted-allocation limits with below/at/over controls | Every declared static dimension | 100% boundary discrimination | Test (TC-009) |
| Monotonic deadline checks before and after each indivisible unit | Every declared unit boundary | 100% stop-state discrimination | Test (TC-009) |
| Work admitted after a limit is exhausted | None | 0 cases | Test (TC-009) |
| Caller-controlled collection or traversal source sites missing or duplicated in the resource inventory | None | 0 unmatched sites or entries | Test (TC-009) |
| Business actions or compensations executed during assessment | None | 0 calls | Test (TC-009) |

## Verification

Retain a complete source-bound inventory of every reviewed Rust request/result
field and production path that can allocate, retain, queue, iterate, or traverse
caller-controlled data. Each unique entry identifies its owning interface,
limit authority, pre-work charge point, finite termination argument, typed
resource-incomplete code, and assigned TC-009 boundary case. Reconcile source
sites and inventory entries in both directions. Boundary tests exercise each
authored maximum and caller-supplied dimension immediately below, at, and above
its limit. Mutation tests delete and duplicate inventory entries, add an
unlisted source site, remove a boundary binding, and remove or reverse each
guard; each assigned bare-traced case must fail for the intended reason. An
action-capable test double proves that assessment invokes no business action or
compensation.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| NFR-002-AC-1 | Every authored maximum and caller-supplied static limit admits its exact inclusive boundary and returns a typed resource-incomplete result before starting the smallest representable excess unit; removing or reversing its guard fails the assigned case. | Test (TC-009) |
| NFR-002-AC-2 | Every declared indivisible unit checks the monotonic deadline before and after the unit, starts no later work after expiry, and reports the observed elapsed time and any indivisible-unit overshoot. | Test (TC-009) |
| NFR-002-AC-3 | Every caller-controlled allocation, retention, queue, iteration and traversal source site has exactly one complete inventory entry and discriminating boundary case, and every deleted, duplicated, unlisted or boundary-unbound inventory mutation fails. | Test (TC-009) |
| NFR-002-AC-4 | An action-capable double records zero business-action and compensation calls during every assessment boundary case. | Test (TC-009) |

## Dependencies

- `ix://agent-ix/quire-specification/NFR-020` is the formal-clause source.
- [FR-001](../functional/FR-001-admit-selected-protocol-profile.md).
- [FR-002](../functional/FR-002-link-finite-protocol-representation.md).
- [FR-003](../functional/FR-003-assess-compensation-state.md).
- [FR-004](../functional/FR-004-assess-finite-global-conformance.md).
- [FR-005](../functional/FR-005-dispatch-independent-protocol-claims.md).
- [FR-006](../functional/FR-006-produce-canonical-protocol-results.md).
- [FR-007](../functional/FR-007-project-results-to-e02.md).
- [FR-008](../functional/FR-008-return-typed-refusals.md).
