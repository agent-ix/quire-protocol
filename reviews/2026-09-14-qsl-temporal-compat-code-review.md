---
id: SR-081
title: "Code and Rust review of the QSL temporal compatibility repin"
type: SpecReview
analysis: code-review
scope: "quire-protocol#56; QSL 7931a00 repin at 3e5ddbd over e5d2afd"
review_set: subset
relationships:
  - { target: ix://agent-ix/quire-protocol/AP-001, type: reviews }
  - { target: ix://agent-ix/quire-protocol/NFR-006, type: reviews }
  - { target: ix://agent-ix/quire-protocol/IT-001, type: references }
  - { target: ix://agent-ix/quire-protocol/Task-009, type: references }
  - { target: ix://agent-ix/quire-protocol/Task-013, type: references }
---
# Code and Rust review of the QSL temporal compatibility repin

## Summary

Reviewed the complete dependency-boundary change at
`3e5ddbd3acde38f783c07f8a05fb8361d9185705` over
`e5d2afd267ef9eaf53749733bc7128679e5b2ae4`. QProtocol now selects the promoted
QSL FR-052 compatibility correction at
`7931a0021612d26132ea8bbef3b0cd19b27248e0`; the manifest, lockfile, dependency
inventory test, rights record and lifecycle/reference statements agree. Two
stale dependency-record statements found during review were repaired before
this artifact was written.

## Verdict

**PASS** — the promoted owner revision compiles through every QProtocol target,
the complete locked local gate passes, and no unresolved finding remains in the
dependency-repin scope.

## Assurance Context

- **Profile:** AP-001, `spec/assurance/AP-001-protocol-implementation.md`,
  profile version 0.2, status `proposed`. It applies because the changed build
  closure is part of the exact Rust candidate selected by the profile.
- **Baseline and candidate:** baseline
  `e5d2afd267ef9eaf53749733bc7128679e5b2ae4`; evaluated candidate
  `3e5ddbd3acde38f783c07f8a05fb8361d9185705` on
  `agent-b/qsl-temporal-compat`. Changed paths are `Cargo.toml`, `Cargo.lock`,
  `DEPENDENCIES.md`, `spec/spec.md`, and
  `tests/it001_dependency_inventory.rs`.
- **Impact scenarios:** `impact-identity-substitution` and
  `impact-assurance-duplication`. The review checked exact manifest/lock/record
  identity, one selected frontend revision, compilation of the downstream
  consumer API, and absence of a second reader, adapter, parser, evaluator,
  callback, runner or evidence path.
- **Architecture and owner context:** AD-001's parser-free consumer boundary,
  NFR-006, IT-001, Plan-001 Tasks 009/013, QSL issue 95 and QProtocol issue 56.
  QSL remains the sole owner of FR-051/FR-052 temporal subject/request/result
  authority; this change imports the promoted owner implementation unchanged.
- **Unavailable or stale selected context:** AP-001 and AD-001 remain proposed.
  No MeasurementPlan, evidence-independence record or producer-reliance
  decision is selected for this dependency-only correction. The repository-wide
  matrix has a pre-existing `Coverage Status`/`Status` validator-version
  mismatch; it is not changed or represented as completion by this repin.
- **Active exceptions:** none.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No unresolved findings | - |

## Review-cycle remediations

| Finding | Resolution |
| --- | --- |
| The lifecycle status still named the pre-merge QSL compatibility candidate after the manifest selected the promoted merge. | Replaced the stale candidate abbreviation with the promoted `7931a00` revision. |
| The touched rights record claimed exact lockfile agreement while naming an obsolete Contract-IR revision and an obsolete QSL selection in its role text. | Reconciled the record to the resolved `quire-contract-ir` `53cc03c...` revision and the promoted QSL selection. |

## Rust review

| Check | Result |
| --- | --- |
| Repository conventions | `AGENTS.md`, `CLAUDE.md`, AP-001, NFR-004 and the repository's bare `ix_trace_rs::trace` convention were applied. |
| Owner boundary | No type, enum, schema, parser or evaluator was declared. The only production change is selection of QSL's promoted owner revision. |
| Dependency identity | `Cargo.toml`, `Cargo.lock`, `DEPENDENCIES.md` and `FRONTEND_REVISION` select exact 40-hex revisions; the lock contains exactly one QSL package and one resolved Contract-IR package. |
| API compatibility | Every library, integration-test and doctest target compiles against QSL's restored non-generic constructor-private temporal authority. No QProtocol public signature or runtime behavior changed. |
| Test quality | The existing IT-001 inventory controls read the real manifest, lockfile and library source, require the exact selected QSL revision, require exactly one frontend instance and forbid an unannounced module/dependency. |
| Panic and unsafe surface | No production Rust changed. No `unsafe`, panic path, integer conversion, async/blocking operation, lock, I/O or warning suppression was introduced. |
| Gate integrity | No workflow, Cargo feature, lint setting, gate command, coverage threshold or hosted workflow changed. The lockfile remained byte-identical throughout the final gate. |

## Gates

Run locally with one Cargo job, the worktree-local target directory and the
shared heavy-check lock:

- `cargo fmt --all -- --check` — pass.
- `cargo clippy --locked --all-targets -- -D warnings` — pass, zero warnings.
- `cargo test --locked` — 219 regular tests and 37 doctests passed, 0 failed;
  23 pre-existing generated version-1 fixture tests were explicitly ignored.
- The three IT-001 dependency inventory controls passed against the promoted
  revision, including the one-frontend and exact-pin assertions.
- `quire validate --scope . spec/spec.md` — pass for the changed specification
  document. All 171 specification documents are grammar-clean; the unchanged
  matrix structural-column mismatch is recorded above rather than hidden.

No hosted workflow was dispatched.

## Boundary

This review establishes only QProtocol issue 56's source-compatible QSL repin.
It does not claim Plan-001, Task-013, qualification, or the wider protocol
implementation complete.
