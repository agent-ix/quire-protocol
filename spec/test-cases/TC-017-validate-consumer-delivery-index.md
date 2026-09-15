---
id: TC-017
title: "Validate the consumer-owned delivery index"
type: TC
relationships:
  - target: ix://agent-ix/quire-protocol/FR-018
    type: verifies
  - target: ix://agent-ix/quire-protocol/NFR-006
    type: verifies
---
# TC-017: Validate the Consumer-Owned Delivery Index

## Description

Verify the already-specified FR-018 consumer record independently from TC-013's
version-lock resolver. The index must retain exact delivered state and report
every malformed or absent state while remaining structurally unable to supply,
select, repair or widen a version-lock entry.

## Test Data

Protocol-owned delivery roots and in-memory byte fixtures covering two producer
revisions; equal identity with differing offer/input/temporal/limit/mutation
state; exact and recanonicalized byte sequences; complete, missing, surplus,
duplicate and reordered temporal declarations; every specified missing or
unreadable index/member state; ambiguous mutations; duplicate and permuted
owner-qualified limits; and same-spelled Protocol/compiled-protocol dimensions.

## Test Procedure

1. Build indices with equal and unequal (`delivery_root`, `producer_revision`)
   keys while varying every non-key field independently.
2. Read indexed offer and input members whose alternate canonical encoding is
   byte-different, then compare the retained/submitted byte sequences.
3. Validate complete bindings, then mutate cardinality, declaration uniqueness
   and ascending declaration order one axis at a time.
4. Exercise missing root, absent index, missing member, unreadable index,
   unreadable member, ambiguous mutation, repeated declaration, unsorted
   binding and unindexed-input-role cases independently.
5. Validate exact owner/contract/dimension limit keys, then duplicate, permute
   and cross owner vocabularies while preserving similar labels.
6. Resolve the same manifest and delivered members with the index absent,
   present, edited to name an uncovered member and deleted again.

## Expected Results

Identity comparison uses only the two specified key components. Exact producer
bytes are returned unchanged and no digest is synthesized. Each malformed or
absent state returns its own typed outcome naming the offending component;
permutation changes no accepted limit set. Protocol and compiled-protocol
dimensions never substitute. All four resolver runs in step 6 return the same
outcome and accepted-byte set, proving the index carries evidence but no
version-lock authority.

## Metadata

- Priority: P0
- Type: Property
- Automation: Automated; Protocol-owned fixtures only

## Traceability

Verifies FR-018-AC-1, FR-018-AC-3, FR-018-AC-5, FR-018-AC-6,
FR-018-AC-8, FR-018-AC-9 and NFR-006-AC-8. FR-018-AC-2 and FR-018-AC-4
remain inspection obligations; FR-018-AC-7 remains an analysis obligation.
