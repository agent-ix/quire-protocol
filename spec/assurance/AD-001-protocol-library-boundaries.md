---
id: AD-001
title: Protocol library ownership and data-flow boundaries
type: ArchitectureDescription
status: proposed
owner: agent-ix-protocol-owner
system: quire-protocol Rust library
relationships:
  - target: ix://agent-ix/quire-protocol/ADR-004
    type: references
  - target: ix://agent-ix/quire-protocol/ADR-005
    type: references
  - target: ix://agent-ix/quire-protocol/ADR-007
    type: references
  - target: ix://agent-ix/quire-protocol/FR-001
    type: realizes
  - target: ix://agent-ix/quire-protocol/FR-002
    type: realizes
  - target: ix://agent-ix/quire-protocol/FR-003
    type: realizes
  - target: ix://agent-ix/quire-protocol/FR-004
    type: realizes
  - target: ix://agent-ix/quire-protocol/FR-005
    type: realizes
  - target: ix://agent-ix/quire-protocol/FR-006
    type: realizes
  - target: ix://agent-ix/quire-protocol/FR-007
    type: realizes
  - target: ix://agent-ix/quire-protocol/FR-008
    type: realizes
  - target: ix://agent-ix/quire-protocol/FR-009
    type: realizes
---
# Protocol library ownership and data-flow boundaries

## System Boundary

`quire-protocol` begins at resolution of its external producers through one
version-lock manifest, then at strict consumption of a compiled, versioned native
protocol artifact and typed observation/temporal facts. It ends after producing
canonical protocol results and optionally deriving E02 results. It owns pure
domain validation and evaluation. Native grammar/compiler, temporal evaluation,
observation collection/correlation, business execution, generic process
execution, evidence retention, verification assessment, and human disposition
remain outside.

### Qualification-evidence intake

Indexing which externally produced artifacts a scenario consumed, at which
pinned revision and digest, is inside the boundary; producing, canonicalizing,
storing, or assessing those artifacts is outside it. This repository therefore
owns a version-lock manifest and a per-scenario evidence record (FR-009,
NFR-005) while owning no producer schema, canonicalizer, evidence store,
catalogue, sufficiency decision, or disposition path. A producer-emitted sidecar
is never an intake source: an offer that supplies its own acceptance context
authorizes itself.

## Views

The planned Rust crate separates eight pure boundaries: profile admission,
representation linking, compensation state, finite conformance, claim dispatch,
canonical result production/reading, E02 projection, and typed refusals. Immutable typed values
flow in that order. Each boundary accepts explicit budgets and returns a value or
typed error; no boundary reads the filesystem, spawns a process, accesses a
network, retains evidence, or invokes a business action.

Manifest resolution is subject to the same containment: it reads no
version-control history, spawns no process and accesses no network, and treats
the immutability and publication of a revision as an assertion its owner
records rather than a property this library checks.

"Retains evidence" is the prohibition. Composing a per-scenario record and
handing it to Quoin is not retention: the record is produced, passed, and not
kept. A boundary that stored such a record, indexed it, or answered questions
from it would be retaining evidence and is forbidden.

The version-lock manifest resolves every external producer before any producer
value is read, and no producer value enters a boundary whose entry is not
`pinned`. `FR-009-lock-external-producer-revisions` admits exactly two statuses,
`pinned` and `unresolved`; an absent entry, a digest mismatch, or a failure to
resolve within the resolution budget all yield `unresolved`, and there is no
partial pin. An earlier revision of this paragraph listed "absent, unpinned,
provisional, or digest-mismatched" as four separate conditions, which named two
statuses the two-status model abolished. `SR-047` FND-008 was closed on the
ground that the `provisional` vocabulary had been replaced -- it had been, in
FR-009, but the word survived here until this correction.
`ADR-009-two-status-manifest-resolution` records the decision. The native compiler supplies
representation artifacts. Observation and temporal
producers supply bound facts. Engineering Assurance invokes configured producers
and transports typed execution results. Quoin retains evidence.
quire-verification strictly reads and assesses E02. Artifact references and
digests join these components without merging their authority.

## Decisions

- Create a separate private Rust repository because every existing crate has a
  narrower authority and combining producer with consumer would weaken independent evidence.
- Implement the reviewed finite replay/conformance slice before any advanced
  projection, realizability, refinement, or composition algorithm.
- Keep canonical protocol results richer than E02 and implement only the reviewed
  total source-bound projection.
- Use RFC 8785 JCS for the preimage of every digest and semantic identity this
  repository *produces*, and for no other preimage. A digest over bytes a
  producer delivered is taken over those exact bytes with no canonicalization,
  re-encoding, or normalization of any kind, because canonicalizing a producer's
  bytes before digesting them would make this repository the authority on their
  spelling. An earlier revision of this bullet read "for every digest or
  semantic identity preimage", which contradicted
  `FR-009-lock-external-producer-revisions` after that requirement was corrected
  under `SR-048` FND-002; the bullet was not corrected at the same time.
  `ADR-006-confine-jcs-to-the-protocol-result-domain` records the decision and
  the scope.
- Admit bare JSON numbers only within the exact interoperable integer interval,
  and require contract-defined canonical non-number encodings for wider integers
  and exact rationals. Producer-owned digest domains remain distinct typed
  inputs, selected from the upstream vocabulary under
  `FR-015-digest-domain-selection` and never re-spelled locally.
- Reuse Engineering Assurance and Quoin rather than adding execution or evidence infrastructure.
- Select no external choreography library until E03 admits an exact candidate.

## Risks

- The composed source remains a working draft; A owns acceptance and the coherent baseline freeze.
- The current composed proposal defines semantic profiles but no accepted
  versioned compiled-protocol artifact interface; admission cannot parse prose or source.
- F's current result consumer omits canonical progress, closure, settlement, and support facts.
- Engineering Assurance issue 34 has not published the required Rust producer boundary.
- `AGPL-3.0-or-later` is selected; every later dependency and retained artifact
  still requires exact license, provenance, and compatibility review before use.
- Performance budgets and the production corpus are owner inputs and cannot be invented.
