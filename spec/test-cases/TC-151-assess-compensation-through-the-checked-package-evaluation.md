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

Verify the compensation assessment over qsl-eval's slot evaluation.

## Test Procedure

Run QSL FR-520-AC-1's outcome rows over **R**'s `Full` and `Partial` compensations; keep `tc_003`'s assertions, restating the report-based ones on the outcome map; supply an incomplete population, a missing target and a missing achieved state and an unavailable binder; run a retry bound of `n` and `n + 1`; run under a denied-syscall harness; scan the source for the retired names and for Boolean coercion; read the charge log before and after the per-binder attribution exists. Tag the tests `#[trace("TC-151", "FR-045-AC-n")]`.

## Expected Results

Each outcome row maps to its successor; tc_003's assertions hold; no true recovery from incomplete evidence; `n` attempts admitted and `n + 1` refused; no I/O; no retired name; total charge recorded with attribution reported unsupported until it exists.
