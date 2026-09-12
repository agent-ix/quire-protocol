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
| FR-029-AC-6 | No claim strength is promoted to another, and two claims of different strength remain distinct. | Analysis |

## Adopted vocabulary

The disposition vocabulary and the non-transfer rules are upstream's. What this
requirement owns is the cardinality rule — one disposition per exact requested
claim — and the requirement that the mapping from requests to dispositions is
total and injective.

| Noun | Upstream object | Identity kind |
| --- | --- | --- |
| Assessment-execution disposition | `FR-241-assessment-execution-disposition` | value |
| Truth disposition | `FR-242-truth-disposition` | value |
| Claim strength | `FR-281-claim-strength` | value |
| Decision-support dependence relation | `FR-282-dependence-relation` | value |
| Projection refusal code | `FR-283-projection-refusal-code` | value |

`FR-241` supplies the two members this requirement's criteria name literally.
Its `unsupported` member states that "One unsupported request does not remove an
independent supported result (FR-061-AC-5)", and its `failed` member states that
the selected analysis backend failed "for this exact claim, without invoking a
fallback or changing independent results (FR-060)". FR-029-AC-3 and FR-029-AC-4
are therefore adopted, not authored here.

`FR-282` supplies the rule behind FR-029-AC-6's second half: `independent` may
not be inferred, and FR-282 records FR-061's exclusion that a strict reader
"SHALL NOT infer independence from distinct display names or implementations".
Independence between two claim dispositions is asserted by this requirement over
the dispatcher's behaviour, never read off two dispositions' spellings.

### Where the upstream vocabulary does not yet reach

- **The claim kinds are not typed.** FR-005 and upstream `FR-060` both name
  global conformance, monitorability, local projection, refinement,
  realizability and composition in prose, and no `FR-2NN` object enumerates
  them: the protocol object set types claim *strength* (`FR-281`) and no claim
  *kind*. FR-029-AC-5 counts dispositions per exact requested claim, so it needs
  an equality over claim kinds that no upstream artifact supplies. The owner is
  `agent-ix/quire-specification`. This requirement names the gap and enumerates
  no kind locally, because a locally authored claim-kind vocabulary is precisely
  the second profile `FR-001` exists to refuse.
- **Claim strength may or may not be single-valued.** `FR-281`'s open gap:
  "FR-061 is the only requirement that names these four values. No requirement
  states whether the set is closed, whether a claim may carry more than one
  strength, or what relation (if any) holds between them." FR-029-AC-6 forbids
  promotion between strengths and asserts nothing about cardinality.
- **A projection refusal's code is contested upstream.** `FR-283`'s open gap
  opens: "**Which code a recomputed-digest mismatch receives is unresolved.**"
  FR-029-AC-1 requires an independent projection refusal and does not select the
  code it carries; `FR-030` records both assignments in full.

## Dependencies

- [FR-005](./FR-005-dispatch-independent-protocol-claims.md) is the parent this
  requirement is part of.
- [FR-004](./FR-004-assess-finite-global-conformance.md) supplies the finite
  conformance result that FR-029-AC-1 requires to survive an unsupported
  projection.
- `ix://agent-ix/quire-specification/FR-060` establishes one independent
  disposition per requested capability and the non-transfer of premises and
  success among them.
