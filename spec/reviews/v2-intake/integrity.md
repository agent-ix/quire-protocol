---
id: SR-039
title: "Integrity review of the IT-001 /2 version-explicit intake scenarios"
type: SpecReview
analysis: integrity
scope: "spec/integration/IT-001-consume-native-protocol-artifact.md SC-06..SC-09; FR-001; FR-002; FR-008; NFR-002; NFR-004; plan/Plan-001-finite-protocol-rust-slice/tasks/Task-009-integrate-native-compiler.md"
review_set: all
relationships:
  - { target: ix://agent-ix/quire-protocol/IT-001, type: reviews }
  - { target: ix://agent-ix/quire-protocol/FR-001, type: reviews }
  - { target: ix://agent-ix/quire-protocol/FR-002, type: reviews }
  - { target: ix://agent-ix/quire-protocol/FR-008, type: reviews }
  - { target: ix://agent-ix/quire-protocol/NFR-002, type: reviews }
  - { target: ix://agent-ix/quire-protocol/NFR-004, type: reviews }
---

## Summary

Completeness, consistency and atomicity analysis of the newly extended
version-explicit `/2` consumer scenarios IT-001-SC-06 through IT-001-SC-09,
read against FR-001, FR-002, FR-008, NFR-002, NFR-004, Task-009, and the
B-owned corpus already written in `tests/support/v2/mod.rs` and
`tests/it001_v2_intake.rs`. The `/2` lane is well isolated — it names no A type,
pins no provisional revision, hand-authors no emitted bytes, adds no feature or
`cfg`, and leaves the `/1` lane untouched — but its scenario clauses are
materially less atomic than the `/1` clauses they parallel, one clause inverts
the boundary polarity NFR-002 fixes, and the central `/2` obligation
(version-explicit entry points, no auto-detection) is stated only in the
integration test and is owned by no requirement.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-001 | high | SC-09 inverts NFR-002's inclusive-maximum polarity: it requires the *one-short* input to return the resource refusal and declares no over-bound input at all, so the below/at/over discrimination NFR-002-AC-1 mandates is both wrong and incomplete. | IT-001-SC-09, IT-001 Inputs, NFR-002 Scope, NFR-002-AC-1 | wrong-requirement |
| FND-002 | high | SC-06 is silently compound: one clause carries byte/reference retention, the `AdmittedPackage` return, both cross-version refusals, the "`admit_and_link` unchanged" precedent, and eleven independent changed-selection refusals — five obligations the `/1` lane deliberately splits across SC-01, SC-02 and SC-03. | IT-001-SC-06, IT-001-SC-01..SC-03 | wrong-requirement |
| FND-003 | high | The defining `/2` rule — one entry point per contract version, with no detection, upgrade or downgrade between them — appears only in IT-001 prose and in the unowned API name `admit_and_link_v2`; neither FR-001 nor FR-002 (the requirements IT-001 declares it verifies) states a version-explicit entry-point obligation, so SC-06's central assertion verifies nothing. | IT-001 Objective, IT-001-SC-06, FR-001 Behavior, FR-002 | missing-requirement |
| FND-004 | high | The `/2` selection set is enumerated three times and no two agree: Inputs lists source, model, definition, profile, configuration, dependency, producer, artifact, result; SC-06 lists artifact, contract, baseline, producer, source, model, dependency, definition, temporal-profile, clock-alternative, clock-parameter; `SelectionRole` lists ten roles again. `profile` is named only in Inputs, and `configuration`/`result` are retained but mutated by no clause. | IT-001 Inputs, IT-001-SC-06, tests/support/v2/mod.rs | wrong-requirement |
| FND-005 | medium | "A typed refusal" is nowhere bound to FR-008's envelope, so no SC-06..SC-09 clause discriminates one refusal from another; SC-08's "names the offending temporal binding axis" is satisfiable by display text, which FR-008-AC-2 forbids as an identity source, and the corpus's `Disposition` records only Admits/RefusesBeforeLinking, letting one blanket error satisfy all fourteen SC-06 refusals. | IT-001-SC-06..SC-09, FR-008-AC-1, FR-008-AC-2, tests/support/v2/mod.rs | wrong-requirement |
| FND-006 | medium | The `/2` lane has no counterpart to SC-02: no clause inspects the linked package's source units, declarations, references or binding path, so IT-001 yields no FR-002 linking evidence for `/2` despite declaring `verifies FR-002`. | IT-001-SC-02, IT-001-SC-06, FR-002-AC-1..AC-4 | correct-requirement-no-evidence |
| FND-007 | medium | Inputs declares signed-64, exact-rational and numeric-representation mutations for both lanes, but no SC-06..SC-09 clause and no `ScenarioKind` variant consumes them, leaving FR-001-AC-4 — whose verification names IT-001 — with `/1` evidence only. | IT-001 Inputs, IT-001-SC-04, FR-001-AC-4, tests/support/v2/mod.rs | correct-requirement-no-evidence |
| FND-008 | medium | SC-08 is compound and duplicative: it bundles six negative temporal-binding mutations with a positive admission clause that restates SC-06's valid case in different words, and the corpus carries no row for that positive clause (all six temporal rows refuse). | IT-001-SC-08, IT-001-SC-06, tests/support/v2/mod.rs | wrong-requirement |
| FND-009 | medium | Task-009 still scopes only "IT-001 SC-01 through SC-05" and the `quire.compiled-protocol/1` contract; SC-06..SC-09, the `/2` pin step, and the already-written `tests/support/v2/mod.rs` and `tests/it001_v2_intake.rs` have no owning subtask or deliverable in the only task that verifies IT-001. | Task-009 Scope, Task-009 Subtasks, IT-001-SC-06..SC-09 | missing-requirement |
| FND-010 | medium | The Notes claim the `/2` parts "fail explicitly rather than passing vacuously", but the single lane test is `#[ignore]`d and NFR-004-AC-1 enumerates `cargo test --locked`, which never runs ignored tests and to which no `--ignored` invocation is added — so the non-vacuity claim is backed by no gate command. | IT-001 Notes, NFR-004-AC-1, NFR-004-AC-2, tests/it001_v2_intake.rs | correct-requirement-no-evidence |
| FND-011 | low | SC-09 leaves the `/2` limit authority unstated — A-declared `v2::Limits` per the Dependencies list, or the caller-supplied budget NFR-002 assigns to the request owner — and "a fresh limit value" fixes no relationship between the retried input and the new bound; NFR-002-AC-2's monotonic-deadline dimension is absent from the `/2` lane entirely. | IT-001-SC-09, IT-001 Dependencies, NFR-002 Statement, NFR-002-AC-2 | missing-requirement |
| FND-012 | low | Wording drifts from the `/1` clauses: SC-06 and SC-08 refuse "before a package is exposed" where SC-03 says "before a linked package is exposed", and SC-06 uses "package" for both `v2::AdmittedPackage` and the linked representation; SC-06 also collapses FR-001's distinct "artifact-contract version" and "contract version" into one `contract` axis. | IT-001-SC-06, IT-001-SC-08, IT-001-SC-03, FR-001 Description | wrong-requirement |

## Completeness Analysis

IT-001 declares `verifies` against FR-001, FR-002 and NFR-002. For `/2` it
supplies FR-001 evidence (identity retention and per-axis refusal) but no FR-002
evidence at all (FND-006), and none of the numeric obligations FR-001-AC-4
routes to IT-001 (FND-007). The NFR-002 edge now asserted by the frontmatter is
discharged by SC-09 alone, which inverts that requirement's polarity (FND-001)
and omits its deadline dimension (FND-011), so the declared edge currently
claims more coverage than the clause delivers. FR-008 constrains every refusal
SC-06..SC-09 assert but is referenced by neither IT-001's relationships nor its
Traceability section, so those refusal clauses carry no declared verification
path back to the requirement that governs them.
Downstream, Task-009 is the only task bound to IT-001 and covers SC-01..SC-05
(FND-009), leaving SC-06..SC-09 with no owning plan item.

## Consistency and Conflict Analysis

The direct conflict is SC-09 against NFR-002's inclusive-maximum rule
(FND-001), and it is mirrored in `LimitBound::OneShort =>
Disposition::RefusesBeforeLinking`, so the corpus encodes the inverted reading
rather than exposing it. The remaining conflicts are enumeration and vocabulary
divergence between Inputs, the SC clauses, and the Rust corpus (FND-004,
FND-012), plus a duplicated positive admission obligation stated twice in
different words (FND-008). Hidden assumptions surface in two probes: an
externally owned boundary not yet published, where the interim state is
documented but its execution gate is not (FND-010), and a limit authority that
is never decided between producer and caller (FND-011).

## Atomicity and Testability Analysis

SC-07 (digest-domain substitution plus recanonicalisation) is two obligations,
SC-08 six negatives plus one positive, SC-09 three dispositions plus a
state-carryover property, and SC-06 five obligations across fourteen corpus
rows (FND-002). The `/1` clauses are the available precedent for the right
granularity: SC-01 retention, SC-02 structure, SC-03 mutation refusal, SC-04
numerics, SC-05 reader inventory — one observable per clause. Testability is
further limited by refusals that are asserted only as "typed" with no
discriminating field (FND-005): every negative row in the `/2` corpus is today
satisfiable by a single undifferentiated error.

## Failure Domain Check

Extension failure and identity keying are well handled: no auto-detection, no
`/1`-for-`/2` substitution, per-role retained selections, and four separated
digest domains with all twelve ordered substitutions enumerated. Evaluation
purity is asserted ("refuses before a package is exposed") but not made
observable (FND-005). Topological robustness of the temporal binding set
(missing, surplus, duplicate, reordered, foreign owner, out of range) is
complete; the resource-bound topology is not, because the over-limit corner is
absent (FND-001).

## Recommendations

1. Correct SC-09 to NFR-002 polarity — one-short admits, exact bound admits,
   smallest representable excess refuses — and add the over-bound input to
   Inputs and to `LimitBound`.
2. Split SC-06 into retention, cross-version isolation, and changed-selection
   refusal clauses mirroring SC-01/SC-03, and add a `/2` structural clause
   mirroring SC-02.
3. Raise the version-explicit entry-point and no-auto-detection rule into FR-001
   (or a new FR) before IT-001 asserts it.
4. Reconcile Inputs, the SC clauses and `SelectionRole` to one enumeration of
   the `/2` selection axes.
5. Bind each refusal clause to the FR-008 field that discriminates it, and carry
   that expected code in `Disposition`.
6. Extend Task-009 to own SC-06..SC-09, the pin step, and the `/2` corpus, and
   record how the `#[ignore]`d lane is executed under the NFR-004 gate.
