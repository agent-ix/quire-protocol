---
id: TC-011
title: "Return typed refusals without text parsing"
type: TC
relationships:
  - target: ix://agent-ix/quire-protocol/FR-008
    type: verifies
---
# TC-011: Return typed refusals without text parsing

## Description

Verify the refusal envelope across every public failure domain, source catalog,
code, stage/prerequisite, identity/location, cause, rendering, unknown-version,
simultaneous-disposition, and partial-output rule.

## Test Procedure

Generate one failure from every public operation and selected source code. Mutate
the domain, diagnostic-catalog identity/revision/digest, code, originating stage,
prerequisites, affected identity, source location, cause order, display text,
locale, source version, and partial-output state independently. Add an unknown
future code and a failed advanced claim beside valid finite conformance. Pass a
failure through a wrapper that erases its stage, remove source context, and use a
repeated display name to attempt location reconstruction.

Exercise paired causes for missing versus conflicting definition, wrong anchor
versus missing observation, unknown syntax versus prohibited recognized
construct, unproved definedness versus runtime logical violation, and resource-
incomplete versus unsupported execution. Combine an independent invalid-input
and missing-input disposition, then set the diagnostic-detail budget to zero.

## Expected Results

Typed fields alone determine identity and behavior. Catalog or stage drift
refuses; unknown contracts retain their opaque original selection/value behind
the local unsupported classification; absent context stays absent; paired and
simultaneous causes remain distinct even with no diagnostic detail. Fatal
operations emit no success value, and only explicitly independent claim results
survive another claim's failure.

## Traceability
This test case additionally backs criteria on the object requirements FR-014, FR-016, which cite it as their verification. Those criteria are exercised by the same procedure above; a change here that stops exercising them is a change to their coverage, not only to this test case's.
