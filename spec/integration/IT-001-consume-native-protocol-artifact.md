---
id: IT-001
title: "Consume the checked protocol package"
type: IT
relationships:
  - { target: ix://agent-ix/quire-protocol/ADR-002, type: references }
  - { target: ix://agent-ix/quire-protocol/ADR-003, type: references }
  - { target: ix://agent-ix/quire-protocol/FR-001, type: verifies }
  - { target: ix://agent-ix/quire-protocol/FR-002, type: verifies }
  - { target: ix://agent-ix/quire-protocol/FR-043, type: verifies }
  - { target: ix://agent-ix/quire-protocol/FR-044, type: verifies }
  - { target: ix://agent-ix/quire-protocol/NFR-002, type: verifies }
---
# IT-001: Consume the checked protocol package

## Objective

Verify that the protocol library admits and links the selected checked package
from recipe **R** through QSL's published reader and typed read view. The
consumer does not parse source, reconstruct checked nodes, or derive causal
edges that QSL owns.

## Target Integration

The producer is `quire_spec_language::checked_handoff`, which emits the three
packages and selection used by recipe **R**. The consumer is this library's
checked-package admission and linking path. The integration uses the producer's
current public interface and the standard reader and selection mutation cases;
it does not maintain a second fixture format or a local reader.

## Owner Boundaries

| Owner | Supplies | This repository must not |
| --- | --- | --- |
| QSL checked-package producer and reader | Recipe R, its package bytes, selection, typed identities, reader outcomes and protocol view | Parse source, reconstruct a package, or reinterpret an identity or refusal |
| This repository | Admission handoff, protocol-node selection, link result and consumer-owned link refusal | Derive source-level semantics or causal edges owned by QSL |

## Preconditions

The integration can call the producer's `checked_handoff` API from the QSL
dependency selected by this repository. Recipe **R** contains the protocol,
model and temporal inputs needed by the scenarios below. The handoff's standard
mutation cases identify the reader and selection defects and their owner-defined
outcomes. No separate version-1 or version-2 compiled-protocol lane is part of
this integration.

## Inputs

The unmodified handoff for **R**; the producer's selected package identities;
the standard reader and selection mutation cases; and bounded admission limits
set to exact usage and one unit below measured usage where a nonzero usage makes
that value representable.

## Test Procedure

1. Generate the handoff for **R** through `quire_spec_language::checked_handoff`
   and admit its three packages through the public protocol boundary.
   - IT-001-SC-01: each admitted package retains the producer-selected identity
     and bytes; no package or selection is read from a consumer-authored copy.
2. Submit every standard reader and selection mutation case with the offer and
   the independent selection held constant except for the mutated axis.
   - IT-001-SC-02: each case returns its producer-defined typed refusal or
     incomplete result before an admitted package or view is exposed; no
     consumer-selected mutation silently admits.
3. Select the protocol nodes from the admitted package, obtain their
   `protocol_view` and `causal_relation` from QSL, and link the selected graph.
   - IT-001-SC-03: the linked node lists and edge set equal the owner's view and
     relation; every linked causal edge is supplied by QSL.
4. Submit the admitted package at exact reader usage and at one unit below each
   nonzero measured limit in turn.
   - IT-001-SC-04: exact usage admits, a one-short limit returns the owner's
     typed incomplete outcome, and each fresh exact-limit retry admits.
5. Offer the retired compiled-protocol formats to the checked-package entry.
   - IT-001-SC-05: each retired format is refused by the owner; no version
     detection, upgrade or downgrade path is exposed by this repository.

## Expected Results

Only the exact checked handoff selected by the caller is admitted. Every reader
or selection defect retains QSL's typed outcome, and every linked node and edge
comes from the owner's view. The library does not claim temporal evaluation,
compensation assessment or complete conformance from admission and linking.

## Metadata

- Priority: P0
- Target Integration: checked native protocol package to protocol library
- Automation: Automated local fixture lane through the QSL dependency selected by Cargo

## Dependencies

QSL publishes `checked_handoff`, the checked-package reader, the independent
selection and the typed protocol view and causal relation. QSL's accepted
recipe **R** is the input contract. This integration does not select or retain
the retired compiled-protocol format lanes.

## Retired Scenario Counterparts

The earlier IT-001 procedure covered two compiled-protocol versions and
version-specific selection, identity, numeric and resource lanes. Recipe **R**
replaces those procedures with one checked-package handoff and the owner's
reader/selection mutation cases. Each former scenario is accounted for here:

| Retired scenario | Recipe R counterpart or reason it ends |
| --- | --- |
| IT-001-SC-01 | IT-001-SC-01 checks the three R packages and selected identities. |
| IT-001-SC-02 | IT-001-SC-01 and IT-001-SC-03 check package identities, protocol nodes and causal edges. |
| IT-001-SC-03 | IT-001-SC-02 checks owner-defined typed refusals for reader and selection mutations. |
| IT-001-SC-04 | IT-001-SC-02 exercises the accepted reader's numeric mutation cases; distinct v1/v2 numeric readers no longer exist. |
| IT-001-SC-05 | IT-001-SC-03 inspects the owner reader and view used on the admitted path. |
| IT-001-SC-06 | Ends: recipe R has one checked-package contract; there is no second versioned reader or cross-version submission. Admission is covered by IT-001-SC-01/02. |
| IT-001-SC-06a | Ends: no legacy `/1` facade remains whose behavior can be compared across a `/2` introduction. |
| IT-001-SC-07 | IT-001-SC-01 compares the admitted handoff with the producer-selected bytes; consumer recanonicalization is not an independent input axis. |
| IT-001-SC-08 | IT-001-SC-02 uses the checked-package owner's selection mutation cases; compiled-protocol temporal-binding wire rules are retired. |
| IT-001-SC-09 | IT-001-SC-04 covers the current checked-package reader limits; the retired version-2 dimension inventory is not used. |

## Traceability

Verifies FR-001, FR-002, FR-043 and FR-044. IT-001-SC-04 also verifies
NFR-002's reader resource boundary.
