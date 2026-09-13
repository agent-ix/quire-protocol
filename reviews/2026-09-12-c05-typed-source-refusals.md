---
id: SR-053
title: "Code review — C-05 typed source refusal retention"
type: SpecReview
analysis: code-review
scope: "src/inventory/map.rs, src/inventory/mod.rs, src/refusal/mod.rs, tests/it001_native_handoff.rs, tests/tc_006_digest_domains.rs"
review_set: subset
relationships:
  - { target: ix://agent-ix/quire-protocol/AP-001, type: reviews }
  - { target: ix://agent-ix/quire-protocol/Task-001, type: reviews }
  - { target: ix://agent-ix/quire-protocol/FR-001, type: references }
  - { target: ix://agent-ix/quire-protocol/TC-001, type: references }
---
# Code review — C-05 typed source refusal retention

## Summary

Reviewed C-05 at `0ab9d91` over `e53fb55`. The admission discriminator now
retains every A-owned error as its complete typed value, including the published
`Producer` and `/2` families that the prior mapper converted into debug text.

## Verdict

**PASS** — the changed Rust boundary, test, and trace claim have no review
finding, and the local Rust gates pass with warnings denied.

## Assurance Context

- **Profile:** AP-001, `spec/assurance/AP-001-protocol-implementation.md`,
  profile version 0.2, status `proposed`. It applies to this exact Rust
  candidate and requires `code-review`.
- **Baseline:** repository base `e53fb554c64ce3da1921063bffbe0aa8bff74ba0`;
  evaluated code revision `0ab9d91` and the five paths named in `scope`.
- **Impact evaluated:** `impact-identity-substitution` and
  `impact-false-conformance`. Converting an A-owned discriminant into rendering
  made a new source refusal indistinguishable without parsing text. Retaining
  the `Error` value and delegating its stable code to `Error::code` removes that
  path.
- **Architecture and owner context:** AD-001's pure typed admission/refusal
  boundary; FR-001/TC-001; FR-008 and NFR-008; and
  `quire-specification` FR-270/FR-272 at `f8b9f9f`. No local parser, source
  catalog, typed-cause vocabulary, I/O path, or new dependency was introduced.
- **Unavailable or stale selected context:** AP-001 and AD-001 remain proposed;
  AD-001's risk text still describes the compiler contract and Engineering
  Assurance producer boundary as unpublished even though later repository
  records say they landed. No selected MeasurementPlan, evidence-independence
  record, or evidence-producer reliance decision was available for this
  increment. C-02/C-03 remain unavailable because FR-270's catalog is not
  adopted and FR-272 supplies no stable cause schema/order.
- **Active exceptions:** none.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings | - |

## Rust review

| Check | Result |
| --- | --- |
| Owner types | `AdmissionRefusal` carries `quire_spec_language::protocol_artifact::Error` directly; no mirror enum or string payload was introduced. |
| Variant identity | The one B-owned refinement remains `Selection { axis, source }`; all other A discriminants remain observable by matching the nested owner enum. |
| Non-exhaustive enums | The compulsory catch-all returns `Source(error)` and therefore cannot coerce a future owner variant into a neighbouring classification. |
| Test quality | The regression test fails if any current A family is dropped or its source value/code changes. It exercises the real public mapper and specifically covers `Producer` and `/2`. |
| Trace honesty | The test carries bare `#[trace("TC-001", "Task-001")]`. The former FR-008 claim was removed because the mapper does not yet construct the complete `ProtocolRefusal`. |
| Panic, unsafe, async, numeric, resource surface | No panic, unsafe, async/blocking, conversion, loop, allocation, or input-processing surface was added. |
| Gate integrity | No workflow, lint, feature, dependency, coverage threshold, or warning suppression changed. No `deny.toml` exists, so there is no `cargo deny` lane to run. |

## Gates

Run locally with the worktree-local `target`, one Cargo job, and the shared
heavy-check lock:

- `cargo fmt --all -- --check` — clean.
- `cargo clippy --locked --all-targets -- -D warnings` — clean, zero warnings.
- `cargo test --locked` — 169 passed, 0 failed, 29 explicitly ignored;
  37 doctests passed.

The ignored tests are the existing external producer-handoff lanes and retain
their named reasons. No hosted workflow was dispatched.

## Boundary

This review does not claim the complete FR-008 envelope. C-05 now preserves the
owner value needed by that later conversion; C-02/C-03 must first receive the
catalog and typed-cause contracts from their upstream owner.
