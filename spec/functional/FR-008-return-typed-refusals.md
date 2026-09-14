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

When any public operation reaches a typed refusal, the library SHALL return one typed refusal envelope that this requirement owns.
The library SHALL construct that envelope from exactly the seven-member domain,
the closed Protocol stage and cause vocabularies, the partial-output state and
the local `unsupported-source-contract` disposition this requirement owns.
The library SHALL satisfy this requirement only by satisfying every one of FR-036 through FR-040, each of which states one atomic, independently verifiable obligation of that envelope.

This requirement is the parent of the envelope. It owns the envelope, its domain,
its partial-output state, its local disposition and its adopted-vocabulary
binding to upstream. It states no obligation its children do not state
atomically, and it weakens none of them.

## Inputs

One resolution, admission, representation, compensation, conformance, claim,
result-reader, projection or evidence-handoff refusal produced under an exact
selected local contract, or one
accepted typed upstream failure response with its declared source contract and
diagnostic-catalog selection.

## Outputs

One `ProtocolRefusal` with a domain from `resolution`, `admission`,
`representation`, `assessment`, `result`, `projection` or `evidence`; affected identities and locations;
the exact originating stage and prerequisites; ordered typed causes; resource
usage; an explicit partial-output state; and the refusal axis under which it was raised, which
[FR-014](./FR-014-typed-outcome-identity.md) uses as an identity component.

## Behavior

Each obligation of the envelope is stated by exactly one child. This table is the
decomposition, not an additional obligation:

| Obligation | Child |
| --- | --- |
| Every declared field is present on exactly one envelope per failing path, and the refusal axis is recorded so injectivity is observable | [FR-036](./FR-036-carry-complete-refusal-envelope.md) |
| No code, stage, identity, location, equality or retry decision is derived from display text, and unavailable context stays explicitly unavailable | [FR-037](./FR-037-derive-no-refusal-fact-from-text.md) |
| Protocol causes use the closed local vocabulary; a known source response is retained under its exact selected contract without remapping; an unknown contract or code is retained under local `unsupported-source-contract` uninterpreted; an erased or replaced stage refuses | [FR-038](./FR-038-retain-unknown-source-contracts.md) |
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
| FR-008-AC-1 | Every public failure path returns the correct typed domain, actual originating stage and prerequisites, affected identity, available or explicitly unavailable location, ordered causes, resource usage, and partial-output state; a source cause additionally retains its selected source contract and stable source code; stated atomically by FR-036-AC-1, FR-036-AC-4 and FR-036-AC-5. | Test (TC-011) |
| FR-008-AC-2 | Changing display text, locale, or cause rendering cannot change a refusal identity, code, equality comparison, or retry decision; stated atomically by FR-037-AC-1 and FR-037-AC-2. | Test (TC-011) |
| FR-008-AC-3 | An unknown source contract, revision, or code becomes local `unsupported-source-contract`, retains the original contract and value uninterpreted, and cannot be interpreted through a nearest known version; stated atomically by FR-038-AC-2. | Test (TC-011) |
| FR-008-AC-4 | Fatal resolution, admission, representation, whole-assessment, result, projection and evidence refusals emit no success value; only FR-005-authorized independent claim outcomes survive a claim-local construction refusal; stated atomically by FR-040-AC-1 and FR-040-AC-2. | Test (TC-011) |
| FR-008-AC-5 | Missing/conflicting definitions, wrong-anchor/missing-observation, unproved-definedness/logical-violation, identity-mismatch/incomplete-evidence, and unsupported-capability/projection-finding-loss cause pairs remain structurally distinct; stated atomically by FR-039-AC-1. | Test (TC-011) |
| FR-008-AC-6 | Changing the selected source-contract identity/revision or erasing/replacing the originating stage refuses the response; stated atomically by FR-038-AC-3. | Test (TC-011) |
| FR-008-AC-7 | Unavailable source context remains explicit and cannot become an invented location through repeated display names, paths, or message text; stated atomically by FR-037-AC-3 and FR-037-AC-5. | Test (TC-011) |
| FR-008-AC-8 | Simultaneous independent invalid and missing-input dispositions both remain represented under a zero-detail control and cannot produce overall success; stated atomically by FR-039-AC-3. | Test (TC-011) |
| FR-008-AC-9 | Every obligation in the Behavior table is carried by exactly one child requirement, every child carries a `part_of` edge to this requirement and this requirement a `contains` edge back, and no obligation listed in FR-008-AC-1 through FR-008-AC-8 is absent from the children's criteria. | Inspection |

## Owned and adopted vocabulary

The refusal **envelope** is Protocol-owned: the seven-member domain, partial-
output state, local stages and causes, `unsupported-source-contract`
disposition, and rule that two distinct refusal axes never share one typed
identity. The local `ProtocolStage` members are exactly `contract-resolution`,
`admission`, `representation`, `compensation`, `conformance`, `claim-dispatch`,
`result-assembly`, `result-read`, `projection`, and `evidence-handoff`. The local
`ProtocolCause` members are exactly `missing-definition`,
`conflicting-definition`, `wrong-anchor`, `missing-observation`,
`unproved-definedness`, `logical-violation`, `identity-mismatch`,
`incomplete-evidence`, `unsupported-capability`, and
`projection-finding-loss`. Payload fields are typed per member; a generic text
payload cannot stand in for one. This closed vocabulary is versioned as
`quire.protocol/refusal/1` and is not an interpretation of an upstream catalog.

The domain/stage map is closed and checked: `contract-resolution` maps to
`resolution`; `admission` to `admission`; `representation` to
`representation`; `compensation`, `conformance` and `claim-dispatch` to
`assessment`; `result-assembly` and `result-read` to `result`; `projection` to
`projection`; and `evidence-handoff` to `evidence`. No other pairing is valid.

Resource exhaustion is not a refusal and therefore is not a `ProtocolCause`.
Before a canonical claim row can exist it returns the independently typed
`ResourceIncomplete` operation outcome from NFR-002; after a claim is admitted
it is FR-241's canonical `resource-incomplete` disposition. Neither is wrapped
or relabelled as `ProtocolRefusal`.

That distinctness rule is *typed* by
[FR-014](./FR-014-typed-outcome-identity.md) — identity kind *value*, authority
*consumer*, key (`integration`, `refusal_axis`, `outcome_kind`) — and FR-014
records why the key cannot be its own oracle: `refusal_axis` is a key component,
so distinctness by key could never fail. Injectivity is therefore checked over
the producer-supplied interior the envelope carries, which the children oblige it
to retain.

An imported **source cause** remains upstream's:

| Interior field | Upstream object |
| --- | --- |
| Source-contract identity and revision | Exact FR-009 resolved capability |
| Stable source code and typed payload | Selected source contract |
| Source stage and locus, when supplied | Selected source contract |

Protocol never merges the local stage axis with a producer stage axis. A known
source contract is matched exhaustively through its typed Rust API. If that API
does not publish a field, Protocol records it unavailable; it does not borrow a
member or spelling from proposed FR-270 through FR-274 material. An adapter may
use a future accepted versioned diagnostic catalog, but the base envelope and
local refusal paths do not require that catalog and are executable without it.

## Dependencies

- [FR-001](./FR-001-admit-selected-protocol-profile.md) through [FR-007](./FR-007-project-results-to-e02.md).
- [FR-036](./FR-036-carry-complete-refusal-envelope.md) through
  [FR-040](./FR-040-emit-no-success-beside-fatal-refusal.md) are this
  requirement's children and carry its obligations atomically.
- [FR-014](./FR-014-typed-outcome-identity.md) types the refusal identity and owns
  the injectivity criterion.
- [FR-009](./FR-009-lock-external-producer-revisions.md) owns the version-lock
  entry that gates the catalog selection.
- `ix://agent-ix/quire-specification/FR-047` and FR-270 through FR-274 are
  informative for source-owner diagnostic contracts; they do not own the local
  Protocol vocabulary or block local envelope execution.
- Each selected upstream contract owns its stable domain-specific refusal codes.
