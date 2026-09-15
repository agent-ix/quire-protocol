---
id: SR-104
title: "Gap analysis of the Task-013 version-lock resolver"
type: SpecReview
analysis: gap-analysis
scope: "Task-013 / quire-protocol#49, FR-009/FR-012/FR-016/NFR-006 resolver slice, TC-013, Plan-001 and TM-001"
review_set: subset
relationships:
  - { target: ix://agent-ix/quire-protocol/Task-013, type: reviews }
  - { target: ix://agent-ix/quire-protocol/Plan-001, type: reviews }
  - { target: ix://agent-ix/quire-protocol/TM-001, type: references }
  - { target: ix://agent-ix/quire-protocol/TC-013, type: reviews }
---
# SR-104: Gap Analysis of the Task-013 Version-Lock Resolver

## Summary

Task-013's exact allocation is complete and semantically aligned: its resolver,
private capability boundary, three resource dimensions, refusal axes and
TC-013 evidence exist and agree. The review also removed false DeliveryIndex
credit and assigned that already-specified object to Task-019/#66 and TC-017,
so this subset PASS does not conceal wider Plan-001 work.

## Verdict

**PASS for Task-013 / quire-protocol#49.** No in-scope plan, matrix, semantic,
trace, reverse-code, stub or validation gap remains. This is not a PASS for
FR-009's concrete integrations, Task-019, or Plan-001 as a whole.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-001 | high | Resolved: TM-001 and TC-013 pre-credited FR-018/NFR-006-AC-8 although neither `main` nor the Task-013 candidate contained a `DeliveryIndex`. Issue #66, Task-019 and TC-017 now own the existing architecture, and Quire reports those rows unbacked. | FR-018; NFR-006-AC-8; #66; TC-017 | correct-requirement-no-evidence |
| FND-002 | medium | Resolved: executable FR-012 identity/sidecar and NFR-006 exact-member controls were described in prose without complete criterion tags. Real assertions now bind FR-012-AC-1/AC-5 and NFR-006-AC-6/AC-7; no tag was added for behavior the tests do not execute. | TC-013; TM-001 | correct-requirement-no-evidence |
| FND-003 | medium | Resolved: key-only `ManifestEntry` equality made two state-retention assertions too weak. Tests now compare key, kind, status, revision, assertion, reason, member and digest independently, and aggregate equality was removed. | FR-012-AC-1/3/8; `assert_entry_exact` | implementation-bug-despite-evidence |

## Target Selection

- Target task: `plan/Plan-001-finite-protocol-rust-slice/tasks/Task-013-lock-producer-revisions-and-compose-evidence.md` in active Plan-001.
- Ticket/PR: quire-protocol#49 / PR #50, current body controlling.
- Specification and evidence: FR-009, FR-012, FR-016-AC-8,
  NFR-006-AC-6/AC-7, TC-013 and TM-001.
- Production/test surface: `src/version_lock.rs`,
  `src/version_lock/tests.rs`, shared resource/refusal inventories, TC-009 and
  TC-012 frozen inventories, plus the library/dependency source inventory.
- Baseline: `origin/main` at
  `0cba7599a64162ade00cf08c01638b6c4cbe20b5`.

## Plan and Matrix Reconciliation

| Task-013 obligation | Executed evidence | Result |
| --- | --- | --- |
| Two-status exact-key manifest | Public `EntryKey`, `ManifestEntry`, `EntryStatus`, exact state accessors; absent/unpinned/unresolved/duplicate and two-version tests | Complete |
| Deterministic required-key resolution | Lexicographic required-key validation; success and multi-error offered-order permutations; exact first key and no partial output | Complete |
| Exact artifact bytes | Exact delivered-byte digest comparison, unchanged retained bytes, missing/unexpected/duplicate/substituted member controls | Complete |
| Authentic interface selection | Independently typed selection, exact owner/version/revision and subject checks, no synthesized digest | Complete |
| Request/value-bound capabilities | Constructor-private typed artifact/interface capabilities; cross-request, key, revision and subject reuse controls | Complete |
| Three independent bounds | Shared entry-count, bytes-digested and resolution-deadline sites; exact, one-short and fresh attempts plus fallible constructor boundaries | Complete |
| Offline containment | Resolver accepts only values and has no filesystem, VCS, network, process, discovery, sidecar or delivery-index authority surface | Complete |
| Tracking | Task/checklist, plan, index, log, TC-013, TM-001, #49 and epic #14 reconciled; DeliveryIndex allocated separately to #66/Task-019/TC-017 | Complete |

Task completion is 1/1 for this allocation. FR-009 remains an in-progress
aggregate because Tasks 009–011 still own concrete producer entry population
and Task-018 owns final evidence composition.

## Criterion and Semantic Review

The semantic review compared each executed assertion with the exact criterion,
not merely with its tag. FR-009-AC-1 through AC-9 all have direct resolver
controls. FR-012-AC-1 through AC-8 have direct exact-state/identity controls.
FR-016-AC-8 has all three independent one-short resource outcomes, and
NFR-006-AC-6/AC-7 have unenumerated-member and same-path byte-substitution
controls with distinct outcomes. No FR-018 or NFR-006-AC-8 tag remains.

The refusal tests inspect stage, partial-output state, exact code, affected
identity and the cause interior where ambiguity was possible. Success tests
inspect every entry state field and the exact released bytes. That closes the
intent-to-test gap rather than only the string-to-tag gap.

## Coverage

- `quire coverage --scope . --json`: 144/342 declared rows backed.
- Rust binding census: 247 candidates, 247 tagged, 247 bound.
- Task-013/TC-013 relevant unmatched tags: 0; relevant status lies: 0.
- FR-009-AC-1..AC-9: all resolver controls bound.
- FR-012-AC-1..AC-8: all bound and executed.
- FR-016-AC-8 and NFR-006-AC-6/AC-7: bound and executed.
- TC-017, FR-018's six Test-designated criteria and NFR-006-AC-8: unbacked and
  explicitly planned under Task-019/#66, exactly as current code reality requires.

Repository-wide unbacked/inspection and unrelated TC-010/TC-014 binding gaps
remain visible in the same output and are outside this ticket. None is used to
claim Task-013 complete.

## Reverse-Gap and Stub Audit

The reverse inventory covered exact bytes, entry key/status/kind/state,
manifest, delivery member, resolution budget/error, request identity, adapter
selection, all seventeen refusal codes, resolved request state, artifact and
interface capabilities, three source sites and the shared deadline/accounting
extension. Each maps to FR-009, FR-012, FR-016-AC-8 or NFR-006-AC-6/AC-7. No
unowned production type, behavior, fallback, approximation, parser, callback,
I/O seam or local owner-contract mirror was introduced.

Searches over the changed production and test surface find no `todo!`,
`unimplemented!`, `dbg!`, placeholder success, ignored Task-013 test or unsafe
block. Production panics are limited to audited post-validation or static
catalog/inventory invariants. The constructor-private path is deliberately live
only in internal tests until Tasks 009–011 consume it, with narrow lint
expectations that fail when stale.

## Validation

The final committed candidate passes the exact repository local gate in order:
format check, locked all-target Clippy with warnings denied, then the complete
locked test suite, all under one Cargo job, the worktree-local heavy lock and
`target-codex-backends`. `quire validate` passes over specification, plan and
review artifacts. Cargo.lock is byte-identical before and after the gate, no
hosted workflow ran, and `resources/native-v1/` is untouched. Exact head and
command output are retained in PR #50 evidence.
