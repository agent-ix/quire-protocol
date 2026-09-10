---
id: SR-018
title: "Evidence review of compiled numeric-domain admission"
type: SpecReview
analysis: evidence
scope: "FR-001-AC-4, TC-001, IT-001-SC-04 and TM-001 at fde0300"
review_set: all
relationships:
  - { target: ix://agent-ix/quire-protocol/FR-001, type: reviews }
---
## Summary

Ran `quoin advise --repo . --json` against the reviewed revision. FR-001-AC-4
is catalogued, conclusive and class-compatible with authored `Test`; its matched
serialization and round-trip characteristics recommend property,
metamorphic and golden tests.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No method mismatch or inconclusive result exists for FR-001-AC-4; implementation must combine generated exact-value round trips with retained canonical byte vectors rather than using the producer as its own oracle. | FR-001-AC-4; TC-001; IT-001-SC-04 |

## Evidence Strategy

- Property tests generate every admitted representation class and one-axis
  invalid forms across field, safe-integer and signed-64 boundaries.
- Metamorphic tests require decode/re-encode byte identity and exact semantic
  value preservation.
- Golden vectors independently pin canonical safe integer, signed-decimal and
  normalized rational forms from the accepted producer contract.
- IT-001 exercises the real compiler-produced artifact through the public Rust
  reader. All rows remain Planned until that producer contract is accepted.
