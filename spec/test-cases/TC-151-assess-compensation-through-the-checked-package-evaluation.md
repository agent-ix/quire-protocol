---
id: TC-151
title: "Assess compensation through the checked-package evaluation"
type: TC
relationships:
  - target: ix://agent-ix/quire-protocol/FR-045
    type: verifies
---
# TC-151: Assess compensation through the checked-package evaluation

## Description

Verify the compensation assessment over qsl-eval's package-only slot
evaluation, including the supported per-binder charge log.

## Test Procedure

Run QSL FR-520's reachable outcome-map rows over **R**'s `Full` and `Partial`
compensations; keep `tc_003`'s assertions, restating report-based assertions on
the outcome map; supply an incomplete population, a missing target and a
missing achieved state and an unavailable binder; run a retry bound of `n` and
`n + 1`; run under a denied-syscall harness; scan the source for the retired
names and Boolean coercion; and inspect the per-binder charge log for each
binder read, an unread binder, the unattributed entry and the call total. Do
not construct a local input for the unsupported-construct row or recovery
predicate rows that recipe **R** cannot reach. Tag each test with the exact
acceptance criterion using `#[trace("FR-045-AC-n", "TC-151")]`.

## Expected Results

Each reachable outcome row maps to its successor; `tc_003`'s assertions hold;
incomplete recovery evidence never yields true; exactly `n` attempts are
admitted and `n + 1` is refused; the assessment performs no I/O and exposes no
retired API; every binder read has its own charge, unread binders have none,
and the binder entries plus unattributed entry equal the call's total.
