---
id: FR-029
title: "Return one disposition per requested claim"
type: FR
relationships:
  - target: ix://agent-ix/quire-protocol/FR-005
    type: part_of
  - target: ix://agent-ix/quire-protocol/US-001
    type: implements
  - target: ix://agent-ix/quire-specification/FR-060
    type: depends_on
  - target: ix://agent-ix/quire-specification/FR-241
    type: references
  - target: ix://agent-ix/quire-specification/FR-242
    type: references
  - target: ix://agent-ix/quire-specification/FR-281
    type: references
  - target: ix://agent-ix/quire-specification/FR-282
    type: references
  - target: ix://agent-ix/quire-specification/FR-283
    type: references
  - target: ix://agent-ix/quire-specification/FR-290
    type: depends_on
---
# FR-029: Return one disposition per requested claim

## Description

When one or more protocol claims are requested, the dispatcher SHALL return exactly one execution disposition per exact requested claim.

## Inputs

The exact set of requested claims, the admitted fragment and selected backend
each claim was requested against, and the per-claim premises the requester
supplied.

## Outputs

One execution disposition per requested claim, each independently readable,
including the explicit `unsupported` and `failed` dispositions.

## Behavior

The dispatcher SHALL NOT derive one claim's disposition from another claim's disposition, premises, backend or result.
The dispatcher SHALL return `unsupported` for an unknown or unimplemented requested claim.
The dispatcher SHALL leave every independently supported result unchanged when another claim returns `refused`, `unsupported`, `resource-incomplete` or `failed`.
The dispatcher SHALL NOT infer a stronger claim from a weaker one.

Independence is the whole point of the dispatcher. Finite conformance stays
available when projection is unsupported, a satisfiable finite trace establishes
neither projectability nor a causal strategy, and mutually circular local
assumptions establish neither realizability nor composition.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-029-AC-1 | A monitorable but unprojectable protocol retains its finite conformance result and an independent projection refusal. | Test (TC-005) |
| FR-029-AC-2 | Satisfiable-but-unrealizable and circular-assumption cases cannot produce realizability or composition success. | Test (TC-005) |
| FR-029-AC-3 | Unknown or unimplemented advanced claims return unsupported with exact premises and cannot erase a supported result. | Test (TC-005) |
| FR-029-AC-4 | A selected backend failure returns failed for only that claim and leaves every other requested claim's disposition unchanged. | Test (TC-005) |
| FR-029-AC-5 | Requesting n claims returns exactly n dispositions, one per exact requested claim, with no claim answered twice and none answered by another claim's result. | Test (TC-005) |
| FR-029-AC-6 | No claim strength is promoted to another, and two claims of different strength remain distinct. | Test (TC-005) |

## Adopted vocabulary

The disposition vocabulary and the non-transfer rules are upstream's. What this
requirement owns is the cardinality rule — one disposition per exact requested
claim — and the requirement that the mapping from requests to dispositions is
total and injective.

| Noun | Upstream object | Identity kind |
| --- | --- | --- |
| Assessment-execution disposition | `FR-241` | value |
| Truth disposition | `FR-242` | value |
| Claim strength | `FR-281` | value |
| Decision-support dependence relation | `FR-282` | value |
| Projection refusal code | `FR-283` | value |
| Protocol claim kind | `FR-290` | value |

`FR-241` supplies the two members this requirement's criteria name literally,
and the obligations FR-029-AC-3 and FR-029-AC-4 adopt rather than author: an
unsupported request removes no independent supported result (`FR-061-AC-5`), and
a backend failure is confined to its exact claim, invoking no fallback and
changing no independent result (`FR-060`).

`FR-282` supplies the rule behind FR-029-AC-6's second half: `independent` may
not be inferred, and under `FR-061` a strict reader never infers independence
from distinct display names or implementations. Independence between two claim
dispositions is asserted by this requirement over the dispatcher's behaviour,
never read off two dispositions' spellings.

### Acceptance state of adopted vocabularies

`ix://agent-ix/quire-specification/FR-290` is the accepted authority for the
capability-kind vocabulary: its **Vocabulary authority** section, landed by
`agent-ix/quire-specification#116`, states that its six labels
— `global-conformance`, `monitorability`, `local-projection`, `refinement`,
`realizability`, `composition` — are the single capability-kind vocabulary
across the ecosystem and that quire-protocol's advertised claim-kind vocabulary
aligns to that set. Protocol dispatches those six exact labels and authors none
of its own. Where this repository's prose names a kind in shorter form —
"conformance" for `global-conformance` and "projection" for `local-projection` —
the FR-290 label is the vocabulary and the prose word is prose; the remaining
four are spelled identically. The single-valued four-member strength that
FR-281 carries is owned by Engineering Assurance
(`ix://agent-ix/engineering-assurance/FR-022`); Protocol imports
`engineering_assurance::claim_strength::ClaimStrength` and parses a
disposition's strength through its `FromStr`, refusing any other label.
FR-283 has already resolved source
digest mismatch to `invalid-source-result` and derived/target identity mismatch
to `identity-mismatch`.

## Dependencies

- [FR-005](./FR-005-dispatch-independent-protocol-claims.md) is the parent this
  requirement is part of.
- [FR-004](./FR-004-assess-finite-global-conformance.md) supplies the finite
  conformance result that FR-029-AC-1 requires to survive an unsupported
  projection.
- `ix://agent-ix/quire-specification/FR-060` establishes one independent
  disposition per requested capability and the non-transfer of premises and
  success among them.
