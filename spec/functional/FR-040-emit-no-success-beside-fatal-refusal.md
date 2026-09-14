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

When an admission, representation, result or projection refusal is fatal, the library SHALL emit no success value for that operation.
When an assessment request refuses, the library SHALL retain no result except the independent claim results FR-005 expressly allows.
The library SHALL NOT treat the absence of a success value as a refusal identity.

## Inputs

One fatal admission, representation, result or projection refusal, or one
assessment request in which at least one claim refused while other claims were
dispatched independently under
[FR-005](./FR-005-dispatch-independent-protocol-claims.md).

## Outputs

For a fatal refusal, the envelope alone and no success value. For a refusing
assessment, the envelope together with exactly those independent claim results
FR-005 authorizes to survive another claim's failure, and no other result.

## Behavior

The library SHALL suppress every declared success value of an operation whose refusal is fatal.
The library SHALL let exactly the FR-005-authorized independent claim results, and no further result, survive another claim's failure.
The library SHALL NOT report a refusal as a completed assessment whose finding set is legitimately empty.
The library SHALL NOT accept the absence of a success value as the distinguishing feature between two refusal axes.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-040-AC-1 | A fatal admission, representation, result or projection refusal emits no success value, and any co-emitted declared success value fails the path. | Test (TC-011) |
| FR-040-AC-2 | Only the independent assessment results FR-005 authorizes survive another claim's failure, and any other retained result under a refusing assessment fails. | Test (TC-011) |
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
by SR-027 on version 1. `tests/it001_v2_handoff.rs` instead compares each
manifest case to the producer's exact `expected_refusal_code`. FR-040-AC-4
therefore fails a two-valued integration rather than crediting it.

### Where the upstream vocabulary does not yet reach

This requirement states no producer-owned noun and therefore adopts no upstream
interior. It nevertheless inherits the surrounding gate: the envelope its
criteria observe carries a selected diagnostic-catalog identity, revision and
digest whose
[FR-009](./FR-009-lock-external-producer-revisions.md) version-lock entry is
`unresolved`, because `ix://agent-ix/quire-specification/FR-270` states of the
only catalog so far offered: "**Proposed, not adopted or implemented.**" A
scenario requiring that entry is reported unexecutable under FR-009 rather than
passing or failing.

**Gap, owner this repository's FR-014 and its cited upstream sources:** FR-014
records that "incomplete" carries unreconciled senses across `NFR-002`, `FR-008`
and `IT-003-SC-06`, and that no functional requirement introduces incompleteness
as a third outcome kind. FR-040-AC-3 is therefore stated over the two outcomes
that are owned — the typed refusal and the completed result with an empty finding
set — and defers the resource-incomplete comparison to FR-014-AC-5 rather than
restating an outcome kind nobody owns.

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

Draft. No execution is claimed. the version 2 compiled-protocol contract is unpinned,
so the admission and representation paths behind these criteria are unexecutable
today.
