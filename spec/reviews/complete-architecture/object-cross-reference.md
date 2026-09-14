---
id: SR-097
title: "Object and cross-reference review of the complete Protocol architecture"
type: SpecReview
analysis: base
scope: "FR-010..FR-018 plus all adopted QSpec/QSL/QObs objects and Plan-001 references"
review_set: all
relationships:
  - { target: ix://agent-ix/quire-protocol/FR-010, type: reviews }
  - { target: ix://agent-ix/quire-protocol/AD-001, type: reviews }
---
# SR-097: Object and cross-reference review of the complete Protocol architecture

## Summary

Local objects, adopted owner objects and their cross-references were checked for
identity completeness, authority, ordering and implementation ownership.

## Verdict

**PASS after remediation.** Every locally owned object has one identity kind,
one authority, explicit equality/absence/order rules, and one implementation
owner. Adopted values remain owner types rather than local wire mirrors.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-001 | high | Resolved: runtime subject references were previously slot-derived and could collide with static requirements. Runtime wrappers now retain the complete QSpec FR-287 authority/kind/opaque-byte key and static definitions use FR-288 componentwise identity. | AD-001 Admission/binding; FR-002; Task-002 | wrong-requirement |
| FND-002 | high | Resolved: version-lock capabilities lacked request/value binding. `ResolvedArtifact<C>` and `ResolvedInterface<C>` are private and retain resolution, request, key, revision and bytes/interface selection. | FR-009; FR-012; Task-013 | missing-requirement |
| FND-003 | medium | Resolved: delivery-index limits had no key/order. FR-018 now uses an owner-qualified tagged dimension, duplicate refusal and deterministic non-causal ordering. | FR-018 | missing-requirement |
| FND-004 | medium | Resolved: local digest names conflicted with QSpec FR-201. FR-015 selects only exact owner labels, and code already exposes the matching two variants. | FR-015; ADR-003; `src/digest/producer.rs` | wrong-requirement |
| FND-005 | low | QSpec artifact `Status` prose for #40 remains stale but issue #40 and PR #44 are accepted. Protocol cites the accepted ticket/merge and semantic content, not the stale status paragraph. | QSpec #40 / PR #44 | wrong-requirement |

The only unresolved semantic object question is QSpec FR-236's per-attempt vs
aggregate outcome partition, explicitly isolated to affected Task-003 mappings.
