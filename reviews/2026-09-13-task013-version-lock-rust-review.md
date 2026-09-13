---
id: SR-055
title: "Rust review — Task-013 version-lock resolution"
type: SpecReview
analysis: code-review
scope: "src/version_lock.rs, src/version_lock/evidence.rs, src/refusal/catalog.rs, tests/tc_013_version_lock.rs, tests/tc_013_scenario_evidence.rs at dae1277 over e53fb55"
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
at PR head `dae1277` over `e53fb55`. The generic resolver preserves byte
identity, enforces the complete manifest and delivery surface under explicit
limits, returns typed refusal, unexecutable and incomplete outcomes, and exposes
no external or self-authorizing capability. The scenario-evidence slice is not
mergeable as reviewed: its constructor accepts an incomplete budget population
as a complete record.

Task-013 itself is not complete. FR-012 requires every concrete entry key to
contain the owner's contract version, but six intended producers publish no
such version. This review does not invent those keys or claim the external
integration scenarios executed.

## Verdict

**FAIL for the delivered B-owned slice; Task-013 remains PARTIAL.** A high-severity
specification/code/test mismatch permits a record with only 3 of the 20 declared
budget dimensions and is positively asserted as complete. The owner-version and
global resource-inventory findings also remain explicitly open and prevent a
whole-task or whole-plan completion claim.

## Assurance Context

- **Profile:** AP-001, profile version 0.2, status `proposed`; the selected
  evidence method for this change is code review under the repository's Rust
  gates.
- **Baseline:** repository base `e53fb554c64ce3da1921063bffbe0aa8bff74ba0`;
  evaluated implementation revision
  `dae1277b7c2cbb32f87c4a646b142b7df6a94c12` and the paths named in `scope`;
  the implementation content is at `adfaa0f809a9b92ecb768cce4e18682ac97c9d3a`.
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
| FND-005 | high | **Open:** `ScenarioEvidenceRecord::new` rejects duplicate budget dimensions but never rejects an absent dimension. The positive fixture supplies only `resolution-entry-count`, `resolution-bytes-digested`, and `resolution-deadline`, then asserts that the record has every required component even though `BudgetDimension` declares 20 dimensions. FR-017 and NFR-005 require granted and consumed figures on every declared dimension, with an unexecuted result when any required component cannot be retained. | `src/version_lock/evidence.rs:134-157,277-340`; `tests/tc_013_scenario_evidence.rs:68-74,105-136`; FR-017 Description, Properties and AC-1/AC-3/AC-5; NFR-005 Statement and Verification |

## Rust review

| Check | Result |
| --- | --- |
| Owner and identity types | `ExactBytes` retains byte identity. `EntryKey` owns owner plus contract version. `ManifestEntry` equality is deliberately key-only as FR-012 requires; separate tests verify revision, digest, status and reason retention. No producer enum or unpublished contract version is mirrored. |
| Invalid-state surface | The private `EntrySubject` makes a pinned artifact without a digest unrepresentable. Artifact and interface constructors are distinct; unresolved artifact and unresolved interface states are explicit. |
| Outcome and refusal identity | `ResolutionOutcome` keeps resolved, refused, unexecutable and resource-incomplete states distinct. All 12 local resolution refusal codes are explicit and centrally registered; tests compare typed variants and keys rather than rendered text. |
| Exact-byte boundary | Artifact verification computes `ByteDigest::of` over the selected delivery member's exact bytes and returns that same byte sequence. No canonicalization or repair path exists. Interface entries refuse synthesized digests. |
| Resource discipline | Resolver entry count covers the full submitted manifest; bytes count only bytes actually digested; the monotonic deadline wraps manifest validation, delivery validation and every digest unit. Every entered deadline unit is closed on mismatch as well as success. Exact-limit, one-short and fresh-attempt controls execute. The evidence-record constructor does not enforce the complete declared budget-dimension population (FND-005). |
| Capability containment | The public resolver accepts manifest, required keys, deliveries, a budget and a `Clock`. It accepts no filesystem path, delivery index, sidecar, VCS, process, network, persistence or evidence-store capability. |
| Panic and unsafe surface | No `unsafe`, `unwrap`, `expect`, `panic!`, filesystem, process or network operation occurs in the production version-lock modules. Checked arithmetic and typed incomplete outcomes cover untrusted size accounting. |
| Async and lock discipline | The slice is synchronous, holds no lock and performs no blocking I/O. The injected monotonic `Clock` makes deadline behavior directly testable. |
| Test quality | Thirteen tests exercise the public resolver/evidence APIs. Resolver mutation controls cover duplicate/missing/unresolved entries, revision/assertion/reason/member defects, delivery selection and digest substitution, adjacent versions, each resolution-budget dimension, equality identity, capability absence and cross-scenario refusal distinctness. The scenario-evidence positive test supplies only three dimensions and has no missing-dimension negative control (FND-005). |
| Trace honesty | TC-013 and Test-method criterion tags name executed behavior. Inspection/Analysis rows receive no Test credit; external IT-002/IT-003 tags are not claimed. |
| Gate integrity | No dependency, feature, workflow, lint rule, warning suppression or hosted job changed. The source inventory and refusal catalog were extended explicitly. |

## Gates

Run locally on the reviewed implementation using the worktree-local target, one
Cargo job and the shared heavy-check lock:

- `cargo fmt --all -- --check` — clean.
- `cargo clippy --locked --all-targets -- -D warnings` — clean, zero warnings.
- `cargo test --locked --all-targets` — 182 passed, 0 failed, 29 explicitly
  ignored; all 211 test symbols were compiled.
- `cargo test --locked --doc` — 37 passed.
- `quire validate` — 148/148 specification artifacts and 16/16 plan artifacts
  grammar-clean. The non-artifact `spec/reviews/README.md` is excluded.
- `quire coverage --json` — 58/322 rows backed and 211/211/211 Rust
  candidates/tagged/bound after removing the inspection-method trace claim.

The installed Quire 0.31.0 CLI (engine 0.46.0 at `ca7362d4`) has no `advise`
subcommand. Validation, coverage reconciliation and manual
intent-to-test-to-code review were used; no advisor result is fabricated. No
hosted workflow was dispatched.

## Boundary

This review does not accept the scenario-evidence constructor or its TC-013
completeness claim until FND-005 is closed. It also does not accept the concrete manifest, producer revision pins,
IT-002/IT-003 execution, TC-009's global resource inventory, PT02, observation
adequacy, replay agreement or campaign completion.
