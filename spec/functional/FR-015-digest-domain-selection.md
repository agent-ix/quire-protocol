---
id: FR-015
title: "Digest domain selection"
type: FR
object: value_object
relationships:
  - target: ix://agent-ix/quire-protocol/ADR-003
    type: references
  - target: ix://agent-ix/quire-protocol/ADR-006
    type: references
  - target: ix://agent-ix/quire-protocol/US-005
    type: implements
  - target: ix://agent-ix/quire-protocol/FR-010
    type: part_of
  - target: ix://agent-ix/quire-protocol/FR-011
    type: depends_on
  - target: ix://agent-ix/quire-protocol/FR-001
    type: specifies
  - target: ix://agent-ix/quire-specification/FR-201
    type: depends_on
---
# FR-015: Digest domain selection

## Description

This repository SHALL mint no digest domain label of its own.

## Properties

- **label** (identity component) — one member of `FR-201`'s published
  vocabulary, spelled exactly as published. Spelling is load-bearing: a
  different label compares unequal over identical bytes.
Where a producer reference arrives already carrying an upstream domain label,
that label is retained exactly and is never re-labelled into another member.

Canonical protocol results and their E02 projections carry the structural
identities defined by QSpec FR-286 and FR-062. This repository computes no
result or projection digest.

## Identity and comparison

- **Identity kind** — *value*, per [FR-011](./FR-011-consumer-identity-discipline.md).
  Authority *producer*: the vocabulary is upstream's and this repository selects
  from it.
- **Key** — the upstream member label itself, matched exactly as `FR-201`
  publishes it. Spelling is load-bearing.
- **Key order** — lexicographic over the label, for collection comparison only.
  It carries no causal meaning and expresses no precedence between members.
- **Causal order** — none.
- **Absence** — an unselected domain compares only to absence. It is never
  defaulted to a member, and a comparison requiring a domain refuses.
- **Digest domain** — the label is the domain; it is not itself digested.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-015-AC-1 | Every digest domain label this repository uses is a member `FR-201` publishes, spelled exactly as published, and no label originates here. | Inspection |
| FR-015-AC-2 | A producer reference arriving with an upstream domain label retains that label exactly through admission, linking and result production, and is never re-labelled. | Test (TC-001, TC-014) |
| FR-015-AC-5 | No requirement, test or implementation in this repository declares a digest domain vocabulary of its own. | Inspection |

## Dependencies

- `ix://agent-ix/quire-specification/FR-201` publishes the vocabulary and the
  cross-domain inequality rule.
- [FR-011](./FR-011-consumer-identity-discipline.md) supplies the identity
  discipline.
- [FR-001](./FR-001-admit-selected-protocol-profile.md) forbids reinterpreting a
  producer digest through another domain.
