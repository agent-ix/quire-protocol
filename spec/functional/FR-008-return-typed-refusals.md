---
id: FR-008
title: "Return typed source-bound refusals"
type: FR
relationships:
  - target: ix://agent-ix/quire-protocol/ADR-008
    type: references
  - target: ix://agent-ix/quire-specification/FR-047
    type: depends_on
  - target: ix://agent-ix/quire-protocol/US-001
    type: implements
  - target: ix://agent-ix/quire-protocol/US-002
    type: implements
  - target: ix://agent-ix/quire-protocol/FR-001
    type: depends_on
  - target: ix://agent-ix/quire-protocol/FR-007
    type: depends_on
  - target: ix://agent-ix/quire-protocol/FR-036
    type: contains
  - target: ix://agent-ix/quire-protocol/FR-037
    type: contains
  - target: ix://agent-ix/quire-protocol/FR-038
    type: contains
  - target: ix://agent-ix/quire-protocol/FR-039
    type: contains
  - target: ix://agent-ix/quire-protocol/FR-040
    type: contains
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
# FR-008: Return typed source-bound refusals

## Description

When any public operation cannot return its declared success value, the library SHALL return one typed refusal envelope that this requirement owns.
The library SHALL construct that envelope from exactly the five-member domain, the partial-output state and the local `unsupported-source-contract` disposition this requirement owns.
The library SHALL satisfy this requirement only by satisfying every one of FR-036 through FR-040, each of which states one atomic, independently verifiable obligation of that envelope.

This requirement is the parent of the envelope. It owns the envelope, its domain,
its partial-output state, its local disposition and its adopted-vocabulary
binding to upstream. It states no obligation its children do not state
atomically, and it weakens none of them.

## Inputs

One admission, representation, compensation, conformance, claim, result-reader,
or projection failure produced under an exact selected local contract, or one
accepted typed upstream failure response with its declared source contract and
diagnostic-catalog selection.

## Outputs

One `ProtocolRefusal` with a domain from `admission`, `representation`,
`assessment`, `result`, or `projection`; the stable source-owned code; affected
identities and locations; selected diagnostic-catalog identity, revision, and
digest; originating stage and prerequisites; ordered typed causes; an explicit
partial-output state; and the refusal axis under which it was raised, which
[FR-014](./FR-014-typed-outcome-identity.md) uses as an identity component.

## Behavior

Each obligation of the envelope is stated by exactly one child. This table is the
decomposition, not an additional obligation:

| Obligation | Child |
| --- | --- |
| Every declared field is present on exactly one envelope per failing path, and the refusal axis is recorded so injectivity is observable | [FR-036](./FR-036-carry-complete-refusal-envelope.md) |
| No code, stage, identity, location, equality or retry decision is derived from display text, and unavailable context stays explicitly unavailable | [FR-037](./FR-037-derive-no-refusal-fact-from-text.md) |
| A code is validated against the exact selected catalog before interpretation; an unknown contract, selection or code is retained under local `unsupported-source-contract` uninterpreted; a changed catalog selection or an erased or replaced stage refuses | [FR-038](./FR-038-retain-unknown-source-contracts.md) |
| Typed cause pairs stay structurally distinct, every independent cause and disposition is retained in the response's declared semantic order, and a zero diagnostic-detail budget erases nothing | [FR-039](./FR-039-preserve-typed-causes-distinctly.md) |
| A fatal refusal emits no success value, and only FR-005-authorized independent assessment results survive another claim's failure | [FR-040](./FR-040-emit-no-success-beside-fatal-refusal.md) |

The load-bearing property across all five is refusal-identity **injectivity**:
one shared error code must not be able to satisfy every negative scenario row.
[FR-014](./FR-014-typed-outcome-identity.md) owns that identity and its
injectivity criterion, and each child above makes the property checkable inside
its own obligation — FR-036 by recording the axis, FR-037 by denying text as a
repair, FR-038 by denying the retention disposition as a catch-all, FR-039 by
keeping the compared cause set unmerged, and FR-040 by refusing absence of
success as an identity.

## Acceptance Criteria

Each criterion below is preserved verbatim in obligation and is stated atomically
by the named child. The child's criteria are the verification; this table is the
aggregation.

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-008-AC-1 | Every public failure path returns the correct typed domain, selected diagnostic-catalog identity/revision/digest, stable source code, originating stage and prerequisites, affected identity, available or explicitly unavailable location, ordered causes, and partial-output state; stated atomically by FR-036-AC-1, FR-036-AC-4 and FR-036-AC-5. | Test (TC-011) |
| FR-008-AC-2 | Changing display text, locale, or cause rendering cannot change a refusal identity, code, equality comparison, or retry decision; stated atomically by FR-037-AC-1 and FR-037-AC-2. | Test (TC-011) |
| FR-008-AC-3 | An unknown source revision, catalog selection, or code becomes local `unsupported-source-contract`, retains the original catalog and value uninterpreted, and cannot be interpreted through a nearest known version; stated atomically by FR-038-AC-2. | Test (TC-011) |
| FR-008-AC-4 | Fatal admission, representation, result, and projection refusals emit no success value; only FR-005-authorized independent assessment results survive another claim's failure; stated atomically by FR-040-AC-1 and FR-040-AC-2. | Test (TC-011) |
| FR-008-AC-5 | Missing/conflicting definitions, wrong-anchor/missing-observation, unknown-syntax/prohibited-construct, unproved-definedness/logical-violation, and resource-incomplete/unsupported cause pairs remain structurally distinct; stated atomically by FR-039-AC-1. | Test (TC-011) |
| FR-008-AC-6 | Changing the selected diagnostic-catalog identity/revision/digest or erasing/replacing the originating stage refuses the response; stated atomically by FR-038-AC-3. | Test (TC-011) |
| FR-008-AC-7 | Unavailable source context remains explicit and cannot become an invented location through repeated display names, paths, or message text; stated atomically by FR-037-AC-3 and FR-037-AC-5. | Test (TC-011) |
| FR-008-AC-8 | Simultaneous independent invalid and missing-input dispositions both remain represented under a zero-detail control and cannot produce overall success; stated atomically by FR-039-AC-3. | Test (TC-011) |
| FR-008-AC-9 | Every obligation in the Behavior table is carried by exactly one child requirement, every child carries a `part_of` edge to this requirement and this requirement a `contains` edge back, and no obligation listed in FR-008-AC-1 through FR-008-AC-8 is absent from the children's criteria. | Inspection |

## Adopted vocabulary

The refusal **envelope** is B's, and this requirement owns it: the five-member
domain, the partial-output state, the local `unsupported-source-contract`
disposition, and the rule that two distinct refusal axes never share one typed
identity. That distinctness rule is *typed* by
[FR-014](./FR-014-typed-outcome-identity.md) — identity kind *value*, authority
*consumer*, key (`integration`, `refusal_axis`, `outcome_kind`) — and FR-014
records why the key cannot be its own oracle: `refusal_axis` is a key component,
so distinctness by key could never fail. Injectivity is therefore checked over
the producer-supplied interior the envelope carries, which the children oblige it
to retain.

The envelope's **interior** is upstream's:

| Interior field | Upstream object |
| --- | --- |
| Diagnostic catalog identity/revision/digest | `FR-270-versioned-diagnostic-catalog` |
| Diagnostic code | `FR-271-diagnostic-code-vocabulary` |
| Typed cause | `FR-272-typed-diagnostic-cause` |
| Originating stage | `FR-273-producer-stage` |
| Location kind | `FR-274-diagnostic-location-kind` |

### Where the upstream vocabulary does not yet reach

**The catalog itself is not adopted.** `FR-270-versioned-diagnostic-catalog`
records that "The only catalog so far offered is the **proposed** native
diagnostic interpretation … which states of itself: '**Proposed, not adopted or
implemented.**' It is source material here, not an adopted selection." Every
criterion below that selects a catalog identity, revision or digest therefore
selects something no adopted artifact supplies. This is a version-lock entry in
FR-009's terms, and it is `unresolved`. Every child inherits that gate and states
it: FR-036-AC-6 and FR-038-AC-6 report the dependent scenarios unexecutable
rather than passing or failing, and `spec.md` section 16 records that this
reference names no adopted artifact yet.

Four of the five interiors carry further open gaps that bear directly on this
requirement's criteria:

- **FR-271** records that "no requirement states which convention belongs to
  which namespace", and places no literal code in a selected catalog.
- **FR-272** records that the catalog "does not give those payload components
  stable field names, a total order, or a spelling convention of their own", and
  supplies none. FR-039 therefore retains the response's own declared semantic
  order and establishes none.
- **FR-273** records a contradiction rather than an absence: "Two sources
  enumerate the same axis and do not agree on its members" — FR-047 names six
  producer stages, the proposed catalog names eleven, and upstream quotes both
  verbatim and prefers neither. FR-008-AC-1 requires the originating stage, and
  FR-008-AC-6 requires erasing or replacing it to refuse; both rest on an axis
  whose membership is contested upstream. FR-038 states the gap and its owner and
  supplies no local member list.
- **FR-274** records that it "states no discriminator tag, identifier spelling or
  serialized form for the kinds themselves", and establishes no key or order for
  a JSON field/index path or a runtime object/value path. FR-008-AC-7 requires
  unavailable source context to stay explicit and never become an invented
  location; the kinds it would discriminate have no serialized form upstream, so
  FR-037 states its criteria over presence versus explicit absence only.

These are cited as gaps, not resolved here. Selecting a member list locally would
author a producer diagnostic vocabulary, which this repository does not own.

## Dependencies

- [FR-001](./FR-001-admit-selected-protocol-profile.md) through [FR-007](./FR-007-project-results-to-e02.md).
- [FR-036](./FR-036-carry-complete-refusal-envelope.md) through
  [FR-040](./FR-040-emit-no-success-beside-fatal-refusal.md) are this
  requirement's children and carry its obligations atomically.
- [FR-014](./FR-014-typed-outcome-identity.md) types the refusal identity and owns
  the injectivity criterion.
- [FR-009](./FR-009-lock-external-producer-revisions.md) owns the version-lock
  entry that gates the catalog selection.
- `ix://agent-ix/quire-specification/FR-047` owns the native typed-cause and
  diagnostic-catalog semantics.
- Each selected upstream contract owns its stable domain-specific refusal codes.
