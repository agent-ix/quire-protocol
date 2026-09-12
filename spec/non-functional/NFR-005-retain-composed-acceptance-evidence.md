---
id: NFR-005
title: "Retain composed acceptance evidence per integration scenario"
type: NFR
quality_attribute: maintainability
relationships:
  - target: ix://agent-ix/quire-protocol/ADR-005
    type: references
  - target: ix://agent-ix/quire-protocol/US-004
    type: derives_from
  - target: ix://agent-ix/quire-protocol/FR-009
    type: constrains
  - target: ix://agent-ix/quire-protocol/FR-006
    type: constrains
  - target: ix://agent-ix/quire-protocol/FR-008
    type: constrains
  - target: ix://agent-ix/quire-protocol/NFR-001
    type: depends_on
  - target: ix://agent-ix/quire-protocol/NFR-002
    type: depends_on
  - target: ix://agent-ix/quire-protocol/NFR-003
    type: depends_on
---
# NFR-005: Retain composed acceptance evidence per integration scenario

## Statement

The library SHALL retain, for every scenario of IT-002 and IT-003 that reaches a
typed outcome, the resolution state it ran against, the owner of each consumed
producer value, its typed outcome, the budget granted and consumed on each
declared dimension, and the trace binding attributing it to its acceptance
criterion.

## Scope

- Applies to: every scenario of IT-002 and IT-003. IT-001 is out of scope: it
  declares neither this requirement nor FR-009 and carries no manifest
  precondition.
- Applies to a scenario that refuses *during* resolution as the partial
  resolution state reached plus the entry that caused the refusal — such a
  scenario has no full snapshot and must not be disqualified for lacking one.
- Does not apply to: retention, storage, sufficiency assessment or human
  disposition, which Quoin and Engineering Assurance own. This requirement fixes
  what a scenario must carry to its handoff, never where it is kept afterwards or
  whether it suffices.

## Rationale

An integration composing several externally owned producers can pass for the
wrong reason: the right outcome from the wrong revision, a refusal sharing one
code with an unrelated axis, or a budget silently carried over from a previous
attempt. Evidence that does not name the resolution state, the owner, the
distinct typed outcome and the per-dimension budget cannot separate those cases
after the fact, and a green scenario then proves less than it appears to.

## Measurement and Evaluation

Every metric is denominated on the scenarios each integration **owes**, not on
those it executed, so a blocked integration scores zero rather than a vacuous
hundred per cent.

| Metric | Target | Threshold | Method |
|--------|--------|-----------|--------|
| Owed scenarios retaining their resolution state | 100% | 100% | Inspection |
| Owed scenarios retaining the owner of every consumed producer value | 100% | 100% | Inspection |
| Refusal axes within one integration whose retained typed outcome is distinct from every other axis in that integration | 100% | 100% | Analysis |
| Owed scenarios retaining granted and consumed budget on every declared dimension | 100% | 100% | Inspection |
| Owed scenarios bound to an acceptance criterion by a compiler-checked trace attribute | 100% | 100% | Analysis |

## Verification

`quire coverage` reports the trace bindings. A static inspection over the
integration lanes asserts that each owed scenario records its resolution state,
the per-value owner, its typed outcome and its per-dimension budget figures, and
reports an owed-but-unexecuted scenario as a zero rather than omitting it from
the denominator.

The distinctness check is applied per **refusal axis**, not per scenario: two
scenarios exercising one axis may legitimately share a typed outcome, while two
distinct axes sharing one may not. The check fails the integration when two
axes collide.

The budget figures are retained per declared dimension. Elapsed time is recorded
but excluded from the reproducibility comparison, because it is not reproducible
and NFR-001 requires the retained evidence to be.

## Dependencies

- **Sequencing**: this requirement cannot be satisfied before FR-009 produces a
  resolution state to retain, which is stage 5 of FR-009's gate order.
- **Upstream**: [FR-009](../functional/FR-009-lock-external-producer-revisions.md)
  supplies the resolution state; [FR-008](../functional/FR-008-return-typed-refusals.md)
  supplies the typed outcome identities; [NFR-002](./NFR-002-enforce-resource-bounds.md)
  supplies the per-dimension budget figures; [NFR-001](./NFR-001-reproduce-semantic-results.md)
  requires the retained evidence to be reproducible; and
  [FR-006](../functional/FR-006-produce-canonical-protocol-results.md) supplies
  the canonical result a scenario's outcome refers to.
- **Downstream**: a scenario record lives only until the scenario hands it to
  Quoin through the accepted structured interface. It is passed, never stored:
  this repository adds no local evidence store, catalogue, sufficiency decision,
  or disposition path, and [NFR-003](./NFR-003-preserve-rust-assurance-boundary.md)
  bounds that.
