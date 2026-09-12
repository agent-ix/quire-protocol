---
id: IT-003
title: "Publish typed assurance and E02 results"
type: IT
relationships:
  - target: ix://agent-ix/quire-protocol/FR-006
    type: verifies
  - target: ix://agent-ix/quire-protocol/FR-007
    type: verifies
  - target: ix://agent-ix/quire-protocol/NFR-003
    type: verifies
  - target: ix://agent-ix/quire-protocol/FR-009
    type: verifies
  - target: ix://agent-ix/quire-protocol/NFR-002
    type: verifies
  - target: ix://agent-ix/quire-protocol/NFR-005
    type: verifies
---
# IT-003: Publish typed assurance and E02 results

## Objective

Verify canonical protocol results and derived E02 results through the accepted
Engineering Assurance transport and Quoin evidence boundary without a local
runner, transcript parser, evidence store, or sufficiency decision.

## Target Integration

The protocol library produces canonical and E02 result values. Engineering
Assurance supplies bounded execution context and typed transport;
quire-verification supplies the E02 strict reader; Quoin retains evidence.

### Owner boundaries

| Owner | Supplies | This repository must not |
| --- | --- | --- |
| A (compiled protocol) | The admitted package the canonical result derives from, at one explicit contract version | Substitute one contract version's result for another's |
| Engineering Assurance | Bounded execution context, typed request/result transport, run and budget identity | Spawn a process, parse stdout, or decide a verdict from transcript text |
| quire-verification (E02) | The strict E02 reader and its finding vocabulary | Author a second reader, relax a finding, or declare sufficiency |
| Quoin | Retained evidence, provenance and limitations | Add a local evidence store, catalogue, or human-disposition path |
| This repository | Canonical protocol results, the total source-bound E02 projection, typed refusals | — |

Every value crossing a row is consumed with its owner retained. A value this
repository derives is never attributed to a producer, and a producer value is
never attributed to this repository.

Provenance divides: Quoin owns the retention of provenance after handoff, and
this repository owns producing the per-scenario record it hands over. Evidence
retention is external. This repository owns what each scenario must carry up to
handoff (NFR-005), never where it is kept afterwards or whether it suffices.

## Preconditions

Every external producer consumed by this integration resolves through the FR-009
version-lock manifest before any request is submitted. The A-owned `/2`
compiled-protocol entry is explicitly unresolved: implemented and under final
integration, not yet published to a remote, so it cannot be pinned. A canonical
result derived from a `/2` admission is therefore out of scope here; SC-01
through SC-09 run on `/1`-derived canonical results only, and no scenario may be
reported as executed by substituting one contract version for the other.

This integration requires four external acceptances. **None holds today**, so
every scenario below is unexecutable and the entries are `unresolved`:

| Required acceptance | Actual state |
| --- | --- |
| Engineering Assurance #34 publishes an accepted Rust request/result interface | `producer_execution` is merged on `origin/main` but `publish = false`; no release contains it, and the cited acceptance revision is unreachable after its PR was squashed |
| The exact E02 revision is accepted | `quire-verification` has zero tags and zero releases; its own interface decision declares the E02 interfaces provisional across repositories, and the code sits on a branch |
| Quoin's evidence interface is pinned | Quoin exposes no Rust-callable boundary — it is TypeScript with no `Cargo.toml` |
| The Quoin containment contract `agent-ix/quire-research#62` is accepted | Open, with every acceptance box unchecked |

Stating these as satisfied preconditions would make the integration look ready
when it is not. They are gates, and FR-009's entry-status table is their record.

## Inputs

Completed, refused, unsupported, exhausted, failed, pending, inadequate,
missing-participant, disagreeing-oracle, malformed-response, and finding-loss cases.

## Test Procedure

1. Submit exact typed producer requests through Engineering Assurance.
   - IT-003-SC-01: request, run, budget, executable, environment, and dependency identities survive transport.
2. Return canonical results through the typed result boundary.
   - IT-003-SC-02: non-success states remain distinct and no transcript text determines a verdict.
3. Project accepted canonical results to E02 and invoke its strict reader.
   - IT-003-SC-03: every mapped finding and exact canonical source reference survives.
4. Retain the evidence through Quoin.
   - IT-003-SC-04: provenance and limitations remain exact, Quoin stays external
     behind the accepted structured interface, and no local store or Quoin runtime
     dependency is created.
5. Exercise malformed, non-completed, and finding-loss paths.
   - IT-003-SC-05: each refuses or remains incomplete without an E02 success or sufficiency decision.

6. Submit a request whose typed result is incomplete rather than refused, and
   one that is refused, through the same boundary.
   - IT-003-SC-06: the three senses of "incomplete" stay separate and none is
     reported as another. NFR-002's resource-incomplete outcome, returned when a
     budget dimension is exhausted, is distinct from FR-008's typed refusal,
     which is returned when an input is invalid, and both are distinct from a
     completed assessment whose finding set is legitimately empty. Neither
     non-success yields an E02 success, and each carries the reason it did not
     complete rather than an empty finding set that reads as clean.
7. Grant a budget exactly sufficient for one unchanged request on each declared
   limit dimension, then one unit short, then resubmit the same request under a
   fresh budget at the exact value.
   - IT-003-SC-07: the exactly sufficient budget completes, the one-short budget
     returns that dimension's typed resource-incomplete outcome — NFR-002's
     outcome kind, not a refusal — with no E02 projection emitted, and the
     fresh-budget resubmission completes, proving no budget is carried across an
     exhausted attempt.
8. Inspect what each scenario retained, one scenario at a time.
   - IT-003-SC-08a: each scenario retains the resolution state it ran against —
     the full resolved manifest where resolution completed, or the partial state
     plus the causing entry where it refused during resolution — the owner of
     each consumed value, its typed outcome, and the budget granted and consumed
     on every declared dimension. A scenario that cannot retain all of these is
     reported unexecuted rather than passing.
   - IT-003-SC-08b: across the whole integration, no two distinct refusal axes
     retain the same typed outcome identity; two scenarios exercising one axis
     may share one. This is an integration-level invariant checked once over
     every scenario's retained outcome.
9. Remove, unpin, repoint, or duplicate one manifest entry at a time. The
   entries this integration requires are declared here, in this test, and not in
   the manifest.
   - IT-003-SC-09: an absent, unpinned or digest-mismatched required entry
     refuses the dependent scenario before any request is submitted, naming that
     entry's (owner, contract version) key; two entries sharing one key refuse
     every scenario before resolution; a scenario requiring an `unresolved` entry
     is reported unexecutable; removing a producer-emitted sidecar changes no
     resolution outcome; editing this test's required-entry list is the only way
     to change what a scenario requires; and repinning one entry changes the
     resolution of no other. The Engineering Assurance seam, the E02 reader and
     the Quoin interface are pinned by revision with no digest, so their entries
     exercise the absent, unpinned, duplicate and unresolved axes only.

## Expected Results

Typed producer context, canonical result meaning, E02 findings, and evidence
provenance remain separately owned and exactly linked across the whole boundary.
An incomplete outcome never reads as a clean one, every outcome is attributable
to one pinned producer revision, and a scenario whose evidence is incomplete
claims nothing.

## Metadata

- Priority: P0
- Target Integration: protocol library, Engineering Assurance, E02, and Quoin
- Automation: Automated after Engineering Assurance issue 34 is accepted

## Dependencies

Engineering Assurance issue 34, quire-verification FR-007/E02, Quoin evidence,
and `agent-ix/quire-research#62` containment interfaces must be accepted at exact
revisions. A's compiled-protocol contract is a fifth gate: this integration
consumes canonical results derived from an admitted package, so an unresolved A
entry leaves those scenarios unexecutable regardless of the other four. FR-009's
gate order and entry-status table are normative for all five.

## Notes

The absent Engineering Assurance producer blocks execution but does not authorize
a repository-local substitute.

This integration is the last in the chain and depends on IT-001 and IT-002
completing. It therefore inherits their unresolved entries: until A's `/2`
contract is published and pinned, no scenario here may claim coverage of a
`/2`-derived canonical result. Closing IT-003 establishes no PT02 result, no F
observation adequacy, no replay agreement and no campaign completion.

## Traceability

Verifies FR-006, FR-007 and NFR-003. SC-06 additionally verifies that NFR-002's
resource-incomplete outcome, FR-008's typed refusal, and a completed assessment
with an empty finding set stay separate; SC-07 verifies NFR-002; SC-08a and
SC-08b verify NFR-005; SC-09 verifies FR-009.
