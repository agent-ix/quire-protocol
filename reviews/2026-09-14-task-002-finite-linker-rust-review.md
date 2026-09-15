---
id: SR-105
title: "Rust review of the Task-002 exact finite linker"
type: SpecReview
analysis: code-review
scope: "Task-002 / quire-protocol#4 candidate diff over c26ae8e"
review_set: subset
relationships:
  - { target: ix://agent-ix/quire-protocol/Task-002, type: reviews }
  - { target: ix://agent-ix/quire-protocol/FR-002, type: reviews }
  - { target: ix://agent-ix/quire-protocol/TC-002, type: reviews }
  - { target: ix://agent-ix/quire-protocol/IT-001, type: references }
  - { target: ix://agent-ix/quire-protocol/NFR-004, type: references }
  - { target: ix://agent-ix/quire-protocol/NFR-006, type: references }
---
# SR-105: Rust Review of the Task-002 Exact Finite Linker

## Summary

Reviewed the complete Task-002 Rust, test, dependency and tracking candidate
against AD-001, FR-002, TC-002, issue #4's current body and the exact pinned QSL
and QObs owner APIs. The final surface separates `/1` compatibility from `/2`
admission, static selection, runtime binding and transactional linking. Every
finding below was fixed before this artifact was finalized.

## Verdict

**PASS.** No unresolved Rust, owner-boundary, identity, refusal, panic,
conversion, test-quality, trace or local-gate finding remains for Task-002/#4.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-001 | high | Resolved: the first v2 projection reused the frozen `/1` `StaticLinkedProtocol`, exposing slot-derived pseudo-runtime IDs. `LinkedGraph` now retains only compiler-validated control/edge structure; every v2 runtime identity is an imported QObs value. | `LinkedGraph`; FR-002-AC-1; TC-002 | implementation-bug-despite-evidence |
| FND-002 | high | Resolved: static selection could omit declaration slots, reorder complete selections, or select a relationship premise from a different authority. Exact missing/surplus/order/kind/correspondence/authority checks now precede runtime binding and emit no linked value. | `validate_static_definition`; FR-002-AC-3 | correct-requirement-no-evidence |
| FND-003 | high | Resolved: runtime identity reuse was guarded for only three occurrence kinds and could merge delivery, attempt, effect, cross-kind or changed-correspondence bindings. Every single-binding occurrence kind now refuses incompatible reuse; compatible participant reuse remains explicit. | `validate_runtime`; FR-002-AC-1/3 | implementation-bug-despite-evidence |
| FND-004 | high | Resolved: external paths, build provenance and source loci were excluded from FR-288 correctly but discarded rather than retained separately. Public `PresentationContext` retains exact owner types and is deliberately outside semantic equality. | `PresentationContext`; AD-001; FR-002 Outputs | implementation-bug-despite-evidence |
| FND-005 | medium | Resolved: static role and slot diagnostics collapsed missing, surplus and reordered cases. Distinct variants now carry exact role/requirement, ordinal, expected handle and real QSL locus whenever one exists. | `BindingRefusal`; FR-002-AC-3 | correct-requirement-no-evidence |
| FND-006 | medium | Resolved: dependency, reader-call and conditional-compilation inventories described the prior QObs pin and root reader layout. All closed inventories now identify the exact pins, both intake readers and every new `cfg(test)` module. | Cargo manifests; IT-001; TC-012 | correct-requirement-no-evidence |
| FND-007 | medium | Resolved: tracking claimed QSL lacked reachability/choice checks and described a lossy numeric/channel projection that the accepted owner boundary superseded. Documentation now assigns compiler checks upstream and records exact typed projections. | FR-002; TM-001; QSL `f1700a9` | wrong-requirement |

## Assurance Context

- **Baseline:** `c26ae8ebe4f0ca8c70dc5b7cc1565ed50281a1ea`, the Task-013 merge and current branch base.
- **Reviewed authority:** AD-001 and SR-089..SR-100; FR-002; TC-002;
  Task-002; issue #4 and epic #14; QSL `f1700a9264d6d3bcdd07e0f77b70f3dae9ed4c07`;
  QObs `93bc565072fb05d1ecf29f15725b75c0a77378fb`.
- **Candidate surface:** versioned intake/model facades; binding, definition,
  FIFO and runtime-model modules; the frozen static linker rename; QObs fixture
  adoption; dependency and closed source inventories; Plan-001/TM-001 evidence.
- **Deliberately later:** the shared bounded `OperationOutcome`/accountant
  convergence belongs to Tasks 006/008; complete scenario composition belongs
  to Task-018. This review gives neither future allocation credit.

## Rust Review

| Check | Result |
| --- | --- |
| Ownership and API boundaries | QSL alone admits compiled bytes and owns wire records; QObs alone constructs qualified subjects and relationships. Protocol owns only its accepted binding selection, linked representation, FIFO partition and definition registry. Root `/1` delegates to `intake::v1`; no root `/2` shortcut remains. |
| Identity and equality | FR-288 is four componentwise typed members with exact finite selection closures, never a digest/fingerprint. Static requirements cannot become runtime identities. Presentation/provenance remains inspectable but outside FR-288 and linked semantic equality. |
| Refusal fidelity | Unsupported binding contracts, selection mismatch, every static/runtime slot defect, identity contradiction, relationship conflict/ambiguity, zero progress and incomplete admitted members are distinct typed results. Every construction failure returns no linked value and commits no registry row. |
| Graph and FIFO fidelity | Complete QSL control/edge/channel values are retained; no wildcard absorbs a QSL semantic alternative. FIFO retains declaration-qualified binder/key/anchor, reachable value/binder/anchor/scope/type closures and the owner's exact generic evaluated value without adding causality. |
| Transactionality | Static, runtime, identity, presentation and FIFO construction complete before registry commit. Exact registry and relationship replay is idempotent; conflicting content refuses without mutation. |
| Panic, conversion and unsafe surface | New production paths contain no `unsafe`, `panic!`, `unwrap`, `expect`, unchecked narrowing, async/blocking bridge or lock. Fallible integer conversions either select an admitted member or return a typed incomplete/refusal result. Test-only assertions are below explicit `cfg(test)` boundaries. |
| Test quality | TC-002 has 66 default-running tests: 57 linker-module tests, seven cross-kind identity tests and two public v2 integrations. They inspect exact variants/interiors, complete static/runtime coverage, owner values, semantic/presentation mutation, temporal selection components, replay and registry state; none is ignored or environment-gated. |
| Repository constraints | Rust only; `publish = false`; no hosted workflow change or dispatch; `resources/native-v1/` is untouched. The lock graph resolves one QSL, one QObs and one Contract-IR revision. |

## Verification

The reviewed candidate passes format checking, locked all-target Clippy with
warnings denied and the complete locked test suite under one Cargo job, the
shared heavy-check lock and `--target-dir target-codex-backends`. The suite
reports 295 passed tests and 23 intentionally ignored legacy producer-fixture
cases; all 66 TC-002 tests run by default. `quire coverage` reports 145/342
matrix rows backed and 272/272 Rust candidates tagged and bound.

Quire reports all 214 documents grammar-clean. Its structural pass is blocked
only by live quire-cli#79: the installed traceability configuration expects a
`Status` column while the active TestMatrix contract requires `Coverage Status`.
That ticket explicitly forbids renaming the consumer header as a workaround.

## Boundary

This PASS covers Task-002/#4 only. It does not close the wider Protocol epic,
pre-credit Tasks 003–011/018/019, or reinterpret repository-wide unbacked rows
as Task-002 failures.
