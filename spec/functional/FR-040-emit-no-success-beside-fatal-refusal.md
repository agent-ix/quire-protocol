---
id: FR-040
title: "Emit no success value beside a fatal refusal"
type: FR
relationships:
  - target: ix://agent-ix/quire-protocol/FR-008
    type: part_of
  - target: ix://agent-ix/quire-protocol/FR-014
    type: depends_on
  - target: ix://agent-ix/quire-protocol/FR-036
    type: depends_on
  - target: ix://agent-ix/quire-protocol/FR-005
    type: depends_on
  - target: ix://agent-ix/quire-protocol/FR-006
    type: depends_on
---
# FR-040: Emit no success value beside a fatal refusal

## Description

When a resolution, admission, representation, whole-assessment, result, projection or evidence refusal is fatal, the library SHALL emit no success value for that operation.
When one admitted claim cannot construct a valid canonical row, the library SHALL retain its claim-local refusal and no result for that claim, while retaining exactly the independent sibling claim outcomes FR-005 expressly allows.
The library SHALL NOT treat the absence of a success value as a refusal identity.

## Inputs

One fatal resolution, admission, representation, whole-assessment, result,
projection or evidence refusal, or one assessment request in which at least one
admitted claim has a construction refusal while other claims were
dispatched independently under
[FR-005](./FR-005-dispatch-independent-protocol-claims.md).

## Outputs

For a fatal refusal, the envelope alone and no success value. For a claim-local
construction refusal, one ordered `ClaimOutcome::Refused` together with exactly
the independent sibling outcomes FR-005 authorizes, and no canonical row for
the affected claim.

## Behavior

The library SHALL suppress every declared success value of an operation whose refusal is fatal.
The library SHALL let exactly the FR-005-authorized independent claim results, and no further result, survive another claim's failure.
The library SHALL NOT report a refusal as a completed assessment whose finding set is legitimately empty.
The library SHALL NOT accept the absence of a success value as the distinguishing feature between two refusal axes.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-040-AC-1 | A fatal resolution, admission, representation, whole-assessment, result, projection or evidence refusal emits no success value, and any co-emitted declared success value fails the path. | Test (TC-011) |
| FR-040-AC-2 | One claim-local construction refusal emits no canonical row for that claim; only the independent sibling outcomes FR-005 authorizes survive, and any other retained result fails. | Test (TC-011) |
| FR-040-AC-3 | A refusal and a completed assessment whose finding set is legitimately empty compare unequal, and neither is reported as the other. | Test (TC-011, IT-003) |
| FR-040-AC-4 | An integration whose negative rows are distinguished only by the absence of a success value fails the distinctness check, because absence of success is not a refusal identity and one shared code behind every such row satisfies the rows as written. | Test (TC-011) |

## Adopted vocabulary

The refusal envelope is [FR-008](./FR-008-return-typed-refusals.md)'s and the
partial-output state it carries is FR-008's; this requirement states what may and
may not accompany it.
[FR-005](./FR-005-dispatch-independent-protocol-claims.md) owns which independent
claim results survive another claim's failure, and
[FR-006](./FR-006-produce-canonical-protocol-results.md) owns the completed
canonical result whose finding set may be legitimately empty.
[FR-014](./FR-014-typed-outcome-identity.md) holds the resource-incomplete
outcome, the typed refusal and the completed-empty result pairwise non-equivalent.

FR-040-AC-4 is this requirement's injectivity obligation. A suppression rule is
the weakest possible oracle: if "returned no success value" is what a negative row
checks, one shared error code satisfies every negative row — the defect recorded
by SR-027 on version 1. A conforming test instead compares each
manifest case to the producer's exact `expected_refusal_code`. FR-040-AC-4
therefore fails a two-valued integration rather than crediting it.

Resource incompleteness is an independently owned operation outcome before a
canonical claim row exists and FR-241's canonical disposition after admission;
it is never a refusal cause. FR-014 owns the pairwise distinction among that
outcome, a refusal and a completed-empty result.

## Dependencies

- [FR-008](./FR-008-return-typed-refusals.md) owns the envelope; this requirement
  is `part_of` it.
- [FR-005](./FR-005-dispatch-independent-protocol-claims.md) authorizes the
  surviving independent claim results.
- [FR-006](./FR-006-produce-canonical-protocol-results.md) owns the completed
  canonical result.
- [FR-014](./FR-014-typed-outcome-identity.md) owns the typed outcome identity and
  the three-way outcome distinction.

## Status

Implemented by Task-006 and TC-011 across the Protocol's current public paths.
