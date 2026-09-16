---
type: index
title: "Plan-001 implementation chunk register"
description: "Recovered status of the implementation chunks evidenced by the current tree and GitHub history."
---
# Plan-001 implementation chunk register

## Contents

This register reconstructs only chunk identifiers evidenced by source ownership
headers, merged pull requests, or the issue history. It is a delivery aid under
[Plan-001](./plan.md), not a second source of requirements. An identifier absent
from the table is deliberately unassigned here rather than inferred.

Status meanings:

- **landed** — the named implementation is present on `main` with merged-PR evidence;
- **partial** — a usable slice is present, but the named chunk's downstream contract is not complete;
- **blocked** — its owning contract is unresolved outside this repository;
- **planned** — the reserved module is still an intentional stub;
- **not landed** — an earlier pull request closed without merging and supplies no current-tree evidence.

| Chunk | Scope | Status | Evidence / remaining boundary | Tracking |
| --- | --- | --- | --- | --- |
| C-00 | Rust facade reservation | landed | PR #29 | #1, epic #14 |
| C-01 | B-owned refusal envelope and local code registry | landed | PR #40 | #8, epic #14 |
| C-02 | Selected source-contract validation | partial | The Task-006 candidate retains known source causes under their exact selected contract and unknown contracts opaquely under `unsupported-source-contract`; status remains partial until the #8 candidate merges to `main`. | #8, epic #14 |
| C-03 | Typed local/source cause taxonomy and ordering | partial | The closed local vocabulary, selected source order and final common routing execute in the Task-006 candidate; status remains partial until #8 merges to `main`. | #8, epic #14 |
| C-04 | Independently selected inventory and selection-axis discriminator | partial | PR #36 landed the top-level inventory; per-source, per-dependency, and per-model selection inventory remains absent | #1, #34, epic #14 |
| C-05 | Preserve A-owned admission errors through B's discriminating layer | partial | PR #36 landed the first mapper; the #8 candidate retains every published A error family and stable code through the shared envelope without text conversion. Status remains partial until #8 merges. | #1, #8, #34, epic #14 |
| C-06 | Repair IT-001 refusal-axis assertions | landed | PR #36 | #1, #34, epic #14 |
| C-07 | Closed protocol identity newtypes | landed | PR #37 | #4, epic #14 |
| C-10 | Opaque producer-domain digests and cross-domain guard | landed | PR #36 | #1, #8, #34, epic #14 |
| C-11 | Result-domain JCS subset | partial | PR #36; JSON numbers remain explicitly refused until the exact ECMAScript serialization algorithm is implemented | #8, epic #14 |
| C-12 | Finite typed linker | landed | PR #43 | #4, epic #14 |
| C-13 | Accumulated located link diagnostics | landed | PR #44 | #4, epic #14 |
| C-14 | Permitted-interleaving proof and FR-050..FR-054 mutation sweep | landed | PR #45 | #4, epic #14 |
| C-15 | Five-axis closure and assessment-outcome lattice | landed | PR #33 | #6, #8, #12, epic #14 |
| C-16..C-19 | Compensation algebra, purity, activation, and recovery | landed | Task-003 derives graph-owned state from the exact linked `/2` context, accepted QObs subjects/position views and QSL guard/retry/recovery evaluation at `c0deb7b`; the graph retains exact evaluator inputs/reports, attempt/effect relationships, full/partial outcomes, selected commit order and finite attempts. Exact replay compares owner evidence and adverse transitions are transactional. The former recovery callback is absent from production and the zero-action capability proof remains executable. | #5, epic #14 |
| C-20..C-21 | Batch and incremental conformance | landed | PR #76 derives one production `ConformanceKernel` from graph/compensation values and proves batch/incremental frontend agreement. | #6, epic #14 |
| C-22 | Six independent claim dispositions | landed | PR #41 | #7, epic #14 |
| C-23..C-24 | Claim premise retention and dispatch | landed | PR #77 consumes only activation-derived conformance output, retains exact claim premises and exposes no arbitrary backend callback or fallback path. | #7, epic #14 |
| C-26..C-28 | Canonical result, global premises, and lineage | partial | Canonical production, strict reading, mapped views, atomic Engine orchestration and private result assembly execute in the #8 candidate. Authenticated oracle/dependence support and applicable correction remain blocked on owner capabilities; merge alone cannot complete this chunk. | #8, epic #14 |
| C-29 | Count, allocation, and deadline guards | landed | PR #42 | #10, epic #14 |
| C-30 | Source-bound resource-site inventory | landed | Task-008 reconciles every final site through `ResourceAccountant`, includes E02 projection, removes the parallel guard architecture and retains owner usage without a local QSL accumulation mirror. | #10, epic #14 |
| C-31 | Reproduction record | landed | PR #38 | #10, epic #14 |
| C-32 | Shared Rust assurance-boundary enforcement | not landed / blocked | PR #32 closed without merge; Task-011 forbids a local replacement and waits for the shared checker tracked by `quire-research#64` | #13, epic #14 |

## Current executable boundary

The standalone kernels above are real but disconnected. The reviewed DAG starts
with Task-017's shared outcomes/refusals and Task-013's request-bound resolver,
then connects the serial graph-to-result core. QSpec #57 resolved the FR-236
partition at `478afa2`; its remaining Protocol bridge is implementation work,
not an external block. qverify E02 acceptance and its dependency-compatible
exact Cargo graph are delivered by issues #32/#44 and Engineering Assurance
#102. Quoin scenario intake remains blocked by `quoin#519`; no local
refusal path waits on a proposed diagnostic catalog.
