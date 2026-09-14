---
id: SR-064
title: "Gap analysis of Task-012 published version 2 handoff closeout"
type: SpecReview
analysis: gap-analysis
scope: "Task-012, IT-001 SC-06 through SC-09, TM-001, Cargo pins, src/, and tests/ on task/48-published-v2-handoff"
review_set: subset
relationships:
  - { target: ix://agent-ix/quire-protocol/Plan-001, type: reviews }
  - { target: ix://agent-ix/quire-protocol/TM-001, type: references }
  - { target: ix://agent-ix/quire-protocol/IT-001, type: references }
  - { target: ix://agent-ix/quire-protocol/Task-012, type: reviews }
---
## Summary

Task-012's issue #48 closeout is complete and its evidence agrees with the
implemented tree. The default local lane reaches the producer's committed
handoff through the exact dependency pin, verifies its closed inventory, and
executes the version 2 admission, mutation, version-isolation, and resource
scenarios. The wider Plan-001 remains intentionally incomplete; in particular,
Task-009 still owns IT-001-SC-06a and the complete version 1 SC-01 through SC-05
two-revision comparison.

## Verdict

**PASS for Task-012 / issue #48.** This is not a PASS for Plan-001 as a whole:
after this task closes, 1 of its 13 tasks is done, 4 are in progress, and 8 are
blocked.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | Plan-001 remains incomplete despite Task-012 passing: SC-06a and the complete version 1 comparison remain under Task-009, and later protocol subsystems remain under their own tasks. TM-001 and the plan retain them as planned or blocked; they are outside issue #48. | Plan-001; Task-009; TM-001; IT-001-SC-06a |
| FND-002 | low | Repository-wide engine coverage remains 37 of 322 rows even though the six version 2 behavior tests execute, because integration credit is tracked separately and the remaining plan is largely unimplemented. Support tests and unrelated criteria were not promoted; no Task-012 remediation is warranted. | TM-001; `quire coverage --json`; `tests/it001_v2_handoff.rs` |

## Scoped reconciliation

| Obligation | Implemented evidence | Result |
| --- | --- | --- |
| Owner-addressed default lane | `Handoff::load` uses `PUBLISHED_HANDOFF`; the environment selector and six ignores are gone. The integrity walk checks 109 listed members plus the manifest itself. | Pass |
| IT-001-SC-06 admission | The exact `/2` offer admits through `admit_and_link_v2` with the published digest and temporal cardinality retained. | Pass |
| IT-001-SC-06 version isolation | A valid `/1` control first admits through `admit_and_link`; both ordered cross-version calls then refuse through their explicit public entries with different typed errors and no translation. | Pass |
| IT-001-SC-07 and SC-08 | All 28 cases over the frozen 21-axis producer manifest are submitted; each returns its declared code, distinct axes share no complete typed error, and the unmutated control admits. | Pass |
| IT-001-SC-09 | All 13 declared dimensions admit at exact usage, refuse one-short with the matching typed dimension/limit, and re-admit with baseline-identical usage under a fresh exact budget. | Pass |
| Corpus authority | The 702-line provisional intake test and 815-line mirrored support module are deleted; current specs and tracking point to the producer manifest. Historical reviews remain historical evidence. | Pass |
| Dependency closure | Cargo and the lockfile fix `quire-spec-language` `93c411b` and one `quire-contract-ir` `04eb6f8` identity; `publish = false`. | Pass |

## Traceability and matrix audit

- `tests/it001_v2_handoff.rs` contains 15 executing tests: 6 behavior tests
  carry bare compiler-checked `#[trace("IT-001", "Task-012")]` attributes and
  9 corpus/integrity support tests carry only `Task-012`. No ignored test exists
  in that file, so support checks do not mint integration credit.
- TM-001 reports exactly four passing version 2 scenarios, with SC-06a planned.
  It assigns only FR-002-AC-5, FR-013-AC-4, NFR-007-AC-3, and NFR-007-AC-4 as
  passing integration-only criteria; it does not promote the wider version 1,
  inherited-axis, PT02, observation, replay, or campaign obligations.
- `quire coverage --json` reports 37 of 322 declared matrix rows backed and a
  Rust census of 176 candidates / 176 tagged / 176 bound. Those repository-wide
  figures remain low because Plan-001 is not complete; they do not contradict
  this narrowly completed task.
- Current matrix arithmetic remains explicit: 288 owed criteria = 233
  test-backed + 8 integration-backed + 47 unbacked; current outcomes are 4 pass,
  1 unexecutable, and 283 planned. Integration scenarios are 4 pass, 6 planned,
  and 22 unexecutable out of 32.

## Reverse-gap audit

No new unowned production behavior was found. The sole production-file edit is
API documentation removing a reference to deleted test code. The test helper
contains no second schema: it deserializes the producer's public handoff types,
uses every owner-exported path/member/format constant, and invokes the existing
public readers. Search confirms that current code retains no reference to the
removed environment variable, provisional loader, or local version 2 corpus;
mentions inside historical SpecReview artifacts are preserved deliberately.

## Validation evidence

- `/specify` plus subset base, dependency, and EARS review: SR-060 through
  SR-062; all four implementation findings remediated.
- Self Rust review: SR-063; both findings remediated.
- Quire: `quire 0.32.0` / engine `0.46.0@a874fb64`; scoped reviews and plan
  artifacts validate grammatically. The repository-wide validator retains two
  pre-existing shape diagnostics (`spec/reviews/README.md` lacks an archetype
  and the configured matrix column says `Status` while the table heading says
  `Coverage Status`); neither was introduced or hidden by this task.
- Rust: `cargo fmt` and clippy with `-D warnings` pass; the full locked suite
  reports 153 passed, 0 failed, and 23 explicitly ignored version 1 fixture
  cases, followed by 37 passing doc-tests. The focused version 2 lane reports
  15 passed, 0 failed, and 0 ignored.

## Remaining work outside this ticket

IT-001-SC-06a and the complete version 1 scenario comparison remain Task-009.
Plan-001's compensation, conformance, claim, canonical-result, observation,
temporal, E02, and final qualification work remains tracked by its other tasks.
Those gaps prevent a whole-plan PASS but do not reopen Task-012.
