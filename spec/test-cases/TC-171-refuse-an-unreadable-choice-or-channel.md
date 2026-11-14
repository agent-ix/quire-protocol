---
id: TC-171
title: "Refuse a package whose choice owner or channel sender is unreadable"
type: TC
relationships:
  - target: ix://agent-ix/quire-protocol/FR-051
    type: verifies
  - target: ix://agent-ix/quire-protocol/FR-052
    type: verifies
---
# TC-171: Refuse a package whose choice owner or channel sender is unreadable

## Description

Verify that a package whose choice `owner`, or channel `from`, is not a role reference is refused by Contract IR's reader as a malformed wire, so the unreadable choice or channel is never linked.

## Test Procedure

Emit a three-role protocol with QSL's writer, replace the `owner` argument of its choice, and separately the `from` argument of its channels, with a text literal (removing the stale dependency), and read each package with Contract IR's reader.

## Expected Results

Each package is refused with `MalformedWire`; none is admitted, so the link's own view refusals are never reached from an admitted package.
