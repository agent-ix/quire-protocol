---
id: US-006
title: "Evaluate within a caller's budget"
type: US
relationships:
  - target: ix://agent-ix/quire-protocol/StR-004
    type: traces_to
---
# US-006: Evaluate within a caller's budget

## Story

**As an** integrator embedding this library in my own service
**I want** to state, per request, how much of each resource the evaluation may use
and to be told which one ran out
**So that** my process stays inside the envelope I am accountable for and I can
decide what to do next instead of only learning that something was too big.

## Context

The integrator's limits are not the protocol author's limits. A protocol carries
semantic maxima its author wrote; the hosting service carries a memory ceiling, a
request deadline, and an input-size policy that vary per deployment and per
request. Both sets have to hold at once, and neither side can supply the other's
figures.

## Acceptance Examples (Illustrative)

### US-006-EX-1: One dimension runs out and says so

- **Given** a request granted a generous byte budget and a small node budget
- **When** the evaluation would exceed the node budget
- **Then** the answer is resource-incomplete and names the node dimension, and the byte budget is untouched

### US-006-EX-2: Exactly at the maximum still answers

- **Given** a request whose consumption lands exactly on an inclusive maximum
- **When** the evaluation completes
- **Then** it returns an answer rather than a refusal

### US-006-EX-3: The deadline stops the next unit, not the current one

- **Given** a request with a short deadline
- **When** the deadline passes during a declared indivisible unit of work
- **Then** no later unit is started and the answer reports the elapsed time observed

## Options (Exploratory)

A caller could be offered a preset budget profile rather than supplying every
dimension. This option is exploratory and does not authorize a default threshold
invented by this repository.

## Constraints (Contextual)

Whole-process containment and enforcement of resources this library does not
itself allocate remain with Engineering Assurance. Concrete budget figures and a
production corpus are owner inputs.

## Dependencies (Contextual)

The upstream resource-limit clause governs the semantic maxima side. The
integrator must supply per-request budgets before this story has anything to
enforce.

## Priority and Risk (Informative)

P1 for the first slice and P0 once a caller runs this in a latency- or
memory-constrained service, because an unbounded evaluation is a liveness failure
in the caller's process.

## Notes (Informative)

A resource-incomplete answer is an honest partial result. Nothing in this story
lets such an answer become conformance, violation, or a completed assessment, and
the distinct outcome identity that keeps it separable is owned elsewhere.

## Traceability (Informative)

NFR-002 and FR-016 implement this story, which refines StR-004.
