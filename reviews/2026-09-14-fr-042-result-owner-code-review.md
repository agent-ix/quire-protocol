---
id: SR-073
title: "Code and Rust review of the FR-042 canonical result owner"
type: SpecReview
analysis: code-review
scope: "quire-protocol FR-042 implementation at 920a909 over e6dfbd5"
review_set: subset
relationships:
  - { target: ix://agent-ix/quire-protocol/AP-001, type: reviews }
  - { target: ix://agent-ix/quire-protocol/FR-042, type: reviews }
  - { target: ix://agent-ix/quire-protocol/TC-015, type: references }
  - { target: ix://agent-ix/tl-syntax/Task-010, type: references }
---
# Code and Rust review of the FR-042 canonical result owner

## Summary

Reviewed the complete FR-042 result producer, strict reader, immutable schemas,
constructor-private typed views, direct correction lineage, selected Contract-IR
mapping, traced tests, and test-status records at
`920a9095c3b2e876c1bfc2451fd4ecadb973fd91` over baseline
`e6dfbd53b0a1f1ecacbaebf5053a1e1bf776b8d9`. The review found and fixed partial
typed access, hand-assembled public wire objects, duplicated mapped source state,
one omitted pair of typed mapping accessors, incomplete result-combination
coverage and enforcement, missing settlement-support cross-checks, and a stale
TC-015 status statement. No unresolved code or Rust finding remains.

## Verdict

**PASS** — every review-cycle finding is resolved, the exact candidate passes
the repository's complete local gate with warnings denied, and no qualification
work or external evidence framework was introduced.

## Assurance Context

- **Profile:** AP-001, `spec/assurance/AP-001-protocol-implementation.md`,
  profile version 0.2, status `proposed`. The profile applies to this exact Rust
  candidate and requires code review and gap analysis.
- **Baseline and candidate:** baseline
  `e6dfbd53b0a1f1ecacbaebf5053a1e1bf776b8d9`; evaluated candidate
  `920a9095c3b2e876c1bfc2451fd4ecadb973fd91` on
  `agent-b/temporal-result-owner`. Changed paths are `src/result/{record,
  contract_ir,lineage,mod}.rs`, `tests/tc_015_result_owner.rs`,
  `spec/tests.md`, and `spec/test-cases/TC-015-result-reader-and-contract-ir-map.md`.
- **Impact scenarios:** `impact-result-loss`, `impact-false-conformance`, and
  `impact-identity-substitution`. The review checked that all non-Boolean states
  survive mapping, no incomplete/open/refused result becomes Boolean success,
  all owner identities remain domain-separated, and correction retains its
  exact direct predecessor.
- **Architecture and owner context:** AD-001's pure typed library boundary,
  FR-042, TC-015, the accepted ecosystem `IF-003` result-authority allocation,
  and PLAN-010 Task-010. The mapping borrows an independently validated source;
  it adds no Contract-IR vocabulary owner, parser, evaluator, callback, trust
  flag, or independently supplied source fields.
- **Unavailable or stale selected context:** AP-001 and AD-001 remain proposed.
  No MeasurementPlan, evidence-independence record, producer-reliance decision,
  or external qualification evidence was selected or required for this
  implementation review. The 23 ignored version-1 producer-fixture tests are
  pre-existing and explicitly named; the implemented FR-042 lane has no ignored
  test.
- **Active exceptions:** none.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No unresolved findings | - |

## Review-cycle remediations

| Finding | Resolution |
| --- | --- |
| The validated and mapped views exposed only a subset of the canonical result, forcing consumers back to owner JSON. | Added constructor-private `AxisView`, `CompletenessView`, and `CorrectionView` plus typed accessors for every canonical result and mapping field. |
| Result and mapping objects were assembled with repeated JSON field-name literals. | Replaced both public emitters with private compiler-checked `Serialize` wire types; the one remaining `json!` object is the internal FR-286 identity preimage, not a published wire schema. |
| The mapped view cloned trusted owner facts into a second state representation. | Made `MappedResultView` borrow its independently validated source and derive all typed fields from that source, eliminating divergence and independent construction. |
| Native subject and correspondence identities existed in mapping bytes but lacked typed mapping accessors. | Added both accessors and asserted equality with the validated source in the traced no-parsing test. |
| TC-015 sampled the result-combination table, and the validator admitted closed-scope truth on an open/incomplete scope, decisive settlement on a closed scope, and settlement references outside the selected support/progress population. | Added an independent 2,400-row Cartesian oracle: 126 valid rows produce, strict-read and map; 2,274 forbidden rows refuse. Closed-scope now requires complete closed decision progress/closure, decisive settlement requires an open decision closure, settlement facts exactly equal decision support, and progress references name only selected progress assertions. Focused mutation controls cover every new invariant. |
| TC-015's own artifact still said planned after its executable suite landed. | Aligned the artifact with the twelve-test implementation and revalidated all 164 spec documents. |

## Rust review

| Check | Result |
| --- | --- |
| Owner types and constructors | Public views retain owner enums and records directly. `ValidatedResult` and `MappedResultView` have no public constructors; no mirror enum, text parser, or Boolean coercion was added. |
| Exhaustiveness | Mapping explicitly enumerates `AssessmentExecution × Truth`; only completed+satisfied and completed+violated yield values. No wildcard arm can absorb a future state. |
| Wire discipline | Private `Serialize` structs define the complete result and mapping field inventories. Strict readers still require canonical byte equality and closed JSON schemas. |
| Identity and lineage | Source result identity/digest, mapping identity, native subject/correspondence identities, assertion identities, and the direct correction predecessor remain separately typed and separately compared. |
| Bounds and allocation | Existing bounded entry points and resource inventory are preserved. Structural input charging precedes JSON parsing; semantic and output charges retain exact/one-short tests; caller ceilings can only lower owner maxima. |
| Panic and unsafe surface | Changed production paths contain no `unsafe`, `unwrap`, `expect`, `panic!`, `todo!`, `unimplemented!`, or new warning suppression. The crate-level unsafe prohibition remains unchanged. |
| Async, locking, numeric boundaries | No async/blocking operation, lock, persistence boundary, integer wire conversion, I/O, or side effect was introduced. |
| Test quality | TC-015 exercises real public producer/read/map/read paths, schema pins, canonical mutations, independent axes, all 2,400 execution/truth/settlement/activation/progress/closure/support combinations, every Boolean/non-value class, direct correction, reproduction, and exact/one-short limits. The added typed-view test proves complete consumption without JSON parsing. |
| Trace honesty | Direct source inventory contains FR-042 AC tags in counts 3, 1, 2, 1, 3, and 2 for AC-1 through AC-6. Quire currently rolls these up as 3/6 because its installed status/criterion extractor is inconsistent; the matrix records direct inventory and engine rollup separately. |
| Gate integrity | No `.github` workflow, dependency, feature, lint, coverage threshold, or warning setting changed. |

## Gates

Run locally at the immutable candidate with one Cargo job, the worktree-local
target, and the shared heavy-check lock:

- `cargo fmt --all -- --check` — clean.
- `cargo clippy --locked --all-targets -- -D warnings` — clean, zero warnings.
- `cargo test --locked` — 209 regular tests and 37 doctests passed, 0 failed;
  23 pre-existing producer-fixture tests were explicitly ignored.
- `quire validate --scope . 'spec/**/*.md' --summary` — 164/164 documents
  grammar-clean.
- `quire coverage --scope .` — 97/329 engine-backed rows; 226/226 Rust symbols
  bound and tagged. This is repository-wide implementation status, not an
  FR-042-only completion percentage.

Toolchain: `rustc 1.98.1`, `cargo 1.98.1`. No hosted workflow was dispatched.

## Boundary

This review establishes QProtocol's FR-042 portion of PLAN-010 Task-010. It does
not claim the whole multi-repository owner task complete: remaining shared
owner decisions and any still-unpromoted owner revisions must close before the
Task-010 checkbox and downstream Contract-IR pins advance.
