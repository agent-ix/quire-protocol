---
id: SR-2127006
title: "QSL-1802 catalog-unavailable adoption: ears-conformance"
type: SpecReview
analysis: ears-conformance
review_set: subset
scope: "agent-ix/quire-protocol; QSL-1802; spec/functional/FR-043-admit-the-checked-package-handoff.md; spec/test-cases/TC-149-admit-the-checked-package-handoff.md"
relationships:
  - { target: ix://agent-ix/quire-protocol/FR-043, type: reviews }
  - { target: ix://agent-ix/quire-protocol/TC-149, type: reviews }
---
## Summary

Reviewed quire-protocol PR #139 frozen head 16dbbda84d7ad5627bfcdb366875c77966fc37ee, diff origin/main...HEAD only, for QSL-1802 using ears-conformance. No findings; owner-preserving catalog failure adoption is ready for a separately accepted implementation plan.

## Assurance Context

AP-001 at spec/assurance/AP-001-protocol-implementation.md was examined as context only: status proposed, decision boundary one exact private implementation candidate, explicit governs edges omit FR-043. No active applicable AssuranceProfile requires an enforced review run for this source-only FR-043 adoption. No exception used. Architecture context is spec/spec.md and CLAUDE.md; no implementation qualification or independence measurement is claimed.

## Evidence and readiness

The two changed documents validate structurally and grammar-clean; git diff --check passes. Full spec validation reports 11 structural failures outside the changed documents, including AP-001 profile_version typing and TM-001 Coverage Status header; none of those files changes in this PR. This review does not certify the repository-wide schema baseline.

Contract IR owns CheckedPackageDispatchResult and CatalogReadError (FR-038-AC-58). Owner dispatch.rs forwards the error; operation_catalog.rs exposes path()/message() but keeps fields pub(super), provides no public constructor or injected-catalog failure API, and reads production catalog bytes internally. Owner-only cfg(test) injection is not a consumer API. Protocol owns only the adapter result carrying that owner value. This is an adoption obligation, not a second error taxonomy or a catalog definition.

AC-9 is one observable dispatch outcome with inseparable preservation/short-circuit invariants. Its Inspection method can conclusively inspect the production arm, imported type, direct move and early return. Manufacturing an opaque error or adding a test-only product API would weaken the boundary. Existing the_three_packages_of_recipe_r_are_admitted_unchanged in tests/tc_149_admit_checked_handoff.rs uses genuine QSL handoffs, asserts the admitted package equals the direct owner's read, and is correctly tagged FR-043-AC-1/TC-149. Run that test against the updated dependency for compatibility; it provides no unavailable-branch runtime credit. No Cargo checks run in this specification review.

Readiness for a separate plan: owner known; upstream IR-1396 Completed per structured dispatch brief and actual public owner API present; dependency update before adapter compilation, then existing real handoff compatibility test and independent code review. Medium implementation risks are dependency skew and accidental classification/selection continuation; unchanged typed forwarding and early return control both. Source-only spec risk is low. Failure domains are explicit: owner catalog failure, invalid package refusal and incomplete read remain distinct; no admitted output or later sidecar work follows catalog failure. No new product API, mirrored catalog, hash/pin tracking, local error vocabulary or additional assurance machinery is required. Human plan acceptance remains a separate gate; this review does not grant it.

## Scope

Every changed requirement unit was examined: FR-043 Outputs (catalog-unavailable result and ownership paragraph), the five added Behavior bullets, FR-043-AC-9, the FR-038 depends_on/dependency prose, TC-149's AC-9 procedure, and its corrected selection-order and catalog-unavailable expected results. The unchanged AC-1 test binding and owner FR-038-AC-58/API were context checks. Existing FR-043 AC-1 through AC-8 were read as context, not requalified.

## Verdict

**PASS** — no defects in this narrowly scoped specification change. Mergeable as a specification change; implementation and human plan acceptance remain future work.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Grammar assessment

Five added Behavior statements have explicit library subject, one shall per statement, and concrete responses. Three event triggers describe the reader's returned unavailable outcome; two ubiquitous prohibitions constrain classification and local vocabulary. The unavailable trigger is a completed read outcome, not an ongoing mode requiring While. Engine validation reports 110/110 documents grammar-clean, zero grammar findings; semantic inspection finds no ambiguity in the changed statements.

## Machine scope

```json
{
  "clean": true,
  "scope": [
    {
      "id": "FR-043/Outputs",
      "path": "spec/functional/FR-043-admit-the-checked-package-handoff.md",
      "role": "examined",
      "excerpt": "Catalog unavailability means the reader could not read its own operation\ncatalog; it is not a judgment that the offered package is invalid. Contract IR\nowns `CatalogReadError`, including its unreadable path and message\n(Contract IR FR-038-AC-58)."
    },
    {
      "id": "FR-043/Behavior/catalog-forward",
      "path": "spec/functional/FR-043-admit-the-checked-package-handoff.md",
      "role": "examined",
      "excerpt": "When the reader returns `CatalogUnavailable(error)`, the library shall return\n  a distinct non-admitted result carrying that same `CatalogReadError` unchanged."
    },
    {
      "id": "FR-043/Behavior/no-admit",
      "path": "spec/functional/FR-043-admit-the-checked-package-handoff.md",
      "role": "examined",
      "excerpt": "When the operation catalog is unavailable, the library shall expose no admitted handoff."
    },
    {
      "id": "FR-043/Behavior/no-sidecars",
      "path": "spec/functional/FR-043-admit-the-checked-package-handoff.md",
      "role": "examined",
      "excerpt": "When the operation catalog is unavailable, the library shall perform no\n  selection, identity-row or equality-row check."
    },
    {
      "id": "FR-043/Behavior/no-reclassification",
      "path": "spec/functional/FR-043-admit-the-checked-package-handoff.md",
      "role": "examined",
      "excerpt": "The library shall not reclassify catalog unavailability as a package refusal\n  or an incomplete package read."
    },
    {
      "id": "FR-043/Behavior/no-vocabulary",
      "path": "spec/functional/FR-043-admit-the-checked-package-handoff.md",
      "role": "examined",
      "excerpt": "The library shall declare no local catalog-error vocabulary."
    },
    {
      "id": "FR-043-AC-9",
      "path": "spec/functional/FR-043-admit-the-checked-package-handoff.md",
      "role": "examined",
      "excerpt": "The production match over `CheckedPackageDispatchResult` has a `CatalogUnavailable` arm that returns a distinct non-admitted result carrying the imported `CatalogReadError` unchanged, including its path and message. That branch returns before selection, identity-row and equality-row checks, produces no admitted handoff, and maps to neither a package refusal nor an incomplete read; no local catalog error or panic substitutes for the owner's value."
    },
    {
      "id": "FR-043/Dependencies",
      "path": "spec/functional/FR-043-admit-the-checked-package-handoff.md",
      "role": "examined",
      "excerpt": "[Contract IR FR-038](ix://agent-ix/quire-contract-ir/FR-038), especially\n  AC-58, owns the operation catalog's typed unreadability result and error."
    },
    {
      "id": "TC-149/Procedure/AC-9",
      "path": "spec/test-cases/TC-149-admit-the-checked-package-handoff.md",
      "role": "examined",
      "excerpt": "For FR-043-AC-9, inspect the production dispatch match: its\n`CatalogUnavailable` arm carries Contract IR's imported `CatalogReadError`\nunchanged and returns before any selection or sidecar check. Check that neither\na package refusal, an incomplete read nor a panic replaces the owner's result.\nThis is inspection of the adapter boundary, not runtime credit for corrupting\nthe production catalog; no consumer test manufactures the opaque owner error."
    },
    {
      "id": "TC-149/Expected/selection",
      "path": "spec/test-cases/TC-149-admit-the-checked-package-handoff.md",
      "role": "examined",
      "excerpt": "selection checked only over a reader-admitted package"
    },
    {
      "id": "TC-149/Expected/catalog",
      "path": "spec/test-cases/TC-149-admit-the-checked-package-handoff.md",
      "role": "examined",
      "excerpt": "The inspected catalog-unavailable branch returns the owner's read error unchanged, exposes no admitted handoff and performs no later selection or sidecar check."
    },
    {
      "id": "FR-043-AC-1",
      "path": "spec/functional/FR-043-admit-the-checked-package-handoff.md",
      "role": "context_only",
      "excerpt": "Over the three packages of **R** (QSL FR-517: event position, fixed sample, timestamped event), the handoff is admitted with every selection identity unchanged and the admitted package is the value `read_checked_package` returned."
    }
  ],
  "findings": [],
  "bindings": [
    {
      "test_id": "the_three_packages_of_recipe_r_are_admitted_unchanged",
      "ac_id": "FR-043-AC-1",
      "trace": "correct"
    }
  ]
}

```
