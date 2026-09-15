---
id: SR-108
title: "Rust review of the Task-019 consumer delivery index"
type: SpecReview
analysis: code-review
scope: "Task-019 / quire-protocol#66 over the delivery-index implementation first merged in PR #68"
review_set: subset
relationships:
  - { target: ix://agent-ix/quire-protocol/Task-019, type: reviews }
  - { target: ix://agent-ix/quire-protocol/FR-018, type: reviews }
  - { target: ix://agent-ix/quire-protocol/NFR-006, type: references }
  - { target: ix://agent-ix/quire-protocol/TC-017, type: reviews }
---
# SR-108: Rust Review of the Task-019 Consumer Delivery Index

## Summary

Reviewed the complete Task-019 Rust and TC-017 evidence as a ticket-specific
promotion of code that first reached `main` inside PR #68 without closing issue
#66. The implementation already held the accepted FR-018 identity, exact-byte,
typed-absence, declaration, mutation, limit and version-lock non-authority
boundaries. This review found and fixed five construction, accounting and
ordering defects. No in-scope finding remains.

## Verdict

**PASS for Task-019 / quire-protocol#66.** The only public completed-index paths
are resource-bounded; validation and multi-dimension charging are transactional;
and every typed ordering/refusal rule is explicit. This subset does not claim
Task-008's plan-wide resource inventory or the remaining Plan-001 work complete.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-001 | high | Resolved: public `DeliveryIndex::new` allowed a caller to retain a completed index without the request accountant, and `observed(Present(..))` used that bypass. The unbounded constructor is now private and every public present path requires `ResourceAccountant`. A one-short adapter-present assertion kills the former bypass. | NFR-002-AC-1/3; FR-018; `src/delivery_index.rs`; TC-017 | implementation-bug-despite-evidence |
| FND-002 | high | Resolved: bounded construction charged exact bytes and allocation in separate mutations, so a later allocation refusal left the earlier byte charge committed although no index existed. Both charges now use `charge_all`; TC-017 consumes the sole allocation, retries, and proves only the first successful transaction remains charged. | NFR-002-AC-1/3; `DeliveryIndex::new_bounded`; TC-017 | implementation-bug-despite-evidence |
| FND-003 | high | Resolved: construction charged request resources before domain validation, allowing duplicate-role or otherwise invalid contents to consume budget. Validation now completes before accounting; TC-017 proves a rejected candidate leaves both relevant dimensions at zero. | NFR-002-AC-1/3; FR-018-AC-5/9; TC-017 | implementation-bug-despite-evidence |
| FND-004 | medium | Resolved: selected temporal declarations were normalized into a set, so duplicate expected declarations passed silently. Cardinality validation now reports the exact repeated declaration before set comparison, with a discriminating TC-017 assertion. | FR-018-AC-3/5; TC-017 | implementation-bug-despite-evidence |
| FND-005 | medium | Resolved: limit ordering converted Rust enum discriminants with `as u8`, coupling the contract to representation details. Exhaustive owner-specific match functions now define every Protocol and compiled-protocol member's order; a future owner variant is a compile-time reconciliation point. | FR-018-AC-9; TC-017 | implementation-bug-despite-evidence |
| FND-006 | medium | Resolved: TC-017's bounded-construction test traced NFR-002-AC-2, which governs monotonic deadlines rather than static limits and source inventory. The symbol now binds AC-1 and AC-3, matching the exercised one-short and inventoried-site behavior. | NFR-002-AC-1/2/3; TC-017 | wrong-requirement |

## Rust Review

| Check | Result |
| --- | --- |
| Ownership and public surface | `DeliveryIndexContents` may stage an adapter value, but only `new_bounded` or `observed` can publish a validated `DeliveryIndex`, and both require the caller's accountant. No parser, filesystem, VCS, network, process, manifest or resolver authority was added. |
| Identity and exact state | Equality uses both `delivery_root` and `producer_revision` and excludes every non-key field. TC-017 now independently changes both key members. Offer and input bytes remain opaque and no digest or canonical form is synthesized. |
| Refusal semantics | Missing root/index/member, unreadable index/member, unindexed/duplicate inputs, missing/surplus/repeated/reordered declarations, ambiguous mutations and repeated typed limits remain distinct `DeliveryIndexError` values. No branch defaults, repairs or approximates. |
| Bounds and transactionality | The complete validated candidate's exact retained byte sequences and one final allocation are charged together before return. One-short, later-charge rollback, invalid-domain no-charge, sufficient-budget and adapter-present controls execute under the production inventory. |
| Ordering and conversions | Temporal records use the producer's ascending declaration rule only. Limit comparison is owner-qualified and exhaustively maps both owners' closed dimension vocabularies; no unchecked integer conversion remains. Checked byte-length addition reports `RetainedBytesOverflow`. |
| Panic, unsafe and concurrency | Production delivery-index code contains no `unsafe`, `unwrap`, `expect`, `panic!`, unchecked indexing, recursion, async/blocking bridge, process, thread or lock. Test-only expectations identify fixture faults. |
| Test quality and trace | Five default-running TC-017 tests inspect exact values and usage, include adverse mutations, and bind all six Test-designated FR-018 criteria plus NFR-006-AC-8. The three declared Inspection/Analysis criteria are not falsely credited as executable coverage. |
| Repository constraints | Rust only, `publish = false`, local `target-codex-backends`, no hosted dispatch, no dependency or workflow change, and no change under `resources/native-v1/`. |

## Verification

Focused TC-017 and focused locked Clippy with warnings denied pass after every
fix. The promotion candidate is required to pass the repository's exact local
gate in order at one unchanged committed head: format check, locked all-target
Clippy with warnings denied, then the complete locked test suite, using one
Cargo job and `--target-dir target-codex-backends`. Rust and Cargo are 1.98.1;
Cargo.lock SHA-256 is
`9b750f798069f5d79216b5efdc70de0acd3d9395ce6fb52534e42e3e4988a882`.
The exact head and command result are retained in the pull-request evidence.

## Boundary

This review closes the Task-019 implementation and promotion surface only. It
does not pre-credit Task-004, Task-008, Task-009, Plan-001, or epic #14.
