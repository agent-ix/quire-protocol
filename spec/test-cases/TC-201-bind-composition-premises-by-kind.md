---
id: TC-201
title: "Bind composition dispositions to their premise lists and component subjects"
type: TC
relationships:
  - target: ix://agent-ix/quire-protocol/FR-201
    type: verifies
---
# TC-201: Bind composition dispositions to their premise lists and component subjects

## Description

Verify separate premise lists, subject binding and undischarged-slot retention, with a stand-in for the QSL call boundary and no local analysis.

## Test Procedure

Request a composition claim with entries in all four premise lists. Feed stand-in results that name an unknown slot, name an unknown subject, return a slot undischarged although a supplied guarantee would discharge it by a local reading, and return an empty discharge record. Drop or merge one list from the disposition. Tag each test with the exact acceptance criterion it verifies.

## Expected Results

Four separate byte-equal lists reach the stand-in and stay in the disposition; unknown slot or subject refuses the disposition with `identity-mismatch` naming it; undischarged slots stay undischarged; no slot is marked discharged by the library.
