---
id: ADR-010
title: "Consume QSL's protocol analysis; own only linking, assessment and dispatch"
type: ADR
status: accepted
relationships:
  - target: ix://agent-ix/quire-protocol/AD-001
    type: relates_to
  - target: ix://agent-ix/quire-protocol/ADR-007
    type: relates_to
  - target: ix://agent-ix/quire-protocol/FR-002
    type: relates_to
  - target: ix://agent-ix/quire-protocol/FR-004
    type: relates_to
  - target: ix://agent-ix/quire-protocol/FR-005
    type: relates_to
  - target: ix://agent-ix/quire-protocol/FR-029
    type: relates_to
  - target: ix://agent-ix/quire-protocol/FR-030
    type: relates_to
  - target: ix://agent-ix/quire-protocol/FR-301
    type: relates_to
  - target: ix://agent-ix/quire-protocol/FR-302
    type: relates_to
  - target: ix://agent-ix/quire-spec-language/ADR-017
    type: depends_on
  - target: ix://agent-ix/quire-spec-language/ADR-020
    type: depends_on
  - target: ix://agent-ix/quire-spec-language/ADR-027
    type: depends_on
---
# ADR-010: Consume QSL's protocol analysis; own only linking, assessment and dispatch

## Status

Accepted. This record fixes the split between this library and QSL's
`qsl-analyze` for the dynamic, projection, realizability, composition and
refinement extension. It authors no requirement; it reserves requirement-id
blocks and names, per capability, what this library owns and what it consumes.
Capabilities QSL does not yet provide are listed in
[QSL gaps](#qsl-gaps-for-the-qsl-code-lead) and are not worked around here.

## Context

The extension tickets TL-6, TL-8, TL-9, TL-10, TL-11 and TL-12 were written
against a QSpec revision in which this library was to implement dynamic roles,
bounded queues, joins, loops, projection, realizability, assume-guarantee
composition and refinement itself. Since then QSL has taken over parts of that
work, and comments of 2026-11-10 on three of those tickets say that
realizability, projection and refinement "now live in QSL". That statement was
checked against the code instead of adopted.

### What this library contains

`src/` holds linking (`link.rs`, `checked_package.rs`), compensation
assessment (`compensation/`), closure and evidence (`closure.rs`,
`evidence.rs`), refusal, bounds, digest and version-lock code. A search of
`src/` for projection, realizability, refinement, quorum and capacity finds no
protocol-analysis code. `link.rs` exposes the checked package's queue nodes as
read-only view data; no queue is simulated. The library has no dispatch module
for the non-conformance claim kinds yet; FR-005, FR-029, FR-030 and FR-301
specify one. It depends on QSL for the checked package and for temporal
evidence (FR-043, FR-044, FR-046).

### What QSL provides, by capability

Read from QSL `main` on 2026-11-10.

| Capability | What the code has | Where |
| --- | --- | --- |
| Dynamic roles | Replicated roles with `spawn`, `retire`, lifetimes `workflow`, `scope`, `until(p)`, a `max` bound, `activation on each` with `max_live_instances` (ADR-027 RR-1..4, ST-13..15) | `qsl-eval/src/simulation/protocol/roles.rs` |
| Bounded queues | `send`, `receive`, `duplicate`, `lose` steps; literal capacity; overflow `block`, `reject`, `loss head/tail/new`; `fifo` per key and `unordered` (ADR-027 ST-3..5, PS-7) | `qsl-eval/src/simulation/protocol/channel.rs` |
| Joins | `parallel` fork/join with `all`, `any`, `quorum(n)` and `outstanding continue/cancel` (ADR-027 ST-6..8, PS-4) | `qsl-eval/src/simulation/protocol/program.rs`, `steps.rs` |
| Loops | `repeat` with iteration counters and a progress check on the body graph (ADR-027 PS-6); `invariant`/`variant` blocks are parsed and lowered into the checked package | `qsl-eval/.../program.rs`; `qsl-forms/src/protocol_control.rs`; `qsl-semantics/src/check/lowering/protocol/controls.rs` |
| Compensation in a model subject | Registration, activation, attempts, commit cutoff, `cend` (ADR-027 CP-1..5) | `qsl-eval/src/simulation/protocol/compensation.rs` |
| Missing bounds | A requires-bound pre-check before exploration | `qsl-analyze/src/model_check/check.rs` |
| Refinement | Safety half on the explicit-state product, liveness half under abstract fairness, a protocol as the abstract side, counterexample replay, simulation certificate (ADR-020; QSL FR-142..149) | `qsl-analyze/src/model_check/refine.rs`, `qsl-replay/src/refinement/protocol_system.rs` |
| Assume-guarantee composition | Planner that discharges `rely` and `liveness-dependency` slots from proved guarantees and fairness implication, and writes the discharge record (QSL FR-530) | `qsl-replay/src/planner/compose.rs` |
| Local projection, realizability | Claim-kind vocabulary (`local-projection`, `realizability`), the protocol-family applicability gate and the routing key only. No engine. A request for either kind that no engine serves settles `unsupported` with `unsupported_projection` / `unsupported-requested-capability` | `qsl-semantics/src/check/capability.rs`, `qsl-route/src/lib.rs`, `qsl-analyze/src/analyze.rs` |

The `Capability::LocalProjection`, `Refinement` and `Realizability` entries in
`qsl-analyze/src/analyze.rs` that the planner comments cite sit in the test
`protocol_kinds_apply_only_to_protocol_claims`. That test proves that the
seven protocol kinds apply to the protocol family and to no other. It is
evidence that the kinds exist and are gated, not that an engine decides them.

### Verdict on the 2026-11-10 staleness comments

- "This library has no projection, queue or refinement code": true.
- "Refinement lives in QSL": true.
- "Realizability and projection live in QSL": true of the claim vocabulary
  and routing, false of any decision procedure. QSL has none. TL-10 is
  therefore not closable as "provided by QSL"; the capability is a QSL gap.
- "Queues, joins, dynamic roles and bounded loops live in QSL": true as an
  exploration semantics over a model subject (QSL decides what a protocol can
  do), with the gaps listed below. It is not an assessment of an observed run,
  which stays this library's job (FR-004).

QSL explores a protocol as a transition system and decides claims about every
behaviour. This library assesses one admitted observation history against the
linked protocol (FR-004) and dispatches claims to the backends that decide
them (FR-005). Those are different questions, and the split follows them.

## Decision

1. **The library implements no projection, realizability, refinement or
   composition decision procedure, and no queue, role, join or loop
   exploration semantics.** It consumes QSL's. It builds no product
   construction, strategy game or fixed-point engine. This is ADR-007 and
   AD-001's boundary table applied to the extension.

2. **The library owns exactly four things for these capabilities.**
   - *Linking:* retaining each authored role, population bound, channel
     premise, capacity, overflow policy, join policy, loop declaration and
     control identity from the checked package without minting identity
     (FR-002, FR-044).
   - *Assessment:* judging an observed history against the authored role
     lifetimes, queue bounds and premises, join thresholds, loop bounds and
     compensation relations, with the progress, closure and completeness axes
     kept independent (FR-004, FR-019..028, FR-041).
   - *Dispatch:* one disposition per requested claim kind, bound to its exact
     fragment, backend, assumptions and premises, with explicit `unsupported`
     where no QSL engine serves the kind (FR-005, FR-029, FR-030).
   - *Carrying:* carrying QSL's result, refusal, counterexample and witness
     into the Protocol result unchanged, joined to the linked graph's source
     identities (FR-006, FR-302).

3. **Unsupported QSL constructs are carried, not repaired.** A QSL
   `Unsupported` cause (symbolic capacity, join predicate, a join with no
   `outstanding` rule, and the others listed below), a requires-bound outcome or
   an unserved kind becomes a typed non-success disposition that names the
   construct. The library never truncates a bound to finish, never substitutes
   a local approximation and never reports `unsupported` as false, satisfied or
   successful.

4. **FR-301's "explicit-state product construction" is QSL's.** The product
   over the concrete and abstract protocol systems is `check_refinement`
   (ADR-020 RE-1). FR-301, spec.md section 5, AD-001 view 7 and TC-147 are
   amended with this record to say that the library dispatches a refinement
   claim to QSL's refinement backend and carries its result, instead of
   building the product over the FR-002 graph.

5. **Requirement-id blocks.** Later waves allocate local FR ids in these
   blocks, one per capability family. Ids are allocated within a block
   and never across blocks.

   | Block | Capability family | Ticket |
   | --- | --- | --- |
   | FR-050 to FR-099 | Dynamic roles, bounded queues, joins, loops | TL-11 |
   | FR-100 to FR-149 | Core roles, messages, channels, control, compensation, replay | TL-6 |
   | FR-150 to FR-199 | Projection, realizability | TL-10 |
   | FR-200 to FR-249 | Assume-guarantee composition, refinement | TL-9 |

   FR-301 and FR-302 stay where they are. The FR-050 to FR-054 ids that
   artifacts cite as "source requirements" are QSpec ids, written
   `ix://agent-ix/quire-specification/FR-0nn`; they are not local ids and do
   not collide with these blocks.

## Per-ticket ownership

| Ticket | Provided by QSL | Remains Protocol-owned | Planner action |
| --- | --- | --- | --- |
| TL-11 dynamic roles, bounded queues, joins, proved loops | Exploration semantics for all of them (table above), except QSL gaps G3 to G5 | FR-050 to FR-099: link the role, queue, join and loop declarations; assess an observed history against role lifetime and population bound, queue capacity and overflow outcome, join settlement per policy, loop bound; map QSL `Unsupported` and requires-bound outcomes to typed dispositions (Decision 3) | Rewrite as assessment and carrying work over QSL's semantics; drop "implement exploration" and "loop invariants and variants" from this repository |
| TL-6 core roles, messages, channels, control, compensation, replay | Compensation and channel steps for model subjects | FR-100 to FR-149: only the delta over the existing FR-002, FR-003, FR-004, FR-019 to FR-028 and FR-041 coverage. Each new requirement names the V1-PROT capability it closes and the existing requirement it extends; a capability those FRs already cover gets no new FR | Rewrite as a delta ticket; close the rows the existing FRs already cover |
| TL-10 projection and realizability | Claim vocabulary and routing only. No engine (gaps G1, G2) | FR-150 to FR-199: dispatch of both kinds to QSL's backend once one exists; premise binding for communication and visibility premises; carrying a QSL witness with trace-position correspondence to the linked graph (FR-302); `unsupported` while no backend exists, with global conformance preserved | Rewrite to dispatch and carrying only; the decision procedures move to a ticket for the QSL code lead (G1, G2); keep blocked by that ticket |
| TL-9 composition and refinement | Refinement engine, protocol abstract side, replay, certificate; composition planner for acyclic discharge. Gap G6 for declared fixed-point cycles | FR-200 to FR-249: dispatch `refinement` and `composition` to QSL's backends (amends FR-301); premise binding; carrying paired counterexamples and certificates; refusing a refinement result whose assumptions are stronger than the request's | Rewrite to dispatch and carrying; remove the product construction and the discharge algorithm from scope |
| TL-8 reference workflows, integration, qualification | Backends for the corpora where QSL has them | Integration tests and evidence for each Protocol-owned cell, as IT and TC artifacts that cite the FR blocks above. No FR block; qualification adds no requirement of its own | Keep as the last layer; scope corpora to the capabilities that have backends, and mark projection and realizability corpora blocked on G1 and G2 |
| TL-12 adopt the extension and delivery DAG | Not applicable | This record is the adoption: it fixes the split, reserves the blocks and names ownership. No further adoption artifact is owed | Close when this record merges |

## Consequences

- The extension shrinks from implementing five analyses to linking, assessing,
  dispatching and carrying. Wave sizing for FR-050 to FR-249 follows from that.
- Projection and realizability cannot ship in this library until QSL provides
  an engine. Until then the library returns `unsupported` for them, as FR-029
  and FR-060 (QSpec) already require, and global conformance remains available.
- The library keeps no copy of QSL's step semantics. Where a requirement here
  must state a threshold or overflow rule for assessment, it cites the QSpec
  clause (FR-170, FR-171) and the QSL record (ADR-027) and does not restate them.
- FR-301, spec.md section 5, AD-001 and TC-147 lose their wording about a
  library-built product construction. Their dispatch, independence and
  premise-binding obligations are unchanged.

## QSL gaps for the QSL code lead

Capabilities this extension needs that QSL `main` does not provide, as read on
2026-11-10. Each is a candidate ticket for the QSL code lead; this repository
does no QSL-side work.

| Id | Gap | Evidence | Needed by |
| --- | --- | --- | --- |
| G1 | No decision procedure for `local-projection`: role-local projection with send/receive correspondence, queue order and merge visibility | Only vocabulary, applicability and routing exist; an unserved request settles `unsupported_projection` / `unsupported-requested-capability` | TL-10 |
| G2 | No decision procedure for `realizability`: local-history strategy game, realizable/unrealizable witnesses, non-conclusive outcome on missing premises | Same as G1 | TL-10 |
| G3 | `join predicate(...)` is not stepped (`Unsupported::JoinPredicate`), and a join that may leave branches running with no `outstanding` rule is refused (`Unsupported::UnruledOutstanding`) | `qsl-eval/src/simulation/protocol/program.rs` | TL-11 |
| G4 | Symbolic channel capacity is not stepped (`Unsupported::SymbolicCapacity`); only literal capacity is | Same file | TL-11 |
| G5 | No discharge of a `repeat`'s `invariant` and `variant` (lexicographic variants included) was found. They parse and lower into the checked package, but a search of `qsl-analyze`, `qsl-eval`, `qsl-replay` and `qsl-route` found no engine that proves them | `qsl-forms/src/protocol_control.rs`, `qsl-semantics/.../controls.rs` | TL-11 |
| G6 | Declared least-fixed-point cycles: the composition planner computes no fixed point and discharges a cycle only from a `fixed-point` slot holding a proved result. No producer of that result was found | `qsl-replay/src/planner/compose.rs` | TL-9 |
| G7 | Other protocol-system constructs that are not stepped and refuse today: `activation on each` with a compensation template, a protocol capture, an `unordered` channel with a `head` or `tail` loss selector, and a step binder of an unenumerable record type | `Unsupported` in `qsl-eval/src/simulation/protocol.rs` | TL-11, TL-6 |

G5 and G6 are findings from a negative search. The QSL code lead should confirm
or correct each before opening a ticket.

## Alternatives considered

**Implement projection, realizability and the rest in this library, as the
tickets were written.** Rejected. It duplicates the transition system QSL
owns (ADR-027), forks the identity and step semantics, and contradicts the
rule that this library has no semantic runtime of its own (AD-001).

**Close TL-10 as provided by QSL.** Rejected. The comment's premise is false
for projection and realizability: QSL has the claim vocabulary and no engine.
Closing would leave the capability with no owner.

**Wait for QSL before reserving blocks.** Rejected. The blocks cost nothing and
let later waves allocate ids independently of the QSL gaps.
