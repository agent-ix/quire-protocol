---
id: US-002
title: "Consume source-bound protocol results"
type: US
relationships:
  - target: ix://agent-ix/quire-protocol/StR-001
    type: traces_to
---
# US-002: Consume source-bound protocol results

## Story

**As a** verification evidence consumer
**I want** canonical protocol results and their E02 projections to retain exact
source identities and independent findings
**So that** I can assess evidence without confusing protocol adequacy,
observation adequacy, execution success, truth, or sufficiency.

## Context

The canonical result is richer than E02. Consumers that need closure, settlement,
decision support, protocol adequacy, or claim meaning read the canonical result;
E02 carries a deliberately narrower portable assessment.

## Acceptance Examples (Illustrative)

### US-002-EX-1: Canonical result remains authoritative

- **Given** a strict-reader-accepted canonical protocol result
- **When** an E02 projection is requested
- **Then** the projection retains the exact canonical source reference and its own distinct identity

### US-002-EX-2: Finding loss refuses projection

- **Given** a canonical result containing two simultaneous independent findings
- **When** a target mapping would erase either finding
- **Then** the adapter returns a typed refusal and emits no E02 result

## Options (Exploratory)

Consumers may store artifact references instead of bytes when retention rights
require it. Such a reference does not become proof that the bytes were retained.

## Constraints (Contextual)

Quoin retains evidence authority and quire-verification retains E02 ownership.

## Dependencies (Contextual)

The canonical result depends on an accepted protocol profile. E02 projection
depends on the exact accepted E02 revision and Engineering Assurance context.

## Priority and Risk (Informative)

P0 because semantic loss at this seam can turn incomplete evidence into a false
qualification result.

## Notes (Informative)

Projection never replaces or mutates the canonical result.

## Traceability (Informative)

FR-006 through FR-008 implement this story.
