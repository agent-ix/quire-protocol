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

Verify the refusal envelope across every public failure domain, source code,
identity/location, cause, rendering, unknown-version, and partial-output rule.

## Test Procedure

Generate one failure from every public operation and selected source code. Mutate
the domain, code, affected identity, source location, cause order, display text,
locale, source version, and partial-output state independently; add an unknown
future code and a failed advanced claim beside valid finite conformance.

## Expected Results

Typed fields alone determine identity and behavior. Unknown contracts remain
unsupported, fatal operations emit no success value, and only explicitly
independent claim results survive another claim's failure.

