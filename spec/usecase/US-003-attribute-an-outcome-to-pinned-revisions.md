---
id: US-003
title: "Attribute an outcome to pinned producer revisions"
type: US
relationships:
  - target: ix://agent-ix/quire-protocol/StR-002
    type: traces_to
---
# US-003: Attribute an outcome to pinned producer revisions

## Story

**As a** qualification-evidence owner defending a protocol result after the run
**I want** every externally produced value that result consumed to be resolved
through one place that pins its owner and its exact revision before the value is
read
**So that** I can say whose artifact, at which revision, produced the outcome,
and can re-derive the identical outcome from that revision set alone.

## Context

Values arrive from several owners with different release habits: some publish
tags, some publish only branches, some publish nothing yet. A delivery usually
arrives with files of its own describing what it contains. The owner asking the
attribution question is not present at the run and has only what the run
recorded.

## Acceptance Examples (Illustrative)

### US-003-EX-1: Unpinned owner stops the run before it reads

- **Given** a required external owner whose entry is present but carries no exact revision
- **When** an integration that consumes that owner's value begins
- **Then** the integration is refused and names that owner's entry, and no value from that owner has been read

### US-003-EX-2: A delivery cannot vouch for itself

- **Given** a delivery that contains a file stating which revision produced it
- **When** the entry for that owner is absent
- **Then** the file does not supply the missing entry, and the integration is refused

### US-003-EX-3: The same pinned set re-derives the same bytes

- **Given** one recorded revision set and one recorded input set
- **When** the assessment is run again on a different machine and in a different arrival order of causally independent observations
- **Then** the canonical result is byte-identical to the first

### US-003-EX-4: Byte-identical is not the same as expected

- **Given** an offered value whose bytes match a value this repository selected
- **When** the two are compared
- **Then** they compare unequal, because provenance differs

## Options (Exploratory)

Resolution could read each entry lazily at first use rather than in one pass
before any read. This option is exploratory and does not authorize reading a
producer value ahead of its entry.

## Constraints (Contextual)

Whether a revision is immutable and published is the owner's own assertion. This
repository indexes that assertion and does not check it; checking it would mean
reading version-control history or the network, which the boundary excludes.

## Dependencies (Contextual)

Each external owner must publish a revision it can assert before its entry can
move off an unresolved state. At the time of writing none of the enumerated
owners has done so for every consumed contract, so the end-to-end shape of this
story is unexecutable.

## Priority and Risk (Informative)

P0 because an outcome attributed to the wrong revision is indistinguishable from
a correct one, and the error surfaces only when the result is challenged.

## Notes (Informative)

Attribution is intake and indexing. This story never asks this repository to
produce, canonicalize, store, or assess an external artifact.

## Traceability (Informative)

FR-009, FR-012, FR-013, FR-018, and NFR-001 implement this story, which refines
StR-002.
