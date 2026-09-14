---
id: SR-082
title: "Gap analysis of the QSL temporal compatibility repin"
type: SpecReview
analysis: gap-analysis
scope: "Plan-001 Tasks 009/013 allocation for quire-protocol#56 at 3e5ddbd"
review_set: subset
relationships:
  - { target: ix://agent-ix/quire-protocol/Plan-001, type: reviews }
  - { target: ix://agent-ix/quire-protocol/Task-009, type: reviews }
  - { target: ix://agent-ix/quire-protocol/Task-013, type: reviews }
  - { target: ix://agent-ix/quire-protocol/NFR-006, type: reviews }
  - { target: ix://agent-ix/quire-protocol/TM-001, type: references }
  - { target: ix://agent-ix/quire-protocol/IT-001, type: references }
---
# Gap analysis of the QSL temporal compatibility repin

## Summary

The issue 56 allocation is complete at
`3e5ddbd3acde38f783c07f8a05fb8361d9185705`: the promoted owner revision is
selected consistently, its downstream public API compiles unchanged, the real
dependency inventory controls pass, and reverse inspection found no new
behavior, unowned vocabulary, source/test stub or hidden adapter. This bounded
result does not relabel Plan-001 or Task-013 complete.

## Verdict

**PASS for the issue 56 allocation** — the compatibility repin has no remaining
plan, matrix, trace, semantic or reverse-code gap in its bounded scope.
Plan-001 and Task-013 remain explicitly in progress for their wider feature
sets.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No unresolved gaps in the issue 56 compatibility allocation | - |

## Plan and task reconciliation

| Obligation | Evidence | Status |
| --- | --- | --- |
| Consume the promoted QSL owner rather than its candidate commit | Exact `7931a002...` manifest and lockfile selection | Complete in candidate |
| Preserve one owner implementation and one resolved frontend | IT-001 manifest/lock/source inventory tests | Complete in candidate |
| Keep build-closure rights/provenance records accurate | `DEPENDENCIES.md` agrees with the resolved QSL, Contract-IR and baseline-producer revisions | Complete in candidate |
| Prove downstream source compatibility | All library, integration and doctest targets compile and pass against the promoted QSL revision | Complete in candidate |

Task-009 and Task-013 remain `in_progress`; their unrelated compiler scenarios,
version-lock mechanism, delivery-index work and composed evidence are outside
issue 56 and are not marked done. The Plan-001 checkboxes are therefore left
unchanged.

## Semantic reconciliation

- **Intent to implementation:** NFR-006 requires an exact immutable dependency
  selection and matching closure records. The manifest, lockfile and rights
  record now name the same promoted QSL revision, while the lock resolves its
  exact transitive producer and Contract-IR revisions.
- **Implementation to test:** `it_001_declares_only_selected_owner_dependencies`
  requires the exact QSL pin in the real manifest;
  `it_001_resolves_one_frontend_in_the_whole_graph` requires that same revision
  in exactly one lockfile source; the full suite compiles the actual QProtocol
  API against it.
- **Test to intent:** deleting or changing the manifest pin, introducing a
  second QSL source, or resolving a different QSL revision fails the existing
  controls. Compilation against the restored non-generic QSL authority kills
  the public-lifetime regression that opened issue 56.
- **Owner boundary:** QProtocol adds no local temporal authority or fallback.
  The QSL merge supplies the implementation; QProtocol only fixes which exact
  owner revision its build consumes.

## Reverse-gap audit

- Changed public production behaviors: 0.
- New public types, schemas, fields, parsers, evaluators or adapters: 0.
- New or changed tests: one existing exact revision constant advanced; its
  three real inventory controls remain traced to IT-001/Task-009 and pass.
- Source/test stubs, ignored tests or weak assertions introduced: 0.
- Unowned constraints introduced: 0. Exact build-closure selection is owned by
  NFR-006 and Plan-001 Tasks 009/013.

## Coverage

- Reconciliation: `quire coverage` using CLI 0.32.0 / engine
  `a874fb641cb70da83c8c8b23f9fea0a44255b88a`.
- Tasks done: issue 56 allocation 4 / 4; Plan-001 and Task-013 not claimed
  complete.
- Rows backed by a tagged test: 108 / 329 repository-wide; this is the wider
  active Plan-001 state, not the completion percentage of this repin.
- Rust evidence symbols: 235 / 235 tagged and bound.
- Untraced behaviors / stubs introduced by this change: 0.
- Semantic review: ran over the exact NFR-006/IT-001 dependency-selection
  intent, tests and changed build records.

## Verification

- Exact NFR-004 gate: pass; 219 regular tests plus 37 doctests passed, 0
  failed, with 23 pre-existing named fixture ignores.
- The lockfile was unchanged by the locked Clippy and test runs.
- Changed spec document: Quire-valid.
- Code/Rust review: SR-081, pass with every review-cycle finding fixed.

## Disposition

Promote the exact QProtocol candidate, record its merge revision on issue 56,
and repin the downstream Contract-IR implementation before resuming its native
temporal projection/join work.
