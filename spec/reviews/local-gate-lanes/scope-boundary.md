---
id: SR-029
title: "Scope review of the local verification gate definition"
type: SpecReview
analysis: scope-boundary
scope: "NFR-004 against NFR-003 and quire-spec-language at 681e25a"
review_set: subset
relationships:
  - { target: ix://agent-ix/quire-protocol/NFR-004, type: reviews }
---
## Summary

Reviewed whether NFR-004 claims anything already owned elsewhere. The boundary
that mattered most is the inherited two-lane directive. That directive is scoped
to `agent-ix/quire-spec-language`, where the `quire-extraction` feature gates a
module, a `deny_unknown_fields` wire field and a whole test file. It describes a
property of that crate's optional Quire consumer, not a general repository
convention, so importing it here produced an unexecutable rule rather than a
shared standard. NFR-004 restates the gate in terms of what it protects, which
resolves the boundary without either repository weakening its own coverage.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The two-lane rule was being applied as a cross-repository convention while its justification was crate-specific. NFR-004 reallocates it: the lane count is a consequence of a crate's declared features and conditional compilation, so A remains two-lane and B is one-lane on the same underlying rule. | NFR-004 Rationale; NFR-004-AC-5 |
| FND-002 | low | No leak into NFR-003. NFR-004 owns which commands run and in what configuration; NFR-003 continues to own language containment, the shared boundary checker, trace bindings and workflow triggers. The Verification section forbids a local lint runner or verdict parser so the gate cannot grow into a second assurance surface. | NFR-004 Verification; NFR-003 |
| FND-003 | low | No leak into TM-001 or the plan bundle. NFR-004 defines the gate; it does not schedule the work that satisfies it, which remains with Plan-001 and the owning issue. | NFR-004 Dependencies |

## Responsibility Allocation

| Boundary | Owner | Contract status |
| --- | --- | --- |
| The `quire-extraction` feature and the lanes it justifies | quire-spec-language | External; consumed as a pinned dependency revision, not as a gate convention |
| Rust-only containment, shared checker, trace bindings, workflow triggers | quire-protocol NFR-003 | Existing B-owned boundary, unchanged |
| Which commands constitute the local gate, in which configurations, with what risk | quire-protocol NFR-004 | B-owned, newly allocated |
| Whether a hosted run is local evidence | quire-protocol NFR-003-AC-1 | Existing; referenced rather than restated |
| Scheduling the implementation of the gate | Plan-001 and repository issue 31 | Out of requirement scope |
