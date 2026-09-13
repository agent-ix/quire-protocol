---
id: SR-055
title: "Rust review — Task-013 version-lock resolution"
type: SpecReview
analysis: code-review
scope: "src/version_lock.rs, src/version_lock/evidence.rs, src/refusal/catalog.rs, tests/tc_013_version_lock.rs, tests/tc_013_scenario_evidence.rs at 582aed5 over e53fb55"
review_set: subset
relationships:
  - { target: ix://agent-ix/quire-protocol/Plan-001, type: references }
  - { target: ix://agent-ix/quire-protocol/Task-013, type: reviews }
  - { target: ix://agent-ix/quire-protocol/FR-009, type: references }
  - { target: ix://agent-ix/quire-protocol/TC-013, type: references }
---
# Rust review — Task-013 version-lock resolution

## Summary

Reviewed the Task-013 B-owned version-lock resolver and scenario-evidence values
at implementation head `582aed5` over `e53fb55`. The generic resolver preserves byte
identity, enforces the complete manifest and delivery surface under explicit
limits, returns typed refusal, unexecutable and incomplete outcomes, and exposes
no external or self-authorizing capability. The scenario-evidence constructor
now rejects every absent member of the complete 20-dimension budget population
with the exact missing dimension, and TC-013 exhaustively exercises that
population.

Task-013 itself is not complete. FR-012 requires every concrete entry key to
contain the owner's contract version, but six intended producers publish no
such version. This review does not invent those keys or claim the external
integration scenarios executed.

## Verdict

**PASS for the delivered B-owned slice; Task-013 remains PARTIAL.** The
high-severity budget-population mismatch is fixed and the exact Rust slice is
mergeable. The owner-version and global resource-inventory findings remain
explicitly open and prevent a whole-task or whole-plan completion claim.

## Assurance Context

- **Profile:** AP-001, profile version 0.2, status `proposed`; the selected
  evidence method for this change is code review under the repository's Rust
  gates.
- **Baseline:** repository base `e53fb554c64ce3da1921063bffbe0aa8bff74ba0`;
  evaluated implementation revision
  `582aed52b9b6252e839fa7695c0dbe39d6c89bd9` and the paths named in `scope`.
- **Impact evaluated:** identity substitution, false conformance, unbounded
  input work and self-authorization. Exact-byte newtypes, key-only entry
  equality, exact delivered-byte digesting, typed outcome separation, explicit
  budgets and a capability-free resolver address the B-owned portion.
- **Owner boundary:** the temporal, observation/output-mapping, Engineering
  Assurance, E02, diagnostic-catalog and Quoin contract versions are not
  published. A concrete current manifest therefore remains unauthorable.
- **Active exceptions:** none.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | **Open:** six intended producer owners publish no contract version, so B cannot author their required `(owner, contract version)` keys or execute the concrete integration manifest without inventing owner identity. | FR-009; FR-012; Task-013 |
| FND-002 | medium | **Fixed:** manifest duplicate validation and required-key lookup initially occurred outside the deadline and the entry budget covered only the required subset. `ef1b19b` starts the deadline before validation and charges the full manifest, with a regression test for both dimensions. | FR-009-AC-6; FR-016-AC-8 |
| FND-003 | medium | **Fixed:** `FR-017-AC-4` uses Inspection evidence but remained attached to a Rust test. `adfaa0f` removed that false Test credit and reconciled the genuine TC-013 AC-5 binding; coverage is now 58/322. | FR-017-AC-4; TM-001 |
| FND-004 | medium | **Open:** the new allocation, retention and iteration sites still require inclusion in C-30's repository-wide bidirectional resource-site inventory. The resolver's local entry, byte and deadline dimensions are enforced, but that does not complete TC-009/NFR-002. | TC-009; NFR-002; Task-008 |
| FND-005 | high | **Fixed at `582aed5`:** `BudgetDimension::ALL` is generated from the enum's declaration, `ScenarioEvidenceRecord::new` returns the exact missing dimension, the positive fixture supplies all 20 dimensions, and one exhaustive negative test removes each member in turn. | `src/version_lock/evidence.rs`; `tests/tc_013_scenario_evidence.rs`; FR-017; NFR-005 |
| FND-006 | medium | **Fixed in the review follow-up:** TM-001's functional-coverage table used the obsolete `Status` heading and failed the installed TestMatrix schema; it now uses `Coverage Status` and the scoped Quire validation passes. | `spec/tests.md`; TM-001 |

## Rust review

| Check | Result |
| --- | --- |
| Owner and identity types | `ExactBytes` retains byte identity. `EntryKey` owns owner plus contract version. `ManifestEntry` equality is deliberately key-only as FR-012 requires; separate tests verify revision, digest, status and reason retention. No producer enum or unpublished contract version is mirrored. |
| Invalid-state surface | The private `EntrySubject` makes a pinned artifact without a digest unrepresentable. Artifact and interface constructors are distinct; unresolved artifact and unresolved interface states are explicit. |
| Outcome and refusal identity | `ResolutionOutcome` keeps resolved, refused, unexecutable and resource-incomplete states distinct. All 12 local resolution refusal codes are explicit and centrally registered; tests compare typed variants and keys rather than rendered text. |
| Exact-byte boundary | Artifact verification computes `ByteDigest::of` over the selected delivery member's exact bytes and returns that same byte sequence. No canonicalization or repair path exists. Interface entries refuse synthesized digests. |
| Resource discipline | Resolver entry count covers the full submitted manifest; bytes count only bytes actually digested; the monotonic deadline wraps manifest validation, delivery validation and every digest unit. Every entered deadline unit is closed on mismatch as well as success. Exact-limit, one-short and fresh-attempt controls execute. The evidence-record constructor requires the complete declared budget-dimension population. |
| Capability containment | The public resolver accepts manifest, required keys, deliveries, a budget and a `Clock`. It accepts no filesystem path, delivery index, sidecar, VCS, process, network, persistence or evidence-store capability. |
| Panic and unsafe surface | No `unsafe`, `unwrap`, `expect`, `panic!`, filesystem, process or network operation occurs in the production version-lock modules. Checked arithmetic and typed incomplete outcomes cover untrusted size accounting. |
| Async and lock discipline | The slice is synchronous, holds no lock and performs no blocking I/O. The injected monotonic `Clock` makes deadline behavior directly testable. |
| Test quality | Fourteen tests exercise the public resolver/evidence APIs. Resolver mutation controls cover duplicate/missing/unresolved entries, revision/assertion/reason/member defects, delivery selection and digest substitution, adjacent versions, each resolution-budget dimension, equality identity, capability absence and cross-scenario refusal distinctness. The scenario-evidence controls require all 20 dimensions and remove each one independently. |
| Trace honesty | TC-013 and Test-method criterion tags name executed behavior. Inspection/Analysis rows receive no Test credit; external IT-002/IT-003 tags are not claimed. |
| Gate integrity | No dependency, feature, workflow, lint rule, warning suppression or hosted job changed. The source inventory and refusal catalog were extended explicitly. |

## Gates

Run locally on the reviewed implementation using the worktree-local target, one
Cargo job and the shared heavy-check lock:

- `cargo fmt --all -- --check` — clean.
- `cargo clippy --locked --all-targets -- -D warnings` — clean, zero warnings.
- `cargo test --locked --all-targets` — 183 passed, 0 failed, 29 explicitly
  ignored; all 212 test symbols were compiled.
- `cargo test --locked --doc` — 37 passed.
- `quire validate` — 148/148 specification artifacts and 16/16 plan artifacts
  grammar-clean. The non-artifact `spec/reviews/README.md` is excluded.
- `quire coverage --json` — 58/322 rows backed and 212/212/212 Rust
  candidates/tagged/bound after removing the inspection-method trace claim.

The installed Quire 0.31.0 CLI (engine 0.46.0 at `ca7362d4`) has no `advise`
subcommand. Validation, coverage reconciliation and manual
intent-to-test-to-code review were used; no advisor result is fabricated. No
hosted workflow was dispatched.

## Boundary

This review accepts the scenario-evidence constructor and TC-013 budget
completeness claim. It does not accept the concrete manifest, producer revision pins,
IT-002/IT-003 execution, TC-009's global resource inventory, PT02, observation
adequacy, replay agreement or campaign completion.
