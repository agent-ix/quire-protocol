---
id: FR-035
title: "Map source states to E02 or refuse"
type: FR
relationships:
  - target: ix://agent-ix/quire-protocol/FR-007
    type: part_of
  - target: ix://agent-ix/quire-protocol/FR-008
    type: depends_on
  - target: ix://agent-ix/quire-specification/FR-062
    type: depends_on
  - target: ix://agent-ix/quire-specification/FR-241
    type: references
  - target: ix://agent-ix/quire-specification/FR-242
    type: references
  - target: ix://agent-ix/quire-specification/FR-244
    type: references
  - target: ix://agent-ix/quire-specification/FR-245
    type: references
  - target: ix://agent-ix/quire-specification/FR-246
    type: references
  - target: ix://agent-ix/quire-specification/FR-248
    type: references
  - target: ix://agent-ix/quire-specification/FR-280
    type: references
  - target: ix://agent-ix/quire-specification/FR-282
    type: references
  - target: ix://agent-ix/quire-specification/FR-283
    type: references
---
# FR-035: Map source states to E02 or refuse

## Description

When each source axis of an accepted canonical result is mapped, the adapter SHALL emit the exact E02 fact the FR-062 mapping table requires for that source state.
When one source state simultaneously implies several portable findings, the adapter SHALL emit all of them.
When a condition in FR-062's refusal set holds, the adapter SHALL emit exactly one code from `FR-283-projection-refusal-code` and no `TechniqueResult`.

## Inputs

One strict-reader-accepted canonical result carrying its execution, truth,
activation, participation, assumption, observation-adequacy and
decision-support axes, the selected source and target contract revisions, and
the exact E02 evidence obligation context.

## Outputs

One E02 `TechniqueResult` whose finding axes are each populated from their own
source axis, or one refusal carrying one `FR-283` code plus the affected exact
identity and no `TechniqueResult`.

## Behavior

Each row of FR-062's mapping table is applied as stated; this requirement
restates neither the source nor the target member lists. FR-062 requires the
adapter to "preserve every independent execution, activation, observation,
participation, assumption and logical finding required by quire-verification
FR-007", and states that a source state implying several such gaps "SHALL
produce all of them after E02 assessment, regardless of the E02 compatibility
summary status".

E02 adequacy is populated from observation adequacy alone. Protocol adequacy,
completeness, decision-scope progress and closure, surrounding-execution
progress and closure, complete-global-conformance closure, settlement basis and
claim are, in FR-062's words, "Retained in the canonical protocol-result
artifact; never substituted for an E02 finding axis." The adapter reuses Quoin
evidence authority and, per FR-062, "SHALL NOT emit a sufficiency decision".

Every `satisfied` or `violated` projection derives its E02 oracle contributions
from the exact decision-support identities and preserves the source
`FR-282-dependence-relation` member and its lineage. FR-062: the "adapter SHALL
NOT invent an independent oracle relation."

A refusal carries exactly one `FR-283` member plus the affected exact identity.
An unrepresented source state carries `unrepresented-source-state` and is not
reclassified as a malformed source. No `TechniqueResult` accompanies a refusal.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-035-AC-1 | Every FR-062 source-state row maps to the exact E02 state and retains all simultaneous independent findings. | Test (TC-007) |
| FR-035-AC-2 | Several findings implied by one source state are all emitted, regardless of the E02 compatibility summary status. | Test (TC-007) |
| FR-035-AC-3 | E02 adequacy derives only from observation adequacy; protocol adequacy and sufficiency cannot cross into it. | Test (TC-007) |
| FR-035-AC-4 | Protocol adequacy, completeness, the four progress/closure axes, complete-global-conformance closure, settlement basis and claim remain in the canonical artifact and never populate an E02 finding axis. | Test (TC-007) |
| FR-035-AC-5 | Each `satisfied` or `violated` projection derives its oracle contributions from the exact decision-support identities and preserves the source `FR-282` relation and lineage, and no independent relation is invented. | Test (TC-007) |
| FR-035-AC-6 | Invalid source, unknown versions/features, identity mismatch, unrepresented source state, missing context, or finding loss returns the stable FR-062 refusal and emits no result. | Test (TC-007) |
| FR-035-AC-7 | An unrepresented source state yields `unrepresented-source-state` and is not reclassified as `invalid-source-result`. | Test (TC-007) |

## Adopted vocabulary

| Noun | Upstream object | Identity kind |
| --- | --- | --- |
| Execution disposition | `FR-241-assessment-execution-disposition` | value |
| Truth | `FR-242-truth-disposition` | value |
| Activation | `FR-244-activation-state` | value |
| Observation adequacy | `FR-245-observation-adequacy-state` | value |
| Protocol adequacy | `FR-246-protocol-adequacy-state` | value |
| Assumptions | `FR-280-assumption-state` | value |
| Decision-support record | `FR-248-decision-support-record` | value, keyed (oracle identity, observation identity, dependence relation, and the predecessor or dependence group that relation requires) |
| Dependence relation | `FR-282-dependence-relation` | value |
| Projection refusal code | `FR-283-projection-refusal-code` | value |

The E02 target vocabularies are `quire-verification`'s, carried into this
repository only through FR-062's mapping table. This requirement spells no E02
member of its own and defines no E02 finding axis.

### Where the upstream vocabulary does not yet reach

- **Which refusal code a recomputed-digest mismatch receives is unresolved, and
  two upstream artifacts assign it differently.** Both readings stand verbatim.
  `FR-249-derived-projection-result`, under `identity-mismatch`: "An exact
  context or canonical-result reference is missing or conflicting; substituting
  any identity or digest refuses the projection (FR-062, FR-062-AC-2)."
  `FR-283-projection-refusal-code`, under `invalid-source-result`: "The
  canonical protocol result did not validate, or its recomputed `sha256-jcs`
  digest did not match, before mapping was attempted (FR-062)." `FR-249`'s Open
  gap records that the two "are not reconcilable by reading" and that "No
  requirement resolves which code a recomputed-digest mismatch receives." This
  requirement obliges the mismatch to refuse and to emit no `TechniqueResult`
  ([FR-034](./FR-034-derive-e02-projection-identity.md)-AC-7); it selects no
  code. Owner: `quire-specification`, by an owner ruling and a requirement that
  names the code.
- **FR-062's mapping table is declared total over two source axes nobody
  enumerates.** FR-062 states the mapping "is total for the selected source and
  target revisions", while `FR-240-canonical-protocol-result` records that
  participation is "unenumerated at the source" and that for the four
  progress/closure axes "no requirement declares a closed vocabulary. No member
  list is invented here." `FR-249`'s Open gap draws the same conclusion:
  "Totality over an unenumerated source domain is not established here". This
  requirement applies every row the table states and asserts no totality.
  Owner: `quire-specification`.
- **The portable finding contract itself was not opened in this pass.** FR-062
  names `ix://agent-ix/quire-verification/FR-007` as the contract whose findings
  must not be lost. That repository is not available to this pass, so no member
  of its finding vocabulary is quoted or restated here, and the finding set is
  adopted only as FR-062 names it. Owner: `quire-verification`.

## Dependencies

- [FR-007](./FR-007-project-results-to-e02.md) is the parent this requirement is
  part of.
- [FR-034](./FR-034-derive-e02-projection-identity.md) supplies the derived
  identity and the retained source reference these findings are emitted with.
- [FR-008](./FR-008-return-typed-refusals.md) supplies the typed-refusal
  discipline the refusal codes are emitted under.
- `ix://agent-ix/quire-specification/FR-062` states the mapping table, the
  retention boundary and the closed refusal set.
- `ix://agent-ix/quire-verification/FR-007` owns the portable finding contract.
