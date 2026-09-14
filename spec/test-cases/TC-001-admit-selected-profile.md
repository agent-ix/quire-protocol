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

The executable closure target is the version 2 handoff published inside the
exact pinned `quire-spec-language` dependency. Its inherited version 1 package
is used only as the explicit opposite-entry-point control. The legacy generated
version 1 corpus remains IT-001/Task-009 work and is not required to make TC-001
run in the default gate.

## Test Procedure

Load the owner-published handoff through its exported address and verify its
closed checksummed inventory before admission. Construct the expected selection
from the retained original inputs, never from the offered bytes. Admit the exact
version 2 offer, then submit every owner-published one-axis mutation and require
its declared typed refusal. Independently substitute the accepted baseline's
identity, revision and digest in the caller selection while leaving the offer
unchanged, and require selection refusal with no admitted package.

Exercise the base package's integer and exact-rational zero, field endpoints and
signed-64 endpoints. Reseal independent one-axis mutations for an unsafe bare
JSON integer, float, negative zero, leading-zero or plus-prefixed decimal,
signed-64 overflow, zero/negative rational denominator, non-reduced rational,
wrong numeric kind and below/above field domain. Replace a producer-owned
model/configuration digest with a result-domain digest and require domain-
specific refusal. Submit the inherited version 1 package at the version 1 entry
point and submit each version's bytes at the other entry point without detection
or translation; the cross-version regression comparison remains IT-001/Task-009.

## Expected Results

Only the exact accepted fixture under the accepted compiled-artifact contract is
admitted. Every mutation refuses before semantic evaluation with the affected
identity and no parser or locally substituted profile. Every accepted numeric
value decodes to the exact native value and re-encodes to identical canonical
artifact bytes; no producer-owned digest is reinterpreted or equated across
domains.

## Traceability
TC-001 closes FR-001-AC-1 through FR-001-AC-4. FR-001-AC-5 remains bound to
IT-001 because its complete two-revision version 1 comparison is not part of
this test case. TC-001 additionally backs criteria on object requirements
FR-013 and FR-015, which cite it as their verification. Those criteria are
exercised by the same procedure above; a change here that stops exercising them
is a change to their coverage, not only to this test case's.
