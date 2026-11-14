---
id: TC-170
title: "Run the timed-refund and compensated fan-out reference workflows end to end"
type: TC
relationships:
  - target: ix://agent-ix/quire-protocol/FR-023
    type: verifies
  - target: ix://agent-ix/quire-protocol/FR-100
    type: verifies
---
# TC-170: Run the timed-refund and compensated fan-out reference workflows end to end

## Description

Verify one end-to-end run of each reference workflow: the timed refund (a wait settles on its timeout and the order is compensated by a retried refund) and the branching compensated fan-out and join (an unsettled join compensates each completed branch).

## Test Procedure

Settle the wait or assess the join over a compiled protocol, register compensation for the established effects, admit failed and successful attempts under the authored bound, and retain the established effects.

## Expected Results

The compensation starts only on the timeout branch (a response starts none) and, in the fan-out run, only for the branches the join observed complete; effects are read from the attempt ledger; both attempts of a retried compensation are retained, one effect is established, a third attempt is refused by the bound, and no duplicate effect is reported.
