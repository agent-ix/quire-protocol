---
id: ADR-002
title: "Version-explicit entry points, never version detection"
type: ADR
status: accepted
relationships:
  - target: ix://agent-ix/quire-protocol/FR-001
    type: relates_to
  - target: ix://agent-ix/quire-protocol/IT-001
    type: relates_to
  - target: ix://agent-ix/quire-protocol/FR-013
    type: relates_to
  - target: ix://agent-ix/quire-protocol/FR-009
    type: depends_on
---
# ADR-002: Version-explicit entry points, never version detection

## Status

Accepted. The version 1 half is implemented; the version 2 half is specified and
unexecutable, because A has published no revision to pin (see Consequences).

## Context

The A-owned native compiler publishes more than one compiled-protocol artifact
contract. version 1 carries `WIRE`-level identity `quire.compiled-protocol/1`; version 2
carries `MEDIA = "application/vnd.quire.compiled-protocol+json;version=2"` and
`SCHEMA = "quire.compiled-protocol.schema/2"`, verified by B directly in A's
local worktree at `agent-a/66-v2-impl` `d03e4ca` and recorded in
[IT-001](../integration/IT-001-consume-native-protocol-artifact.md)'s
Dependencies.

Two contracts and one consumer create an obvious convenience: read a few bytes,
decide which contract this is, dispatch. That convenience was rejected.

The reason is that this repository's entire value is that its selection is
independent of the offer. [FR-001](../functional/FR-001-admit-selected-protocol-profile.md)
requires admission only when the submitted artifact's identities "exactly match
one immutable protocol profile frozen by an accepted coherent native Quire
baseline". A detector inverts that: the offered bytes would choose the contract
under which they are judged, and therefore choose their own acceptance context.
That is the same self-authorization shape
[FR-009](../functional/FR-009-lock-external-producer-revisions.md) forbids for
sidecars, one level down — not at the level of *which values are expected* but
at the level of *which contract is in force*.

A detector also destroys the only property that makes version 1 evidence durable. If
dispatch is inferred, adding version 2 changes what version 1 bytes mean, and every version 1
result already recorded becomes conditional on the detector's behaviour at the
revision that produced it.

### The defect that fixed the wording

The decision itself was not reached by defect; the *falsifiability* of it was.
An earlier IT-001-SC-06 carried the clause that `admit_and_link` "keeps its version 1
behaviour unchanged". SR-044 FND-003 recorded that "that last clause names no
observable and no baseline, so the scenario cannot fail for a stated reason"
([spec/reviews/v2-intake/ears-conformance.md](../reviews/v2-intake/ears-conformance.md)).
An unfalsifiable no-regression clause is worse than none: it reads as a
guarantee and discharges nothing. IT-001-SC-06a exists because of that finding.
It names the observable — re-run SC-01 through SC-05 on the same version 1 fixture at
the revision that adds version 2, obtain the same admitted identities and the same
typed refusal per axis as at the revision before it, with `admit_and_link`'s
signature and refusal set unchanged.

## Decision

Each published compiled-artifact contract version is admitted only through its
own explicit entry point. `admit_and_link` consumes version 1. A separate
`admit_and_link_v2` consumes version 2. The library infers no contract version from
offered bytes, and it does not upgrade, downgrade, repair or substitute one
version's artifact for another's. Bytes offered at an entry point other than
their own contract version's return a typed refusal, and the two versions
mutually refuse.

Publishing a later version leaves every already-published version's admission
behaviour unchanged. version 1 is never weakened to accommodate version 2: where version 2
needs a wider expectation, it composes version 1's rather than relaxing it —
`v2::Expected` is `{ inherited: protocol_artifact::Expected, temporal:
&[ExpectedTemporal] }`, so the axes version 1 already discriminates stay
discriminated by version 1's own code.

This is recorded in FR-001's Behavior, in FR-001-AC-5, and in IT-001-SC-06 and
IT-001-SC-06a.

## Consequences

**The cost is paid in surface area, and it grows linearly.** Every future
contract version adds a public entry point, a fixture lane, a mutation corpus
and a cross-version mutual-refusal pair. Nothing amortizes. A caller holding
bytes of unknown provenance gets no help from this library at all: it must know
which contract it holds before it can call anything, and if it does not know,
the library's answer is a refusal rather than a best effort. That is the
intended answer, and it is inconvenient.

**Both cross-version refusal directions are unexecuted.** IT-001-SC-06 requires
version 1 bytes at `admit_and_link_v2` and version 2 bytes at `admit_and_link` each to
return a typed refusal. `admit_and_link_v2` does not exist in this repository's
`src/` today, and the version 2 reader is present only in a local worktree. Under
FR-009 the version 2 entry is `unresolved`, so SC-06 through SC-09 assert nothing
about A's behaviour and remain unexecuted. This ADR records a decision, not a
passing test.

**SC-06a is a two-revision comparison, which no single run can produce.** It
requires the same fixture at the revision before version 2 and at the revision that
adds it. That is an obligation on how the version 2 change is landed, not a test that
can be written once and left. It is unexecuted for the same reason as SC-06.

**Cross-version refusals must be distinguishable from each other and from
everything else.** SC-06 states that each refusal is distinguishable from the
others by its typed identity and that one shared cause across two axes does not
discharge the scenario. [ADR-008](./ADR-008-refusal-identity-injectivity.md)
carries that obligation; without it, "wrong entry point" could collapse into the
same code as eleven selection mismatches and this decision would be untestable.

## Alternatives considered

**Sniff the wire/media/schema identity and dispatch.** Rejected. The offer would
select its own contract, which is the self-authorization FR-009 forbids, and it
would make version 1's meaning depend on the set of versions compiled in.

**One entry point taking an explicit caller-supplied version selector.**
Rejected, but it is the closest alternative and the rejection is a judgement
rather than a derivation. The selector is caller-supplied, so it authorizes
nothing the offer controls, and it would hold the surface at one function. It
was rejected because the selector's value space then has to be validated at run
time against a set the type system already knows, and because a shared body
behind the selector is exactly where version 1 gets weakened to accommodate version 2 —
one shared `Expected` with optional temporal fields, rather than version 2 composing
version 1. Separate entry points make that pressure a compile error.

**Upgrade version 1 bytes into version 2 and admit once.** Rejected outright. An upgrade
path would have this repository derive a version 2 artifact from version 1 bytes, which is
producing a compiled artifact — A's authority, not B's — and would require
knowledge of version 2's construction rules that this repository has no business
holding.
