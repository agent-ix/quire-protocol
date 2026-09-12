---
id: US-005
title: "Adopt upstream nouns by reference"
type: US
relationships:
  - target: ix://agent-ix/quire-protocol/StR-003
    type: traces_to
---
# US-005: Adopt upstream nouns by reference

## Story

**As an** upstream formal-clause owner whose vocabulary this consumer reads
**I want** the consumer to reach my nouns, identity kinds, and digest domains by
reference and to label plainly which side authored every object it adds
**So that** I can change my own artifact without breaking a second definition of
it that I did not know existed.

## Context

The consumer needs objects of its own — entries, records, indexes, vocabularies
for things only it does. It also handles values whose meaning is entirely
upstream. Nothing in a file's appearance distinguishes an adopted noun from a
locally invented one, so a reader cannot tell them apart unless the file says
which it is.

## Acceptance Examples (Illustrative)

### US-005-EX-1: A consumed value keeps its own identity

- **Given** a value produced by an external owner and crossing into the consumer
- **When** it is retained locally
- **Then** its own identity is preserved and only objects this repository owns are added around it

### US-005-EX-2: Every local object says who authored its identity

- **Given** an object this repository defines whose identity is a plain value
- **When** a reader inspects it
- **Then** the object declares its identity kind and, separately, which side authored that identity

### US-005-EX-3: No new digest domain appears locally

- **Given** two digests to compare
- **When** the comparison names its domain
- **Then** the domain label comes from the upstream identity-domain vocabulary and no local label was minted

## Options (Exploratory)

Where an upstream vocabulary lacks a label a consumed value needs, the gap could
be raised as an upstream question or the value could be left uncomparable until
it is filled. This option is exploratory and does not authorize a local label.

## Constraints (Contextual)

Where an upstream artifact does not establish an identity, key, or order this
repository needs, the missing element and its owner are recorded and the owner is
asked. This repository supplies neither.

## Dependencies (Contextual)

The upstream assessment-domain and canonical-identity-domain artifacts must
publish the identity kinds and domain labels this story reads. Questions this
repository has already raised against that layer remain open.

## Priority and Risk (Informative)

P0 because a forked noun is invisible at run time and surfaces as two accepted
artifacts disagreeing, which is the most expensive class of defect to unwind.

## Notes (Informative)

The same failure mode is already named one level up for the profile itself: the
consumer must not stand up a second locally authored profile. This story extends
that rule to every noun below it.

## Traceability (Informative)

FR-010, FR-011, and FR-015 implement this story, which refines StR-003.
