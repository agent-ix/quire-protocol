---
id: Task-001
title: "Adopt owner catalog error and refresh moving-main lock"
type: Task
status: not_started
track: A
priority: P1
relationships:
  - target: ix://agent-ix/quire-protocol/FR-043
    type: references
  - target: ix://agent-ix/quire-protocol/TC-149
    type: verifies
---
# Task-001: Adopt owner catalog error and refresh moving-main lock

## Scope

Adapt src/checked_package.rs to the current owner dispatch result and refresh Cargo.lock without changing moving-main declarations.

## Subtasks

- [ ] Read current owner API and inspect existing tests before editing; identify genuine regression assertions.
- [ ] Import CatalogReadError and add the exhaustive CatalogUnavailable arm; return its existing payload unchanged before later sidecar checks, with no admitted handoff, panic or package-refusal classification.
- [ ] Refresh held moving-main dependencies to current upstream heads; preserve current carried_member production-catalog support. Record application version and actual validation outcomes, without revision pins or an authored revision ledger; perform user-required exact-head checks transiently when routing or guarding a merge.
- [ ] Run focused current catalog/adapter validation; retain authentic trace attributes and add only tests backed by reachable behavior. Use Inspection for the opaque catalog-error path.

## Deliverables

One small adapter/lock change and focused validation evidence, ready for independent review.

## Notes

No catalog copies, local error mirrors, compatibility taxonomy, manufactured owner error or test-only public API. Unrelated baseline findings remain explicitly identified rather than silently fixed.
