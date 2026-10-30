---
id: FR-005
title: "Dispatch independent protocol claims"
type: FR
relationships:
  - target: ix://agent-ix/quire-protocol/US-001
    type: implements
  - target: ix://agent-ix/quire-protocol/FR-004
    type: depends_on
  - target: ix://agent-ix/quire-protocol/FR-029
    type: contains
  - target: ix://agent-ix/quire-protocol/FR-030
    type: contains
  - target: ix://agent-ix/quire-specification/FR-060
    type: depends_on
  - target: ix://agent-ix/quire-specification/FR-241
    type: references
  - target: ix://agent-ix/quire-specification/FR-249
    type: references
  - target: ix://agent-ix/quire-specification/FR-280
    type: references
  - target: ix://agent-ix/quire-specification/FR-281
    type: references
---
# FR-005: Dispatch independent protocol claims

## Description

When one or more protocol claims are requested, the dispatcher SHALL return one independent execution disposition per exact claim through the atomic requirements FR-029 and FR-030.
When either FR-029 or FR-030 is unsatisfied for a requested claim, the dispatcher SHALL refuse that claim's disposition rather than returning a weaker or inferred one.

## Inputs

The exact conformance report produced by FR-004 plus ordered requests naming one
`ix://agent-ix/quire-specification/FR-290` claim kind each —
`global-conformance`, `monitorability`, `local-projection`, `refinement`,
`realizability` or `composition` —
with admitted fragments, closed backend selections, assumptions, budgets, and
source/derived-artifact identities. Production callers SHALL NOT supply an
arbitrary backend implementation or callback.

## Outputs

One source-bound disposition per requested claim, including explicit unsupported
or failed results and preserved independent supported results. Every disposition
retains its exact admitted fragment, backend, assumptions, communication and
visibility premises, source identities, and derived-artifact correspondence.

## Behavior

This requirement is the aggregate. It establishes no obligation of its own beyond
composition: each obligation it previously stated in prose is now stated, once
and independently verifiably, by exactly one of its parts.

| Part | Obligation it owns |
| --- | --- |
| [FR-029](./FR-029-return-one-disposition-per-requested-claim.md) | One disposition per exact requested claim, no stronger-claim inference, explicit `unsupported`, and independent supported results preserved when another claim refuses, fails or is exhausted. |
| [FR-030](./FR-030-bind-each-disposition-to-its-exact-premises.md) | Every disposition stays bound to its exact fragment, backend, assumptions, premises, source identities and derived-artifact correspondence; a mismatch refuses that disposition, and a failed selected backend invokes no fallback. |

The two are separate requirements because their failures are separate failures. A
dispatcher can keep claims independent while losing the premises a disposition
was derived under, and it can retain every premise while inferring one claim's
success from another's. FR-029 answers "how many dispositions, and whose"; FR-030
answers "derived under what, and refused when that changes".

The rule that finite conformance stays available when projection is unsupported
is FR-029's; the rule that a failed backend is never replaced by an unreviewed
fallback is FR-030's. Neither is restated here.

The dispatcher SHALL derive finite conformance directly from the supplied
report. It SHALL invoke only a reviewed built-in backend selected by the closed
capability table. A claim without such a backend is `unsupported`; a test double
or caller callback is not a capability. Dispositions preserve request order and
are never sorted by their result or claim identities.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-005-AC-1 | Claim independence over the whole dispatch is exactly what [FR-029](./FR-029-return-one-disposition-per-requested-claim.md) establishes: a monitorable but unprojectable protocol retains its finite conformance result and an independent projection refusal, and no aggregate path derives one claim's disposition from another's. | Test (TC-005) |
| FR-005-AC-2 | Non-inference over the whole dispatch is exactly what [FR-029](./FR-029-return-one-disposition-per-requested-claim.md)-AC-2 enforces: satisfiable-but-unrealizable and circular-assumption cases cannot produce realizability or composition success. | Test (TC-005) |
| FR-005-AC-3 | Unsupported handling over the whole dispatch is exactly what [FR-029](./FR-029-return-one-disposition-per-requested-claim.md)-AC-3 enforces: unknown or unimplemented advanced claims return unsupported with exact premises and cannot erase a supported result. | Test (TC-005) |
| FR-005-AC-4 | Backend failure over the whole dispatch is what [FR-029](./FR-029-return-one-disposition-per-requested-claim.md)-AC-4 and [FR-030](./FR-030-bind-each-disposition-to-its-exact-premises.md)-AC-3 jointly enforce: a selected built-in backend failure returns failed for only that claim and invokes no fallback, while an absent built-in backend is unsupported and cannot be replaced by a caller callback. | Test (TC-005) |
| FR-005-AC-5 | Premise binding over the whole dispatch is exactly what [FR-030](./FR-030-bind-each-disposition-to-its-exact-premises.md) enforces: every claim disposition retains its exact admitted fragment, backend, assumptions, communication/visibility/environment premises and source/derived-artifact correspondence, and removing or substituting any one refuses that disposition without changing independent results. | Test (TC-005) |

## Adopted vocabulary

Every noun below is typed upstream and adopted by reference. Each part carries
the slice of this table it actually uses; the table is retained here as the
shared mapping the decomposition was derived from, not as a second definition.

| Noun | Upstream object | Identity kind |
| --- | --- | --- |
| Assessment-execution disposition | `FR-241-assessment-execution-disposition` | value |
| Assumption state | `FR-280-assumption-state` | value |
| Claim strength | `FR-281-claim-strength` | value |
| Derived portable projection result | `FR-249-derived-projection-result` | `resultId` per FR-062 |

The two disposition words this requirement's Outputs use literally,
`unsupported` and `failed`, are `FR-241` members, and their upstream
descriptions already carry this requirement's rules: one unsupported request
does not remove an independent supported result, and a failed selected backend
fails for that exact claim without invoking a fallback or changing independent
results. The Outputs list is therefore a reference, not a local enumeration.

### Accepted owner rulings

`ix://agent-ix/quire-specification/FR-290`'s **Vocabulary authority** section,
landed by `agent-ix/quire-specification#116`, is the accepted authority for the
six capability kinds this requirement dispatches; Protocol's advertised
claim-kind vocabulary is exactly those six labels and it authors none of its own.
QSpec #29/#31/#40 fixes FR-290 claim kinds, FR-281 strength/cardinality, FR-291
admitted fragments, FR-292 analysis backends, FR-217 role internality,
FR-210/FR-287 subject membership and FR-283 projection refusal codes. Protocol
imports those exact values. Production exposes no arbitrary backend callback;
an advanced claim without an accepted local implementation returns
`unsupported` and invokes no fallback.

## Dependencies

- [FR-004](./FR-004-assess-finite-global-conformance.md).
- `ix://agent-ix/quire-specification/FR-060`.
- Parts: [FR-029](./FR-029-return-one-disposition-per-requested-claim.md),
  [FR-030](./FR-030-bind-each-disposition-to-its-exact-premises.md).
