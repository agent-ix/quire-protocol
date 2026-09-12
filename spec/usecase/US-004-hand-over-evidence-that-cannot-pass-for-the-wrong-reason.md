---
id: US-004
title: "Hand over evidence that cannot pass for the wrong reason"
type: US
relationships:
  - target: ix://agent-ix/quire-protocol/StR-002
    type: traces_to
---
# US-004: Hand over evidence that cannot pass for the wrong reason

## Story

**As a** qualification-evidence owner reviewing an integration scenario after it ran
**I want** each scenario that reached an outcome to hand over the resolution state
it ran against, the owner of each value it consumed, its own distinct outcome
identity, and the budget granted and consumed on each dimension
**So that** I can tell a scenario that passed for the stated reason from one that
passed for a different reason, and can tell a scenario that did not run from one
that did.

## Context

An integration composes several externally owned producers. Three ways for such a
scenario to look green while proving nothing recur: the right outcome obtained
from the wrong revision, two unrelated failure axes sharing one outcome code, and
a budget silently carried over from a previous attempt. None of the three is
visible in a pass or fail alone.

## Acceptance Examples (Illustrative)

### US-004-EX-1: Two failure axes stay apart

- **Given** two scenarios in one integration that fail for unrelated reasons
- **When** the owner compares their outcome identities
- **Then** the two identities differ, and neither had to be read out of display text or a rendered cause

### US-004-EX-2: An incomplete record is not a pass

- **Given** a scenario that reached an outcome but cannot report the budget consumed on one declared dimension
- **When** the scenario result is handed over
- **Then** it is reported as unexecuted rather than as passing

### US-004-EX-3: A refusal during resolution still carries what it reached

- **Given** a scenario refused part-way through resolving its owners
- **When** the record is composed
- **Then** it carries the partial resolution state reached and the entry that caused the refusal, and is not disqualified for lacking a full snapshot

## Options (Exploratory)

The per-scenario record could be composed by the harness or by the library
boundary under test. This option is exploratory and does not authorize keeping,
indexing, or querying the record after handoff.

## Constraints (Contextual)

Retention, storage, sufficiency assessment, and human disposition belong to Quoin
and Engineering Assurance. This story fixes what a scenario carries to its
handoff, never where it is kept afterwards or whether it suffices.

## Dependencies (Contextual)

The resolution state this record reports must exist before the record can report
it, so the attribution story is upstream of this one. A callable typed evidence
interface on the shared platform side is an external obligation and is not
published at the time of writing.

## Priority and Risk (Informative)

P0 because a scenario that passes for the wrong reason removes the only signal
the qualification owner has.

## Notes (Informative)

IT-001 is outside this story: it declares neither the manifest nor the
per-scenario record and carries no resolution precondition.

## Traceability (Informative)

FR-014, FR-017, and NFR-005 implement this story, which refines StR-002.
