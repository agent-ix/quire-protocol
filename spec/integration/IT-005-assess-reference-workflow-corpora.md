---
id: IT-005
title: "Assess the reference workflow corpora over the Protocol-owned cells"
type: IT
relationships:
  - target: ix://agent-ix/quire-protocol/FR-050
    type: verifies
  - target: ix://agent-ix/quire-protocol/FR-100
    type: verifies
  - target: ix://agent-ix/quire-protocol/FR-020
    type: verifies
---
# IT-005: Assess the reference workflow corpora over the Protocol-owned cells

## Objective

Show, for each corpus the Protocol-owned V1 cells name, which living tests run it and what each asserts, so every cell has attributed evidence or a stated gap. This library assesses an observed history; QSL owns the exploration engines (ADR-010).

## Target Integration

QSL compiles the choreographies and owns the exploration engines; this library admits, links and assesses an observed history against the authored declarations (ADR-010). It adds no engine.

## Preconditions

QSL's writer and Contract IR's reader are available in process. QSL's linked engines serve `local-projection` and `realizability` only; `composition` and `refinement` have none.

## Inputs

The recipe model's dynamic-roles choreography, and finite observed histories of it: a conforming run and a faulty run. The corpora below name the evidence for each cell.

## Test Procedure

Run each living test named in the table. Each corpus row names the cells it backs and the assertion made.

## Corpora

| Corpus | Cells | Run by | What is asserted |
| --- | --- | --- | --- |
| Finite replay | V1-PROT-025, V1-RUN-003 | TC-167; TC-008 | A finite observed history is assessed on the role, queue, join and loop axes with the same result on a second pass; the reproduction record's seed, toolchain and corpus identity are typed and refuse when absent. |
| Timed refund | V1-RUN-011, V1-PROT-019 | TC-154 | Inclusive deadline branches settle only on complete progress; captures bind once, in order. No end-to-end refund workflow runs: the cell is partial. |
| Fan-out and join | V1-RUN-012, V1-PROT-010 | TC-053; TC-167 | Each join policy, threshold and `outstanding` rule is judged on an observed history; a mixed run holds or fails on the join axis independently of the others. |
| Retry | V1-PROT-021, V1-PROT-023 | TC-003 (FR-022) | The initial attempt counts, the authored retry bound admits exactly and one over refuses; attempts, effects and outcomes stay distinct. |
| Compensation | V1-PROT-020, V1-PROT-022 | TC-003 (FR-019..FR-021) | Registration needs its matching effect and is replay-idempotent; commit and recovery orderings follow only the authored causal order. |
| Dynamic roles | V1-PROT-012, V1-PROT-013 | TC-051, TC-052, TC-167 | Lifetime and population bound of a replicated role; queue capacity and overflow; a faulty run is judged on each axis separately. |
| Projection | V1-PROT-027 | TC-164, TC-165 (IT-004) | QSL's linked engine decides it. |
| Realizability | V1-PROT-028 | TC-166 | QSL's linked engine decides it over a subject built from QSL's public front end; the witness carrier is batch D (FR-302). |
| Composition, refinement | V1-PROT-029, V1-PROT-030 | TC-200..TC-204 | Dispatch and carriage against a stand-in; no linked QSL engine serves either kind. |

## Expected Results

Every corpus row passes. A cell with no row is a stated gap in the V1-PROT inventory of `spec/tests.md`, not an unnamed omission.

## Dependencies

The result-publication cells (V1-RUN-005, V1-RUN-006 as published) wait on Quoin #519 (IT-003). Composition and refinement wait on a QSL engine. The `v1-draft.2` witness carrier (FR-302) is batch D.
