---
id: SR-075
title: "Code and Rust review of the Task-014 local gate policy"
type: SpecReview
analysis: code-review
scope: "quire-protocol Task-014, NFR-004, TC-012 and tests/tc_012_local_gate.rs at ad97c46 over 36af8d7"
review_set: subset
relationships:
  - { target: ix://agent-ix/quire-protocol/AP-001, type: reviews }
  - { target: ix://agent-ix/quire-protocol/NFR-004, type: reviews }
  - { target: ix://agent-ix/quire-protocol/TC-012, type: references }
  - { target: ix://agent-ix/quire-protocol/Task-014, type: references }
---
# Code and Rust review of the Task-014 local gate policy

## Summary

Reviewed the Task-014 test-owned Rust policy model, documented command surface,
Cargo-feature and conditional-compilation inventory, negative controls and
tracking changes at `ad97c468803c58ba313fda32f17f67dedd52a36c` over main
`36af8d7bb4753ea89f020fe1e5080cef21879b65`. The review found two fail-closed
feature-selection gaps; both were fixed and killed by discriminating tests
before this artifact was written. No unresolved code or Rust finding remains.

## Verdict

**PASS** — the reviewed scope is idiomatic test-only Rust, every supported
repository state is checked exactly, unsupported configuration states refuse,
and the change introduces no production runner, parser or assurance framework.

## Assurance Context

- **Profile:** AP-001, `spec/assurance/AP-001-protocol-implementation.md`,
  profile version 0.2, status `proposed`. It applies because Task-014 controls
  the local evidence gate and the assurance-duplication impact boundary.
- **Baseline and candidate:** baseline
  `36af8d7bb4753ea89f020fe1e5080cef21879b65`; evaluated candidate
  `ad97c468803c58ba313fda32f17f67dedd52a36c` on
  `task/31-local-gate`.
- **Impact scenario:** `impact-assurance-duplication`. The review checked that
  the implementation remains a test-owned policy model and adds no subprocess
  runner, rendered-output parser, evidence store, hosted qualification path or
  false feature lane.
- **Architecture and owner context:** PR #15 remains the fixed architecture;
  NFR-004, TC-012 and Plan-001 Task-014 own this repository-local gate. Agent
  B's TL-* lane and Agent E's Contract-IR bridge are unaffected.
- **Unavailable or stale selected context:** AP-001 remains proposed. No
  external qualification evidence, MeasurementPlan or producer-reliance
  decision is selected for this local gate-definition ticket.
- **Active exceptions:** none.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No unresolved findings | - |

## Review-cycle remediations

| Finding | Resolution |
| --- | --- |
| A `cfg(feature = "ghost")` condition without a declared Cargo feature reached only the missing-lane classification. | The policy now returns the stable `gate-feature-undeclared` identity, with a fixture that supplies an otherwise adequate opposing lane so the mutation isolates declaration failure. |
| An optional dependency could be represented by two empty feature lanes. | Opposing-lane admission now requires a non-empty selected lane; the two-empty-lane mutant refuses and a selected implicit-dependency lane admits. Whitespace-insensitive `optional=true` detection closes the compact manifest spelling. |

## Rust review

| Check | Result |
| --- | --- |
| Repository conventions | `AGENTS.md`, `Cargo.toml`, AP-001 and the repository's bare `ix_trace_rs::trace` convention were applied. |
| Placement and seam | The model lives only in `tests/tc_012_local_gate.rs`. Source inspection is appropriate because the subject is the repository definition itself and Task-014 forbids a production runner or subprocess parser. |
| Exactness and refusal | Command set/order, risk binding, serialization, warning channels, Cargo features, conditions, target directory and evidence state have distinct stable defect identities. Unknown target predicates and undeclared/phantom features refuse; none is approximated. |
| Test quality | Ten traced tests exercise the actual documented command block, manifest and four tracked conditional sites plus independent mutations. Stable-code uniqueness, wrong-order/set, empty opposing lane, undefined feature, phantom feature, unselected target and state changes are discriminating rather than tautological. |
| Panic and unsafe surface | No production code changed. Test-only `panic!`/`expect` calls identify impossible fixture/read failures; no unsafe code or warning suppression was added. |
| Determinism and resources | The inventory is rooted at `CARGO_MANIFEST_DIR`, files and identities use deterministic `BTreeSet`/sorted order, and no time, random, network, port, environment-home or concurrent state is consulted. |
| Gate integrity | No workflow, dependency, feature, lint table or coverage threshold changed. The documented compiling commands gained only the required literal worktree target directory. |

## Review evidence

- TDD red: all 10 TC-012 scaffolds failed explicitly before implementation.
- Targeted green at the reviewed candidate: 10 passed, 0 failed, 0 ignored.
- Targeted Clippy with `-D warnings`: pass.
- Cargo feature graphs: default, no-default-features and all-features normalize
  identically to SHA-256
  `46f12ad2da0ebc3bd7044774b3d5b11b4be9f8c2c7d21af5e84b11b676deb40e`.
- Real compiler negative controls: removing the manifest Clippy-deny channel
  still fails the introduced `clippy::len_zero` under CLI `-D warnings`; removing
  CLI `-D warnings` still fails it under manifest `clippy::all = "deny"`.
- Undeclared `--features quire-extraction`: Cargo exit 101 with no feature
  declaration or substitute lane.

## Boundary

This review passes only Task-014/NFR-004/TC-012. It does not qualify the whole
protocol implementation, close Task-011, or claim any TL, Contract-IR,
observation, E02 or public-release result.
