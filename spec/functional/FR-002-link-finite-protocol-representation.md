---
id: FR-002
title: "Link the finite protocol representation"
type: FR
relationships:
  - target: ix://agent-ix/quire-protocol/US-001
    type: implements
  - target: ix://agent-ix/quire-protocol/FR-001
    type: depends_on
  - target: ix://agent-ix/quire-specification/FR-050
    type: depends_on
  - target: ix://agent-ix/quire-specification/FR-054
    type: depends_on
  - target: ix://agent-ix/quire-specification/FR-210
    type: references
  - target: ix://agent-ix/quire-specification/FR-216
    type: references
  - target: ix://agent-ix/quire-specification/FR-217
    type: references
  - target: ix://agent-ix/quire-specification/FR-220
    type: references
  - target: ix://agent-ix/quire-specification/FR-221
    type: references
  - target: ix://agent-ix/quire-specification/FR-227
    type: references
  - target: ix://agent-ix/quire-specification/FR-228
    type: references
  - target: ix://agent-ix/quire-specification/FR-252
    type: references
  - target: ix://agent-ix/quire-specification/FR-255
    type: references
---
# FR-002: Link the finite protocol representation

## Description

When an admitted protocol is linked, the library SHALL construct the exact finite
typed representation required by source requirements FR-050 through FR-054 or
return located refusals for every invalid binding that prevents construction.

## Inputs

An admitted protocol artifact with typed role, role-instance, workflow-instance,
component, endpoint, channel, message, send, receive, delivery, attempt, effect,
choice, control, join, iteration, termination, and visibility declarations.

## Outputs

One finite linked representation with exact source spans and identities, plus
independent diagnostics for definitions that remain inspectable after a refusal.

## Behavior

The linker SHALL keep definitions, role instances, workflow instances,
participants, components, endpoints, channels, messages, sends, receives,
deliveries, attempts, and effects distinct. It SHALL derive causal edges only
from authored control and channel premises. It SHALL refuse duplicate role
authorities, unresolved, ambiguous, wrong-kind or name-only identity bindings,
missing or invalid joins, unbounded or zero-progress iteration, causal cycles,
unreachable termination, missing or conflicting sender, receiver, message-type,
ordering or delivery premises, and ownerless, multiply owned, unlabeled, overlapping or
owner-unobservable choices. If any refused binding prevents construction, then
the linker SHALL emit no linked representation. The refusal SHALL contain only
the independent located diagnostics established from the submitted artifact.

Where a compiled-artifact contract carries temporal selections, the linker SHALL
treat the temporal profile, clock alternative, and clock parameters as
independently retained selection identities rather than as offered values. When
a temporal binding set is missing, surplus, duplicated, reordered, owned by a
foreign owner, or outside the declared range, the linker SHALL refuse it,
naming the offending binding axis and emitting no linked representation.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-002-AC-1 | Concurrent workflow instances, static roles, role instances, participants, components, endpoints, channels, messages, sends, receives, duplicate deliveries, attempts, and business effects retain distinct exact identities. | Test (TC-002) |
| FR-002-AC-2 | Every permitted interleaving of bounded parallel branches preserves authored causal edges and satisfies its declared join. | Test (TC-002) |
| FR-002-AC-3 | Each duplicate authority, unresolved/ambiguous/wrong-kind/name-only reference, missing/invalid join, cycle, unbounded/zero-progress iteration, unreachable termination, missing/conflicting channel premise, and ownerless/multiply-owned/unlabeled/overlapping/unobservable choice mutation is refused at its source location with no linked representation. | Test (TC-002) |
| FR-002-AC-4 | Reordering equal-time input or changing presentation text cannot add a causal edge or change linked meaning, while changing an authored branch, join, iteration maximum, channel premise or visibility premise changes the linked protocol identity. | Test (TC-002) |
| FR-002-AC-5 | A changed temporal-profile, clock-alternative or clock-parameter selection, and a temporal binding set that is missing, surplus, duplicated, reordered, foreign-owned or out of the declared range, each return a typed refusal naming the offending axis with no linked representation, while a complete, singly owned, correctly ordered, in-range binding set links. | Test (IT-001) |

## Adopted vocabulary

This requirement defines no protocol noun. Every subject it links is typed
upstream, and the linker's obligation is to preserve those identities, never to
establish them:

| Noun | Upstream object | Identity kind |
| --- | --- | --- |
| Subject kind discrimination | `FR-210-protocol-subject-vocabulary` | value |
| Channel premise (all six components) | `FR-216-channel-premise` | value |
| Role internality / visibility | `FR-217-role-internality` | value |
| Causal control graph | `FR-220-causal-control-graph` | derived |
| Node occurrence | `FR-221-protocol-node-occurrence` | derived, keyed (static declaration path, workflow instance identity, iteration identity) |
| Owned choice | `FR-227-owned-choice` | derived, via FR-221 |
| Bounded control construct | `FR-228-bounded-control-construct` | value |
| Temporal clock binding | `FR-252-temporal-clock-binding` | value |
| Bounded temporal interval | `FR-255-bounded-temporal-interval` | value |

### Where the upstream vocabulary does not yet reach

Upstream records these as open gaps in its own artifacts. This requirement's
obligations stand, but they rest on identities upstream has not established, and
they cannot be discharged until it does:

- **Delivery** — `FR-224-delivery` states that it "establishes no component that
  distinguishes one delivery from another", and that neither a key nor a total
  order exists in the prose. FR-002-AC-1 requires duplicate deliveries to retain
  distinct exact identities; that criterion currently has no identity to retain.
- **Operation attempt** — `FR-225-operation-attempt` records that "a retried
  attempt is required to be retained as a second subject, and the prose supplies
  no component that separates it from the first".
- **Business effect** — `FR-226-business-effect` states that its declared key "is
  constant across two effect claims at one static node within one workflow
  instance, so two duplicate effects compare equal under it and the duplicate the
  evaluator is required to detect is undetectable".
- **Role, role instance, workflow instance, message, channel** — `FR-211` through
  `FR-215` each declare identity kind *authored* and then state that "the prose
  does not fix the key components". They may be cited for what identity excludes,
  never for what constitutes it.

## Dependencies

- [FR-001](./FR-001-admit-selected-protocol-profile.md).
- `ix://agent-ix/quire-specification/FR-050` through `FR-054` define the source
  semantics; `FR-210` through `FR-228`, `FR-252` and `FR-255` type the subjects.
