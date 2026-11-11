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

Verify that the backend receives, and the disposition retains, the exact channel premises and role internality, that an omitted per-claim premise refuses only its disposition, and that an omitted premise or an `Inconclusive` outcome yields no truth `satisfied` or `violated`.

## Test Procedure

Dispatch both claim kinds over a checked package with channels and internal and external roles, using a call-recording stand-in backend registered in QSL's registry snapshot. Vary one component of one channel premise; omit one channel premise and one role internality from the requester's per-claim premise set in turn, then restore each; have the stand-in return the `Inconclusive` verdict with an `InconclusiveCause` and a requires-bound outcome. Tag each test with the exact acceptance criterion it verifies.

## Expected Results

Recorded calls carry the authored premises and nothing else, with `external` unchanged. Request identity and provenance differ per varied component. An omitted premise refuses that disposition under FR-030 with no backend call, retains no value for it and leaves independent dispositions unchanged; restoring it sends exactly the restored value. `Inconclusive`, requires-bound and premise-absent dispositions hold neither `satisfied` nor `violated`, and the library supplies no bound.
