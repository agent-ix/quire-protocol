---
id: SR-103
title: "Rust review of the Task-013 version-lock resolver"
type: SpecReview
analysis: code-review
scope: "Task-013 / quire-protocol#49 candidate diff over 0cba759"
review_set: subset
relationships:
  - { target: ix://agent-ix/quire-protocol/Task-013, type: reviews }
  - { target: ix://agent-ix/quire-protocol/FR-009, type: reviews }
  - { target: ix://agent-ix/quire-protocol/FR-012, type: references }
  - { target: ix://agent-ix/quire-protocol/FR-016, type: references }
  - { target: ix://agent-ix/quire-protocol/NFR-006, type: references }
  - { target: ix://agent-ix/quire-protocol/TC-013, type: references }
  - { target: ix://agent-ix/quire-protocol/AP-001, type: references }
---
# SR-103: Rust Review of the Task-013 Version-Lock Resolver

## Summary

Reviewed the complete Task-013 Rust, test and tracking candidate against the
current Task-017 foundation on `origin/main`. The final resolver is offline,
bounded, deterministic and incapable of minting a capability until exact
request, key, revision, subject and byte checks complete. Every in-scope finding
was fixed before this artifact was finalized.

## Verdict

**PASS.** No unresolved Rust, identity, refusal-cause, panic-surface, resource,
test-quality, trace or local-gate finding remains in Task-013/#49.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-001 | high | Resolved: derived equality on manifest/resolution aggregates inherited FR-012's key-only entry equality and could call differing interface revisions equal. Aggregate equality was removed; exact-state assertions compare every retained field. | `VersionLockManifest`; `ResolvedRequest`; FR-012 | implementation-bug-despite-evidence |
| FND-002 | medium | Resolved: presenting an artifact entry to the interface selector emitted an identity-mismatch cause with equal revision bytes instead of the actual subject-kind mismatch. The path now emits exact `interface` versus `artifact` anchors and a killing test observes both. | `ResolvedRequest::interface`; FR-009-AC-8 | implementation-bug-despite-evidence |
| FND-003 | medium | Resolved: several negative tests asserted only refusal-code spelling, so a regression could name the wrong key/member and remain green. The tests now inspect exact affected identities and exact mismatch/reason interiors. | TC-013; FR-009-AC-2/3/5/8 | correct-requirement-no-evidence |
| FND-004 | medium | Resolved: the first budget constructor narrowed the deadline with a panic-capable conversion. `ResolutionBudget::new` is fallible, refuses zero and unrepresentable grants explicitly, and has boundary tests. | `ResolutionBudget`; FR-009-AC-6 | implementation-bug-despite-evidence |

## Assurance Context

- **Profile:** `spec/assurance/AP-001-protocol-implementation-assurance-profile.md`.
  Its no-exception policy applies; this ticket adds no exception.
- **Baseline:** merge base and current `origin/main` are
  `0cba7599a64162ade00cf08c01638b6c4cbe20b5` (Task-017 / PR #65).
- **Candidate surface:** `src/version_lock.rs` and its internal tests; the shared
  resource inventory/deadline path; the refusal catalog; module/conditional and
  dependency inventories; Task-013, TC-013, Plan-001 and TM-001 tracking. Cargo,
  lockfile, workflows and `resources/native-v1/` are unchanged.
- **Impact scenarios reviewed:** critical identity substitution, false
  completed resolution, byte substitution after verification, ambiguous first
  cause, resource exhaustion reported as a domain refusal, and self-authorized
  producer context.
- **Available authority:** AD-001, ADR-009, FR-009, FR-012, FR-016, NFR-006,
  TC-013, Task-013, issue #49's current body, Task-017's shared outcome/refusal
  API, and the final Rust/tests in this candidate.
- **Unavailable or deliberately later:** concrete QSL/QObs/EA/E02/Quoin entry
  population belongs to Tasks 009–011; final scenario composition belongs to
  Task-018. The distinct FR-018 consumer `DeliveryIndex` is absent and is now
  honestly assigned to Task-019/#66 and TC-017; this review gives it no credit.

## Rust Review

| Check | Result |
| --- | --- |
| Ownership and public surface | Protocol owns manifest/key/status/delivery/budget/refusal vocabulary. Resolver, selections, resolved request and capabilities remain crate-private so callers cannot self-authorize adapter selections. Public entry fields are read-only exact-byte accessors. |
| Identity and equality | `EntryKey` is exact non-empty owner plus contract-version bytes. `ManifestEntry` equality is intentionally key-only per FR-012 and documented as such; aggregate types do not inherit that equality. Capability checks separately compare request, key, revision and Rust contract marker. |
| Refusal fidelity | Seventeen resolution axes have unique registered spellings in the Resolution domain. Every path uses the Task-017 envelope at `ContractResolution`, emits no partial value, and retains an exact key/member plus a typed cause. Resource exhaustion remains `ResourceIncomplete`, never a refusal. |
| Exact bytes and containment | Digests are computed over delivered bytes and the verified byte allocation is retained for capability release. No parser, normalization, filesystem, VCS, network, process, discovery or sidecar-selection path exists. |
| Bounds | The shared accountant adds independent entry-count, bytes-digested and monotonic resolution-deadline dimensions. Charges precede validation/digest work, maxima are inclusive, one-short attempts stop structurally, and fresh requests receive fresh accounting. |
| Panic and conversion surface | Caller-controlled key/request/budget construction is fallible where absence or representation can fail. Production `expect`/`unreachable` sites are post-validation or compile-time catalog/inventory invariants; no caller input can violate their stated premise. No unchecked narrowing, indexing, recursion, `unsafe`, async/blocking bridge or lock was added. |
| Future private foundation | Narrow `cfg_attr(not(test), expect(dead_code, reason = ...))` annotations cover only the crate-private resolver/capability path awaiting Tasks 009–011. The warnings become unfulfilled under `-D warnings` once adoption makes an expectation stale; no blanket `allow` exists. |
| Test quality | Eleven default-running tests exercise success and every refusal family, exact identity/cause interiors, key-only versus state equality, add/remove/repin independence, both manifest permutations, all three exact/one-short/fresh budgets, exact-byte substitution, subject mismatch and cross-request/key/revision capability reuse. No test is ignored, environment-gated, mock-only or tautological. |
| Repository constraints | Rust only and `publish = false` remain unchanged. Cargo.toml, Cargo.lock (SHA-256 `2824b45d644459229f0eb2293cc6f1bff12ee77dd16dd83c73e93f9fc0cbd3b5`), dependency records, workflows and native-v1 resources are unchanged; no hosted CI was dispatched. |

## Verification

The final committed candidate passes the repository-defined local gate in its
required order with Rust/Cargo 1.98.1, `CARGO_BUILD_JOBS=1`, the worktree-local
heavy lock, `--locked`, and `--target-dir target-codex-backends`: format check,
all-target Clippy with warnings denied, then the complete test suite. The
eleven-test TC-013 lane and its adjacent inventory/catalog/gate lanes pass with
zero ignored Task-013 tests. Quire validation and the scoped coverage audit also
pass; exact head and command output are retained in PR #50 evidence.

## Boundary

This PASS covers the resolver/capability mechanism only. It neither pre-credits
concrete producer integration nor the missing DeliveryIndex, finite linker,
assessment engine, E02 projection, scenario evidence or qualification work.
