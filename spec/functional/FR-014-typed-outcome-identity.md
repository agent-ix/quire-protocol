---
id: FR-014
title: "Typed outcome identity"
type: FR
object: value_object
relationships:
  - target: ix://agent-ix/quire-protocol/ADR-008
    type: references
  - target: ix://agent-ix/quire-protocol/US-004
    type: implements
  - target: ix://agent-ix/quire-protocol/FR-010
    type: part_of
  - target: ix://agent-ix/quire-protocol/FR-011
    type: depends_on
  - target: ix://agent-ix/quire-protocol/FR-008
    type: specifies
  - target: ix://agent-ix/quire-protocol/NFR-005
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
# FR-014: Typed outcome identity

## Description

When an integration reaches a non-success outcome, the library SHALL return a typed outcome identity distinct from that of every other refusal axis in that integration.
Two scenarios exercising one refusal axis MAY share one typed outcome identity.
The library SHALL derive a typed outcome identity from no display text, locale or cause rendering.

## Properties

- **integration** (identity component) — the integration within which
  distinctness is required. Distinctness is scoped per integration, exactly as
  [NFR-005](../non-functional/NFR-005-retain-composed-acceptance-evidence.md)
  scopes it: "two axes sharing one may not" collide *within* one integration.
- **refusal_axis** (identity component) — the named way the outcome differs from
  success. For the version 2 integration, the producer's published
  `MutationManifest` enumerates the cases and their expected refusal codes. The
  manifest is a scenario index and not a locally re-declared refusal vocabulary:
  the refusal a caller receives is the producer's own value, from
  `protocol_artifact::Error` and its typed payloads, carried unmodified.

  Five of the corpus axes name conditions the producer already has codes for —
  a wrong contract version is `Unsupported::Wire`, a cross-domain digest is
  `Invalid::Seal`, a recanonicalized input is `Invalid::Canonical`, a temporal
  binding fault is `Invalid::Binding` or `Invalid::Order`, and an exhausted
  budget is `Error::Incomplete`. This requirement selects none of them, spells
  none of them, and does not re-partition them: the producer decides how coarse
  its codes are, and several corpus axes may share one code.

  The injectivity obligation is this requirement's and is stated over the
  producer-supplied interior, not over this key: `refusal_axis` is a key
  component, so a criterion asserting that distinct axes carry distinct
  identities could never fail. `FR-014-AC-1` states it over the producer's code
  and cause instead.
- **outcome_kind** (identity component) — which non-equivalent outcome sense
  this is. The distinction below records which of them this repository returns;
  it establishes no vocabulary. Where an outcome reports whether a requested
  evaluator ran, the vocabulary is
  `ix://agent-ix/quire-specification/FR-241`'s five members — `completed`,
  `refused`, `unsupported`, `resource-incomplete`, `failed` — and that is the
  spelling used.
- **envelope interior** (referenced, not owned) — the selected diagnostic-catalog
  identity/revision/digest, the diagnostic code, the typed causes, the
  originating stage and the location kind. These are upstream's `FR-270` through
  `FR-274`, referenced through
  [FR-008](./FR-008-return-typed-refusals.md)'s adopted-vocabulary table and
  restated here in no form.

### Envelope and interior

The refusal **envelope** is this repository's and `FR-008` owns it. This artifact
types the envelope's **identity** only: which outcomes are the same outcome and
which are different ones. The envelope's interior is upstream's, and where
upstream records an open gap — `FR-271` on namespace convention, `FR-272` on
payload field names, order and spelling, `FR-273` on a producer-stage membership
its two sources contradict, `FR-274` on discriminator tag and serialized form —
this artifact cites the gap and supplies no local member list, key or order.
A local list would author a producer diagnostic vocabulary this repository does
not own.

Injectivity is nevertheless statable without those gaps closing, because it is a
property of the mapping from axis to identity and not of the interior's spelling.

### Injectivity

The mapping from refusal axis to typed outcome identity is injective within one
integration: distinct axes map to distinct identities, and the mapping onto a
producer's own typed refusal enum stays injective across the pin step.

A two-valued admits/refuses oracle is satisfied by one shared error code across
every negative row. That is the defect this artifact exists to prevent. The
version 2 lane therefore compares every executed mutation against the exact
producer-published `expected_refusal_code` through `Error::code`; it never
scores a negative row from `is_err()` alone.

Sharing in the other direction is legitimate and is not a collision: many
scenarios may exercise one axis, and they carry that axis's one identity.

### The three-way outcome distinction

These outcomes are non-equivalent, and none is reportable as another
([IT-003](../integration/IT-003-publish-assurance-and-e02-results.md)-SC-06):

| Outcome kind | `FR-241` member | Returned when | Owner |
| --- | --- | --- | --- |
| Resource-incomplete | `resource-incomplete` | A declared budget dimension is exhausted | [NFR-002](../non-functional/NFR-002-enforce-resource-bounds.md) |
| Typed refusal | `refused` | An input is invalid | [FR-008](./FR-008-return-typed-refusals.md) |
| Unsupported | `unsupported` | The requested capability is unsupported | `FR-241`; one unsupported request does not remove an independent supported result (FR-061-AC-5) |
| Failed | `failed` | The selected analysis backend failed for this exact claim | `FR-241` |
| Completed, empty | `completed` | An assessment completed and its finding set is legitimately empty | [FR-006](./FR-006-produce-canonical-protocol-results.md) |

An earlier revision of this table carried three rows and called them "the three
non-equivalent outcome senses". That collapsed `FR-241`'s five published members
into three, folding `unsupported` and `failed` into "typed refusal" — a
distinction `FR-062` projects separately to E02 as `unsupported` and `failed`,
so the collapse was not merely coarse, it was lossy at a boundary this
repository crosses.

No non-success outcome yields an E02 success, and each carries the reason it
did not complete rather than an empty finding set that reads as clean.

## Identity and comparison

- **Identity kind** — *value*, authority *consumer*. The identity is compared
  componentwise and is this repository's own label, not a producer's. Under
  [FR-011](./FR-011-consumer-identity-discipline.md) it crosses no boundary as
  authority: no producer accepts it as a selector, and it appears in no canonical
  result as a producer reference. The producer's own code, catalog and stage
  inside the envelope stay producer-authored and are copied byte-exactly.
- **Key** — (`integration`, `refusal_axis`, `outcome_kind`). Equality is
  componentwise over exactly those three.

  **This key cannot be the injectivity oracle, and no criterion here treats it
  as one.** `refusal_axis` is itself a key component, so "two distinct axes carry
  distinct identities" is true by construction: the check could never fail, and
  the defect it exists to catch — one shared producer code across two unrelated
  axes — would stay invisible behind it. The injectivity obligation below is
  therefore stated over the *observed interior* the envelope carries, which the
  key does not determine. Display text, locale, cause rendering,
  message wording, path and span never participate, and none reconstructs an
  absent component (FR-008-AC-2, `FR-204`). Scenario identity is deliberately
  absent from the key: including it would make every scenario's outcome trivially
  distinct and destroy the collision check.
- **Key order** — lexicographic over (`integration`, `refusal_axis`,
  `outcome_kind`), for collection comparison and collision detection only. It
  carries no causal meaning, expresses no severity ranking, and implies no
  precedence between refusal axes or between outcome kinds.
- **Causal order** — none between outcome identities. Within one refusal the
  typed causes carry a declared semantic order that `FR-008` retains; that order
  is the interior's and is not this key's.
- **Absence** — an absent refusal axis or outcome kind compares only to absence
  and is never defaulted. An outcome carrying no axis is a collision waiting to
  happen and is reported as an unassigned axis rather than folded into a
  catch-all identity. Absence of an interior component stays explicitly
  unavailable and never becomes an invented location (FR-008-AC-7).
- **Digest domain** — not applicable to the identity itself. The envelope's
  interior carries the selected diagnostic catalog's identity, revision and
  digest; that digest is `FR-270`'s and its domain selection is upstream's, not
  recomputed here.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-014-AC-1 | Within one integration, for every ordered pair of distinct refusal axes, the producer-supplied interior the envelope carries — its `FR-271` code together with its `FR-272` typed cause — differs between the two. Two axes returning one shared code and cause fail this criterion, and the failure is reported against the pair rather than absorbed as a passing row. | Test (TC-011) |
| FR-014-AC-2 | Two scenarios exercising one refusal axis carry one typed outcome identity, and that sharing fails no distinctness check. | Test (TC-011) |
| FR-014-AC-3 | The mapping from this repository's outcome identities onto a producer's typed refusal enum is injective, and a mapping collapsing two axes onto one producer code fails the integration. | Analysis |
| FR-014-AC-4 | An integration in which every negative row carries one shared identity fails the distinctness check rather than reporting full coverage. | Test (TC-011) |
| FR-014-AC-5 | A resource-incomplete outcome, a typed refusal and a completed assessment with an empty finding set compare pairwise unequal, and none is reported as another. | Test (TC-011, IT-003) |
| FR-014-AC-6 | No non-success outcome yields an E02 success, and each carries the reason it did not complete. | Test (IT-003) |
| FR-014-AC-7 | Changing display text, locale or cause rendering changes no typed outcome identity, no equality comparison and no retry decision. | Test (TC-011) |
| FR-014-AC-8 | No local member list, key or order is supplied for the interior axes upstream records as open, and each such dependency cites the upstream gap. | Inspection |

## Dependencies

- [FR-008](./FR-008-return-typed-refusals.md) owns the refusal envelope, its
  five-member domain, the partial-output state, and the adopted-vocabulary table
  binding the interior to upstream.
- [NFR-005](../non-functional/NFR-005-retain-composed-acceptance-evidence.md)
  states the per-refusal-axis distinctness rule and the check that fails an
  integration when two axes collide.
- [NFR-002](../non-functional/NFR-002-enforce-resource-bounds.md) owns the
  resource-incomplete outcome.
- [FR-006](./FR-006-produce-canonical-protocol-results.md) owns the completed
  canonical result whose finding set may be legitimately empty.
- [FR-011](./FR-011-consumer-identity-discipline.md) supplies the kind and
  authority axes.
- `ix://agent-ix/quire-specification/FR-270` through `FR-274` own the envelope's
  interior; each is referenced, and each open gap is cited rather than closed
  locally.

## Status

Draft. This artifact records existing meaning structurally and establishes no new
obligation. The three-way outcome distinction is recorded as a conflict in the
source material rather than resolved here: SR-046 FND-003 and the
failure-domain review's FND-010 both record that "incomplete" carries
unreconciled senses across `NFR-002`, `FR-008` and `IT-003-SC-06`, and that no
functional requirement introduces incompleteness as a third outcome kind. This
artifact types the distinction `IT-003-SC-06` depends on and names that gap; it
does not author the incomplete-reason vocabulary, which originates outside this
boundary.
