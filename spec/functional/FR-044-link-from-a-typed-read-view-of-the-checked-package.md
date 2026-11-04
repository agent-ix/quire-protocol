---
id: FR-044
title: "Link from a typed read view of the checked package"
type: FR
relationships:
  - { target: ix://agent-ix/quire-protocol/US-001, type: implements }
  - { target: ix://agent-ix/quire-protocol/FR-002, type: references }
  - { target: ix://agent-ix/quire-protocol/FR-043, type: depends_on }
  - { target: ix://agent-ix/quire-spec-language/FR-519, type: depends_on }
  - { target: ix://agent-ix/quire-specification/FR-464, type: references }
---
# FR-044: Link from a typed read view of the checked package

## Description

When an admitted checked package is linked, the library SHALL build the finite
protocol representation from a typed read view over the nodes of the
`CheckedPackageV2`, and SHALL take the causal relation from QSL's
`qsl_package::protocol_view::causal_relation`, imported and never re-derived.

## Inputs

An admitted `CheckedPackageV2` (FR-043) and the `CheckedNodeId` of a
`protocol_clause` node.

## Outputs

The linked finite protocol representation, or a located typed refusal. The retired
`wire::Package` (`src/link/mod.rs`, `src/model/v2.rs`) is not an input or an
output.

## Behavior

- The read view shall list the clause, its roles, channels, queues, controls in
  layout order, compensations, required temporal clauses, `terminal`,
  `scheduling` and the activation rows as node ids, obtained through QSL's
  `protocol_view`.
- The read view shall own no table of its own for those nodes.
- The link layer shall take every causal edge from `causal_relation`.
- The link layer shall construct no causal edge.
- Each record the link layer read from `wire::Package` shall have the successor
  that QSL FR-519's record-to-node map names; a window binding is the one record
  with no successor, and it stays on its retained `wire.rs` spelling until its
  owner removes it.
- The link layer shall keep its own refusals: zero-progress iteration, duplicate
  role authorities, conflicting channel premises and the others its module
  documentation lists, because the checked package does not refuse them.
- The link layer shall refuse an absent node, a non-protocol node and a
  `protocol_reference` carrier with a typed refusal.
- The link layer shall return no partial representation with a refusal.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-044-AC-1 | Over **R**'s protocol, the linked representation lists the roles, channels, queues, controls, compensations, required temporal clause and activation rows equal to `protocol_view`'s node ids, and its edge set equals `causal_relation(package, protocol)`, back edges of every `repeat` included. | Test (TC-150) |
| FR-044-AC-2 | No function of the crate constructs an `EdgeKind` or `CausalEdge` value or derives an edge from a node: the edge set comes only from `qsl_package::protocol_view::causal_relation`. | Inspection (TC-150) |
| FR-044-AC-3 | Over views edited so that an iteration makes no progress, two roles claim one authority, and two channels hold conflicting premises, linking returns the consumer's own located refusal for each and no representation. | Test (TC-150) |
| FR-044-AC-4 | An absent node id, the id of a `protocol_reference` carrier and the id of a non-protocol node each return a typed refusal naming the cause and no partial representation. | Test (TC-150) |
| FR-044-AC-5 | The signature of the link entry names `CheckedPackageV2` and `CheckedNodeId` and names no `wire::Package`, `Handle` or `wire::Declaration`; a compile-fail use of each of those retired types through the crate's public surface fails to resolve, while `BindingKind::Window` (AC-6) still resolves. | Test (TC-150) |
| FR-044-AC-6 | A window binding is read through the retained `wire.rs` `BindingKind::Window` spelling, the one `wire.rs` item that stays public, and the crate imports no window view from QSL. | Test (TC-150) |
| FR-044-AC-7 | The criteria FR-002-AC-1 to FR-002-AC-5 hold over **R**'s package: concurrent instances stay distinct, every permitted interleaving keeps the causal edges and joins, each consumer-owned refusal refuses, reordering equal-time input adds no edge, and a changed temporal-profile or clock selection changes the result. | Test (TC-150) |

## Dependencies

- QSL FR-519 (`protocol_view`, `causal_relation`, the record-to-node map), QSpec
  FR-464 (the causal relation), FR-043.
- [FR-002](FR-002-link-finite-protocol-representation.md) states the linking
  contract for the retired wire; this requirement is its successor.
