---
id: FR-025
title: "Assess compensation without performing I/O"
type: FR
relationships:
  - target: ix://agent-ix/quire-protocol/FR-003
    type: part_of
  - target: ix://agent-ix/quire-protocol/FR-010
    type: depends_on
  - target: ix://agent-ix/quire-specification/FR-055
    type: references
  - target: ix://agent-ix/quire-specification/FR-056
    type: references
  - target: ix://agent-ix/quire-specification/FR-057
    type: references
---
# FR-025: Assess compensation without performing I/O

## Description

When compensation state is assessed, the library SHALL perform no network, storage, telemetry, business-operation or compensation-operation input or output on any path, including refusing and incomplete paths.
When the evaluator API or its dependency inventory is mutated to admit one network, storage, telemetry, business-operation or compensation-operation capability, the purity boundary SHALL reject that mutation before any assessment runs.

## Inputs

The compensation assessment surface of
[FR-019](./FR-019-derive-obligation-activation-state.md) through
[FR-024](./FR-024-decide-recovery-only-by-authored-relation.md), action-capable
business-operation and compensation-operation doubles with call counters, and the
declared dependency inventory of the assessment path.

## Outputs

Zero-valued call counters on every action-capable double, and one rejection per
admitted I/O capability naming the capability and the surface that admitted it.

## Behavior

The prohibition is upstream's and is adopted, not invented. `FR-055` Outputs
states that "no business action is executed"; `FR-056` Outputs states that the
evaluator "does not invoke the compensating operation"; `FR-057` states that the
evaluator "SHALL NOT infer recovery success from compensation-operation
success", which presupposes that the operation is not invoked here.

The obligation is stated over the assessment surface as a whole rather than per
child requirement, because an I/O capability admitted anywhere on the path
defeats it everywhere on the path. Each of FR-019 through FR-024 additionally
carries its own no-invocation criterion where its own prose names an action.

Engineering Assurance owns whole-process containment; this requirement covers
only the library's own API surface and declared dependency inventory, as
`ix://agent-ix/quire-protocol/NFR-002` already scopes for resource enforcement.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-025-AC-1 | Every action-capable business-operation and compensation-operation double reports a zero call count after a full compensation assessment, including assessments that refuse or return incomplete. | Test (TC-003, TC-009) |
| FR-025-AC-2 | Mutating the evaluator API to admit one network, storage, telemetry, business-operation or compensation-operation capability is rejected before assessment, and the rejection names that capability. | Test (TC-003, TC-009) |
| FR-025-AC-3 | Mutating the declared dependency inventory to admit one such capability is rejected before assessment independently of the API mutation. | Test (TC-003, TC-009) |

## Adopted vocabulary

This requirement defines no protocol noun. It constrains the execution of the
requirements that do, and names no subject upstream types.

### Where the upstream vocabulary does not yet reach

No upstream artifact enumerates the capability axes this requirement rejects.
The five axes named here — network, storage, telemetry, business operation and
compensation operation — are FR-003's own existing list, carried forward
unchanged; they are this repository's consumer-side enumeration, not an adopted
upstream vocabulary, and FR-003-AC-7 and TC-003 already state them in exactly
these terms.

## Dependencies

- [FR-003](./FR-003-assess-compensation-state.md) aggregates this requirement.
- [FR-010](./FR-010-consumer-boundary-domain.md) bounds what this repository owns
  versus what an external producer supplies.
- `ix://agent-ix/quire-specification/FR-055`, `FR-056` and `FR-057` establish the
  no-business-action and no-compensating-operation rules this requirement
  enforces as a boundary.
