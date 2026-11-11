---
id: TC-161
title: "Bind communication and visibility premises"
type: TC
relationships:
  - target: ix://agent-ix/quire-protocol/FR-151
    type: verifies
---
# TC-161: Bind communication and visibility premises

## Description

Verify that the backend receives, and the disposition retains, the exact channel premises and role internality, that a missing premise refuses only its disposition, and that a missing premise or non-conclusive outcome yields no truth `satisfied` or `violated`.

## Test Procedure

Dispatch both claim kinds over a checked package with channels and internal and external roles, using a call-recording stand-in backend. Vary one component of one channel premise; remove one channel premise and one role internality in turn; have the stand-in return a requires-bound outcome and a non-conclusive result. Tag each test with the exact acceptance criterion it verifies.

## Expected Results

Recorded calls carry the authored premises and nothing else, with `external` unchanged. Request identity and provenance differ per varied component. A removed premise refuses that disposition under FR-030 with no backend call and leaves independent dispositions unchanged. Non-conclusive and premise-absent dispositions hold neither `satisfied` nor `violated`, and the library supplies no bound.
