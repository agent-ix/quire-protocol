---
id: US-001
title: "Assess one finite native protocol"
type: US
relationships:
  - target: ix://agent-ix/quire-protocol/StR-001
    type: traces_to
---
# US-001: Assess one finite native protocol

## Story

**As a** protocol verification owner
**I want** one exact finite native protocol and observation history assessed
**So that** I can distinguish conformance, violation, pending work, missing facts,
and compensation outcomes without executing business actions.

## Context

The selected profile contains concurrent roles, causal and channel premises,
bounded iteration, retries, commits, and authored recovery relations. Inputs may
be open, incomplete, duplicated, or inconsistent.

## Acceptance Examples (Illustrative)

### US-001-EX-1: Split shipment conforms

- **Given** an accepted protocol and complete observations for two permitted shipment branches
- **When** the owner requests finite global replay
- **Then** the result identifies every satisfied obligation and the complete conformance basis

### US-001-EX-2: Missing provider fact stays incomplete

- **Given** an open history missing one required external provider observation
- **When** the owner requests finite global replay
- **Then** the result identifies the missing fact and does not report complete conformance

## Options (Exploratory)

Batch and incremental evaluators may share a pure transition core. This option
does not authorize separate semantic engines or a streaming deployment surface.

## Constraints (Contextual)

The first implementation uses Rust and the reviewed finite profile only.

## Dependencies (Contextual)

Native compiler, observation, temporal, and profile owners must publish exact
accepted interfaces before end-to-end execution.

## Priority and Risk (Informative)

P0 because every downstream result and qualification claim depends on correct
finite assessment.

## Notes (Informative)

The evaluator observes compensation effects; it never invokes compensation.

## Traceability (Informative)

FR-001 through FR-005 and FR-008 implement this story.
