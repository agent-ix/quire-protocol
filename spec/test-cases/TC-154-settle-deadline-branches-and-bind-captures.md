---
id: TC-154
title: "Settle deadline branches and bind captures"
type: TC
relationships:
  - target: ix://agent-ix/quire-protocol/FR-100
    type: verifies
  - target: ix://agent-ix/quire-protocol/FR-101
    type: verifies
---
# TC-154: Settle deadline branches and bind captures

## Description

Verifies the TL-6 delta over FR-019 and FR-028 (ADR-010). Unsupported constructs (ADR-010 gap G7) are verified by FR-057's test case, not here.

## Test Procedure

Tag each test `#[trace("TC-154", "FR-1nn-AC-n")]`. Deadline: responses at, after and before the deadline, with each combination of progress open/closed and completeness complete/incomplete, a false constraint, a valueless constraint (an unavailable slot outcome and a non-Boolean value), a response observed before the window's lower bound, an `after` observation that is absent, admitted late, offered twice or on another clock family, a response arriving after timeout but observed at the deadline, a response observed after the deadline, a false or valueless observation and a regressed progress or completeness fact admitted after timeout, a progress view for another boundary, a control that is not a declared `await`, completeness contradicted, progress open together with incomplete or contradicted completeness, and two waiting instances of one branch. Captures: ordered, duplicated, mutated-source, wrong-boundary and one-undefined cases, and a supplied value that differs from the local expression result (AC-5).

## Expected Results

Each criterion's observable holds and no business operation is invoked.
