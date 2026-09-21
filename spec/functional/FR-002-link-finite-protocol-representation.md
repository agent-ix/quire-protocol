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
  - target: ix://agent-ix/quire-specification/FR-262
    type: references
  - target: ix://agent-ix/quire-specification/FR-287
    type: references
  - target: ix://agent-ix/quire-specification/FR-288
    type: references
---
# FR-002: Link the finite protocol representation

## Description

When an admitted protocol is linked, the library SHALL construct the exact finite
typed representation required by source requirements FR-050 through FR-054 or
return located refusals for every invalid binding that prevents construction.

## Inputs

An admitted `quire.compiled-protocol/2` artifact with typed role,
role-instance, workflow-instance, component, endpoint, channel, message, send,
receive, delivery, attempt, effect, choice, control, join, iteration,
termination, and visibility declarations; one accepted version-1 static
binding definition; one finite runtime binding set; and a definition registry.
The static binding definition carries its exact artifact identity, revision and
digest, every role's exact FR-217 internality, and each independently selected
slot-to-producer-kind/correspondence fact. The runtime set targets that accepted
definition, keys each subject entry by the exact static binding-requirement
handle plus an imported FR-287 `QualifiedSubject`, and keys each relationship
entry by the exact static relationship requirement plus an imported FR-262
`Relationship`. A static binding requirement identifies a slot and never
substitutes for its runtime value. The definition registry retains each FR-288
key with its complete static consistency content across link operations.

## Outputs

One finite linked representation with an FR-288 qualified definition identity,
complete typed semantic content, exact runtime identities and retained source
spans, plus independent diagnostics for definitions that remain inspectable
after a refusal. A diagnostic names the affected static handle or runtime-entry
ordinal and retains an originating source locus when one exists; it never
manufactures a source span for an unlocated runtime value. A FIFO channel
additionally exposes a partition constructor whose key is the exact static
channel plus the caller's exact typed evaluated key value.

## Behavior

When an admitted version-2 protocol, accepted version-1 binding definition and
runtime binding set are linked, the linker SHALL keep definitions, static roles,
role instances,
workflow instances, participants, components, endpoints, channels, messages,
sends, receives, deliveries, attempts, and effects distinct. It SHALL preserve
each runtime identity from the corresponding FR-287 value and SHALL NOT mint a
runtime identity from a binding-requirement coordinate, display name or table
position. Each static role SHALL carry exactly one `internal` or `external`
member. The linker SHALL refuse a missing, surplus, duplicate, wrong-kind,
foreign, incompatible or slot-derived runtime binding without choosing a
default.

The accepted static binding definition SHALL identify its contract as
`quire.protocol.binding-definition/1`, retain its independently selected
artifact identity/revision/digest, and target exactly one admitted protocol
declaration. The runtime binding set SHALL identify its contract as
`quire.protocol.runtime-binding/1` and target that exact accepted static
definition. If either binding contract version is absent, unknown or older, the
library SHALL return an unsupported-contract outcome without reading it as
version 1. The linker SHALL compare each producer-local subject kind only
through the accepted slot compatibility fact; it SHALL NOT infer a universal
producer spelling from the QSL `BindingKind` label.
If an admitted compiled artifact has a version other than
`quire.compiled-protocol/2`, the versioned linker SHALL return an
unsupported-artifact outcome without inferring internality or runtime subjects.

The linker SHALL construct the FR-288 definition identity componentwise from
the admitted source-document key, source-inventory-qualified declaration
handle, formal owner and clause, and complete semantic selection closure. The
linker SHALL include the accepted static binding-definition selection in that
closure. The linker SHALL exclude concrete runtime subjects, relationships,
observations and resource ceilings from that closure. The linker SHALL retain
the complete typed declaration content as consistency
content together with the static role classifications and slot compatibility
facts. When one FR-288 key is associated with different static consistency
content, the library SHALL refuse an identity contradiction through the
definition registry rather than flattening or fingerprinting either graph. The
linker SHALL keep the key unchanged when only external path, locus, build
provenance, runtime subject, runtime relationship, observation or
resource-ceiling content changes.

The linker SHALL retain all five FR-216 channel premises. For FIFO, it SHALL
retain the qualified definition, declaration-local binder, typed key-expression
definition and its complete selected type and semantic dependencies. When a
caller forms a runtime FIFO partition, the partition SHALL compare the exact
static channel together with the exact typed evaluated value supplied by that
value's owner; equal values on different channels SHALL remain distinct and no
partition key SHALL add a causal edge.

The linker SHALL retain supplied FR-262 relationships without converting them
to subjects. When the same complete relationship key and facts are replayed for
one required slot, the library SHALL preserve one idempotent relationship. If
relationship facts conflict under one key, the library SHALL refuse the
contradiction. When two distinct relationship keys satisfy one
required slot, the linker SHALL return the FR-262 ambiguous outcome. When two
distinct relationship keys satisfy two distinct slots, the linker SHALL retain
both relationships as distinct from each other and from every effect.

The linker SHALL rely on the admitted compiler artifact for compiler-owned
structural validation and SHALL derive no causal edge from source order,
timestamp, arrival order or key order. It SHALL independently refuse the
consumer-owned duplicate role authorities, invalid runtime bindings,
zero-progress iteration and identity contradictions. If any refused binding
prevents construction, the linker SHALL emit no linked representation. The
refusal SHALL contain only the independent located diagnostics established from
the submitted inputs.

Where a compiled-artifact contract carries temporal selections, the linker SHALL
treat the temporal profile, clock alternative, and clock parameters as
independently retained selection identities rather than as offered values. When
a temporal binding set is missing, surplus, duplicated, reordered, owned by a
foreign owner, or outside the declared range, the linker SHALL refuse it,
naming the offending binding axis and emitting no linked representation.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-002-AC-1 | Concurrent workflow instances, static roles, role instances, participants, components, endpoints, channels, messages, sends, receives, duplicate deliveries, attempts, and business effects retain distinct exact identities from FR-288 or FR-287 as applicable; no static slot identifies a runtime subject and no QSL binding-kind label is treated as a universal producer subject-kind spelling. | Test (TC-002) |
| FR-002-AC-2 | Every permitted interleaving of bounded parallel branches preserves authored causal edges and satisfies its declared join. | Test (TC-002) |
| FR-002-AC-3 | Each consumer-owned duplicate authority, missing/surplus/duplicate/wrong-kind/foreign/incompatible/slot-derived runtime binding, invalid internality, zero-progress iteration, FR-288 content contradiction and FR-262 conflicting/ambiguous relationship is refused with no linked representation and names its exact static handle or runtime-entry ordinal plus any originating source locus; compiler-owned unresolved references, invalid joins, cycles, unreachable termination, channel omissions and invalid choices remain refusals of the admitted compiler boundary. | Test (TC-002) |
| FR-002-AC-4 | Reordering equal-time input, replaying one exact relationship, changing a concrete runtime value or changing external presentation metadata cannot add a causal edge or independently change FR-288, while changing an authored branch, join, iteration maximum, five-component channel premise, FIFO expression definition, accepted role internality, slot compatibility fact or complete typed semantic content requires another FR-288 and changes the linked protocol identity. | Test (TC-002) |
| FR-002-AC-5 | A changed temporal-profile, clock-alternative or clock-parameter selection, and a temporal binding set that is missing, surplus, duplicated, reordered, foreign-owned or out of the declared range, each return a typed refusal naming the offending axis with no linked representation, while a complete, singly owned, correctly ordered, in-range binding set links. | Test (IT-001) |

## Adopted vocabulary

This requirement defines no protocol noun. Every subject it links is typed
upstream, and the linker's obligation is to preserve those identities, never to
establish them:

| Noun | Upstream object | Identity kind |
| --- | --- | --- |
| Subject kind discrimination | `FR-210` | value |
| Channel premise (all five components) | `FR-216` | value |
| Role internality / visibility | `FR-217` | value |
| Causal control graph | `FR-220` | derived |
| Node occurrence | `FR-221` | derived, keyed (static declaration path, workflow instance identity, iteration identity) |
| Owned choice | `FR-227` | derived, via FR-221 |
| Bounded control construct | `FR-228` | value |
| Temporal clock binding | `FR-252` | value |
| Bounded temporal interval | `FR-255` | value |
| Runtime relationship | `FR-262` | value |
| Runtime subject | `FR-287` | value |
| Static protocol definition | `FR-288` | derived |

### Accepted owner boundary

The former identity and premise gaps are closed by Quire Specification PRs #41
and #42. FR-287 supplies every concrete runtime-subject key, FR-288 supplies the
static definition key, FR-216 supplies exactly five channel premises and derives
duplication permission from delivery cardinality, FR-217 supplies the exact
closed internality vocabulary, and FR-262 supplies runtime relationships.
Quire Observation PR #20 publishes the accepted FR-287 and FR-262 Rust values.
This repository imports those values and implements the linker-owned binding;
it does not redeclare their wire records or infer their missing members.

## Dependencies

- [FR-001](./FR-001-admit-selected-protocol-profile.md).
- `ix://agent-ix/quire-specification/FR-050` through `FR-054` define the source
  semantics; `FR-210` through `FR-228`, `FR-252` and `FR-255` type the subjects.
