---
id: SR-2127002
title: "Rust review of QSL-1740 FR-044 linker residuals"
type: SpecReview
analysis: code-review
scope: "QSL-1740 / FR-044 / `src/link.rs`, TC-150, handoff test support"
review_set: subset
relationships:
  - { target: ix://agent-ix/quire-protocol/FR-044, type: reviews }
  - { target: ix://agent-ix/quire-protocol/TC-150, type: reviews }
  - { target: ix://agent-ix/quire-protocol/Task-022, type: references }
---
# SR-2127002: Rust Review of QSL-1740 FR-044 Linker Residuals

## Summary

Reviewed the changed Rust implementation and tests against the repository's
owner-boundary rules, public API conventions, test-trace requirements, error
behavior, bounded-input assumptions, and the local gate.

## Verdict

**PASS for the changed Rust paths.** No unresolved Rust implementation,
boundary, lint, or test-execution finding remains in this subset.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder). | `src/link.rs`; TC-150; `tests/support/handoff.rs` |

## Rust Review

| Check | Result |
| --- | --- |
| API and docs | `LinkRefusal` and each public variant field are documented. The new `LinkedProtocol` accessors document their values and ordering. The public link function describes owner and consumer refusals. |
| Owner boundary | Checked content is read through Contract IR's `graph`, `term`, source-map and temporal-binding accessors. QSL owns protocol listing, delivery, repeat operation, temporal selection, causal relation and source loci. The linker does not parse package bytes or construct causal edges. |
| Refusals | Owner `ViewRefusal` values remain wrapped unchanged. Consumer refusals identify checked nodes and retain only QSL-provided source-map entries. No partial `LinkedProtocol` is returned after refusal. |
| Tests | TC-150 trace-binds all seven FR-044 criteria. Adverse cases first pass the checked-package reader, then exercise the linker. The temporal-selection test changes the selected authority and confirms the owner causal relation remains unchanged. |
| Safety and bounds | No unsafe code, unchecked narrowing, production panic path, async work, or mutable shared state was added. Pairwise channel comparison runs over the checked package's bounded node list. |
| Changes outside Rust | No Cargo manifest, lockfile, or workflow gate was changed. |

## Verification

The repository-defined serialized gate passed over one unchanged head:
formatting; locked all-target Clippy with warnings denied; the full locked test
suite; and rustdoc with warnings denied. The suite passed all unit, integration,
and documentation tests, including ten TC-150 cases.

## Boundary

This review covers the QSL-1740 changed Rust paths. It does not certify other
repository modules or replace the Linear review required for promotion.
