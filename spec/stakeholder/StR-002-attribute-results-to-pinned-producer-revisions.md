---
id: StR-002
title: "Attribute every result to the exact producer revisions behind it"
type: StR
relationships:
  - target: ix://agent-ix/quire-protocol/FR-009
    type: satisfied_by
  - target: ix://agent-ix/quire-protocol/FR-012
    type: satisfied_by
  - target: ix://agent-ix/quire-protocol/FR-013
    type: satisfied_by
  - target: ix://agent-ix/quire-protocol/FR-014
    type: satisfied_by
  - target: ix://agent-ix/quire-protocol/FR-017
    type: satisfied_by
  - target: ix://agent-ix/quire-protocol/FR-018
    type: satisfied_by
  - target: ix://agent-ix/quire-protocol/NFR-001
    type: satisfied_by
  - target: ix://agent-ix/quire-protocol/NFR-005
    type: satisfied_by
---
# StR-002: Attribute every result to the exact producer revisions behind it

## Stakeholder Need

The qualification-evidence owner SHALL receive, with every protocol result, the identity of each external owner whose value that result consumed and the exact immutable revision of each.

The qualification-evidence owner SHALL re-derive the identical canonical result from that recorded revision set and its recorded inputs alone, on another machine and in another arrival order of causally independent observations.

## Rationale

This repository consumes values from seven external owners and holds authority
over none of them. When a result is later defended, the question asked of its
owner is not whether the library computed correctly but whose artifact, at which
revision, it computed over. A result that cannot answer that question is
indistinguishable from a correct outcome obtained from the wrong revision, from a
refusal that shares one code with an unrelated failure axis, or from a budget
carried over from an earlier attempt. The owner who must stand behind the result
then has no way to separate those cases after the fact, and the outcome — green
or red — proves less than it appears to.

The second half of the need is what makes the first half worth anything. An
attribution to a revision set is only meaningful if that revision set determines
the result. If the same pinned inputs can yield two different byte sequences,
then the revision set names nothing and the attribution is decoration.

## Validation Criteria

| ID | Criteria | Validation |
| --- | --- | --- |
| StR-002-VC-1 | For every externally produced value a run consumed, the owner can read one external owner identity and one exact pinned revision, and can see that no required entry was absent, unpinned, or declared unresolved when the value was read. | Demonstration (TC-013) |
| StR-002-VC-2 | No file supplied by a producer alongside its own delivery can supply, select, or repair the revision a value is attributed to. | Test (TC-013) |
| StR-002-VC-3 | Two runs over the identical pinned revision set and identical inputs yield byte-identical canonical results, and a change to any pinned identity changes the result identity or returns a typed refusal. | Test (TC-008) |
| StR-002-VC-4 | A scenario that reached a typed outcome carries that outcome, the resolution state it ran against, the owner of each consumed value, and the budget granted and consumed on each declared dimension, or is reported as unexecuted rather than as passing. | Demonstration (IT-003) |
| StR-002-VC-5 | Two distinct failure axes reached in one integration are distinguishable from each other without reading display text, locale, or a cause rendering. | Test (TC-011) |

## Stakeholders

The primary stakeholder is the qualification-evidence owner: the person
accountable for a retained protocol result once it leaves this repository
through Engineering Assurance and Quoin. Affected parties are the external
producer owners this repository consumes — the native compiler, observation,
temporal, diagnostic-catalog, Engineering Assurance, Quoin, and E02 owners — each
of whom is exposed when a result attributes an outcome to the wrong authority.
quire-verification is an affected consumer, because it strictly reads the E02
projection derived from the attributed result.

## Context and Assumptions

This repository is a consumer. Every semantic input it reads is authored
elsewhere, and its own correctness is not sufficient for a defensible result.
It is assumed that an external owner can state which revision of its artifact is
immutable and published; this need does not assume that this repository can check
that statement, and AD-001 places that check outside the boundary.

## Stakeholder Constraints (Contextual)

Attribution is an intake and indexing concern only. Producing, canonicalizing,
storing, or assessing an external artifact remains outside this repository, as
does deciding whether the attributed evidence suffices.

## Dependencies

The need depends on each external owner publishing an exact revision it can
assert. It drives manifest resolution, entry comparison, provenance-bearing
comparison, delivery indexing, deterministic result identity, and the
per-scenario evidence handoff.

## Priority and Risk (Informative)

Priority is P0. An unattributable or non-reproducible result can authorize a
production claim that no owner is able to defend, so the impact is critical.

## Notes (Informative)

SR-049 FND-001 recorded that FR-009 descended from no stakeholder statement in
this repository; this need is that statement. Nothing here asserts that any
external owner has yet published a revision that can be pinned: SR-047 FND-002
records the opposite for the live producer set, so VC-1, VC-2, and VC-4 are
currently unexecutable end to end and their integration-level validation is
planned, not performed.

## Traceability

FR-009, FR-012, FR-013, FR-014, FR-017, FR-018, NFR-001, and NFR-005 satisfy this
stakeholder need. US-003 and US-004 refine it; TM-001 records planned evidence.
