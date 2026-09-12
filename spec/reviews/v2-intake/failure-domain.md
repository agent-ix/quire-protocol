---
id: SR-038
title: "Failure-domain review of the version-explicit /2 intake lane"
type: SpecReview
analysis: failure-domain
scope: "spec/integration/IT-001-consume-native-protocol-artifact.md scenarios IT-001-SC-06..SC-09; FR-001, FR-002, FR-008, NFR-002, NFR-004; plan/Plan-001-finite-protocol-rust-slice/tasks/Task-009-integrate-native-compiler.md; as authored in tests/support/v2/mod.rs and tests/it001_v2_intake.rs on agent-b/it001-v2-intake over 992904e"
review_set: all
relationships:
  - { target: ix://agent-ix/quire-protocol/IT-001, type: reviews }
  - { target: ix://agent-ix/quire-protocol/FR-001, type: references }
  - { target: ix://agent-ix/quire-protocol/FR-002, type: references }
  - { target: ix://agent-ix/quire-protocol/FR-008, type: references }
  - { target: ix://agent-ix/quire-protocol/NFR-002, type: references }
  - { target: ix://agent-ix/quire-protocol/NFR-004, type: references }
  - { target: ix://agent-ix/quire-protocol/Task-009, type: references }
---
## Summary

This failure-domain analysis examines the newly extended version 2 consumer
scenarios IT-001-SC-06 through IT-001-SC-09 — the version-explicit compiled-
protocol intake — against the four checklist domains: trust boundaries and
extension-point failure policy, entity identity and uniqueness keys, evaluation
purity, and topological robustness. It reviews the scenario text together with
the B-owned corpus that was authored ahead of A's delivery
(`tests/support/v2/mod.rs`, `tests/it001_v2_intake.rs`), and treats A's
`protocol_artifact::v2` boundary as the unpublished trust boundary it is.

Twelve findings are recorded. Four are high: SC-09 states the resource boundary
backwards and has no over-limit control at all; no artefact supplies inputs at
those bounds; the lane's only outcome oracle cannot distinguish a typed refusal
from a panic; and nothing in the spec separates a selection B *derived* from its
retained inputs from a value B *transcribed* out of A's delivery. The remaining
eight are medium or low and concern purity, identity keys, and the gate.

## Verdict

**CONDITIONAL** — SC-06 through SC-09 are the right scenarios and they assert
nothing today, which is honest. They are not yet *safe* to switch on: as
written, three of the four would admit a wrong implementation once A delivers
(FND-001, FND-003, FND-004). Those three should be closed in the IT-001 text
before A's revision is pinned; the rest can be closed as the lane is built out.

## Assurance Context

Evaluated tree: `agent-b/it001-v2-intake` at `992904e` with the IT-001 edit and
both new test files in the working tree. No A revision is pinned, no
`protocol_artifact::v2` symbol is named anywhere in B, and no emitted bytes are
hand-authored — the loader's own fixtures write opaque non-artifact bytes and
are never submitted to an admission API. The version 1 lane
(`admit_and_link`, `tests/it001_native_handoff.rs`) is untouched.

Unavailable selected context, declared rather than reconstructed: A's public
version 2 type names, its declared limit set and their exact bounds, its refusal
taxonomy, and its documented per-axis refusal expectations. Every finding below
is therefore a statement about the *specification and the corpus*, not about A's
behaviour, which cannot be observed from here.

## Checklist Application

**1. Extension points (trust boundaries).** The one extension point is A's
`v2::read` and the `v2::AdmittedPackage` / refusal pair it returns. IT-001
requires "a typed refusal" for every negative row but never states the failure
policy for the boundary itself: what happens when A's reader panics, aborts,
returns a success with an internally inconsistent report, or returns a code
absent from the selected diagnostic catalog. FR-008 answers the last case for
B-owned failures (`unsupported-source-contract`, FR-008-AC-3) but is not bound
into any SC-06..SC-09 outcome. See FND-003, FND-005, FND-012.

**2. Entity identity.** Four version 2 entity families gained no uniqueness key in
this extension: temporal bindings (`owner` + `ordinal`), declared resource
limits (`name`), mutation fixtures (`scenario`), and the digest domains as they
key into slots. See FND-008, FND-009, FND-004.

**3. Evaluation purity.** Two purity questions are open: whether the
cross-version refusal is decided from the caller's entry point or from the
offered bytes (which would be the auto-detection SC-06 forbids, used to produce
the refusal), and whether the recanonicalisation probe has an observable
difference to detect at all. See FND-005, FND-007. The resource-retry purity
question — what state may not survive a refused attempt — is FND-006.

**4. Topological robustness.** The temporal binding set is the only new
structure with ordering and ownership. SC-08 enumerates six flat mutations but
states no termination argument over owner structure and no relation between
binding cardinality and the NFR-002 limits. See FND-008, FND-011.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-001 | high | SC-09 states the resource boundary backwards and omits the over-limit control. NFR-002 makes maxima inclusive — at the bound admits, the first excess unit refuses — but SC-09 requires "the one-short input returns the typed resource refusal for that limit". An input one unit *short* of a maximum is under it and must admit. There is no above-bound case anywhere in the scenario, so NFR-002-AC-1's "removing or reversing its guard fails the assigned case" is not discharged for version 2: an implementation with the guard deleted passes every SC-09 row. `LimitBound` encodes the inversion (`OneShort => RefusesBeforeLinking`) and has no over-bound variant, so the defect is now in two places. | IT-001-SC-09; NFR-002; NFR-002-AC-1; `tests/support/v2/mod.rs` `LimitBound` | wrong-requirement |
| FND-002 | high | Nothing supplies the inputs SC-09 needs. B owns no emitter and may not hand-author bytes, so an artefact sized exactly at a declared limit can only come from A — yet neither IT-001's Dependencies list nor `REQUIRED_A_DELIVERABLES` asks A for boundary-sized artefacts; both stop at "the per-axis mutation fixtures". `IndexedLimit` carries `name` and `exact` only, with no fixture path, so `resource_scenarios` mints three rows per limit that no delivery can satisfy. SC-09 is presently unconstructible rather than merely unexecuted. | IT-001-SC-09; IT-001 Dependencies; `REQUIRED_A_DELIVERABLES`; NFR-002-AC-1 | missing-requirement |
| FND-003 | high | The lane's outcome oracle is two-valued, so any non-admission passes. `Disposition` distinguishes only `Admits` from `RefusesBeforeLinking`; SC-06..SC-09 likewise say "returns a typed refusal" without binding it to FR-008's envelope. A panic, an untyped error, a refusal from the wrong stage, a refusal carrying a shared code across three different axes, or a refusal raised *after* a package was constructed all satisfy every negative row as written. This is the exact defect SR-027 recorded on the version 1 lane — one shared `Invalid::Selection` code standing for three axes — reintroduced structurally in the version 2 corpus before A has delivered. | IT-001-SC-06..SC-09; FR-008-AC-1; FR-008-AC-5; SR-027; `tests/support/v2/mod.rs` `Disposition` | correct-requirement-no-evidence |
| FND-004 | high | Independent selection and transcription are not separated. SC-06 requires `v2::Expected`/`v2::ExpectedTemporal` be built "without reading A's offered bytes", but B's evidence index is written by B at pin time from A's delivery directory: `IndexedOffer.reference` is documented as "copied verbatim" from A, and the temporal profile, clock alternative, clock parameters and limit bounds are likewise transcribed. Nothing in IT-001 says which manifest fields must be *derived from B's retained producer inputs* and which may be transcribed, so comparing a transcribed value against its own source is a tautology that would still pass SC-06. `the_loader_preserves_the_offered_bytes_and_reference` already asserts one such tautology (the loader's reference equals the manifest's reference). | IT-001-SC-06; IT-001 Inputs; FR-001-AC-1; FR-001-AC-3; `tests/it001_v2_intake.rs` | missing-requirement |
| FND-005 | medium | The cross-version refusal may itself be the auto-detection SC-06 forbids. SC-06 demands version 1 bytes at `admit_and_link_v2` and version 2 bytes at `admit_and_link` refuse "with no auto-detection, upgrade, or downgrade", but does not say the mismatch must be decided by comparing the offered contract identity against B's independently selected one at the entry point the caller chose. An implementation that sniffs the offered bytes, recognises the other version, and refuses satisfies the text while discriminating on exactly the signal the requirement exists to forbid; only the *outcome* is constrained, not the decision procedure. | IT-001-SC-06; FR-001; `CrossVersion` | missing-requirement |
| FND-006 | medium | SC-09's "fresh retry" has no defined state set. It requires the exact-bound input to re-admit "without carrying state from the refused attempt" without naming what must not carry: accounted allocation, a consumed deadline, a partially decoded buffer, an interned identity table, or a poisoned reader. NFR-002-AC-2's elapsed-time dimension cannot be made fresh at all, and SC-09 does not say whether time limits are in scope of the retry or whether the retry uses the same reader instance. The criterion is unobservable as stated. | IT-001-SC-09; NFR-002-AC-2; NFR-002 "Work admitted after a limit is exhausted" | missing-requirement |
| FND-007 | medium | The recanonicalisation probe can be vacuous. SC-07 requires that "a producer input that B recanonicalized instead of copying byte for byte refuses rather than admitting an equivalent form", but if a producer input is already in its canonical form, recanonicalising it is the identity function and the probe submits the valid input while the corpus asserts `RefusesBeforeLinking`. IT-001 does not oblige A to deliver originals with a distinguishable non-canonical form, so up to four `Recanonicalized` rows may be indistinguishable from `Valid` — failing for the wrong reason, or being quietly relaxed later. | IT-001-SC-07; FR-001 Behavior; `ProducerInput` | wrong-requirement |
| FND-008 | medium | Temporal bindings have no uniqueness key, so four of SC-08's six mutations are undefined. "Duplicate", "reordered", "surplus" and "missing" all presuppose an identity for a binding and a total order over the set, but IT-001 states neither. `IndexedBinding` carries `owner` and `ordinal: u32` with no stated requirement that ordinals are unique per owner, dense, zero-based, or globally ordered — so a "reordered" set and a "duplicate" set may be the same set, and an implementation that keys bindings by `owner` alone refuses both for one reason and still passes. | IT-001-SC-08; FR-002-AC-1; FR-002-AC-3; `IndexedBinding` | missing-requirement |
| FND-009 | medium | Limit names and mutation scenario ids have no declared uniqueness, and the composed corpus is never checked for collisions. Scenario ids are minted as `v2-limit-<name>-<bound>`, so two limits sharing a name silently collapse three rows into three duplicates; `static_corpus_ids_are_unique` covers only the static corpus and never `scenarios_for`. Symmetrically, `IndexedMutation.scenario` is not required to be unique, and `mutation_bytes` returns the first match, so a second fixture claiming the same scenario is silently unused while the delivery still validates. | `tests/support/v2/mod.rs` `resource_scenarios`, `mutation_bytes`; `tests/it001_v2_intake.rs` `static_corpus_ids_are_unique` | missing-requirement |
| FND-010 | medium | The loader converts a read failure into "absent", contradicting its own stated policy. `Absence` exists so that "every variant is reported; none is treated as a pass", but `input_bytes` and `mutation_bytes` both end their chain in `fs::read(..).ok()`: an indexed member that is present at load time and unreadable afterwards (truncated, permissions, replaced by a directory) becomes `None`, indistinguishable from "not indexed for this delivery". FR-008-AC-7's rule that unavailable context stays explicitly unavailable is the same rule one level down. | `tests/support/v2/mod.rs` `input_bytes`, `mutation_bytes`; FR-008-AC-7 | implementation-bug-despite-evidence |
| FND-011 | medium | The one test that reports the version 2 lane's blocked state is never run by the gate. `v2_lane_is_blocked_on_a_s_published_boundary` is `#[ignore]`, and NFR-004-AC-1 fixes the gate at `cargo fmt`/`cargo clippy`/`cargo test --locked` — which does not run ignored tests and does not fail on a non-zero ignored count. IT-001's Notes claim the lane "fails explicitly rather than passing vacuously"; against the enumerated gate it does neither, because it does not execute. Nothing obligates an `--ignored` run or a recorded ignored-count budget. | IT-001 Notes; NFR-004-AC-1; NFR-004-AC-2; `tests/it001_v2_intake.rs` | correct-requirement-no-evidence |
| FND-012 | low | Loader scratch roots collide across concurrent runs. `scratch()` builds `std::env::temp_dir().join("quire-protocol-v2-<name>")` — a fixed shared path — and begins with `remove_dir_all`, so two simultaneous test processes delete each other's fixtures mid-run. `flock` on the gate (NFR-004-AC-3) serialises gate runs but not ad-hoc `cargo test` runs on the same shared workstation, which is the condition NFR-004-AC-3 and AC-9 exist to address. | `tests/it001_v2_intake.rs` `scratch`; NFR-004-AC-3; NFR-004-AC-9 | implementation-bug-despite-evidence |

## Coverage assessed

| Criterion | Status | Evidence |
| --- | --- | --- |
| IT-001-SC-06 | Planned | The corpus enumerates one valid admission, both cross-version submissions and all eleven changed-selection axes, and asserts only `Valid` admits. No A boundary is called, so nothing about A's behaviour is asserted. FND-003, FND-004 and FND-005 must close before this scenario can be switched on. |
| IT-001-SC-07 | Planned | All twelve ordered digest-domain substitutions and all four recanonicalised producer inputs are enumerated and asserted refusing. FND-007 records that the recanonicalisation half may have nothing to observe. |
| IT-001-SC-08 | Planned | All six temporal-binding mutations are enumerated. FND-008 records that four of them lack the identity key that would give them meaning. |
| IT-001-SC-09 | Blocked | The probe shape exists but the criterion is inverted (FND-001) and unconstructible (FND-002), and its retry oracle is undefined (FND-006). This scenario is not merely unexecuted; it is not yet correct. |
| FR-001-AC-1..AC-3 | Unchanged | version 2 adds no new evidence for FR-001 today; the version 1 lane's standing status is untouched. |
| FR-002-AC-1, AC-3 | Unchanged | Temporal-binding distinctness and located refusal are enumerated only, pending FND-008. |
| FR-008-AC-1, AC-5 | Not evidenced | No version 2 row inspects a refusal envelope, its domain, its stage, or its code (FND-003). |
| NFR-002-AC-1, AC-2 | Not evidenced | See FND-001, FND-002, FND-006. |
| NFR-004-AC-7 | Satisfied | The new files add no `[features]` entry, no `optional = true` dependency, and no `cfg` attribute or `cfg!` macro, so the crate stays single-lane and no second lane is obligated. |
| NFR-004-AC-1 | Partial | The corpus and loader tests run in the enumerated gate; the lane test does not (FND-011). |

## Proposed additions

Stated as the smallest wording that closes each high finding; all belong to
IT-001 and NFR-002, and none is authored here.

* **FR-shaped, SC-09.** Restate the boundary in NFR-002's own direction: the
  input at the exact bound and the input one unit *below* it both admit; the
  input at the smallest representable excess over the bound returns that limit's
  typed resource-incomplete refusal. Add the over-bound case as a third probe.
  (FND-001)
* **StR-shaped, IT-001 Dependencies and A's deliverable list.** Add "an offered
  artefact at each declared limit's exact bound, one below it, and one above it"
  to what `quire-spec-language#40` owes, and give `IndexedLimit` a fixture path
  per bound. (FND-002)
* **FR-shaped, SC-06..SC-09.** Every negative row's refusal is checked as an
  FR-008 envelope — domain, stage, catalog selection, affected identity — and
  no two distinct axes may share a code. A panic or an untyped error is a
  failure of the scenario, not a refusal. (FND-003)
* **StR-shaped, IT-001 Inputs.** Mark each version 2 selection field as *derived*
  (recomputed by B from a retained original producer input) or *transcribed*
  (copied from A's delivery); a transcribed field may not stand as evidence for
  independent selection, and SC-06 compares derived fields only. (FND-004)
* **NFR-shaped, NFR-004.** The gate records the ignored-test count for one
  unchanged head, and an ignored test that stands for an unexecuted acceptance
  scenario must be listed with the external blocker it waits on. (FND-011)

## Gates run

None. This review edits one Markdown file under `spec/reviews/` and no Rust
source, so the Cargo gate of NFR-004-AC-1 has nothing to detect here and was not
run; claiming it would be an untrue assurance claim under NFR-003. The
specification gate that does apply was run:

- `quire validate --scope /home/peter/dev/quire-protocol "spec/**/*.md"` — see
  the parent report for its result on this document.
