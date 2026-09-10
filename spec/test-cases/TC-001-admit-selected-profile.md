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
one-axis artifact-contract, producer, accepted source-baseline, revision, digest,
dependency, profile, feature, and numeric-wire mutation.

## Test Procedure

Build one valid retained compiled-artifact fixture after the upstream baseline
and artifact contract are accepted. Admit it, then independently mutate every
identity-bearing field, artifact-contract version, and producer revision; replace
its bytes while resealing an internal field; select every unaccepted baseline
state; and remove each required feature. For every compiled integer and exact
rational field, exercise its own declared minimum and maximum plus zero. Use
separate fields whose declared source domains admit the signed-64 minimum and
maximum, and encode those extrema in the accepted exact form. Independently
mutate the values below a field minimum, above a field maximum, to an unsafe bare
JSON integer, float, negative zero, leading-zero or plus-prefixed decimal,
signed-64 overflow, zero/negative rational denominator, non-reduced rational and
a value that changes across RFC 8785 serialization and decoding. Replace a
producer-owned model/configuration digest with an otherwise similar
artifact-domain JCS digest.

## Expected Results

Only the exact accepted fixture under the accepted compiled-artifact contract is
admitted. Every mutation refuses before semantic evaluation with the affected
identity and no parser or locally substituted profile. Every accepted numeric
value decodes to the exact native value and re-encodes to identical canonical
artifact bytes; no producer-owned digest is reinterpreted or equated across
domains.
