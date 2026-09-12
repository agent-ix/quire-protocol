---
id: StR-003
title: "Keep upstream owners the only authority for the nouns consumed here"
type: StR
relationships:
  - target: ix://agent-ix/quire-protocol/FR-010
    type: satisfied_by
  - target: ix://agent-ix/quire-protocol/FR-011
    type: satisfied_by
  - target: ix://agent-ix/quire-protocol/FR-015
    type: satisfied_by
---
# StR-003: Keep upstream owners the only authority for the nouns consumed here

## Stakeholder Need

The upstream formal-clause owner and each producer team SHALL remain the only authority for every noun, identity domain, and wire contract this repository consumes.

A reviewer of this repository SHALL receive, from every object it does define, an explicit declaration of whether that object's identity was authored upstream, derived here, or minted here.

## Rationale

A consumer that re-spells an upstream noun creates a second definition of it, and
a second definition is a fork whether or not anyone intended one. The upstream
owner then cannot change its own artifact without silently breaking a meaning it
did not know existed, and cannot rely on its acceptance of a clause meaning the
same thing on both sides. FR-001 already names this failure mode for the profile
itself — a second locally authored profile — but the same failure recurs one
level down, in every identity domain, digest label, and object name the consumer
touches.

The second half of the need is what makes the first auditable. A reader who
cannot tell which side authored an identity cannot tell whether this repository
adopted a noun or invented one, so the boundary is unenforceable by inspection
even when it is respected in fact.

## Validation Criteria

| ID | Criteria | Validation |
| --- | --- | --- |
| StR-003-VC-1 | Every noun this repository consumes from an upstream owner resolves to that owner's artifact by reference, and is not restated, re-enumerated, or re-spelled locally. | Inspection |
| StR-003-VC-2 | Every object this repository defines declares its identity kind and declares separately which side authored that identity, including objects whose identity is a value. | Inspection (TC-014) |
| StR-003-VC-3 | Every digest comparison this repository performs names a domain label selected from the upstream identity-domain vocabulary, and this repository mints no domain label of its own. | Test (TC-014) |
| StR-003-VC-4 | A value consumed from outside retains its own identity across the consumer boundary, and the boundary adds only objects this repository owns. | Analysis |

## Stakeholders

The primary stakeholders are the upstream formal-clause owner
(`agent-ix/quire-specification`) and the native compiler producer team
(`agent-ix/quire-spec-language`), whose accepted artifacts define what the nouns
mean and how they are encoded. The observation, temporal, diagnostic-catalog, and
E02 owners are affected parties on the same footing for their own vocabularies.
The protocol library owner is the party constrained by the need.

## Context and Assumptions

The upstream domain object layer already publishes the identity-kind and
identity-domain vocabularies this need depends on. It is assumed that where those
vocabularies are incomplete for a value this repository must consume, the gap is
raised with the owner rather than filled locally.

## Stakeholder Constraints (Contextual)

Adoption is by reference only. Where an upstream artifact does not establish an
identity, key, or order this repository needs, the missing element and its owner
are recorded; this repository supplies neither.

## Dependencies

The need depends on the upstream assessment-domain and canonical-identity-domain
artifacts. It drives the consumer boundary, the identity discipline every local
object satisfies, and digest domain selection.

## Priority and Risk (Informative)

Priority is P0. A forked noun is not detectable at run time and surfaces as a
disagreement between two accepted artifacts, so the impact is high and the
detection cost after the fact is severe.

## Notes (Informative)

This need constrains what this repository may define; it does not assert that the
upstream vocabularies are complete. Open questions this repository has raised
against the upstream object layer remain open and are not resolved here.

## Traceability

FR-010, FR-011, and FR-015 satisfy this stakeholder need. US-005 refines it;
TM-001 records planned evidence.
