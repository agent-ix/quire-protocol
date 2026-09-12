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

Verify the F/E-owned observation, decision-scope progress/closure,
surrounding-execution progress/closure, and temporal decision facts plus the
B-owned complete-global-conformance closure record through compensation, finite
conformance, and canonical result production without duplicating producer
semantics.

## Target Integration

The observation producer is a Rust interface: `quire-observation` carries
`ScopeFact` across all four progress/closure axes plus
`GlobalConformanceClosure`/`GlobalClosureState` at
`PRODUCER_INTERFACE_VERSION = "1.2.0"`. It is `publish = false` and this crate
declares no dependency on it, so there is no edge to pin yet.

The temporal producer has **no Rust interface at any state**. E's contribution is
merged prose in `quire-specification` (FR-090..FR-092, FR-095 and the
`proposals/quire-v1/definitions/temporal-*` set). FR-009 cannot pin a typed
temporal producer that does not exist, so that entry is `unresolved` and every
scenario requiring it is unexecutable.

The consumer is the protocol evaluator and canonical result producer.

### Owner boundaries

| Owner | Supplies | This repository must not |
| --- | --- | --- |
| F (observation) | Observation facts, decision-scope and surrounding-execution progress/closure, settlement basis, decision support, completeness | Collect, correlate, or re-derive an observation; infer a missing fact |
| E (temporal) | The accepted temporal profile and temporal decision facts. No Rust interface exists yet; this entry is `unresolved` | Evaluate a temporal formula or author a second profile |
| A (compiled protocol) | The linked protocol representation the facts are assessed against | Parse native source or reinterpret an identity |
| Engineering Assurance | The bounded execution context and the budget each scenario is granted | Grant itself a budget, or continue past an exhausted one |
| Quoin | Retention of the scenario evidence record after handoff | Retain that record locally, or decide whether it suffices |
| This repository | Compensation state, finite conformance, claim dispatch, canonical result production, typed refusals, version-lock resolution, the complete-global-conformance closure record and its premises, the scenario evidence record up to handoff | — |

Every value crossing a row is consumed with its owner retained. A value this
repository derives is never attributed to a producer, and a producer value is
never attributed to this repository.

The complete-global-conformance closure record is **B-authored**, not an
external producer's. It was specified in `quire-specification` through PR #22
from `agent-b/global-conformance-closure`, and `quire-research#48` (PT02) is an
open research ticket that names this repository as the intended implementation
home. It therefore takes no version-lock entry: there is no external owner to
assert anything about it. What SC-03 and SC-04 verify is that a scoped closure
cannot substitute for it, not that it arrived from elsewhere.

## Preconditions

Every external producer consumed by this integration resolves through the
FR-009 version-lock manifest before any fact is read. The A-owned `/2`
compiled-protocol entry is explicitly unresolved: that contract is implemented
and under final integration but not yet published to a remote, so it cannot be
pinned. No scenario below depends on that entry, and none may be reported as
executed by substituting the `/1` entry for it.

The observation-output consumer preserves decision-scope and surrounding
execution progress/closure, settlement basis, decision support, completeness,
and exact producer identities. PT02 adds an independently typed
complete-global-conformance closure record with exact execution, branch, and
workflow premises. The temporal profile is accepted.

## Inputs

Complete and open valid histories, a decisive witness, a decisive
counterexample, a non-decisive prefix, missing support, each
complete-global-conformance closure state and premise set, changed closure,
mis-correlated observation, duplicate delivery, and late contradiction.

## Test Procedure

1. Submit complete and open histories through the accepted typed interfaces.
   - IT-002-SC-01: producer facts retain their exact identities and authorities.
2. Assess compensation and finite conformance.
   - IT-002-SC-02: every outcome follows only the selected protocol and temporal premises.
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
     repinning one entry changes the resolution of no other.

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
- Automation: Automated after producer contracts are accepted

## Dependencies

F-owned FR-115/FR-116, TC-145/TC-146, IT-051 and the output-mapping contract plus
the E-owned temporal profile must preserve the complete canonical-result input
shape.

## Notes

Quire Specification PR #15 at
`d8d984b542fe30f65b6e07f4a07b2d1f81d3962b` now specifies the required
independent decision-scope and surrounding-execution progress/closure axes plus
the complete-global-conformance closure record and premise mutations through
**one** F consumer path: it touches FR-116 and TC-146 and leaves FR-115 and
TC-145 unchanged, so the second consumer path is not yet specified to this
shape. The content dependency is ready, while the revision
remains unaccepted until the composed baseline is reviewed and promoted. This
integration test remains planned.

## Traceability

Verifies FR-003, FR-004 and FR-006. SC-05a through SC-05d additionally verify
FR-004-AC-4; SC-06 verifies NFR-002; SC-07a and SC-07b verify NFR-005; SC-08
verifies FR-009.
