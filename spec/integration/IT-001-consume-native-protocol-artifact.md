---
id: IT-001
title: "Consume the native compiler protocol artifact"
type: IT
relationships:
  - target: ix://agent-ix/quire-protocol/FR-001
    type: verifies
  - target: ix://agent-ix/quire-protocol/FR-002
    type: verifies
---
# IT-001: Consume the native compiler protocol artifact

## Objective

Verify that the native Quire compiler's selected protocol artifact is admitted
and linked through the public Rust boundary without a copied parser, alternate
formal frontend, or identity reinterpretation.

## Target Integration

The producer is the A-owned native Quire compiler artifact interface. The
consumer is this library's strict protocol admission and linking API.

## Preconditions

The composed baseline has an accepted immutable protocol candidate, and the
native compiler publishes an accepted versioned compiled-protocol artifact
contract plus exact artifact bytes, producer revision, source revision,
selected-rule digests, dependency closure, profile, and required features.
The producer contract also declares one lossless numeric wire profile for native
signed-64 integers and exact rationals before the consumer reads semantic fields.

## Inputs

One accepted compiled finite protocol plus changed-version, changed-digest,
unknown-feature, ambiguous-binding, stale-candidate, unsafe-number,
non-canonical-exact-number and cross-domain-digest variants.

## Test Procedure

1. Submit the exact accepted artifact to the public Rust admission API.
   - IT-001-SC-01: admission succeeds and retains every producer identity.
2. Link the admitted artifact.
   - IT-001-SC-02: the finite graph and source references match the producer contract.
3. Submit each one-axis producer-contract mutation.
   - IT-001-SC-03: every mutation refuses before semantic evaluation.
4. Submit exact signed-64 and rational boundaries plus each invalid or lossy
   numeric representation.
   - IT-001-SC-04: exact values round-trip byte-identically; unsafe, ambiguous,
     out-of-range and cross-domain forms refuse before semantic evaluation.
5. Inspect the dependency and source inventory.
   - IT-001-SC-05: no second parser, formal frontend, or foreign runtime participates.

## Expected Results

Only the exact selected artifact is admitted and linked. Every mismatch is a
typed refusal at the affected identity, and the integration uses the native
producer contract directly.

## Metadata

- Priority: P0
- Target Integration: native compiler artifact to protocol library
- Automation: Automated after the producer interface is accepted

## Dependencies

The A-owned immutable profile candidate and versioned compiled-protocol artifact
contract must both be accepted. A prose definition or native source file cannot
substitute for the compiled interface because this repository owns no parser.

## Notes

The current composed baseline remains draft and existing candidate `1-draft.3`
predates integrated FR-061/FR-062 bytes, so this test is planned rather than run.

## Traceability

Verifies FR-001 and FR-002.
