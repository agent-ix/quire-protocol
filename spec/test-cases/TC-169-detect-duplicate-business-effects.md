---
id: TC-169
title: "Detect duplicate business effects independently of delivery"
type: TC
relationships:
  - target: ix://agent-ix/quire-protocol/FR-023
    type: verifies
---
# TC-169: Detect duplicate business effects independently of delivery

## Description

Verify that a duplicate business effect is decided from effect identity alone, so a redelivered observation of one effect is not a duplicate and a second distinct effect for one registration is.

## Test Procedure

Retain the effects observed for one registration as one effect, the same effect twice, two distinct effects, and a mixed sequence with a redelivery.

## Expected Results

The first effect is established; a distinct second effect is a duplicate; a repeated identity is retained once; redelivery never changes the verdict.
