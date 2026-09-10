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
profile, feature, and numeric-wire mutation.

## Test Procedure

Build one valid retained compiled-artifact fixture after the upstream candidate
and artifact contract are accepted. Admit it, then independently mutate every
identity-bearing field, artifact-contract version, and producer revision; replace
its bytes while resealing an internal field; select every non-promoted candidate
state; and remove each required feature. For every compiled integer and exact
rational field, exercise the declared minimum and maximum plus zero, then
substitute the signed-64 extrema in the accepted exact form. Independently
mutate those values to an unsafe bare JSON integer, float, negative zero,
leading-zero or plus-prefixed decimal, signed-64 overflow, zero/negative rational
denominator, non-reduced rational and a value that changes across RFC 8785
serialization and decoding. Replace a producer-owned model/configuration digest
with an otherwise similar artifact-domain JCS digest.

## Expected Results

Only the exact accepted fixture under the accepted compiled-artifact contract is
admitted. Every mutation refuses before semantic evaluation with the affected
identity and no parser or locally substituted profile. Every accepted numeric
value decodes to the exact native value and re-encodes to identical canonical
artifact bytes; no producer-owned digest is reinterpreted or equated across
domains.
