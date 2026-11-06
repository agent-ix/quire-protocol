---
id: Task-022
title: "QSL-1740 — complete the checked-package linker residuals"
type: Task
status: in_progress
track: Core
priority: P0
relationships:
  - { target: ix://agent-ix/quire-protocol/Task-002, type: depends_on }
  - { target: ix://agent-ix/quire-protocol/Task-017, type: depends_on }
  - { target: ix://agent-ix/quire-protocol/FR-044, type: references }
  - { target: ix://agent-ix/quire-protocol/TC-150, type: verifies }
---
# Task-022: QSL-1740 — Complete the Checked-Package Linker Residuals

## Scope

Finish FR-044 over the admitted `CheckedPackageV2` after QSL-1549's stage-2
subset. Keep owner facts behind Contract IR and QSL accessors, add the consumer
refusals and the no-window package boundary, and preserve recipe-R behavior.
The work lands as one QSL-1740 PR.

## Subtasks

- [ ] Extend `TC-150` first with positive controls, one-axis adverse cases, and
  exact `FR-044-AC-3`, `AC-6`, and `AC-7` trace tags. Cover zero maximum,
  duplicate role authority, conflicting channel premises, the absent package
  window member, recipe-R identity/interleaving behavior, and changed temporal
  binding selection.
- [ ] Implement a consumer-owned `LinkRefusal` that carries QSL `ViewRefusal`
  unchanged for owner-read failures and identifies offending checked node ids
  plus available QSL `locus` entries for consumer refusals.
- [ ] Reject zero-progress repeats, duplicate role authorities, and conflicting
  channel premises from checked node/term content without parsing bytes,
  duplicating a reader, constructing causal edges, or returning a partial
  `LinkedProtocol`.
- [ ] Treat windows only as FR-516 evaluation inputs. Read required clock rows
  through QSL `temporal_binding`; expose no package window binding or local
  compatibility type.
- [ ] Run focused `TC-150`, then the NFR-004 local gates in order; self-review
  Rust changes and run semantic gap-analysis, resolving findings before PR.

## Deliverables

Updated `src/link.rs`, focused `tests/tc_150_link_from_a_typed_read_view.rs`,
trace coverage for FR-044-AC-1..7, and the QSL-1740 PR.

## Notes

Contract IR owns checked-package decoding and node/term access. QSL owns
`protocol_view`, `causal_relation`, `control_operation`, `delivery`,
`queue_capacity`, `binder`, `temporal_binding`, `attempts_maximum`, and `locus`.
Only quire-protocol owns the three residual consumer refusals. `Window` is not a
package member and no first-party dependency lock is changed for this task.
