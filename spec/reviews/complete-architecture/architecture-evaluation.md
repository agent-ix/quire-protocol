---
id: SR-098
title: "Implemented-reality evaluation of the complete Protocol architecture"
type: SpecReview
analysis: architecture-evaluation
scope: "AD-001 and Plan-001 evaluated against quire-protocol main 34d1752 and current accepted owner revisions"
review_set: all
relationships:
  - { target: ix://agent-ix/quire-protocol/AD-001, type: reviews }
  - { target: ix://agent-ix/quire-protocol/Plan-001, type: reviews }
---
# SR-098: Implemented-reality evaluation of the complete Protocol architecture

## Summary

The proposed architecture and task graph were evaluated against Protocol main
`34d1752`, current owner code and live issue state rather than prior tracking
claims.

## Verdict

**PASS as the implementation architecture; implementation remains partial.**
The design preserves useful landed kernels, names every missing composition
boundary and assigns each gap to one ordered task. It does not mistake module or
type presence for end-to-end acceptance.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-001 | high | Resolved: the previous plan treated disconnected kernels as a nearly composed path. AD-001 now names every typed transition and Task-002 through Task-006 own the missing serial composition. | AD-001; Plan-001; source inventory | wrong-requirement |
| FND-002 | high | Resolved: PR #50/#49 combined resolver and evidence work at opposite ends of the graph. The plan requires narrowing that candidate to Task-013 and defers evidence to Task-018. | Task-013; Task-018 | wrong-requirement |
| FND-003 | medium | Resolved: QObs, QCI, Engineering Assurance and QSL tracking facts were stale. Exact current accepted revisions and directions now appear in the architecture and dependency review. | SR-092; AD-001 Risks | wrong-requirement |
| FND-004 | medium | Open implementation gap, fully allocated: no composed Engine currently connects versioned linking, compensation, conformance, claims and result production. Tasks 002..006 own the ordered closure and their TCs define the oracle. | Plan-001; Tasks 002..006 | correct-requirement-no-evidence |

## Reality map

| Subsystem | Implemented reality at `34d1752` | Reviewed destination |
| --- | --- | --- |
| Admission | strict `/1` and accepted `/2` owner reader/handoff paths exist | versioned module APIs; frozen `/1` facade only |
| Linking | version-1 graph/interleaving kernels exist; v2 identities/content are incomplete | exact typed v1/v2 links, FR-287/288, FR-262, five channel premises |
| Compensation | registration/retry/commit/recovery kernels exist independently | Engine-owned graph-derived state, no public recovery callback |
| Conformance | batch/incremental kernels and closure-axis tests exist | one `ConformanceKernel` over linked/compensation/QObs/QSL values |
| Claims | dispositions and validation exist with a callback seam | graph-derived claims, no arbitrary backend fallback |
| Results | canonical producer/reader and complete borrowed result view exist | typed AssessmentBundle assembly with two-level outcomes |
| Bounds | inventories/accountants exist in two surfaces | one request accountant using owner accumulation rules |
| Resolution/evidence | PR #50 combines an unmerged resolver and evidence candidate | Task-013 resolver/capabilities; Task-018 evidence after prerequisites |
| E02/Quoin | no accepted complete handoff | remain explicit unresolved adapters, never substitutes |

## Architectural fitness

- Cohesion: stages group by resolution, versioned intake/model, assessment,
  output and qualification rather than by a flat list of ticket functions.
- Coupling: external owners enter only through exact versioned adapters and
  request-bound capabilities.
- Change isolation: a future owner version adds an adapter/version path, not a
  reinterpretation of an existing value.
- Testability: each boundary has a named TC; transactional and differential
  invariants have observable oracles.
- Resource safety: every public operation has one accountant owner and explicit
  resource-incomplete behavior.

No later major architectural decision is knowingly deferred within Tasks
002..013 and 017..018. A genuinely changed external contract reopens the gate;
ordinary implementation against this packet does not.
