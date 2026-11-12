---
id: SR-2127059
title: "TL-390 PR #151 code-review (rust-review lane)"
type: SpecReview
analysis: code-review
review_set: subset
scope: "agent-ix/quire-protocol; TL-390; PR #151 head e239071; Cargo.lock, src/deadline.rs, src/declarations.rs, src/lib.rs, src/observation_link.rs, tests/tc_154_settle_deadline_captures.rs"
relationships:
  - { target: ix://agent-ix/quire-protocol/FR-100, type: reviews }
---
## Summary

Ticket: TL-390. Code review with the Rust lane (rust-review, rust-style) folded in, scoped to `git diff origin/main...HEAD` of PR #151 at head e239071. Focused check: `cargo test --locked --test tc_154_settle_deadline_captures` on the head. No full gate run.

## Examined

- FR-100-AC-23 (spec/functional/FR-100-settle-inclusive-deadline-branches-on-complete-progress.md)
- FR-100-AC-24 (spec/functional/FR-100-settle-inclusive-deadline-branches-on-complete-progress.md)
- src/observation_link.rs (RoleMapping, Unlinked, Basis)
- src/deadline.rs (Wait::new, Wait::admit, assess, WaitRefusal::ObservationUnlinked)
- src/declarations.rs (await_control, AwaitAnchorDeclaration)
- Cargo.lock (qsl-package rev bump and co-bumped git sources)

## Verdict

**FAIL — two medium findings to fix before merge.** The anchor is read from QSL's `protocol_view::await_anchor` on the checked await node; the caller-attributed path is gone with no compatibility twin; `Basis::verify` runs before any wait state mutates, so a refused link changes nothing; the refusal is typed (`WaitRefusal::ObservationUnlinked(Unlinked)`); tests use `#[trace]`. The Cargo.lock bump is acceptable: every bumped source is a `branch = "main"` git dependency, and the new qsl-foundation pulls quire-canonical, so the QSL rev needs a consistent closure of its sibling git sources. That is a normal lock refresh, not a pin. Defects: the subject match drops two of the three FR-287 key members, and the AC-24 "unaltered" assertion cannot fail.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | `Basis` keeps only `subject().identity`, the producer-local object-identity bytes, and `verify` compares them with `WorkflowInstanceId::identity_bytes()`. The FR-287 subject key is (authority, kind, identity), and `SubjectRef` documents `identity` as producer-local. A record from another producer authority, or of another subject kind, that happens to share identity bytes is linked to the waiting instance. QSpec FR-260-AC-6 keys the instance on the subject, not on its local bytes. | src/observation_link.rs:66-80, 101 |
| FND-002 | medium | The AC-24 test asserts "the wait is unaltered" with `wait.current().is_none()`, which already holds for a fresh wait whatever `admit` does. It would still pass if a refused link set `last` or `anchored_by`. Assert it by admitting a linked `after` observation at the same position after each refusal and expecting `Ok`. | tests/tc_154_settle_deadline_captures.rs:1047-1085 |
| FND-003 | low | The AC-23 test checks the second linked `after` observation with `is_err()`, which passes on any refusal, including an unlinked one. Assert `WaitRefusal::AmbiguousAnchor { .. }`. | tests/tc_154_settle_deadline_captures.rs:1040 |
| FND-004 | low | `RoleMapping` stores each binding's nodes in a `Vec` and `resolving` extends it, so declaring the same node twice keeps duplicates. FR-261 describes a set, so `BTreeSet<CheckedNodeId>` fits the model and makes `contains` logarithmic. | src/observation_link.rs:23-48 |
| FND-005 | low | The module docs cite "QSpec FR-260-AC-5..9", but the occurrence rank of FR-260-AC-7 is not derived here (FR-100 Inputs says so). The citation overclaims. Cite AC-5, AC-6 and AC-9. | src/deadline.rs:11; src/lib.rs:57; src/observation_link.rs:3 |

## New findings (disposition pass 1)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-006 | low | `Wait::new` gained the `WaitRefusal::SubjectNotInstance` refusal, but its `# Errors` section still lists only `UndeclaredAwait`. | src/deadline.rs:698-712 |
| FND-007 | low | In the test file, the old `mapping` doc comment ("The declared role mapping: ...") now sits above `fn subject`, which therefore carries two unrelated doc paragraphs, and `fn mapping` carries none. | tests/tc_154_settle_deadline_captures.rs:124-138 |

## Dispositions

Round 1, fix commit 4f0d3f4 (reviewed as `git diff e239071..4f0d3f4`).

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | 4f0d3f4: `Basis` keeps the authority (`AdmittedBundleKey`), kind and identity, and `verify` compares all three with the wait's `Link` `QualifiedSubject`. `the_subject_key_is_compared_whole` asserts that equal bytes under another kind are refused with `OtherInstance`. |
| FND-002 | fixed | 4f0d3f4: after each refusal, the AC-24 test admits a linked `after` observation at the same position and expects `Ok`. |
| FND-003 | fixed | 4f0d3f4: AC-23 was narrowed to the anchor source and the loose `is_err()` was removed. `AmbiguousAnchor` stays asserted exactly at tests/tc_154_settle_deadline_captures.rs:919. |
| FND-004 | fixed | 4f0d3f4: `RoleMapping` holds `BTreeSet<CheckedNodeId>`, and an empty set resolves as unresolved. |
| FND-005 | fixed | 4f0d3f4: the citations read FR-260-AC-5, FR-260-AC-6, FR-260-AC-9 in src/deadline.rs, src/lib.rs, src/observation_link.rs and FR-100 Inputs. |
