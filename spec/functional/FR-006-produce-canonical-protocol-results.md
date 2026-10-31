---
id: FR-006
title: "Produce canonical protocol results"
type: FR
relationships:
  - target: ix://agent-ix/quire-protocol/ADR-006
    type: references
  - target: ix://agent-ix/quire-protocol/US-002
    type: implements
  - target: ix://agent-ix/quire-protocol/FR-004
    type: depends_on
  - target: ix://agent-ix/quire-protocol/FR-005
    type: depends_on
  - target: ix://agent-ix/quire-protocol/FR-031
    type: contains
  - target: ix://agent-ix/quire-protocol/FR-032
    type: contains
  - target: ix://agent-ix/quire-protocol/FR-033
    type: contains
  - target: ix://agent-ix/quire-protocol/FR-015
    type: depends_on
  - target: ix://agent-ix/quire-specification/FR-061
    type: depends_on
  - target: ix://agent-ix/quire-specification/FR-201
    type: references
  - target: ix://agent-ix/quire-specification/FR-240
    type: references
  - target: ix://agent-ix/quire-specification/FR-241
    type: references
  - target: ix://agent-ix/quire-specification/FR-242
    type: references
  - target: ix://agent-ix/quire-specification/FR-243
    type: references
  - target: ix://agent-ix/quire-specification/FR-244
    type: references
  - target: ix://agent-ix/quire-specification/FR-245
    type: references
  - target: ix://agent-ix/quire-specification/FR-246
    type: references
  - target: ix://agent-ix/quire-specification/FR-247
    type: references
  - target: ix://agent-ix/quire-specification/FR-248
    type: references
  - target: ix://agent-ix/quire-specification/FR-280
    type: references
  - target: ix://agent-ix/quire-specification/FR-281
    type: references
---
# FR-006: Produce canonical protocol results

## Description

When an assessment disposition is available, the result producer SHALL emit one strict-reader-valid `quire.protocol.result/v1-draft.1` artifact through the atomic requirements FR-031 through FR-033 that together discharge source requirement FR-061.
When any single atomic requirement FR-031 through FR-033 is unsatisfied, the result producer SHALL emit a typed refusal rather than an aggregate result.

## Inputs

One constructor-private completed assessment context, its compensation snapshot,
conformance report and ordered claim dispositions; the shared typed progress,
closure, completeness, activation, participation, assumption, support, adequacy,
provenance and limitation values derived by those stages; and any applicable
validated prior result. Production callers SHALL NOT assemble a wire-shaped
`Input` by independently restating those values.

## Outputs

One ordered `ClaimOutcome` per admitted requested claim: either one canonical
protocol result carrying the structural FR-286 `resultKey`, or one typed
construction/strict-reader refusal for that claim.
Independent sibling outcomes remain present.

## Behavior

This requirement is the aggregate. It establishes no obligation of its own
beyond composition: each obligation it previously stated in prose is now stated,
once and independently verifiably, by exactly one of its parts.

| Part | Obligation it owns |
| --- | --- |
| [FR-031](./FR-031-canonical-result-identity-and-digest.md) | Contract version, kind, structural result key, RFC 8785 canonical bytes, and provenance identity matching. |
| [FR-032](./FR-032-retain-orthogonal-result-axes.md) | Separate retention of every result axis, the FR-061 result-combination table, adequacy-domain disjointness, open-subject settlement, and the complete-global-conformance closure record with its premise set. |
| [FR-033](./FR-033-preserve-result-lineage-integrity.md) | Decision-support dependence relations and their lineage refusals, and the superseding or invalidating result that preserves the earlier bytes and identity. |

The three parts are separate requirements because their failures are separate
failures: a result may be byte-canonical and still carry a forbidden axis
combination, and it may satisfy the combination table and still carry a cyclic
support lineage. No part's success substitutes for another's.

The structural result identity enters this requirement through
[FR-031](./FR-031-canonical-result-identity-and-digest.md), which consumes the
FR-286 `resultKey` without deriving a digest. RFC 8785 canonicalization governs
the emitted bytes as stated by FR-240.

Activation, assumptions, adequacy, participation, claim kind/strength,
settlement and decision support SHALL be the same domain values consumed or
produced by the preceding subsystems. Result assembly is crate-private; strict
reading is the only constructor of `ValidatedResult` from external bytes.

The assessment operation uses two levels. `OperationOutcome<T>` is exactly
`Completed(T)`, `Refused(ProtocolRefusal)` or
`ResourceIncomplete(ResourceIncomplete)`. A completed assessment contains an
`AssessmentBundle` with one `ClaimOutcome` for each admitted claim. A canonical
row may itself carry any FR-241 semantic disposition; a claim-local row-
construction failure instead carries `ClaimOutcome::Refused` and no row.
Resolution, admission, linking or whole-assessment invariant failure produces
no bundle. This algebra is shared with FR-008, FR-014 and NFR-002; no subsystem
declares a substitute.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-006-AC-1 | Every valid FR-061 result-combination row derived from the typed assessment stages is accepted and every one-axis forbidden combination or caller-built substitute stage value is refused, which is exactly [FR-032](./FR-032-retain-orthogonal-result-axes.md)-AC-1, and no aggregate path admits a row that part refuses. | Test (TC-006) |
| FR-006-AC-2 | Settled truth on an open subject requires exact complete decisive support whose truth is preserved by every admitted continuation, which is exactly [FR-032](./FR-032-retain-orthogonal-result-axes.md)-AC-2. | Test (TC-006) |
| FR-006-AC-3 | Assumption, observation-adequacy, protocol-adequacy, execution, truth, activation, participation, decision-scope progress, decision-scope closure, surrounding-execution progress, surrounding-execution closure, complete-global-conformance closure, settlement, claim, and completeness mutations remain independently observable, per [FR-032](./FR-032-retain-orthogonal-result-axes.md)-AC-3 and -AC-4. | Test (TC-006) |
| FR-006-AC-4 | Every source, profile, model, protocol, request, binding, input, backend, decision-scope progress/closure, surrounding-execution progress/closure, complete-global-conformance closure, settlement, support, producer, and prior-result identity is recomputed or strictly matched: [FR-031](./FR-031-canonical-result-identity-and-digest.md)-AC-5 covers the provenance identities, [FR-032](./FR-032-retain-orthogonal-result-axes.md)-AC-7 the axis identities, and [FR-033](./FR-033-preserve-result-lineage-integrity.md)-AC-3 and -AC-4 the support and prior-result identities. | Test (TC-006) |
| FR-006-AC-5 | Support lineage rejects missing predecessors, cycles, self-reference, and invented independence while retaining shared and derived relations, which is exactly [FR-033](./FR-033-preserve-result-lineage-integrity.md)-AC-1. | Test (TC-006) |
| FR-006-AC-6 | A later contradiction emits a new linked result and cannot rewrite the earlier artifact bytes or identity, which is exactly [FR-033](./FR-033-preserve-result-lineage-integrity.md)-AC-2. | Test (TC-006) |
| FR-006-AC-7 | Independently omitting, duplicating, cross-wiring, changing scope on, or changing the identity of every required execution, branch, and workflow global-closure premise refuses the record or prevents complete global success while leaving an independently settled per-obligation truth unchanged, which is exactly [FR-032](./FR-032-retain-orthogonal-result-axes.md)-AC-5. | Test (TC-006) |
| FR-006-AC-9 | The public assessment operation returns exactly the three-member operation outcome; each completed bundle has one ordered canonical row or claim-local refusal per admitted claim, a whole-operation refusal/resource-incomplete outcome has no bundle, and no outcome is accepted in another's place. | Test (TC-006, TC-011) |

## Adopted vocabulary

Every axis this requirement keeps separate is an upstream object. Each part
carries the slice of this table it uses; the table is retained here as the
shared mapping the decomposition was derived from, not as a second definition.

| Axis | Upstream object |
| --- | --- |
| Result fact set | `FR-240-canonical-protocol-result` |
| Execution disposition | `FR-241-assessment-execution-disposition` |
| Truth | `FR-242-truth-disposition` |
| Settlement basis | `FR-243-settlement-basis` |
| Activation | `FR-244-activation-state` |
| Observation adequacy | `FR-245-observation-adequacy-state` |
| Protocol adequacy | `FR-246-protocol-adequacy-state` |
| Complete-global-conformance closure | `FR-247-complete-global-conformance-closure-state` |
| Decision support | `FR-248-decision-support-record` |
| Assumptions | `FR-280-assumption-state` |
| Claim strength | `FR-281-claim-strength` |

`FR-247`'s five members — `closed`, `open`, `incomplete`, `contradicted`,
`not-required` — match this requirement's previous local enumeration exactly,
including the rule that a claim which is not global conformance carries no
global premise set. The local enumeration is therefore replaced by the reference
rather than reconciled.

### Accepted upstream rulings

The implementation consumes the promoted shared object authority rather than
retaining earlier gap descriptions. FR-286 fixes the complete derived result
identity preimage and strict-reader order; FR-284 fixes each of the four
progress/closure axes to independent `open`/`closed`; FR-295 fixes completeness
to `complete`/`incomplete`/`contradicted`; FR-243 fixes all five settlement
members; and FR-285 fixes participation to its exact known/unknown form. No
local aliases or compatibility fallbacks survive these rulings.

## Dependencies

- [FR-031](./FR-031-canonical-result-identity-and-digest.md),
  [FR-032](./FR-032-retain-orthogonal-result-axes.md) and
  [FR-033](./FR-033-preserve-result-lineage-integrity.md) are this requirement's
  parts.
- [FR-004](./FR-004-assess-finite-global-conformance.md).
- [FR-005](./FR-005-dispatch-independent-protocol-claims.md).
- `ix://agent-ix/quire-specification/FR-061`.
