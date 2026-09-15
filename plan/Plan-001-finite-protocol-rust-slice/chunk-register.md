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
| C-02 | Selected source-contract validation | partial | Local refusal execution needs no external catalog. Task-017/006 retain a known source cause under its exact contract and keep an unknown contract opaque under `unsupported-source-contract`. | #8, epic #14 |
| C-03 | Typed local/source cause taxonomy and ordering | partial | Protocol-local causes are closed; source causes retain their selected contract order or explicit source-wire order. Final common routing remains Task-006. | #8, epic #14 |
| C-04 | Independently selected inventory and selection-axis discriminator | partial | PR #36 landed the top-level inventory; per-source, per-dependency, and per-model selection inventory remains absent | #1, #34, epic #14 |
| C-05 | Preserve A-owned admission errors through B's discriminating layer | partial | PR #36 landed the first mapper; the current increment retains every published A error family and stable code without text conversion. Task-017/006 add the shared envelope without an external catalog blocker. | #1, #8, #34, epic #14 |
| C-06 | Repair IT-001 refusal-axis assertions | landed | PR #36 | #1, #34, epic #14 |
| C-07 | Closed protocol identity newtypes | landed | PR #37 | #4, epic #14 |
| C-10 | Opaque producer-domain digests and cross-domain guard | landed | PR #36 | #1, #8, #34, epic #14 |
| C-11 | Result-domain JCS subset | partial | PR #36; JSON numbers remain explicitly refused until the exact ECMAScript serialization algorithm is implemented | #8, epic #14 |
| C-12 | Finite typed linker | landed | PR #43 | #4, epic #14 |
| C-13 | Accumulated located link diagnostics | landed | PR #44 | #4, epic #14 |
| C-14 | Permitted-interleaving proof and FR-050..FR-054 mutation sweep | landed | PR #45 | #4, epic #14 |
| C-15 | Five-axis closure and assessment-outcome lattice | landed | PR #33 | #6, #8, #12, epic #14 |
| C-16..C-19 | Compensation algebra, purity, activation, and recovery | partial | Standalone registration/retry/commit/recovery kernels and tests exist; Task-003 derives them from the linked graph, removes the public recovery callback, and closes transactionality. QSpec #57 resolved FR-236 at `478afa2`; Protocol retains distinct attempt, effect, and recovery outcome domains, while the complete typed-expression bridge remains Task-003 work. | #5, epic #14 |
| C-20..C-21 | Batch and incremental conformance | partial | Both kernels execute independently; Task-004 derives one production `ConformanceKernel` from graph/compensation values and proves frontend agreement. | #6, epic #14 |
| C-22 | Six independent claim dispositions | landed | PR #41 | #7, epic #14 |
| C-23..C-24 | Claim premise retention and dispatch | partial | Premise validation/dispatch execute; Task-005 consumes only conformance output and removes the public arbitrary backend callback/fallback path. | #7, epic #14 |
| C-26..C-28 | Canonical result, global premises, and lineage | partial | Canonical production, strict reading, lineage and mapped views execute; Task-006 owns Engine/bundle orchestration, shared domains and private result assembly. | #8, epic #14 |
| C-29 | Count, allocation, and deadline guards | landed | PR #42 | #10, epic #14 |
| C-30 | Source-bound resource-site inventory | partial | The current-site inventory/accountant exists. Task-008 covers all final sites, internalizes parallel guards and consumes QSL's published accumulation accessor. | #10, epic #14 |
| C-31 | Reproduction record | landed | PR #38 | #10, epic #14 |
| C-32 | Shared Rust assurance-boundary enforcement | not landed / blocked | PR #32 closed without merge; Task-011 forbids a local replacement and waits for the shared checker tracked by `quire-research#64` | #13, epic #14 |

## Current executable boundary

The standalone kernels above are real but disconnected. The reviewed DAG starts
with Task-017's shared outcomes/refusals and Task-013's request-bound resolver,
then connects the serial graph-to-result core. QSpec #57 resolved the FR-236
partition at `478afa2`; its remaining Protocol bridge is implementation work,
not an external block. The remaining external blocks are qverify E02 acceptance
(`quire-verification#32`) and Quoin scenario intake (`quoin#519`); no local
refusal path waits on a proposed diagnostic catalog.
