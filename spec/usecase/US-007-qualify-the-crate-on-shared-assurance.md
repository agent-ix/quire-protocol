---
id: US-007
title: "Qualify the crate on shared assurance"
type: US
relationships:
  - target: ix://agent-ix/quire-protocol/StR-005
    type: traces_to
---
# US-007: Qualify the crate on shared assurance

## Story

**As a** promotion owner for this library
**I want** the gate I am asked to trust to be an enumerated list of commands that
each run against the crate as it stands and each name the risk it detects, with
producer execution and evidence left to the shared platform
**So that** a green gate means something specific was protected, and this
repository's evidence is legible next to every other repository's.

## Context

Gate instructions outlive the crate they were written for. A directive inherited
from a sibling repository can name a configuration lane this crate cannot
express, in which case running it reports success while covering nothing extra.
The same pull exists at the platform edge: producer execution and evidence
retention are easy to reimplement locally and hard to compare afterwards.

## Acceptance Examples (Illustrative)

### US-007-EX-1: The written gate and the run gate agree

- **Given** the gate steps named in the repository's own instructions
- **When** the promotion owner runs them
- **Then** every named step executes, and no step in the run is absent from the instructions

### US-007-EX-2: A lane that covers nothing is not counted

- **Given** a crate with a single unit graph
- **When** two configuration lanes are proposed for the gate
- **Then** only one lane is part of the gate, and the trigger that would oblige a second is stated instead

### US-007-EX-3: Evidence is composed and passed, not kept

- **Given** a scenario record composed for handoff
- **When** the boundary finishes with it
- **Then** it has been passed to the platform owner and not stored, indexed, or queried locally

## Options (Exploratory)

The obligation to add a configuration lane could be written as a trigger in the
gate definition or carried in the crate's own review checklist. This option is
exploratory and does not authorize declaring a feature with no gated code behind
it.

## Constraints (Contextual)

Build and qualification workflows remain manual dispatch. No hosted status
stands as evidence for the local gate. The existing CLA administration workflow
retains its separate triggers, and heavy local activity shares one workstation
lock.

## Dependencies (Contextual)

The shared platform must publish a typed producer boundary and a typed evidence
interface callable from this crate. Neither is published at the time of writing,
so the delegation half of this story cannot be exercised end to end.

## Priority and Risk (Informative)

P0 at promotion. A promotion authorized by a green step that covered nothing is
unrecoverable once downstream claims rest on it.

## Notes (Informative)

While the repository remains specification-only, several gate steps detect
little. That is stated rather than hidden, and a step's coverage is expected to
grow with the suite rather than be introduced later as a new gate.

## Traceability (Informative)

NFR-003 and NFR-004 implement this story, which refines StR-005.
