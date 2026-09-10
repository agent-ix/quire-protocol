---
id: TC-003
title: "Assess compensation, retry, commit, and recovery"
type: TC
relationships:
  - target: ix://agent-ix/quire-protocol/FR-003
    type: verifies
---
# TC-003: Assess compensation, retry, commit, and recovery

## Description

Verify compensation registration and recovery across successful/failed effects,
duplicates, retries, commits, partial recovery, missing facts, and finite bounds.

## Test Procedure

Replay one successful effect, failed attempt, duplicate delivery, early
compensation, late registration, forbidden post-commit compensation, timed-out
retry, duplicate effect, partial recovery, and missing population/completeness case.

## Expected Results

Only the paired successful effect registers compensation. Each attempt/effect and
ordering outcome remains distinct; recovery follows only the authored relation;
missing authority cannot become success; no business I/O occurs.

