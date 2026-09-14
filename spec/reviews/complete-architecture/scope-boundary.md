---
id: SR-095
title: "Scope-boundary review of the complete Protocol architecture"
type: SpecReview
analysis: scope-boundary
scope: "Protocol library and all external producer/consumer seams in AD-001"
review_set: all
relationships:
  - { target: ix://agent-ix/quire-protocol/AD-001, type: reviews }
  - { target: ix://agent-ix/quire-protocol/NFR-003, type: reviews }
---
# SR-095: Scope-boundary review of the complete Protocol architecture

## Summary

Each pipeline responsibility and external seam was allocated to exactly one
owner and checked for prohibited local substitutes.

## Verdict

**PASS.** Protocol owns resolution, admission/binding, pure assessment,
canonical Protocol results, E02 adaptation and transient evidence composition;
it owns none of its producers' languages, facts or execution and owns no
evidence store or campaign verdict.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-001 | high | Resolved: QCI's native/TL join was treated as a Protocol input. It is downstream of the Protocol leaf-result view. | AD-001; IT-002 | wrong-requirement |
| FND-002 | high | Resolved: a production recovery callback could import an arbitrary evaluator. Only the admitted QSL expression and capability remain public. | FR-024; Task-003 | wrong-requirement |
| FND-003 | medium | Resolved: evidence “retention” wording implied a Protocol store. The record is composed and passed to Quoin, never stored/read/scored by Protocol. | FR-017; NFR-005; ADR-005 | wrong-requirement |

## Boundary checks

- QSL owns grammar, compilation, compiled protocol and native temporal
  evaluation. Protocol consumes typed artifacts and evaluates no replacement
  language.
- QObs owns collection, runtime subject/relationship values and authority
  assertions. Protocol validates bindings and never repairs observations.
- Engineering Assurance owns producer execution. Protocol adapts typed request
  and result values and never spawns/parses a producer process.
- QCI is downstream of `MappedResultView`; Protocol never consumes its
  native/TL join.
- qverify owns the E02 reader contract. Protocol projects only after the exact
  accepted capability resolves.
- Quoin owns storage/indexing. Protocol passes one transient typed scenario
  record and never stores, reads back, scores or disposes it.
- Agent B's TL libraries remain outside this lane and are not edited by this
  campaign.

The frozen root `/1` facade is a narrowly documented compatibility exception,
not an alternate integration path. New full paths use the versioned modules and
resolved capabilities.
