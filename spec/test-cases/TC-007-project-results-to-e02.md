---
id: TC-007
title: "Project canonical protocol results to E02"
type: TC
relationships:
  - target: ix://agent-ix/quire-protocol/FR-007
    type: verifies
  - target: ix://agent-ix/quire-protocol/FR-034
    type: verifies
  - target: ix://agent-ix/quire-protocol/FR-035
    type: verifies
---
# TC-007: Project canonical protocol results to E02

## Description

Verify the total FR-062 mapping, exact derived identity, canonical-source
retention, adequacy separation, oracle lineage, and stable refusals.

## Test Procedure

Project one strict result for every source-state row and every simultaneous
finding combination. Mutate source/target versions, source digest, every target
binding, adapter identity/version, semantic order, adequacy domain, support
relation, native artifact reference, and each condition in the refusal set.

## Expected Results

Every source row maps exactly and preserves all findings. Every semantic mutation
changes the derived identity or refuses. Protocol adequacy never becomes E02
adequacy or sufficiency. Refusal emits no `TechniqueResult`.
