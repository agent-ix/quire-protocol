---
id: FR-017
title: "Scenario evidence record"
type: FR
object: entity
relationships:
  - target: ix://agent-ix/quire-protocol/US-004
    type: implements
  - target: ix://agent-ix/quire-protocol/FR-010
    type: part_of
  - target: ix://agent-ix/quire-protocol/FR-011
    type: depends_on
  - target: ix://agent-ix/quire-protocol/NFR-005
    type: specifies
  - target: ix://agent-ix/quire-protocol/FR-016
    type: depends_on
  - target: ix://agent-ix/quire-protocol/FR-012
    type: depends_on
---
# FR-017: Scenario evidence record

## Description

When an integration scenario of IT-001, IT-002 or IT-003 reaches a typed outcome, the library SHALL compose one scenario evidence record carrying the resolution state it ran against or its declared legacy-compatibility selection, the owner of each consumed producer value, its typed outcome, the budget granted and consumed on each declared dimension, and its trace binding.
When a scenario cannot retain every one of those components, the library SHALL report that scenario as unexecuted rather than as passing.

## Properties

- **scenario** (identity component) — the integration scenario this record is
  composed for, such as `IT-002-SC-07a`. One scenario composes at most one
  record.
- **integration** (identity component) — the integration the scenario belongs to:
  `IT-001`, `IT-002` or `IT-003`.
- **resolution_state** (state) — the state of `FR-009` resolution this scenario
  ran against, in exactly one of two admitted shapes:
  - the **full resolved manifest**, where resolution completed; or
  - the **partial resolution state reached plus the entry that caused the
    refusal**, where the scenario refused *during* resolution.

  Both shapes are complete evidence. `NFR-005` Scope is explicit that the second
  "has no full snapshot and must not be disqualified for lacking one", and
  IT-002-SC-07a and IT-003-SC-08a admit both. Requiring a full snapshot would
  disqualify exactly the scenarios that demonstrate resolution refusing, which
  are the scenarios that prove resolution works.
  IT-001-SC-06a's frozen legacy `/1` facade has no resolver capability; its
  present compatibility-selection shape carries the exact selected QSL owner,
  contract and immutable revision instead of pretending to hold one.
- **value_owners** (state) — the owner of each consumed producer value, recorded
  per value rather than per scenario. The owner is the `owner` component of the
  version-lock entry the value resolved through
  ([FR-012](./FR-012-version-lock-entry.md)); it is producer-authored and copied,
  never inferred from a path, a display name or a file location.
- **typed_outcome** (state) — the exact shared operation outcome and, where an
  assessment completed, its ordered claim outcomes: canonical FR-241 rows,
  claim-local typed refusals ([FR-008](./FR-008-return-typed-refusals.md)), or a
  typed pre-row resource-incomplete identity
  ([FR-016](./FR-016-resource-budget-dimension.md)).
- **budget_granted** and **budget_consumed** (state) — the granted and consumed
  figures, retained per declared dimension of
  [FR-016](./FR-016-resource-budget-dimension.md) and never as one scalar total.
  The `elapsed-time` and `resolution-deadline` figures are recorded and excluded
  from the reproducibility comparison.
- **trace_binding** (state) — the compiler-checked trace attribute binding this
  scenario to its acceptance criterion. `NFR-005` measures it by `quire
  coverage`; a scenario whose binding is absent is unbound evidence, not
  evidence of a passing scenario.

### Distinctness is an integration-level invariant

No single record asserts anything about another record. The distinctness
obligation — that no two distinct refusal axes within one integration retain the
same typed outcome identity — is checked **once over every record of that
integration**, per refusal axis and not per scenario. Two scenarios exercising
one axis may legitimately share an identity; two axes may not
(IT-002-SC-07b, IT-003-SC-08b, `NFR-005` Verification). This artifact records the
invariant as a property of the collection, not as a component of the record's
key.

### Passed, never stored

A record lives only until its scenario hands it to Quoin through the accepted
structured interface. `NFR-005` fixes the boundary in both directions: it is
"passed, never stored", and this repository "adds no local evidence store,
catalogue, sufficiency decision, or disposition path". Retention, storage and
indexing are Quoin's; Engineering Assurance owns producer execution;
sufficiency assessment and human disposition remain with their campaign/human
owners; and
[NFR-003](../non-functional/NFR-003-preserve-rust-assurance-boundary.md) bounds
that. This requirement fixes what a scenario must carry to its handoff, never
where it is kept afterwards or whether it suffices.

Consequently the record has no lifecycle, no revision, no supersession and no
disposition state, and nothing in this repository reads one back.

## Identity and comparison

- **Identity kind** — *authored*, authority *consumer*, per
  [FR-011](./FR-011-consumer-identity-discipline.md). The record is composed by
  this repository about its own execution. No producer writes one, and a
  producer-emitted value supplies, selects or repairs no component of it. Being
  consumer-authored, it crosses no boundary as authority: it appears in no
  canonical result as a producer reference, and no producer accepts it as a
  selector. The producer identities it *carries* stay producer-authored and are
  copied byte-exactly; carrying them does not make the record's own identity
  producer-authored.
- **Key** — (`integration`, `scenario`). Equality is componentwise over exactly
  those two. The resolution state, the value owners, the typed outcome, the
  budget figures and the trace binding are state, never identity: two runs of one
  scenario under two budgets are one record's subject, not two subjects, and
  `ix://agent-ix/quire-specification/FR-204` forbids field equality from standing
  in for object identity. A display name, source path, span, message text,
  locale, rendering, wall-clock timestamp or execution position never
  participates.
- **Key order** — lexicographic over (`integration`, `scenario`), for collection
  comparison and duplicate detection only. It carries no causal meaning, is not
  an execution order, and expresses no precedence between integrations or between
  scenarios. The integration-level distinctness check runs over the whole
  collection and does not depend on this order.
- **Causal order** — none between records. Within one record the resolution state
  precedes the outcome, because `FR-009` resolves every required entry before any
  producer value is read; that is `FR-009`'s gate order and not an order over
  records. `NFR-005` Dependencies states the sequencing as a gate: this
  requirement "cannot be satisfied before FR-009 produces a resolution state to
  retain, which is stage 5 of FR-009's gate order."
- **Absence** — an absent record compares only to absence and is never defaulted
  to an empty record, to a passing scenario, or to the record of another
  scenario. A scenario that cannot retain every required component is reported
  unexecuted; an owed-but-unexecuted scenario is scored as a zero rather than
  omitted from the denominator (`NFR-005` Measurement and Evaluation). A partial
  resolution state plus its causing entry is a *present* state, not absence. An
  absent budget grant on a dimension compares only to absence and is not a grant
  of zero.
- **Digest domain** — the record carries no digest of its own. Every digest
  inside the resolution state it retains keeps the domain it was selected under,
  [FR-015](./FR-015-digest-domain-selection.md), and a digest from one domain is
  unequal to one from another over identical bytes.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-017-AC-1 | Every owed scenario of IT-001, IT-002 and IT-003 composes exactly one record carrying its resolution state or declared legacy-compatibility selection, the owner of every consumed producer value, its typed outcome, its per-dimension granted and consumed budget figures, and its trace binding. | Test (TC-016, IT-001, IT-002, IT-003) |
| FR-017-AC-2 | A scenario that refused during resolution composes a record carrying the deterministic partial resolution state reached plus the first causing entry, and is not disqualified for lacking a full resolved manifest. | Test (TC-016, IT-001, IT-002, IT-003) |
| FR-017-AC-3 | A scenario that cannot retain every required component is reported unexecuted rather than passing, and an owed-but-unexecuted scenario is scored as a zero rather than omitted from the denominator. | Inspection |
| FR-017-AC-4 | Across one integration, no two distinct refusal axes retain the same typed outcome interior, while two scenarios exercising one axis may share one. | Test (TC-016) |
| FR-017-AC-5 | Budget figures are retained per declared dimension and never as one scalar total, and the `elapsed-time` and `resolution-deadline` consumed figures participate in no reproducibility comparison. | Test (TC-008, TC-009, TC-016) |
| FR-017-AC-6 | Two records whose integration and scenario are equal compare equal regardless of resolution state, owners, outcome or budget figures, and two records differing in either component compare unequal regardless of identical remaining state. | Test (TC-016) |
| FR-017-AC-7 | No record is written to a local evidence store, catalogue or disposition path, no record is read back after handoff, and no sufficiency decision is taken in this repository. | Test (TC-010, TC-016) |
| FR-017-AC-8 | No record appears in a canonical result as a producer reference, and no producer accepts a record or any component this repository authored in it as a selector. | Test (TC-014, TC-016) |

## Dependencies

- [NFR-005](../non-functional/NFR-005-retain-composed-acceptance-evidence.md)
  establishes the retained component set, the partial-resolution admission, the
  per-axis distinctness check, and the passed-never-stored boundary.
- [FR-009](./FR-009-lock-external-producer-revisions.md) produces the resolution
  state and fixes the gate order this record sits at stage 5 of.
- [FR-012](./FR-012-version-lock-entry.md) supplies the `owner` component
  recorded per consumed producer value.
- [FR-008](./FR-008-return-typed-refusals.md) supplies the typed refusal
  identities the outcome may take.
- [FR-016](./FR-016-resource-budget-dimension.md) supplies the declared
  dimensions the budget figures are retained on.
- [FR-015](./FR-015-digest-domain-selection.md) selects the domain of every
  digest inside a retained resolution state.
- [FR-006](./FR-006-produce-canonical-protocol-results.md) supplies the canonical
  result a scenario's outcome refers to.
- [NFR-001](../non-functional/NFR-001-reproduce-semantic-results.md) requires the
  retained evidence to be reproducible.
- [NFR-003](../non-functional/NFR-003-preserve-rust-assurance-boundary.md) bounds
  the handoff and keeps storage and disposition outside this boundary.
- [FR-011](./FR-011-consumer-identity-discipline.md) supplies the kind and
  authority axes and the rule that a consumer-authored identity crosses no
  boundary as authority.

## Status

Specified. QSL, QObs and Engineering Assurance contracts are pinnable;
E02/Quoin retain the exact blockers in FR-009.
Task-018 implements final composition after the shared outcome and resource
types exist; consuming integration tasks populate their concrete values.
