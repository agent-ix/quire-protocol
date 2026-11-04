---
id: SR-2127001
title: "spec-review of quire-protocol PR 127 (checked-package spec batch)"
type: SpecReview
analysis: base
scope: "agent-ix/quire-protocol; FR-043 to FR-047; TC-149 to TC-153; FR-001; FR-002; spec/tests.md"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-protocol/FR-043
    type: references
---

# Spec review of quire-protocol PR 127

## Summary

Ticket: QSL-1549. Head bd361459f0fa20d1646b8ca5532137db9922bd51, checked against QSL main 5d34a5b1e (FR-519, FR-520, FR-521, FR-515 to FR-518, FR-511 and the qsl-package, qsl-eval and checked_handoff code). Hand-applied EARS, integrity, dependency and criterion-strength lenses. Verified: ids FR-043 to FR-047 and TC-149 to TC-153 are free on main and on the other open PR; FR-044 matches FR-519's record-to-node map and its `protocol_view`, `causal_relation` and refusal names; FR-046 matches FR-521 rows 3 and 4 and the `evaluate_temporal_evidence(&CheckedNodeId, &EvidenceRequest, &mut Meter)` signature and `PackageMismatch` on main; FR-047 matches FR-521 rows 2, 14 and 15 and FR-511's end condition; FR-045 matches FR-520's outcome map and `ProtocolSlotEvaluation::Unavailable`; the owner-ruling item is flagged and tests.md marks it pending; no ticket number in any AC; no new digest introduced. quire validate exits are in the Linear comment.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-001 | high | FR-045 and FR-046 take a `CheckedPackageEvaluation` and an `EvidencePackage` 'over the admitted package' that FR-043 gets from `read_checked_package`, but QSL main has no such constructor. `CheckedPackageEvaluation` is sealed and implemented for the in-process `CheckedPackage` (qsl-eval value/expression/mod.rs, family.rs); `EvidencePackage::from_emission` and `from_read` and `AdmittedEvaluationView` all take an `AdmittedPackageRead`, which only `read_evaluation_package` and `read_evaluation_emission` build, by emitting a QSL source `CheckedPackage` (qsl-package/src/checked_v2.rs: 'callers cannot pair an admitted IR graph with an unrelated QSL package'). QSL FR-515, as updated by its open spec batch, also puts a bytes-only evaluator out of scope. A consumer that reads a handoff directory has a `CheckedPackageV2` and no source `CheckedPackage`, so FR-045 and FR-046 cannot be implemented from FR-043's output. State where the in-process source package comes from (for example the recipe is compiled in the consumer's test) or raise the missing bytes-only entry with the QSL planner; do not leave the Inputs as written. FR-044 is fine: `protocol_view` and `causal_relation` take `&CheckedPackageV2`. | spec/functional/FR-046-consume-temporal-evidence-through-the-evidence-types.md | wrong-requirement |
| FND-002 | high | FR-043 says the library compares the offered identities with the selection before it reads a field that depends on them, and AC-3 says the offered bytes are not read before the comparison. QSL FR-517 defines `Selection::check(&self, &CheckedPackageV2)` over an already admitted package, and FR-518 gives each mutation case the class `reader` (refused by `read_checked_package`) or `selection` (admitted by the reader, then refused by `Selection::check`). So the bytes are read before the selection check and AC-3 cannot pass as written. The Outputs and the third Behavior bullet also say the selection check returns a refusal code, but `SelectionRefusal` carries a `SelectionAxis`, not a code. Restate the order as read first, then `Selection::check`, and the selection refusal as an axis. | spec/functional/FR-043-admit-the-checked-package-handoff.md | wrong-requirement |
| FND-003 | medium | The Description and AC-3 list `EvidenceRef` and `ActivationState` as retired and forbid naming them, but the new qsl-eval `EvidenceResult` carries `activation: ActivationState` and `correspondence: EvidenceRef` (qsl-eval/src/temporal/evidence/request.rs, structs of those names), and FR-046 itself reads the activation state from the result. An inspection that greps for the names fails on the legitimate new types. Qualify the retired ones by path (`native_temporal::v2::...`). | spec/functional/FR-046-consume-temporal-evidence-through-the-evidence-types.md | wrong-requirement |
| FND-004 | medium | FR-047-AC-2 says no source names `quire.compiled-protocol/1`, `/2` or `quire.protocol.v2-mutations/1`, while FR-043-AC-5 requires a test that offers bytes of `quire.compiled-protocol/1` and `/2` and checks the refusal, which puts the string in test fixtures that AGENTS.md holds to the same rule. FR-043-AC-5 also asks for checked-package bytes offered at a retired entry point, which cannot exist once the repoint removes them, and FR-001 keeps them in force until then. Say whether AC-2 covers production source only, how the retired-format test builds its input without naming the identity, and that the retired-entry-point half applies only before the repoint. | spec/functional/FR-047-end-the-retired-compiler-surfaces-and-the-integrity-lanes.md | wrong-requirement |
| FND-005 | medium | The owner ruling is flagged, but the requirement still decides it: the Description says the library SHALL remove the IT-001 checksum and seal lanes 'pending an owner ruling', and AC-4 states option A as the criterion. Make the Description conditional and keep AC-4 out of the in-force criteria until the owner rules (the open question belongs in the text, the criterion is written after the ruling). AC-4 also says no digest of an admitted package exists in the crate, which can be read as banning the producer content digests FR-009 and FR-015 keep; limit it to the package-identity digest, the checksum and seal lanes and `ArtifactRef.digest`/`revision`. Digests that carry trust (the FR-009 lock) are not touched by it and the spec should say so. | spec/functional/FR-047-end-the-retired-compiler-surfaces-and-the-integrity-lanes.md | wrong-requirement |
| FND-006 | medium | AC-7 does not contradict QSL FR-520-AC-11, which is gated on IR-1280, but it fixes that attribution is 'read from the Meter charge log', where FR-520's text says IR-1280 decides where it lives, and one AC holds two states (before and after the attribution exists), so the 'until then' branch is untestable once it lands. Split it: an AC for the current state (record the total charge, report attribution as unsupported, not zero) and an AC mirroring FR-520-AC-11 for when the per-binder log and `accounting_version` exist, both naming the gate. The 'unsupported result' has no named owner type; say which owner's type it is (AGENTS.md: find the owner). | spec/functional/FR-045-assess-compensation-through-the-checked-package-evaluation.md | wrong-requirement |
| FND-007 | low | Small gaps: tests.md still says 'Behavioural totals across these 35 rows: 198 acceptance criteria' after five rows and 31 criteria were added; QSL FR-521 row 5 (keep the `temporal::{Closure, Completeness, ...}` surface until the repoint and IR-1005, and fix the `Position.instance_valuations` break in the consumer) has no requirement in FR-045 to FR-047; FR-044-AC-5 forbids retired types on the public surface while AC-6 keeps a window binding on the `wire.rs` spelling, so say which `wire.rs` item stays public. Process: AGENTS.md requires `/spec-review` and `/spec-to-plan` before implementation; neither was run for this batch. | spec/tests.md | wrong-requirement |

## New findings (disposition pass 1)

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-008 | low | FR-043's Outputs give two outcomes, admitted or one of two refusal kinds, but `read_checked_package` has a third result, `CheckedPackageDispatchResult::Incomplete(CheckedPackageIncomplete)` (the first exhausted read limit, with `limit_kind`, `limit`, `consumed` and a pointer), which is neither admitted nor a refusal. State how the library carries it (a typed non-conclusive outcome, unchanged) and give it a criterion; the `CheckedPackageRefusal` also holds `loci` (nonempty list, each with a pointer and source region), `cause` and a node `locus`, so 'its optional cause and JSON pointer' should say loci. SelectionRefusal and SelectionAxis (src/checked_handoff/selection.rs: `Selection::check(&self, &CheckedPackageV2) -> Result<(), SelectionRefusal>`, `SelectionRefusal { axis, clause }`) match the text. | spec/functional/FR-043-admit-the-checked-package-handoff.md | missing-requirement |
| FND-009 | low | FR-045 and FR-046 now depend on a QSL entry that evaluates from a `CheckedPackageV2` alone, but QSL main has no requirement for it (QSL FR-515 still calls an evaluator over `formula_graph()` alone 'a separate future capability'), so the dependency names no requirement id. Name the QSL requirement when it exists, and mark FR-045 and FR-046 as blocked on it in spec/tests.md; the wording is abstract and invents no API, which is right. | spec/functional/FR-045-assess-compensation-through-the-checked-package-evaluation.md | missing-requirement |

## Dispositions

| FND | Outcome | Reason |
| --- | --- | --- |
| FND-001 | fixed | Verified in fix commit 3d9f8dccc against the current text. |
| FND-002 | fixed | Verified in fix commit 3d9f8dccc against the current text. |
| FND-003 | fixed | Verified in fix commit 3d9f8dccc against the current text. |
| FND-004 | fixed | Verified in fix commit 3d9f8dccc against the current text. |
| FND-005 | fixed | Verified in fix commit 3d9f8dccc against the current text. |
| FND-006 | fixed | Verified in fix commit 3d9f8dccc against the current text. |
| FND-007 | still-open | Partly fixed: FR-046-AC-7 and the keep-the-temporal-surface Behavior bullet cover FR-521 row 5, FR-044-AC-5 and AC-6 now name `BindingKind::Window` as the one public `wire.rs` item, and the tests.md rows and the intro (49 FRs, 24 test cases) are updated. Still open: the intro's acceptance-criteria count reads 345 (315 + 30) but the five new requirements hold 6, 7, 8, 7 and 4 criteria, 32 in all, so the author's own method gives 347; and 'Behavioural totals across these 35 rows: 198 acceptance criteria' is unchanged although five rows were added. Process note: no /spec-review artifact existed before this review and no plan is written. |
| FND-008 | still-open | new this round. |
| FND-009 | still-open | new this round. |

## New findings (disposition pass 2)

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-010 | low | Checked against the Contract IR reader (quire-contract-model checked_package): `exceeds(consumed, limit)` is `consumed > limit`, so one below usage returns `Incomplete` and the exact limit admits, and `CheckedPackageIncomplete` carries `limit_kind` (Bytes, Nodes, Edges, Occurrences, Diagnostics, Work), `limit` and `consumed`, so AC-7 matches the code. TC-149 says it reads 'with each read limit one below and at its exact usage', but a limit whose measured usage on an admitted package is 0 (the diagnostic budget on R, probably) has no representable one-below; say 'each limit with nonzero measured usage'. The `⛔ Blocked` status is new to tests.md, which has no status legend; non-blocking. | spec/functional/FR-043-admit-the-checked-package-handoff.md | wrong-requirement |

## Dispositions (round 2)

| FND | Outcome | Reason |
| --- | --- | --- |
| FND-007 | fixed | Round 2: 348 and 231 recomputed as 315 + 33 and 198 + 33; 40 rows; 226 test-backed. |
| FND-008 | fixed | Round 2: FR-043 Outputs add the incomplete read, FR-043-AC-7, TC-149; matches CheckedPackageIncomplete and exceeds(consumed, limit). |
| FND-009 | fixed | Round 2: FR-045 and FR-046 and TC-151 and TC-152 marked Blocked; TC-150 stays Planned. |
| FND-010 | still-open | new this round, low, non-blocking. |

## Verdict

Not mergeable until FND-001 and FND-002 are resolved; FND-003 to FND-006 are medium.

Round 1: FND-001 to FND-006 fixed; FND-007 (count arithmetic) still open; FND-008 and FND-009 new, low. Not mergeable until FND-007 and FND-008 are fixed.

Round 2: FND-007, FND-008 and FND-009 fixed; FND-010 is new, low, non-blocking. Mergeable.
