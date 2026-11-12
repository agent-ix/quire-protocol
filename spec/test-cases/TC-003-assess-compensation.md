---
id: TC-003
title: "Assess compensation, retry, commit, and recovery"
type: TC
relationships:
  - target: ix://agent-ix/quire-protocol/FR-003
    type: verifies
---
# TC-003: Assess compensation, retry, commit, and recovery

## Description

Verify activation, compensation registration, and recovery across exact trigger,
capture, anchor, effect, retry, commit, full/partial relation, population,
missing-fact, duplicate, and finite-bound states.

## Test Procedure

Exercise an established trigger, an absent trigger under complete closure, and an
unavailable trigger under open/incomplete input. Independently substitute its
workflow, role, effect, predicate, capture, choice-visibility evidence (choice, owner or visible set), and evaluation-anchor identities.
Replay one successful effect twice; separately submit a send, receive, duplicate
delivery, failed attempt, early compensation, late registration, forbidden
post-commit compensation, bounded and one-over retries, timed-out attempt, and
duplicate effect. Evaluate the same achieved state under authored full-restoration
and partial-recovery relations, then vary captured targets, relationship,
population membership, and completeness authority independently.
Run every assessment with action-capable business-operation and compensation-
operation doubles and assert their zero-call counters. Mutate the evaluator API
and dependency inventory independently to admit one network, storage, telemetry,
business-operation, or compensation-operation capability, and require the
purity boundary test to reject each mutation before assessment.

## Expected Results

Activation states and every captured identity remain exact. Only the paired
successful effect creates one registration, including after replay. Each
attempt/effect and ordering outcome remains distinct; full and partial recovery
follow only their exact authored relation and complete inputs; missing authority
cannot become success. Every action-capable counter remains zero, and each
I/O-capability API or dependency mutation fails for its intended reason.
