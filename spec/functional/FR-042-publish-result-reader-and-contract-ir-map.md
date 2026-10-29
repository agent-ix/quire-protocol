---
id: FR-042
title: "Publish the canonical result reader and selected Contract-IR mapping"
type: FR
relationships:
  - { target: ix://agent-ix/quire-protocol/FR-006, type: depends_on }
  - { target: ix://agent-ix/quire-protocol/FR-031, type: depends_on }
  - { target: ix://agent-ix/quire-protocol/FR-032, type: depends_on }
  - { target: ix://agent-ix/quire-protocol/FR-033, type: depends_on }
  - { target: ix://agent-ix/quire-specification/FR-240, type: depends_on }
  - { target: ix://agent-ix/quire-specification/FR-286, type: depends_on }
  - { target: ix://agent-ix/quire-specification/FR-295, type: depends_on }
  - { target: ix://agent-ix/tl-syntax/IF-003, type: implements }
  - { target: ix://agent-ix/tl-syntax/VO-005, type: implements }
---
# FR-042: Publish the canonical result reader and selected Contract-IR mapping

## Description

When a caller produces or consumes a protocol result, the library SHALL expose
the complete `quire.protocol.result/v1-draft.1` contract through one bounded
canonical producer and strict public reader and SHALL publish a separate exact
`quire.protocol.contract-ir-result-map/v1` validated mapping view.

## Public subsystem

The implementation SHALL organize the owner surface as
`result::{record,reader,global,lineage,contract_ir}` while preserving existing
public paths by re-export. It SHALL publish immutable `SCHEMA_BYTES` and
`SCHEMA_SHA256` for both contracts and expose:

```text
result::produce(Input, Limits) -> Result<Document, Refusal>
result::read(bytes, Expected, Limits) -> Result<ValidatedResult, Refusal>
result::contract_ir::map(&ValidatedResult, MappingSelection, Limits)
  -> Result<MappedResultView, Refusal>
result::contract_ir::read(bytes, &ValidatedResult, MappingSelection, Limits)
  -> Result<MappedResultView, Refusal>
```

`ValidatedResult` and `MappedResultView` have no public constructors. `Document`
exposes the complete canonical bytes and FR-286 result identity.

## Canonical result admission

The producer and reader SHALL enforce every FR-061/FR-240/FR-286 field and
combination in FR-031 through FR-033. The four decision/surrounding progress and
closure axes each carry only independently selected `open`/`closed` states and
exact owner assertion references. Completeness is a separate embedded
`quire.observation.completeness-assertion/v1` selection with one FR-295 state.
Truth uses only `satisfied`, `violated`, `pending`, `unavailable`; activation,
settlement, participation, adequacy, global closure, claim and correction
lineage retain their separately owned closed values and required premises.

The reader SHALL reject unknown/duplicate/missing/out-of-order fields, trailing
data, noncanonical JSON, unknown contracts/labels, noncanonical JCS, same
identity on unequal canonical bytes, invalid
axis combinations, incomplete support, invalid global premises, missing/cyclic/
self lineage, and a non-direct correction predecessor.

## Contract-IR mapping contract

The mapping selection fixes the exact mapping contract/schema digest and the
source result contract/schema/revision. Its canonical mapped view contains:
source result identity/digest, native subject/correspondence identities,
execution, `value` or `nonValue`, the four independent progress/closure owner
references and states, completeness owner reference/state/fact identities,
settlement, exact decision support, and correction relation/direct predecessor.
Each mapped progress/closure owner reference SHALL expose its selected owner
contract and schema digest, canonical assertion identity and digest, authority
identity, revision and digest, scope identity, population identity, and
independent state through borrowed typed accessors; a downstream structural
consumer SHALL NOT need to parse the mapping JSON to compare any retained owner
reference field.

Only a `completed` source result with `satisfied` maps to `value:true`; only a
`completed` source result with `violated` maps to `value:false`. Every other
valid combination maps to one typed `nonValue` retaining the exact source
execution/truth/settlement/completeness facts. Pending, unavailable, incomplete,
unsupported, failed and refused values are never Boolean-coerced. The mapping
view is derived entirely from a `ValidatedResult`; it has no callback, parser,
evaluator, trust flag or independently supplied source fields.

The mapping identity is lowercase SHA-256 over its contract label, a zero byte,
and canonical mapping bytes with `identity` omitted. It is distinct from and
cannot replace the source result identity. A reader re-derives the mapping from
the independently validated source result and demands byte equality.

## Limits

Producer, reader and mapping limits independently bound bytes, JSON depth,
strings, claims, premises, support records, limitations, lineage nodes and total
visited fields. Work is charged before allocation/traversal. Exact limits pass;
one-over returns a typed resource refusal with no partial result/view. A retry
uses fresh accounting; caller limits may lower but not raise owner maxima.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-042-AC-1 | Every valid canonical-result combination produces deterministic FR-286 bytes/identity/digest and strict-reads to a constructor-private view; every one-axis forbidden combination refuses. | Test (TC-015) |
| FR-042-AC-2 | Omission, duplication, reordering, unknown fields/labels/contracts, trailing/noncanonical bytes and every identity, scope, decision-premise or lineage mutation refuse without partial output. | Test (TC-015) |
| FR-042-AC-3 | Every independent combination of the four open/closed axes and FR-295 completeness remains representable and no axis supplies another. | Test (TC-015) |
| FR-042-AC-4 | The exact selected mapping derives true only from completed+satisfied and false only from completed+violated; all other valid rows remain typed non-values with complete source facts. | Test (TC-015) |
| FR-042-AC-5 | Mapping bytes/read views are fully re-derived from the validated result; every retained progress/closure owner-reference field is available through borrowed typed accessors without JSON parsing; and source/mapping identities, vocabularies and schema digests cannot substitute for one another. | Test (TC-015) |
| FR-042-AC-6 | Exact and one-over limits cover every bounded dimension and refuse before excess retention; no public constructor, callback, parser, evaluator or trust flag bypasses admission. The one exception is the opt-in `test-support` Cargo feature: its `result::test_support` constructors are compiled only when a dependent enables the feature for testing, they run the canonical producer and the strict reader over their input, and the default build exposes none of them. | Test (TC-015) |

## Dependencies

FR-006 and FR-031 through FR-033 supply canonical result construction,
orthogonal axes and lineage. `quire-specification` FR-240/286/287 supply the
shared result identity and completeness authority. Quire Observation supplies
embedded assertion views; Contract IR is a downstream mapped-view consumer.

## Status

Implemented by Protocol PR #51 and completed for downstream consumption by
Protocol #54 / PR #55. QCI FR-025 and #71 consume the resulting borrowed views
without making QCI an upstream Protocol dependency.
