---
id: FR-036
title: "Carry the complete typed refusal envelope"
type: FR
relationships:
  - target: ix://agent-ix/quire-protocol/FR-008
    type: part_of
  - target: ix://agent-ix/quire-protocol/FR-014
    type: depends_on
  - target: ix://agent-ix/quire-protocol/FR-009
    type: depends_on
  - target: ix://agent-ix/quire-specification/FR-047
    type: depends_on
  - target: ix://agent-ix/quire-specification/FR-270
    type: references
  - target: ix://agent-ix/quire-specification/FR-271
    type: references
  - target: ix://agent-ix/quire-specification/FR-272
    type: references
  - target: ix://agent-ix/quire-specification/FR-273
    type: references
  - target: ix://agent-ix/quire-specification/FR-274
    type: references
---
# FR-036: Carry the complete typed refusal envelope

## Description

When a public operation cannot return its declared success value, the library SHALL return exactly one typed refusal envelope carrying every field declared in Outputs below.
The library SHALL record on that envelope the refusal axis under which the refusal was raised.
The library SHALL report an envelope carrying no refusal axis as an unassigned axis rather than folding it into a catch-all identity.
The library SHALL treat an absent declared envelope field as a failure of this requirement rather than defaulting it.

## Inputs

One admission, representation, compensation, conformance, claim, result-reader or
projection failure produced under an exact selected local contract, or one
accepted typed upstream failure response with its declared source contract and
its diagnostic-catalog selection.

## Outputs

One `ProtocolRefusal` carrying exactly these fields:

| Field | Owner | Note |
| --- | --- | --- |
| Domain | This requirement's parent [FR-008](./FR-008-return-typed-refusals.md) | One of `admission`, `representation`, `assessment`, `result`, `projection`. |
| Stable code | The selected source contract | Source-owned and copied byte-exactly; `FR-271` types the vocabulary. |
| Selected diagnostic-catalog identity, revision and digest | `ix://agent-ix/quire-specification/FR-270` | Selected, never defaulted. |
| Originating stage and every available prerequisite | `ix://agent-ix/quire-specification/FR-273` | Membership of the stage axis is contested upstream; see the gap section. |
| Affected identity | The failing operation's own subject | Retained, never reconstructed. |
| Location, available or explicitly unavailable | `ix://agent-ix/quire-specification/FR-274` | Absence is a value, not a gap to fill. |
| Ordered typed causes | `ix://agent-ix/quire-specification/FR-272` | Order and multiplicity are [FR-039](./FR-039-preserve-typed-causes-distinctly.md)'s obligation. |
| Partial-output state | This requirement's parent FR-008 | Explicit on every envelope. |
| Refusal axis | [FR-014](./FR-014-typed-outcome-identity.md) | The identity component that makes injectivity observable. |

## Behavior

The library SHALL populate every field above on every public failure path, or fail this requirement.
The library SHALL record the refusal axis as the `FR-014` identity component it is, so that the pairwise injectivity check of FR-014-AC-1 is evaluable from the envelope alone.
The library SHALL NOT substitute a two-valued admits/refuses disposition for the recorded axis, because such a disposition is satisfied by one shared error code across every negative row.
The library SHALL refuse an envelope whose domain is absent or is a sixth value outside the five closed members.

Field *content* rules live in the sibling requirements: text-independence in
[FR-037](./FR-037-derive-no-refusal-fact-from-text.md), unknown-contract
retention in [FR-038](./FR-038-retain-unknown-source-contracts.md), cause order
and multiplicity in [FR-039](./FR-039-preserve-typed-causes-distinctly.md), and
suppression of a co-emitted success value in
[FR-040](./FR-040-emit-no-success-beside-fatal-refusal.md). This requirement owns
presence and cardinality only.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-036-AC-1 | Every public failure path returns exactly one envelope carrying the typed domain, the selected diagnostic-catalog identity/revision/digest, the stable source code, the originating stage and every available prerequisite, the affected identity, an available or explicitly unavailable location, the ordered typed causes, and the partial-output state; an absent field fails the path rather than being defaulted. | Test (TC-011) |
| FR-036-AC-2 | Every returned envelope records the refusal axis under which it was raised, and an envelope recording no axis is reported as an unassigned axis and satisfies no negative scenario row. | Test (TC-011) |
| FR-036-AC-3 | The recorded axis and the retained interior are together sufficient to evaluate FR-014-AC-1's pairwise comparison of distinct axes without reading any display text, and a corpus whose refusing rows record only a two-valued admits/refuses disposition fails this criterion. | Test (TC-011) |
| FR-036-AC-4 | An envelope whose domain is absent, or is a sixth value outside `admission`, `representation`, `assessment`, `result` and `projection`, is refused. | Test (TC-011) |
| FR-036-AC-5 | The partial-output state is explicit on every envelope and is inferred from neither an empty output nor the absence of a success value. | Test (TC-011) |
| FR-036-AC-6 | The catalog identity/revision/digest field is populated only from a catalog pinned by an FR-009 manifest entry; while that entry's status is `unresolved`, every criterion here that requires it is reported unexecutable rather than passing. | Inspection |

## Adopted vocabulary

The envelope is this repository's and FR-008 owns it. Its interior is upstream's
and is adopted by reference, not restated:
`ix://agent-ix/quire-specification/FR-270` (catalog identity, revision, digest),
`FR-271` (diagnostic code), `FR-272` (typed cause), `FR-273` (originating stage),
`FR-274` (location kind). The refusal identity this requirement makes observable
is [FR-014](./FR-014-typed-outcome-identity.md)'s: identity kind *value*,
authority *consumer*, key (`integration`, `refusal_axis`, `outcome_kind`).

FR-014 records why that key cannot itself be the injectivity oracle:
`refusal_axis` is a key component, so distinctness by key is true by
construction. This requirement therefore obliges the envelope to *carry* the axis
and the producer-supplied interior, so that FR-014-AC-1 can compare interiors
across a pair of distinct axes and fail a shared one. A criterion that cannot
fail is not asserted here.

### Where the upstream vocabulary does not yet reach

**The selected catalog is not adopted, and this requirement inherits that gate.**
`FR-270-versioned-diagnostic-catalog` states of the only catalog so far offered:
"**Proposed, not adopted or implemented.**" and calls it source material here,
not an adopted selection. FR-009's version-lock manifest carries the diagnostic
catalog as an entry whose status is `unresolved`, and `spec.md` section 16 records
that this reference names no adopted artifact yet. FR-036-AC-1 and FR-036-AC-6
therefore select something no adopted artifact supplies; under FR-009 every
scenario requiring that entry is reported unexecutable rather than passing or
failing.

- **Originating stage** — `FR-273` records a contradiction, not an absence: "Two
  sources enumerate the same axis and do not agree on its members." FR-036-AC-1
  requires the stage field to be present; it establishes no member list, because
  the axis membership is contested upstream and belongs to upstream.
- **Location kind** — `FR-274` states "no
  discriminator tag, identifier spelling or serialized form" for the kinds
  themselves. FR-036-AC-1 therefore obliges presence-or-explicit-absence only and
  fixes no spelling.
- **Diagnostic code** — `FR-271` records that "no requirement states which
  convention belongs to which namespace". No literal code is placed in a selected
  catalog here.
- **Typed cause** — `FR-272` records that the catalog "does not give those
  payload components stable field names, a total order, or a spelling convention
  of their own". No payload field vocabulary is supplied here.

No local member list, spelling or order is authored for any of the four. Doing so
would author a producer diagnostic vocabulary this repository does not own.

## Dependencies

- [FR-008](./FR-008-return-typed-refusals.md) owns the envelope, its five-member
  domain and its partial-output state; this requirement is `part_of` it.
- [FR-014](./FR-014-typed-outcome-identity.md) owns the refusal identity, its key
  and its injectivity obligation.
- [FR-009](./FR-009-lock-external-producer-revisions.md) owns the version-lock
  entry that gates the catalog selection, and reports a scenario requiring an
  `unresolved` entry as unexecutable.
- `ix://agent-ix/quire-specification/FR-047` owns the native typed-cause and
  diagnostic-catalog semantics.

## Status

Draft. No execution is claimed. the version 2 compiled-protocol contract is unpinned
and the diagnostic-catalog entry is `unresolved`, so the scenarios behind the
Test criteria above are unexecutable today.
