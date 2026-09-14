---
id: TC-015
title: "Produce, read and map canonical protocol results"
type: TC
relationships:
  - { target: ix://agent-ix/quire-protocol/FR-042, type: verifies }
---
# TC-015: Produce, read and map canonical protocol results

## Description

Planned complete result-owner and selected-mapping checks for FR-042. No new
implementation coverage is claimed.

## Test Procedure

Rust combination/property tests enumerate the valid canonical-result table and
each one-axis invalid mutation, recompute FR-286 identities/JCS digests, replay
strict-reader mutations, enumerate the four progress/closure axes independently
of completeness, and derive/read the selected Contract-IR mapping for every
execution/truth/settlement row. Exact and one-over limits cover every bounded
collection and byte/depth/work dimension.

## Expected Results

Only exact canonical source bytes return `ValidatedResult`; only the mapping
re-derived from that view returns `MappedResultView`. Completed+satisfied and
completed+violated are the only Boolean rows. Every other row is a lossless
typed non-value, and no refusal exposes a partial view.

## Status

Planned for `quire-protocol#8`.
