---
id: StR-004
title: "Bound an evaluation the embedding caller is accountable for"
type: StR
relationships:
  - target: ix://agent-ix/quire-protocol/NFR-002
    type: satisfied_by
  - target: ix://agent-ix/quire-protocol/FR-016
    type: satisfied_by
---
# StR-004: Bound an evaluation the embedding caller is accountable for

## Stakeholder Need

The integrator embedding this library SHALL set, per evaluation request, a limit on every resource dimension its own service is accountable for.

The integrator SHALL receive, when one of those limits is reached, a partial answer naming the exhausted dimension rather than an unbounded run.

## Rationale

This library runs inside somebody else's process. The integrator — not this
repository — is the party paged when that process exceeds its memory limit,
misses its deadline, or is killed by its scheduler, and the integrator is the
party who knows what those limits are for the request in hand. A library that
enforces only the limits a protocol author wrote leaves the integrator with no
way to express its own, and a library that invents one universal threshold is
wrong for every caller.

The dimension matters as much as the bound. An integrator told only that an
evaluation ran out of resources cannot decide what to do next; an integrator told
which dimension was exhausted can raise that one budget, reject the input as too
large, or shed the request. A single scalar total collapses those choices into
one, and an answer of "too big" with nothing named is operationally the same as
no answer.

## Validation Criteria

| ID | Criteria | Validation |
| --- | --- | --- |
| StR-004-VC-1 | An integrator can supply, per request, a positive limit on each resource dimension the library accounts, and the library enforces the integrator's limit alongside every limit the protocol author authored. | Demonstration (TC-009) |
| StR-004-VC-2 | Exhausting one dimension yields a resource-incomplete answer naming that dimension, and consumption on one dimension does not consume another dimension's budget. | Test (TC-009) |
| StR-004-VC-3 | A request whose consumption sits exactly at an inclusive maximum is answered rather than refused, and only an excess beyond that maximum is refused. | Test (TC-009) |
| StR-004-VC-4 | No evaluation continues past the request deadline, and the answer returned reports the elapsed time observed. | Demonstration (TC-009) |

## Stakeholders

The primary stakeholder is the integrator that embeds this library: the native
Quire compiler and runtime integrators named as intended users of this
specification, who host the evaluation inside their own resource envelope.
Engineering Assurance is an affected party because it owns whole-process
containment and external resource enforcement around this library, not inside it.
The protocol verification owner is an affected party because a resource-exhausted
answer must stay distinguishable from a violation.

## Context and Assumptions

Budget figures are owner inputs. It is assumed the integrator supplies concrete
per-request limits and that a production corpus establishing typical costs is
provided by the owner rather than invented here.

## Stakeholder Constraints (Contextual)

Containment of the whole process, and enforcement of resources this library does
not itself allocate, remain with Engineering Assurance. This need covers only
what the library accounts for and refuses on its own.

## Dependencies

The need depends on the upstream resource-limit clause and on the integrator
supplying per-request budgets. It drives bound enforcement across every
evaluation stage and the dimension vocabulary those budgets are accounted on.

## Priority and Risk (Informative)

Priority is P1 for the first slice, rising to P0 at the point a caller runs this
in a latency- or memory-constrained service. An unbounded evaluation is a
liveness failure in the caller's process, not in this repository's.

## Notes (Informative)

A resource-incomplete answer is an honest partial result, not a failure to
comply. Nothing here permits promoting such an answer to conformance, to
violation, or to a completed assessment.

## Traceability

NFR-002 and FR-016 satisfy this stakeholder need. US-006 refines it; TM-001
records planned evidence.
