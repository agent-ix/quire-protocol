---
id: TC-152
title: "Consume temporal evidence through the in-memory evidence types"
type: TC
relationships:
  - target: ix://agent-ix/quire-protocol/FR-046
    type: verifies
---
# TC-152: Consume temporal evidence through the in-memory evidence types

## Description

Verify the assessment over QSL's in-memory evidence entry and the caller-owned
request boundary.

## Test Procedure

For each of **R**'s four temporal clauses, construct the `EvidenceRequest` at
the caller boundary, pass it through the checked-package evidence entry, and
compare the returned `EvidenceResult` fields with the assessment record. Verify
that the request and result remain in-memory values, and that the boundary
accepts no retired temporal request/result types. Exercise each of the six
FR-046-AC-4 causes reachable from a caller-built request over an admitted
package. Confirm the reader refuses duplicate temporal-clause name rows as
`AmbiguousName`, so no admitted package reaches the `PackageMismatch`
construction refusal. Run the IT-002 scenarios on the evidence route with an
unavailable and a withdrawn anchor, and compare the position values supplied by
the assessment with those read by QSL. Tag each test with the exact acceptance
criterion it verifies using `#[trace("FR-046-AC-n", "TC-152")]`; record the
AC-5 duplicate-row analysis under FR-043's reader evidence.

## Expected Results

The assessment record equals the QSL result; the caller-supplied request is
passed unchanged as an in-memory value; the six reachable causes retain their
names with no charge; duplicate temporal-clause name rows are refused by the
reader before assessment; unavailable and withdrawn anchors never become
Boolean truth; and the position values QSL reads equal those the assessment
supplies.
