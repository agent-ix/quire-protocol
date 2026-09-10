---
id: TC-001
title: "Admit only the selected protocol profile"
type: TC
relationships:
  - target: ix://agent-ix/quire-protocol/FR-001
    type: verifies
---
# TC-001: Admit only the selected protocol profile

## Description

Verify strict admission for exact compiled protocol bytes and refusal for every
one-axis artifact-contract, producer, candidate, revision, digest, dependency,
profile, and feature mutation.

## Test Procedure

Build one valid retained compiled-artifact fixture after the upstream candidate
and artifact contract are accepted. Admit it, then independently mutate every
identity-bearing field, artifact-contract version, and producer revision; replace
its bytes while resealing an internal field; select every non-promoted candidate
state; and remove each required feature.

## Expected Results

Only the exact accepted fixture under the accepted compiled-artifact contract is
admitted. Every mutation refuses before semantic evaluation with the affected
identity and no parser or locally substituted profile.
