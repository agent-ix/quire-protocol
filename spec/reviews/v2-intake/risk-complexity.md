---
id: SR-042
title: "Risk and complexity review of the version-explicit /2 consumer intake"
type: SpecReview
analysis: risk-complexity
scope: "IT-001 SC-06..SC-09, FR-001, FR-002, Task-009, tests/support/v2/mod.rs, tests/it001_v2_intake.rs"
review_set: all
relationships:
  - { target: ix://agent-ix/quire-protocol/IT-001, type: reviews }
---
## Summary

The `/2` intake is high volatility and medium technical risk: B has authored a
scenario corpus, an evidence-index schema and an absence-reporting loader ahead
of A's unpublished `protocol_artifact::v2` boundary, while correctly refusing to
pin a provisional revision, hand-author emitted bytes, or disturb the `/1` lane.
The dominant hazards are not the churn — which is bounded to one B-owned test
support file — but an inverted resource-limit obligation in IT-001-SC-09 that
the corpus has now hard-coded, and a blocked-lane test that the NFR-004 gate
never executes because it is `#[ignore]`d.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-001 | high | IT-001-SC-09 inverts the resource-limit obligation: it requires the input *one unit short* of a declared limit to return that limit's typed refusal while the exact-bound input admits, so a smaller input is illegal and the over-bound case is never probed; `LimitBound::OneShort => RefusesBeforeLinking` and two tests now encode the inversion, so the defect is spec, corpus and test at once. | IT-001-SC-09; tests/support/v2/mod.rs `LimitBound`, `resource_scenarios`; tests/it001_v2_intake.rs `every_declared_limit_gets_an_exact_a_one_short_and_a_fresh_retry_probe` | wrong-requirement |
| FND-002 | high | The only test that speaks for SC-06..SC-09 is `#[ignore]`d, and the NFR-004 local gate is `cargo test --locked`, which never runs ignored tests; IT-001's claim that the lane "fails explicitly rather than passing vacuously" holds only under a `-- --ignored` invocation that no declared gate command makes, so the whole `/2` obligation is silently green today. | IT-001 Notes; NFR-004-AC-7, NFR-004-AC-10; AGENTS.md gate commands; tests/it001_v2_intake.rs `v2_lane_is_blocked_on_a_s_published_boundary` | correct-requirement-no-evidence |
| FND-003 | medium | High volatility, bounded cost: B's `b-v2-manifest.json` schema, its closed 10-variant `SelectionRole`, 11-variant `SelectionAxis`, 4 digest domains and 6 temporal mutations are all guesses at an unpublished A surface, and every index struct is `#[serde(deny_unknown_fields)]`, so any added field, renamed role, or extra identity axis A ships is a hard load failure across the entire corpus; the churn is real but confined to one ~600-line test-support file with no `src/` coupling, no pinned A commit and no `/1` impact, so the mitigation is to accept the rewrite rather than defend the guess — keep the index lenient on unknown members until A's names are published, and treat the enumerations as B's obligation list, not A's API mirror. | tests/support/v2/mod.rs `Manifest`, `SelectionRole`, `SelectionAxis`, `REQUIRED_A_DELIVERABLES`; IT-001 Dependencies | missing-requirement |
| FND-004 | medium | Three enumerations of the same selection disagree: IT-001's `/2` Inputs retain *configuration* and *result* selections, `SelectionRole` carries `Configuration` and `Result`, but `SelectionAxis` carries neither — so two independently retained axes have no changed-selection scenario and SC-06 never mutates them, while `SelectionAxis` adds temporal axes `SelectionRole` cannot index. | IT-001 Inputs, IT-001-SC-06; tests/support/v2/mod.rs `SelectionRole` vs `SelectionAxis` | missing-requirement |
| FND-005 | medium | The loader admits a partial mutation corpus: it checks that each *delivered* mutation names an owed scenario, but never that each owed refusing scenario has a delivered fixture, and `mutation_bytes` silently returns the first match for a scenario id. A delivery carrying one mutation loads clean, so under-delivery by A reads as an assembled lane rather than as an absence. | tests/support/v2/mod.rs `Fixture::load_from`, `mutation_bytes`; tests/it001_v2_intake.rs `the_loader_indexes_the_temporal_and_mutation_evidence`; IT-001-SC-06..SC-08 | correct-requirement-no-evidence |
| FND-006 | medium | `producer_revision` is free text inside B's own JSON, not a pin: nothing binds the offered bytes to an immutable A revision, no digest of the offer is recorded or re-checked after read, and any 40-character string satisfies the loader — so a stale or re-emitted delivery directory is indistinguishable from the pinned one that IT-001 Dependencies demand. | IT-001 Dependencies; tests/support/v2/mod.rs `Manifest::producer_revision`, `OfferedArtifact`; FR-001-AC-1 | correct-requirement-no-evidence |
| FND-007 | medium | Gate evidence depends on ambient, shared state: the fixture root is the process environment variable `QUIRE_PROTOCOL_A_V2_FIXTURE`, and the loader's own tests write fixed-name directories under `std::env::temp_dir()` and `remove_dir_all` them at entry, so two worktrees or two concurrent gate runs destroy each other's fixtures — against the worktree-owned-evidence rule the gate already enforces with `flock` and a per-worktree target directory. | NFR-004-AC-9, NFR-004-AC-10; AGENTS.md; tests/it001_v2_intake.rs `scratch`, `write_delivery`; tests/support/v2/mod.rs `FIXTURE_ENV` | correct-requirement-no-evidence |
| FND-008 | medium | Task-009 was not extended with the spec: its subtasks name only "IT-001 SC-01 through SC-05", its deliverables name only the `/1` contract, and there is no subtask for pinning A's `/2` revision, rederiving B's selection, running the owed scenarios, or binding SC-06..SC-09 to FR-001/FR-002 criteria — so the plan now understates the specified work and no task owns the unblocking step. | Task-009 Subtasks/Deliverables/Notes; IT-001-SC-06..SC-09 | missing-requirement |
| FND-009 | medium | Thirteen passing corpus self-tests carry `#[trace("IT-001", "Task-009")]` while asserting nothing about A's behaviour; they correctly claim no FR acceptance-criterion id, but they can still make the IT-001 integration row read as exercised while SC-06..SC-09 are unexecuted. The matrix must record SC-06..SC-09 as unbacked explicitly rather than relying on the row's aggregate status. | spec/tests.md IT-001 row; tests/it001_v2_intake.rs trace attributes; FR-001-AC-4, FR-002-AC-3 | correct-requirement-no-evidence |
| FND-010 | low | SC-07's recanonicalization scenarios need a recanonicalized form of a producer input, which B cannot produce without the canonicalizer it is forbidden to own; `REQUIRED_A_DELIVERABLES` asks A for "per-axis mutation fixtures" generically and never names the recanonicalized variants, so this axis can be delivered incomplete and only discovered at pin time. | IT-001-SC-07; tests/support/v2/mod.rs `ProducerInput`, `REQUIRED_A_DELIVERABLES` | missing-requirement |

## Risk Register

| Req | Tech Risk | Volatility | Drivers | Mitigation |
| --- | --- | --- | --- | --- |
| FR-001 | Medium | High | Admission identity set is fixed by A's unpublished `v2::Expected`/`ExpectedTemporal` constructors; numeric and digest-domain obligations are already exercised on `/1` | Keep `/1` frozen as the compatibility precedent; re-derive the `/2` selection from A's published recipe only; never detect a version from offered bytes |
| FR-002 | Medium | Medium | Linking is B-owned and already specified; the `/2` delta is the temporal binding set (ownership, ordering, range) | Bind SC-08 to FR-002-AC-3 once A's refusal type names the offending temporal axis |
| IT-001-SC-06 | Medium | High | Version-explicit entry points plus 11 changed-selection axes guessed ahead of A's API; cross-version submissions must refuse without auto-detection | FND-003 mitigation; assert the two cross-version refusals first when A lands, before any positive admission |
| IT-001-SC-07 | Medium | High | Four digest domains and four recanonicalized producer inputs, all supplied by A; B owns no canonicalizer | Name the recanonicalized fixtures explicitly in `REQUIRED_A_DELIVERABLES` (FND-010) |
| IT-001-SC-08 | Low | High | Temporal profile, clock alternative, clock parameters and binding-set mutations are entirely A-defined vocabulary | Treat the six mutation variants as B's obligation list; expect renames on delivery |
| IT-001-SC-09 | Medium | High | Resource limits are A-declared, and the stated at-bound/one-short obligation is inverted (FND-001); the fresh-retry case also asserts no state carries across a refusal | Fix SC-09's direction before A lands, then add an over-bound probe; keep the fresh-retry probe |
| Task-009 | Medium | High | Task text predates the `/2` extension and owns no unblocking step | FND-008: add SC-06..SC-09 subtasks, the revision pin, and the `quire coverage` credit step |

## Top hazards

1. **FND-001** — SC-09's inverted limit obligation is now spec, corpus and test;
   every later reader will implement the wrong direction.
2. **FND-002** — the `/2` lane cannot fail the gate, so the whole SC-06..SC-09
   obligation is invisible to the only verification NFR-004 defines.
3. **FND-003** — the B-authored evidence-index schema and axis enumerations will
   churn when A publishes; accept the rewrite rather than harden the guess.
4. **FND-005 / FND-006** — a partial or unpinned delivery currently reads as an
   assembled lane; absence must stay louder than assembly.
5. **FND-008** — no task owns pinning A's revision and running the owed
   scenarios, so unblocking has no owner when `quire-spec-language#40` closes.

## Failure-domain gaps

Cross-reference: `spec/reviews/native-compiler-it001/failure-domain.md` (SR-025)
and `spec/reviews/digest-domain-discrimination.md`. Open gaps carried into `/2`:
the positive parser-set proof for SC-05 (unchanged, still an open obligation),
the configuration and result selection axes with no mutation scenario (FND-004),
and the recanonicalization axis B cannot construct itself (FND-010).
