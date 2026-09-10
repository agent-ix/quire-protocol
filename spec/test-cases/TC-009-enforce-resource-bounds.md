---
id: TC-009
title: "Enforce every protocol resource boundary"
type: TC
relationships:
  - target: ix://agent-ix/quire-protocol/NFR-002
    type: verifies
---
# TC-009: Enforce every protocol resource boundary

## Description

Verify each authored protocol and caller-supplied resource boundary and
discriminate every guard.

## Test Procedure

Derive a resource inventory independently from every reviewed Rust interface and
production source site that allocates, retains, queues, iterates, or traverses
caller-controlled data, then reconcile it bidirectionally with the retained
inventory. For each input-byte, definition, node, edge, role, message, branch,
iteration, active-instance, observation, lineage, result-item, derived-artifact,
output-byte, operation, accounted-allocation, and elapsed-time dimension,
execute cases below, at, and above every authored maximum and one positive
caller-supplied limit. Delete and duplicate each inventory entry, add an
otherwise valid unlisted source site, remove its boundary-case binding, and
remove or reverse each guard in isolated mutations. Substitute an action-capable
test double for business and compensation execution.

## Expected Results

Every source site has exactly one complete inventory entry and every entry maps
to one real source site and discriminating boundary case. Below-limit work may
proceed, every inclusive at-limit case remains eligible, and over-limit work
returns resource-incomplete before starting the excess unit. The implementation
reports elapsed overshoot only for a declared indivisible unit. Each inventory
or guard mutant is killed by its assigned test for the intended reason, and the
action-capable double records zero calls.
