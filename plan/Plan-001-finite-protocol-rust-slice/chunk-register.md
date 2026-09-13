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
| C-02 | Selected source diagnostic-catalog validation | blocked | `quire-specification` FR-270 still says the offered catalog is proposed, not adopted or implemented; no local catalog may substitute | #8, epic #14 |
| C-03 | Typed source-cause taxonomy and ordering | blocked | `quire-specification` FR-272 publishes no stable field names, total order, or spelling convention; `src/refusal/cause.rs` stays empty | #8, epic #14 |
| C-04 | Independently selected inventory and selection-axis discriminator | partial | PR #36 landed the top-level inventory; per-source, per-dependency, and per-model selection inventory remains absent | #1, #34, epic #14 |
| C-05 | Preserve A-owned admission errors through B's discriminating layer | partial | PR #36 landed the first mapper; the current increment retains every published A error family and its stable code without text conversion. Conversion into `ProtocolRefusal` still waits on C-02/C-03 | #1, #8, #34, epic #14 |
| C-06 | Repair IT-001 refusal-axis assertions | landed | PR #36 | #1, #34, epic #14 |
| C-07 | Closed protocol identity newtypes | landed | PR #37 | #4, epic #14 |
| C-10 | Opaque producer-domain digests and cross-domain guard | landed | PR #36 | #1, #8, #34, epic #14 |
| C-11 | Result-domain JCS subset | partial | PR #36; JSON numbers remain explicitly refused until the exact ECMAScript serialization algorithm is implemented | #8, epic #14 |
| C-12 | Finite typed linker | landed | PR #43 | #4, epic #14 |
| C-13 | Accumulated located link diagnostics | landed | PR #44 | #4, epic #14 |
| C-14 | Permitted-interleaving proof and FR-050..FR-054 mutation sweep | landed | PR #45 | #4, epic #14 |
| C-15 | Five-axis closure and assessment-outcome lattice | landed | PR #33 | #6, #8, #12, epic #14 |
| C-16..C-19 | Compensation algebra, purity, activation, and recovery | planned | Reserved stubs under `src/compensation/` | #5, epic #14 |
| C-20..C-21 | Batch and incremental conformance | planned | Reserved stubs under `src/conformance/` | #6, epic #14 |
| C-22 | Six independent claim dispositions | landed | PR #41 | #7, epic #14 |
| C-23..C-24 | Claim premise retention and dispatch | planned | Reserved stubs under `src/claims/` | #7, epic #14 |
| C-26..C-28 | Canonical result, global premises, and lineage | planned | Reserved stubs under `src/result/` | #8, epic #14 |
| C-29 | Count, allocation, and deadline guards | landed | PR #42 | #10, epic #14 |
| C-30 | Source-bound resource-site inventory | planned | `src/bounds/inventory.rs` is an intentional stub | #10, epic #14 |
| C-31 | Reproduction record | landed | PR #38 | #10, epic #14 |
| C-32 | Shared Rust assurance-boundary enforcement | not landed / blocked | PR #32 closed without merge; Task-011 forbids a local replacement and waits for the shared checker tracked by `quire-research#64` | #13, epic #14 |

## Current executable boundary

C-05 is the last independently executable refusal increment at the current
owner revisions. C-02 and C-03 cannot proceed until the upstream catalog and
cause contracts are adopted. The remaining Plan-001 work follows the task
dependency graph; this register does not authorize work past a blocked owner.
