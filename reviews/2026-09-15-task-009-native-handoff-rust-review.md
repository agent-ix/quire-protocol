---
id: SR-110
title: "Rust review of the Task-009 native compiler handoff"
type: SpecReview
analysis: code-review
scope: "Task-009 / quire-protocol#11 over candidate 345b459 and its review fixes"
review_set: subset
relationships:
  - { target: ix://agent-ix/quire-protocol/Task-009, type: reviews }
  - { target: ix://agent-ix/quire-protocol/FR-001, type: reviews }
  - { target: ix://agent-ix/quire-protocol/NFR-007, type: reviews }
  - { target: ix://agent-ix/quire-protocol/IT-001, type: reviews }
---
# SR-110: Rust Review of the Task-009 Native Compiler Handoff

## Summary

Reviewed Task-009's QSL pin, explicit version-1 intake/static-link path,
owner-addressed fixtures, paired version isolation, inventory checks and
tracking against issue #11. Every in-scope Rust finding was fixed before this
artifact was finalized.

## Verdict

**PASS for Task-009 / quire-protocol#11.** No unresolved Rust, owner-boundary,
typed-refusal, fixture-authority, trace, panic, unsafe, resource or test-quality
finding remains in this subset. The wider Plan-001 pipeline remains incomplete
and receives no credit from this review.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-001 | medium | Resolved: the version-1 integration asserted only that linked controls were non-empty, so losing 15 of 16 controls or any causal edge still passed. It now requires exactly 16 controls and 36 owner-validated causal edges, alongside two roles and one channel. | `tests/it001_native_handoff.rs::it_001_accepts_and_links_the_actual_four_unit_handoff`; IT-001-SC-01 | correct-requirement-no-evidence |
| FND-002 | medium | Resolved: the new inherited-axis comparison initially checked only `is_err()`, allowing `/2` to relabel an inherited refusal as a temporal/version-2-only cause. All eight cases now require exact `/1`–`/2` error equality and reject a `v2.*` code. | `tests/it001_v2_handoff.rs::every_inherited_selection_axis_refuses_at_both_versions`; NFR-007-AC-7 | correct-requirement-no-evidence |
| FND-003 | medium | Resolved: the six-ID multi-line trace on mutual refusal compiled but the installed Quire scanner did not bind the symbol. Removing undeclared task-only IDs and formatting the four owning IT/criterion IDs as one attribute raises FR-001 from 4/5 to 5/5 and NFR-007's current executable bindings from one to three. | `tests/it001_v2_handoff.rs::contract_versions_mutually_refuse_without_translation`; FR-001-AC-5; NFR-007-AC-3/AC-4 | correct-requirement-no-evidence |

## Rust Review

| Check | Result |
| --- | --- |
| Ownership and public surface | Protocol imports QSL's `Selection`, `SelectedDependency`, handoff paths and member constants. No owner type, schema, key, parser, wire reader or compatibility dispatcher is mirrored. Production keeps the root `/1` facade and explicit `intake::v1`/`intake::v2` paths; the only production edit routes the discriminator directly through `intake::v1::admit`. |
| Fixture authority | B's selection is fixed before the offer is read. Every selected source, model and dependency is loaded from QSL's pinned committed handoff and checked against its selected digest. The fixture equals the owner selection byte-for-byte except for B's terminating newline; no ambient environment path or producer invocation remains. |
| Version isolation | Both ordered cross-version submissions refuse with distinct typed causes. A second paired test changes artifact, contract, baseline, producer, language, sources, dependencies and models; both readers return the exact same inherited typed error for each, and no inherited error is relabeled `v2.*`. Temporal axes remain in the separate owner manifest. |
| Admission and linking | The default `/1` lane enters `intake::v1::admit`, preserves four sources and seven declarations, and reaches `model::v1::link_static` with exact cardinalities: two roles, one channel, 16 controls and 36 causal edges. Numeric extrema, rationals, field domains, digest domains, producer identity, feature and binding mutations refuse explicitly. |
| Errors and panics | Changed production code adds no panic, unwrap, unsafe, unchecked indexing, allocation loop, async/blocking bridge, process, thread or lock. Test `expect`/`panic` sites identify corrupt immutable evidence or a violated assertion. All unsupported and invalid paths remain typed QSL or Protocol errors; no approximation was added. |
| Bounds and conversions | No new numeric conversion or production accumulator exists. Both readers receive QSL's bounded `Limits`; the owner `/2` exact/one-short/fresh dimension tests remain default-running. |
| Test quality and trace | The three inventory tests, 24 `/1` tests and 18 `/2` tests run real public boundaries with no ignores. New assertions are discriminating: exact graph sizes, exact cross-version inequality, exact inherited error equality, and explicit axis names in failures. Quire reports FR-001 5/5 and NFR-007 current test criteria AC-3/AC-4/AC-7 bound. |
| Repository constraints | Rust only, `publish = false`, local `target-codex-backends`, no hosted dispatch, no workflow change, and no change under `resources/native-v1/`. Cargo and the lockfile pin QSL exactly to `6c8c401bb807a417215fae7593ef27fd9bab051e`. |

## Verification

Format and locked all-target Clippy with warnings denied pass after the
implementation changes. After review fixes, the focused default-running lanes
pass 3 dependency-inventory tests, 24 version-1 tests and 18 version-2 tests,
with zero failure or ignore. Quire validates all 236 specification, plan and
review documents grammar-clean. The repository-defined full unchanged-head gate
is the next promotion step; its exact head and results belong in the
pull-request evidence rather than being predicted here.

## Boundary

This PASS covers Task-009's native compiler handoff and version-isolation
allocation only. It does not claim Task-004 through Task-008, Task-010,
Task-011, Task-018, Plan-001 or epic #14 complete.
