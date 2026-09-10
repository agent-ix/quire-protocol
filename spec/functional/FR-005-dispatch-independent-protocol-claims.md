---
id: FR-005
title: "Dispatch independent protocol claims"
type: FR
relationships:
  - target: ix://agent-ix/quire-protocol/US-001
    type: implements
  - target: ix://agent-ix/quire-protocol/FR-004
    type: depends_on
  - target: ix://agent-ix/quire-specification/FR-060
    type: depends_on
---
# FR-005: Dispatch independent protocol claims

## Description

When one or more protocol claims are requested, the dispatcher SHALL return one
independent execution disposition per exact claim without inferring a stronger
claim or invoking an unreviewed fallback.

## Inputs

Exact conformance, monitorability, projection, realizability, refinement, or
composition requests with admitted fragments, backends, assumptions, budgets,
and source/derived-artifact identities.

## Outputs

One source-bound disposition per requested claim, including explicit unsupported
or failed results and preserved independent supported results. Every disposition
retains its exact admitted fragment, backend, assumptions, communication and
visibility premises, source identities, and derived-artifact correspondence.

## Behavior

The dispatcher SHALL keep finite conformance available when projection is
unsupported. It SHALL NOT infer projectability or realizability from a
satisfiable trace, infer composition from circular local assumptions, or replace
a failed selected backend with an unreviewed fallback. Unimplemented advanced
claims SHALL return unsupported without weakening the finite replay result. The
dispatcher SHALL refuse a disposition whose fragment, backend, assumption,
premise, source or derived-artifact identity does not match its exact request.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-005-AC-1 | A monitorable but unprojectable protocol retains its finite conformance result and an independent projection refusal. | Test (TC-005) |
| FR-005-AC-2 | Satisfiable-but-unrealizable and circular-assumption cases cannot produce realizability or composition success. | Test (TC-005) |
| FR-005-AC-3 | Unknown or unimplemented advanced claims return unsupported with exact premises and cannot erase a supported result. | Test (TC-005) |
| FR-005-AC-4 | A selected backend failure returns failed for only that claim and invokes no fallback. | Test (TC-005) |
| FR-005-AC-5 | Every claim disposition retains its exact admitted fragment, backend, assumptions, communication/visibility/environment premises and source/derived-artifact correspondence; removing or substituting any one refuses that disposition without changing independent results. | Test (TC-005) |

## Dependencies

- [FR-004](./FR-004-assess-finite-global-conformance.md).
- `ix://agent-ix/quire-specification/FR-060`.
