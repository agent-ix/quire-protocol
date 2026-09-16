---
id: IT-002
title: "Consume observation and temporal facts"
type: IT
relationships:
  - target: ix://agent-ix/quire-protocol/FR-003
    type: verifies
  - target: ix://agent-ix/quire-protocol/FR-004
    type: verifies
  - target: ix://agent-ix/quire-protocol/FR-006
    type: verifies
  - target: ix://agent-ix/quire-protocol/FR-009
    type: verifies
  - target: ix://agent-ix/quire-protocol/NFR-002
    type: verifies
  - target: ix://agent-ix/quire-protocol/NFR-005
    type: verifies
---
# IT-002: Consume observation and temporal facts

## Objective

Verify QObs-owned observation, decision-scope progress/closure and surrounding-
execution progress/closure facts plus QSL-owned native-temporal results and the
Protocol-owned complete-global-conformance closure record through compensation, finite
conformance, and canonical result production without duplicating producer
semantics.

## Target Integration

QObs publishes the accepted typed Rust runtime facts, authority bundles and
bounded lineage at revision
`924006300f45b38483be1cbdf99b68f899b7d368`. Protocol consumes the owner crate's
contract constants and immutable schema bytes directly; it does not restate an
owner payload shape or validation rule. QSL first published the native-temporal
request/result and checked activation-guard selection at `f1700a9`; Protocol's
current exact QSL pin `2736b62b2f741496590d099d71478b3044d638d6` also publishes the
native-temporal v2 opaque-trigger and activation-fact interface, plus the
source-authorized strict `/3` activation-mapping producer and reader Protocol
uses to validate FR-300 bindings. Protocol invokes no temporal
evaluator and does not consume a QCI join result; QCI consumes Protocol's result
map downstream.

The consumer is the protocol evaluator and canonical result producer. Its
closed interface inventory contains the fifteen QObs entries below, four QSL
native-temporal request/result entries, the consumed compiled-protocol `/2`
interface, and the strict activation-mapping `/3` interface: twenty-one entries
selected independently by the integration test.

### Owner boundaries

| Owner | Supplies | This repository must not |
| --- | --- | --- |
| QObs | Observation facts, decision-scope and surrounding-execution progress/closure, settlement basis, decision support, completeness, partial facts, activation authority, revisioned authority bundles and bounded lineage | Collect, correlate, re-derive or repair an observation; infer a missing fact; execute QObs repair or aggregate-query semantics without an accepted Protocol adapter |
| QSL native temporal | Exact checked native-temporal results, activation-guard selection, opaque trigger, captures and evaluation anchor | Evaluate a temporal formula, select a QObs subject or Protocol control, or author a second profile |
| QSL compiled protocol | The admitted `/2` definition from which Protocol builds the linked graph | Parse native source or reinterpret an identity |
| This repository | Compensation state, finite conformance, claim dispatch, canonical result production, typed refusals, version-lock resolution, the complete-global-conformance closure record and its premises, the scenario evidence record up to handoff | — |

Every value crossing a row is consumed with its owner retained. Protocol alone
derives the FR-300 binding from one linked control, QSL's strict v2 request/result
facts and QObs's qualified subject; it never attributes that join to either
producer or accepts a caller-built equivalent. A value this repository derives
is never attributed to a producer, and a producer value is never attributed to
this repository.

Protocol's accepted QObs interface inventory additionally recognizes these
owner contracts and schema digests at the exact accepted revision:

| Owner contract | Schema SHA-256 |
| --- | --- |
| `quire.observation.partial-fact/v1` | `9dc068376a7db5a7509810910c36e2efff6b9c3701d83c474701ec572f13f23c` |
| `quire.observation.activation-scope-authority/v1` | `c17ad8b3ec10c48e535bc3a762738b979206c01af20c31775d8a9a9e0a97fab6` |
| `quire.observation-authority/v1` | `9614d8077bd1e667507b2107eab08e0f7934337115ce3e10c8c55b5d5538cdab` |
| `quire.observation-authority/v2` | `f92335c5e5e1e3434f1c2484557ca1f512c22b496eb97319c94fcac3491f116a` |
| `quire.observation-authority-lineage/v1` | `d58c70516e53bde40ab2d276df143fffb05740348b98d09a71e89857a82a22ef` |

These entries authorize only the published Rust interfaces. QObs repair and
aggregate-query are consumer capabilities, not wire contracts. Until Protocol
implements adapters for them, its closed capability inventory reports each as
a distinct typed `unsupported` value and creates no invented contract key,
reader, or semantic substitute.

The complete-global-conformance closure record is Protocol-authored, not an
external producer's. It therefore takes no version-lock entry. SC-03 and SC-04
verify that a scoped closure cannot substitute for it, not that it arrived from
QObs, QSL, QCI or a research artifact.

## Preconditions

Every external contract consumed by this integration resolves through FR-009
before any fact is read. QSL's native-temporal v1 interface originated at
`f1700a9`; Protocol consumes it, the compiled `/2` dependency and the
opaque-trigger v2 entries from current exact pin `2736b62`. QObs runtime and authority interfaces are pinned at
`924006300f45b38483be1cbdf99b68f899b7d368`. Every scenario resolves the exact
interface set it consumes before it receives an owner value.

The QObs consumer preserves decision-scope and surrounding-execution
progress/closure, settlement basis, decision support, completeness, and exact
producer identities. Protocol derives complete-global-conformance closure from
the linked graph, accepted QObs/QSL facts and exact execution, branch and
workflow premises.

## Inputs

Complete and open valid histories, complete valid FR-300 activation bindings, a decisive witness, a decisive
counterexample, a non-decisive prefix, missing support, each
complete-global-conformance closure state and premise set, changed closure,
mis-correlated observation, duplicate delivery, and late contradiction.

## Test Procedure

1. Submit complete and open histories through the accepted typed interfaces.
   - IT-002-SC-01: producer facts retain their exact identities and authorities.
2. Assess compensation and finite conformance.
   - IT-002-SC-02: every outcome follows only the selected protocol and temporal premises; each event-triggered obligation reaches Protocol through one complete FR-300 binding, while a missing, foreign, cross-control, cross-subject, cross-trigger or contradictory binding refuses and no caller-built obligation is accepted.
3. Produce canonical results for decisive and non-decisive open prefixes.
   - IT-002-SC-03: settlement, pending state, support, decision-scope
     progress/closure, surrounding-execution progress/closure, and
     complete-global-conformance closure remain distinct; independently change
     each progress/closure axis and require exact retention or typed refusal
     without cross-scope substitution.
4. Submit missing, changed, mis-correlated, duplicate, and contradictory facts.
   - IT-002-SC-04: independently omit, duplicate, cross-wire, scope-mutate, or
     identity-mutate each required global execution, branch, and workflow
     premise; each adverse input yields the exact refusal, violation,
     incompleteness, or linked supersession without changing independent truth.

5. Submit one history through batch assessment and through settled-incremental
   assessment, then permute the incremental arrival order within equal time,
   then change one authored premise and repeat both paths.
   - IT-002-SC-05a: batch replay and settled incremental assessment are
     semantically identical for every permutation of the same admitted facts,
     as FR-004-AC-4 states it: for each assessed value the two paths agree on
     settled-or-pending, and where settled, on the settled value, the
     decision-scope and surrounding-execution progress/closure, and the
     complete-global-conformance closure.
   - IT-002-SC-05b: the incremental path settles at least one value strictly
     before the final fact arrives, on a history where that value's support is
     complete at an earlier prefix. A path that buffers every fact and evaluates
     once at the end fails this scenario rather than satisfying SC-05a
     vacuously.
   - IT-002-SC-05c: permuting equal-time arrivals, under the time key the
     accepted temporal profile declares, changes no assessed value on either
     path, and the permutation set used is non-singleton.
   - IT-002-SC-05d: changing one authored premise changes the same assessed
     values to the same new values on both paths. A divergence between the two
     paths returns a typed refusal carrying its own identity, distinct from
     every other refusal in this integration, rather than reporting agreement.
6. Grant a budget exactly sufficient for one unchanged history on each declared
   limit dimension, then one unit short, then resubmit the same history under a
   fresh budget at the exact value.
   - IT-002-SC-06: the exactly sufficient budget completes, the one-short budget
     returns that dimension's typed resource-incomplete outcome — NFR-002's
     outcome kind, not a refusal — with no partial assessment retained, and the
     fresh-budget resubmission completes, proving no budget is carried across an
     exhausted attempt. Manifest resolution consumes its own declared budget
     dimensions under FR-009 before this scenario's budget applies.
7. Inspect what each scenario retained, one scenario at a time.
   - IT-002-SC-07a: each scenario retains the resolution state it ran against —
     the full resolved manifest where resolution completed, or the partial state
     plus the causing entry where it refused during resolution — the owner of
     each consumed producer value, its typed outcome, and the budget granted and
     consumed on every declared dimension. A scenario that cannot retain all of
     these is reported unexecuted rather than passing.
   - IT-002-SC-07b: across the whole integration, no two distinct refusal axes
     retain the same typed outcome identity. Two scenarios exercising one axis
     may share an identity; two axes may not. This is an integration-level
     invariant, checked once over every scenario's retained outcome, not a claim
     any single scenario makes about the others.
8. Remove, unpin, repoint, or duplicate one manifest entry at a time. The
   entries this integration requires are declared here, in this test, and not in
   the manifest.
   - IT-002-SC-08: an absent, unpinned or digest-mismatched required entry
     refuses the dependent scenario before any producer fact is read, naming that
     entry's (owner, contract version) key; two entries sharing one key refuse
     every scenario before resolution; a scenario requiring an `unresolved` entry
     is reported unexecutable rather than passing or failing; removing a
     producer-emitted sidecar changes no resolution outcome; editing this test's
     required-entry list is the only way to change what a scenario requires; and
     repinning one entry changes the resolution of no other. The inventory also
     binds each QObs contract above to the owner's exported contract constant,
     exported schema bytes and recorded schema SHA-256. Repair and aggregate
     query remain two distinct typed unsupported consumer capabilities, create
     no manifest entries, and cannot be reported as supported merely because
     the pinned owner crate exports their implementation modules.

## Expected Results

The protocol library consumes producer facts without reinterpreting them. No
missing or contradictory fact improves truth, adequacy, recovery, or
conformance, and no scoped closure substitutes for complete-global-conformance
closure. Batch and settled-incremental assessment agree on every settled value
and disagree only by refusing. Every outcome is attributable to one pinned
producer revision, and a scenario whose evidence is incomplete claims nothing.

## Metadata

- Priority: P0
- Target Integration: observation and temporal producers to protocol evaluator
- Automation: Automated against the exact accepted QObs revision

## Dependencies

QSL native-temporal v1 interface originating at `f1700a9`, the compiled-protocol
`/2` dependency and FR-053 opaque-trigger v2 interface at current exact pin
`2736b62`, and QObs
runtime/authority interfaces at `9240063`. Protocol owns global closure and
result assembly. QCI issue 71 is a downstream consumer and is not a prerequisite.

## Notes

The owner contracts and Task-010's SC-01 through SC-04 composition path are
delivered. Task-020 separately delivers SC-08's exact contract/schema
inventory and negative resolution controls. Task-021 delivers SC-05 and SC-06
with replay, evidence-correlation and resource-boundary coverage. SC-07 remains
the retained-evidence extension owned by Task-018 / issue #63.

## Traceability

Verifies FR-003, FR-004 and FR-006. SC-05a, SC-05b and SC-05d exercise
FR-041-AC-1 through FR-041-AC-4; SC-05c verifies FR-026-AC-2; together they
exercise FR-004-AC-4. SC-07b completes FR-041-AC-4's integration-wide
refusal-axis comparison. SC-06 verifies NFR-002; SC-07a and SC-07b verify
NFR-005; SC-08 verifies FR-009.
