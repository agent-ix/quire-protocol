---
id: FR-010
title: "Compiled-artifact consumer boundary"
type: FR
object: domain
relationships:
  - target: ix://agent-ix/quire-protocol/ADR-007
    type: references
  - target: ix://agent-ix/quire-protocol/US-005
    type: implements
  - target: ix://agent-ix/quire-specification/FR-200
    type: depends_on
  - target: ix://agent-ix/quire-protocol/FR-011
    type: contains
  - target: ix://agent-ix/quire-protocol/FR-012
    type: contains
  - target: ix://agent-ix/quire-protocol/FR-013
    type: contains
  - target: ix://agent-ix/quire-protocol/FR-014
    type: contains
  - target: ix://agent-ix/quire-protocol/FR-015
    type: contains
  - target: ix://agent-ix/quire-protocol/FR-016
    type: contains
  - target: ix://agent-ix/quire-protocol/FR-017
    type: contains
  - target: ix://agent-ix/quire-protocol/FR-018
    type: contains
---
# FR-010: Compiled-artifact consumer boundary

## Description

When a value produced outside this repository is consumed, the consumer boundary SHALL retain that value's own identity and add only the objects this context owns.

## Bounded Context

This context begins where an externally produced artifact is offered and ends
where a canonical result or a typed refusal is returned. It owns the act of
*consuming*: selecting independently, resolving a producer to a pinned revision,
discriminating what was offered from what was selected, refusing with a distinct
typed identity, and composing the evidence a scenario carries to its handoff.

It owns no protocol, temporal or observation semantics. Those are
`ix://agent-ix/quire-specification`'s, which publishes them as the domain object
layer under `FR-200-quire-assessment-domain`. This context references that layer
and never restates it. A noun this context redefines locally would be the "second
locally authored profile" FR-001 exists to refuse.

### What distinguishes an object in this context

Every object here exists because *consumption* needs it and the producer has no
reason to have one. A producer has no concept of "the selection I retained
before I looked at your bytes", of "the revision I pinned you at", or of "the
axis along which your offer differed from my selection". Those are relationships
between a consumer and a producer, and they belong to neither alone.

The test for whether an object belongs here: does it describe a producer's
subject, or the consumer's relationship to that subject? The first is upstream's.
The second is this context's.

## Entities

The objects this context contains are listed in the `contains` edges above and
specified individually. Summarised:

| Object | What it is |
| --- | --- |
| `FR-011` | The identity discipline this context applies, reconciled to upstream's |
| `FR-012` | A version-lock entry pinning one producer at one contract version |
| `FR-013` | The discrimination between what was offered and what was selected |
| `FR-014` | A typed outcome identity — the refusal envelope's key |
| `FR-016` | A resource budget dimension |
| `FR-017` | A scenario evidence record |
| `FR-018` | A delivery index — evidence about what a producer delivered |

## Adoption status

An adoption pass replaces a requirement's local definitions with upstream
references and adds a section recording where the upstream vocabulary does not
reach. Three artifacts have not had one, and claiming otherwise would be the
same over-claim this context exists to prevent.

| Requirement | Adoption pass |
| --- | --- |
| FR-002 through FR-008 | Done |
| NFR-001, IT-002, IT-003 | Not started |
| FR-001, FR-009 | Not required — neither defines a protocol noun |

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-010-AC-1 | Every object specified in this context describes a consumer-to-producer relationship, and none restates a protocol, temporal or observation subject that `quire-specification` types. | Inspection |
| FR-010-AC-2 | In each requirement that has had an adoption pass, every protocol, temporal or observation noun resolves to an `ix://agent-ix/quire-specification` reference rather than a local definition. FR-002 through FR-008 have each had one, together with every atomic part they contain. NFR-001, IT-002 and IT-003 have not, and carry no such reference; this criterion does not yet assert anything about them. | Inspection |
| FR-010-AC-3 | Where an upstream artifact records an open gap, a requirement depending on it cites the gap rather than supplying the missing identity, key or order locally. | Inspection |

## Dependencies

- `ix://agent-ix/quire-specification/FR-200` supplies the identity discipline
  this context conforms to and the object layer it references.
