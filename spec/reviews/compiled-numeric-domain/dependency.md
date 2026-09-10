---
id: SR-017
title: "Dependency review of compiled numeric-domain admission"
type: SpecReview
analysis: dependency
scope: "FR-001 and IT-001 at fde0300"
review_set: all
relationships:
  - { target: ix://agent-ix/quire-protocol/FR-001, type: reviews }
---
## Summary

Reviewed the producer and semantic dependency order. The compiler-owned
artifact contract selects its numeric wire profile before B admits or links any
semantic value; no reverse dependency or local producer substitute is added.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No issues found; quire-spec-language#40 and composed candidate adoption remain hard external gates, followed by FR-001 admission and only then FR-002 linking. | FR-001; IT-001 |

## Dependency Order

1. A accepts the composed semantic candidate and compiler artifact contract,
   including one lossless numeric wire profile.
2. FR-001 recomputes identities and admits exact bytes.
3. FR-002 and later protocol evaluation consume only the admitted typed value.

No dependency cycle exists. Engineering Assurance is an execution dependency
for later integration evidence and is not a semantic decoder dependency.
