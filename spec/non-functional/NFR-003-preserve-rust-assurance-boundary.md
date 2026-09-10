---
id: NFR-003
title: "Preserve the Rust assurance boundary"
type: NFR
quality_attribute: maintainability
relationships:
  - target: ix://agent-ix/quire-protocol/FR-001
    type: constrains
  - target: ix://agent-ix/quire-protocol/FR-002
    type: constrains
  - target: ix://agent-ix/quire-protocol/FR-003
    type: constrains
  - target: ix://agent-ix/quire-protocol/FR-004
    type: constrains
  - target: ix://agent-ix/quire-protocol/FR-005
    type: constrains
  - target: ix://agent-ix/quire-protocol/FR-006
    type: constrains
  - target: ix://agent-ix/quire-protocol/FR-007
    type: constrains
  - target: ix://agent-ix/quire-protocol/FR-008
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
the accepted Rust-native language-boundary checker delivered through
`agent-ix/quire-research#64`, the exact Rust review, and semantic gap analysis run
over one unchanged head. The repository SHALL consume the shared checker's
versioned structured result and SHALL NOT implement a local replacement.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| NFR-003-AC-1 | Every hosted workflow has only `workflow_dispatch` and no hosted run is used as local evidence. | Test (TC-010) |
| NFR-003-AC-2 | An unsafe Rust block is rejected unless a later reviewed requirement and owner disposition explicitly authorize that exact block. | Test (TC-010) |
| NFR-003-AC-3 | Every first-party production, qualification, generator, validator, canonicalizer, adapter, fixture-audit, test-assertion and command path is Rust, and the accepted shared checker rejects an unapproved tracked source, shebang, manifest, inline workflow command, generated executable template, runtime dependency, extensionless executable, rename, or Rust-wrapper bypass. | Test (TC-010) |
| NFR-003-AC-4 | The dependency and source inventory contains no local producer runner, stdout verdict parser, evidence store, method catalogue, sufficiency or human-disposition implementation; accepted versioned Engineering Assurance and Quoin interfaces remain the only corresponding boundaries. | Test (TC-010, IT-003) |
| NFR-003-AC-5 | Every automated acceptance test carries a bare compiler-checked `ix_trace_rs::trace` binding to each obligation it claims, and removing or misspelling one binding fails compilation or coverage. | Test (TC-010) |
| NFR-003-AC-6 | Existing Quoin execution remains external behind its accepted structured interface, no Quoin implementation is copied or added as a runtime dependency, and no new JavaScript or TypeScript executable path is introduced. | Test (TC-010, IT-003) |

## Dependencies

- Engineering Assurance issue 34 owns reusable producer execution and typed transport.
- Quoin owns retained evidence and assurance reports.
- `agent-ix/quire-research#62` owns the Quoin containment boundary.
- `agent-ix/quire-research#64` owns the reusable Rust-native language-boundary checker.
