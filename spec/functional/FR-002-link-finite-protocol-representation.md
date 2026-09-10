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

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-002-AC-1 | Concurrent workflow instances, static roles, role instances, participants, components, endpoints, channels, messages, sends, receives, duplicate deliveries, attempts, and business effects retain distinct exact identities. | Test (TC-002) |
| FR-002-AC-2 | Every permitted interleaving of bounded parallel branches preserves authored causal edges and satisfies its declared join. | Test (TC-002) |
| FR-002-AC-3 | Each duplicate authority, unresolved/ambiguous/wrong-kind/name-only reference, missing/invalid join, cycle, unbounded/zero-progress iteration, unreachable termination, missing/conflicting channel premise, and ownerless/multiply-owned/unlabeled/overlapping/unobservable choice mutation is refused at its source location with no linked representation. | Test (TC-002) |
| FR-002-AC-4 | Reordering equal-time input or changing presentation text cannot add a causal edge or change linked meaning, while changing an authored branch, join, iteration maximum, channel premise or visibility premise changes the linked protocol identity. | Test (TC-002) |

## Dependencies

- [FR-001](./FR-001-admit-selected-protocol-profile.md).
- `ix://agent-ix/quire-specification/FR-050` through `FR-054`.
