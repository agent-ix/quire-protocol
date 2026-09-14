---
id: NFR-005
title: "Compose acceptance evidence through each integration handoff"
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
# NFR-005: Compose acceptance evidence through each integration handoff

## Statement

The library SHALL compose and carry through handoff, for every scenario of
IT-001, IT-002 and IT-003 that reaches a
typed outcome, the resolution state it ran against, the exact owner, contract
version and immutable revision of each consumed producer value, its typed outcome, the budget granted and consumed on each
declared dimension, and the trace binding attributing it to its acceptance
criterion.

## Scope

- Applies to: every scenario of IT-001, IT-002 and IT-003. IT-001's normal
  version-explicit integration paths resolve QSL capabilities under FR-009; its
  SC-06a legacy-facade comparison records the selected immutable QSL revision
  as an explicit compatibility exception rather than minting a capability.
- Applies to a scenario that refuses *during* resolution as the partial
  resolution state reached plus the entry that caused the refusal — such a
  scenario has no full snapshot and must not be disqualified for lacking one.
- Does not apply to: storage, sufficiency assessment or human disposition, which
  Quoin and human owners control. Engineering Assurance owns producer execution,
  not evidence storage. This requirement fixes
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
| Owed scenarios retaining the exact owner, contract version and immutable revision of every consumed producer value | 100% | 100% | Inspection |
| Refusal axes within one integration whose retained typed outcome is distinct from every other axis in that integration | 100% | 100% | Analysis |
| Owed scenarios retaining granted and consumed budget on every declared dimension | 100% | 100% | Inspection |
| Owed scenarios bound to an acceptance criterion by a compiler-checked trace attribute | 100% | 100% | Analysis |

## Verification

`quire coverage` reports the trace bindings. A static inspection over the
integration lanes asserts that each owed scenario records its resolution state,
the per-value owner/contract/revision, its typed outcome and its per-dimension budget figures, and
reports an owed-but-unexecuted scenario as a zero rather than omitting it from
the denominator.

The distinctness check is applied per **refusal axis**, not per scenario: two
scenarios exercising one axis may legitimately share a typed outcome, while two
distinct axes sharing one may not. The check fails the integration when two
axes collide.

The budget figures are retained per declared dimension. Elapsed time is recorded
but excluded from the reproducibility comparison, because it is not reproducible
and NFR-001 requires the retained evidence to be.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| NFR-005-AC-1 | Every owed IT-001, IT-002 and IT-003 scenario carries its complete resolution state, or the deterministic partial state and first causing entry when resolution does not complete; an omitted unexecuted scenario counts zero rather than disappearing from the denominator. | Test (TC-016, IT-001, IT-002, IT-003) |
| NFR-005-AC-2 | Every consumed producer value names its exact owner, contract version and immutable revision, and substituting any component refuses evidence composition. | Test (TC-016, IT-001, IT-002, IT-003) |
| NFR-005-AC-3 | Every scenario carries its exact typed operation/claim outcome and distinct refusal axes within one integration have distinct retained interiors; a shared catch-all outcome fails the integration. | Test (TC-016, IT-001, IT-002, IT-003) |
| NFR-005-AC-4 | Every declared resource dimension carries its granted and consumed values under the correct accountant owner/version; missing, duplicated, cross-wired or merged dimensions refuse composition. | Test (TC-016) |
| NFR-005-AC-5 | Every scenario carries a compiler-checked acceptance-criterion trace binding, and a runtime string without that binding does not satisfy it. | Test (TC-016) |
| NFR-005-AC-6 | The composed value is passed to the selected Quoin intake and is never stored, read back, indexed, scored or disposed by Protocol; before that intake is accepted, the handoff returns the exact unexecutable/refusal outcome and no local substitute. | Test (TC-010, TC-016, IT-003) |

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
