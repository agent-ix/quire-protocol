---
id: FR-044
title: "Link from a typed read view of the checked package"
type: FR
relationships:
  - { target: ix://agent-ix/quire-protocol/US-001, type: implements }
  - { target: ix://agent-ix/quire-protocol/FR-002, type: references }
  - { target: ix://agent-ix/quire-protocol/FR-043, type: depends_on }
  - { target: ix://agent-ix/quire-protocol/FR-008, type: references }
  - { target: ix://agent-ix/quire-spec-language/FR-519, type: depends_on }
  - { target: ix://agent-ix/quire-spec-language/FR-516, type: references }
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

The linked finite protocol representation, or a typed `LinkRefusal`. An
upstream `ViewRefusal` is carried unchanged as an owner-read refusal. A
consumer-owned refusal identifies its checked node ids and retains the source
map loci returned by QSL's `locus` accessor for those ids; it never manufactures
a locus. The retired `wire::Package` (`src/link/mod.rs`, `src/model/v2.rs`) is
not an input or an output.

## Behavior

- The read view shall list the clause, its roles, channels, queues, controls in
  layout order, compensations, required temporal clauses, `terminal`,
  `scheduling` and activation rows as node ids, obtained through QSL's
  `protocol_view`. It shall read node content from the admitted
  `CheckedPackageV2` through Contract IR's checked node and term accessors, and
  use QSL's `control_operation`, `delivery`, `queue_capacity`, `binder`,
  `temporal_binding` and `attempts_maximum` accessors wherever those accessors
  own the value. It shall not parse checked-package bytes or implement a second
  node reader.
- The read view shall own no table of its own for those nodes.
- The link layer shall take every causal edge from `causal_relation`.
- The link layer shall construct no causal edge.
- Each record the link layer read from `wire::Package` shall have the successor
  that QSL FR-519's record-to-node map names. A window is an evaluation input,
  not a checked-package member; the linker shall not expect a window-binding
  node or import a window accessor from QSL.
- The link layer shall keep its own refusals because the checked package does
  not refuse them. It shall refuse zero-progress iteration when a `repeat`
  control's maximum is zero; it shall read selected temporal bindings for the
  required temporal clauses through QSL's `temporal_binding`;
  duplicate role authority by reading every listed role node's authority
  through its checked node content; and conflicting channel premises by reading
  each listed channel's premise terms and QSL's `delivery`. It shall retain the
  offending checked node ids and their available `locus` entries in each
  refusal and return no representation. Owner read failures remain the exact
  `ViewRefusal` returned by QSL inside `LinkRefusal`; the linker does not
  relabel them as consumer failures.
- The link layer shall refuse an absent node, a non-protocol node and a
  `protocol_reference` carrier with a typed refusal.
- The link layer shall return no partial representation with a refusal.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-044-AC-1 | Over **R**'s protocol, the linked representation lists the roles, channels, queues, controls, compensations, required temporal clause and activation rows equal to `protocol_view`'s node ids, and its edge set equals `causal_relation(package, protocol)`, back edges of every `repeat` included. | Test |
| FR-044-AC-2 | No function of the crate constructs an `EdgeKind` or `CausalEdge` value or derives an edge from a node: the edge set comes only from `qsl_package::protocol_view::causal_relation`. | Test (TC-150) |
| FR-044-AC-3 | Over checked packages whose `protocol_view` identifies respectively a zero-progress `repeat` with maximum zero, two role nodes with the same authority, and two channel nodes with mutually conflicting premise terms, linking returns a consumer-owned `LinkRefusal` naming the offending node ids and retaining their available `locus` entries, with no representation. The linker obtains ids from `protocol_view`, reads content through Contract IR's checked node/term accessors, uses QSL owner accessors for content they publish, parses no bytes and maintains no second reader. Required temporal clauses resolve their selected binding through QSL's `temporal_binding`. | Test |
| FR-044-AC-4 | An absent node id, the id of a `protocol_reference` carrier and the id of a non-protocol node each return a typed refusal naming the cause and no partial representation. | Test |
| FR-044-AC-5 | The signature of the link entry names `CheckedPackageV2` and `CheckedNodeId`, returns `LinkedProtocol` or `LinkRefusal`, and names no `wire::Package`, `Handle` or `wire::Declaration`; compile-fail uses of each retired type through the crate's public surface fail to resolve. | Test |
| FR-044-AC-6 | The linker reads no window-binding node from `CheckedPackageV2` and imports no window view from QSL. A window value is accepted only at the evaluation input owned by FR-516; absence of a window member in the checked package has no effect on linking. | Test |
| FR-044-AC-7 | The criteria FR-002-AC-1 to FR-002-AC-5 hold over recipe **R**'s checked package: concurrent instances stay distinct; every permitted interleaving preserves the owner-supplied causal edges and joins; each consumer-owned refusal refuses; reordering equal-time input adds no edge; and changing the selected temporal profile or clock binding changes the linked result or returns its typed refusal. | Test |

## Dependencies

- QSL FR-519 (`protocol_view`, `causal_relation`, the record-to-node map), QSpec
  FR-464 (the causal relation), FR-043.
- [FR-002](FR-002-link-finite-protocol-representation.md) states the linking
  contract for the retired wire; this requirement is its successor.
