---
id: NFR-003
title: "Preserve the Rust assurance boundary"
type: NFR
quality_attribute: maintainability
relationships:
  - target: ix://agent-ix/quire-protocol/FR-001
    type: constrains
  - target: ix://agent-ix/quire-protocol/FR-007
    type: constrains
---
# NFR-003: Preserve the Rust assurance boundary

## Statement

The repository SHALL implement every first-party production and qualification
path in Rust while delegating producer execution to Engineering Assurance and
retained evidence to Quoin through accepted typed interfaces.

## Scope

Applies to library code, generators, validators, canonicalization, adapters,
fixture audits, test assertions, and any future command-line surface.

## Rationale

The protocol path may enter production qualification. Reusing shared typed
infrastructure prevents language drift, transcript scraping, duplicate evidence
models, and consumer-specific assurance behavior.

## Measurement and Evaluation

| Metric | Target | Threshold | Method |
| --- | --- | --- | --- |
| First-party executable languages in scoped paths | Rust only | 100% Rust | Test (TC-010) |
| Local process runners, stdout verdict parsers, evidence stores, method catalogues, or human disposition paths | None | 0 implementations | Test (TC-010) |
| Rust acceptance tests with bare compiler-checked trace bindings | Every automated acceptance test | 100% bound | Test (TC-010) |

## Verification

Static source inventory, Cargo metadata, Quire coverage, architecture inspection,
the exact Rust review, and semantic gap analysis run over one unchanged head.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| NFR-003-AC-1 | Every hosted workflow has only `workflow_dispatch` and no hosted run is used as local evidence. | Test (TC-010) |
| NFR-003-AC-2 | Unsafe Rust is forbidden unless a later reviewed requirement and owner disposition explicitly authorize an exact block. | Analysis (TC-010) |

## Dependencies

- Engineering Assurance issue 34 owns reusable producer execution and typed transport.
- Quoin owns retained evidence and assurance reports.
