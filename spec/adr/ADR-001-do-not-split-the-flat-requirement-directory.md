---
id: ADR-001
title: "Do not split the flat requirement directory"
type: ADR
status: accepted
---
# ADR-001: Do not split the flat requirement directory

## Status

Accepted.

## Context

`agent-ix/quire-specification` reorganized `spec/functional/` into owner-aligned
subsystem directories (`foundation/`, `protocol/`, `temporal/`, `observation/`)
on `origin/main` at `bd387dd`. This repository was asked whether to mirror that
layout.

Symmetry with an upstream repository is not a reason to move files. The question
was measured instead, and the measurement changed the answer.

### What upstream's split was actually driven by

A prior brief described upstream's trigger as "13 of 38 FRs double-claimed and 5
of 8 subsystems owner-spanning". That measurement does not exist upstream. A
search over `origin/main` for double-claim language returns nothing, and
upstream's own commit message records a different driver:

```
functional/foundation/    41  A  FR-001..020, FR-030..049, FR-130
functional/protocol/      13  B  FR-050..062
functional/temporal/       6  E  FR-090..095
functional/observation/    7  F  FR-110..116
```

67 requirements, four subsystems, and **every one maps to exactly one owner** —
read off a pre-existing `docs/functional-capability-ledger.md` that already
assigned identifier ranges to workstreams. Upstream did not split because
subsystems overlapped. It split because a structure that already existed, with
one owner per range, was invisible in the layout. That is a different
justification, and it is the one that decides this question.

### The equivalent measurement here

Candidate subsystems derived from what FR-001..FR-009 actually cover:

| Candidate | FRs it would specify | Owner |
| --- | --- | --- |
| Admission / contract intake | FR-001 | B |
| Linking / finite representation | FR-002 | B |
| Compensation assessment | FR-003 | B |
| Global conformance assessment | FR-004 | B |
| Claim dispatch | FR-005 | B |
| Canonical result production | FR-006 | B |
| E02 projection | FR-007 | B |
| Refusal / diagnostics | FR-008 | B |
| Producer-revision locking | FR-009 | B |
| Temporal binding intake | FR-002 only, via FR-002-AC-5 | B, consuming E |
| Observation intake | FR-004 only, via IT-002 | B, consuming F |

- **FRs claimed by more than one candidate subsystem: 2 of 9.** FR-002 and
  FR-004.
- **Candidate subsystems spanning more than one owner: 0 of 9.** Every
  requirement under `spec/` is B-authored and B-owned. A, E, F, Engineering
  Assurance, E02 and Quoin appear only as external producers behind FR-009's
  version-lock manifest and in IT-001..IT-003; none owns an artifact here.
  FR-009 names seven external owners and is still single-owner, because what it
  specifies is B's manifest, not their contracts.

## Decision

Keep `spec/functional/` flat.

## Consequences

Upstream's trigger — many requirements across several owning workstreams, with a
published ledger already assigning them — is absent here. This repository has
nine requirements, one owner, one crate, and a directory that costs nothing to
scan.

The two double-claimed requirements are not evidence of subsystem tension. They
are evidence that temporal intake and observation intake have no requirement of
their own: each exists only as an acceptance criterion on another. A directory
move would not fix that, and would obscure it by creating `temporal/` and
`observation/` directories holding zero requirements.

A split now would produce four directories holding nine files, at least two of
them empty, and would rewrite every relative link in the repository for an owner
boundary that does not exist.

Revisit when a second owner authors a requirement in this repository, or when the
requirement count passes roughly 25. Recording the numbers here is the point: the
question should not return as symmetry pressure.

## Alternatives considered

**Mirror upstream's four directory names.** Rejected: it copies a layout whose
justification — one owner per range, already published in a ledger — does not
hold here, and it creates empty directories.

**Split by lifecycle stage** (admission, linking, assessment, result). Rejected
for the same reason and a worse one: FR-002 and FR-004 each straddle two stages,
so the split would raise the double-claim count rather than lower it.
