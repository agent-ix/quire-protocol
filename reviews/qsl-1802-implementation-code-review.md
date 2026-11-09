---
id: SR-2127008
title: "QSL-1802 implementation code-review review"
type: SpecReview
analysis: code-review
review_set: subset
scope: "agent-ix/quire-protocol; QSL-1802; PR #140; changed source/tests/lock and Plan-001-catalog-hold-adoption"
relationships:
  - { target: ix://agent-ix/quire-protocol/FR-043, type: references }
---
## Summary

Independent code-review review of Protocol PR #140 for QSL-1802. No introduced or in-scope defect found. Code-review includes the Rust language lane; plan analyses are direct analyses selected by the code lead.

## Assurance Context

AP-001 (spec/assurance/AP-001-protocol-implementation.md) was read as context. It is proposed, governs one exact private candidate, and has no governs edge to FR-043, FR-044 or FR-046. It is not an active applicable profile for this PR. No enforced profile-review acceptance is claimed. Architecture context is spec/spec.md and CLAUDE.md. The existing merged-spec readiness reviews remain the specification authority. No new formal requirement, vocabulary, schema or assurance control is introduced by the plan. The code-lead selected base, integrity and dependency lenses for the new plan files; no EARS lens is applicable to an unchanged requirement statement. These are direct analyses, not a claimed accepted enforced review workflow.

## Verdict

**PASS** — no introduced or in-scope defect. PR #140 is mergeable after the lead runs the final current four-command gate and commits these artifacts.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Coverage

Reviewed every changed Rust file, README paragraph, Cargo.lock delta and all plan files in Protocol PR #140. Application: quire-protocol 0.1.0. No new manifest pin, copied owner type/catalog/source, compatibility branch, CI change, unsafe operation, allocation policy, narrowing cast or blocking path is added. Existing moving-main declarations are unchanged. The coherent lock refresh updates the held catalog/reader plus consumers that must agree on owner types. Registry changes are transitive resolution from those owners; there is no new direct dependency.

FR-043-AC-9: src/checked_package.rs imports CatalogReadError, stores the exact moved owner value in Refusal::CatalogUnavailable, and matches all four owner dispatch outcomes exhaustively. The branch returns immediately before Selection::read/check, ReadIdentityRows::read or ReadEqualityRows::read; it cannot construct AdmittedHandoff or be relabeled Reader/Incomplete. Display delegates directly to the owner. Owner operation_catalog.rs keeps error fields private and exposes path()/message(); there is no public constructor or injected-catalog product seam. Inspection is the declared verification; no synthetic error or runtime corruption claim is made. The current owner also has the production carried_member constraint reader. Existing TC-149 real handoffs exercise that catalog and compare the admitted package to the direct owner's result.

The linker uses CheckedJson::as_str() and keeps the prior decimal-only/u64 parsing rule. Temporal assessment reads EvaluatedEvidence::result(), the documented borrow of the complete original EvidenceResult; the result record's fields and typed CallFailure path are unchanged. The retired empty native_valuations field is removed from the test request; actual Position.instance_valuations remain supplied and asserted.

The two rewritten mutation fixtures start from real emitted handoff bytes, call the owner's resolve_operand_slots/write_operand_rows public APIs, then require the real checked reader to admit the edited package before the linker assertion can run. The carrier test now passes the document-held resolved node identity and asserts the exact NotAProtocolClause refusal, compares it to protocol_view, and retains the unedited linking control. The zero-maximum test requires the exact ZeroProgressIteration variant, offending control identity and nonempty locus. Neither can pass merely because setup/reader admission failed. There is no duplicated operand encoder or weakened oracle. No mutation was made to the frozen branch for this review.

Independent reviewer command: cargo test --locked --test tc_149_admit_checked_handoff --test tc_150_link_from_a_typed_read_view --test tc_152_temporal_evidence, under the complete user thermal wrapper and this worktree's own target. It passed 9 + 15 + 6 = 30 tests, exit=0; /tmp/qsl1802-code-review/focused-tests.log. Coder fmt and confined Clippy logs were read, both exit=0. No aggregate gate ran during review. The lead must run current main's four ordered commands after artifact custody. The user thermal override supersedes local jobs=1/quire-heavy lock/literal target-codex-backends; literal NFR-004-AC-3/9 procedure compliance is not claimed.

## Machine scope

```json
{
  "clean": true,
  "scope": [
    {
      "id": "FR-044-AC-1",
      "path": "spec/functional/FR-044-link-from-a-typed-read-view-of-the-checked-package.md",
      "role": "examined",
      "excerpt": "Over **R**'s protocol, the linked representation lists the roles, channels, queues, controls, compensations, required temporal clause and activation rows equal to `protocol_view`'s node ids, and its edge set equals `causal_relation(package, protocol)`, back edges of every `repeat` included."
    },
    {
      "id": "FR-044-AC-2",
      "path": "spec/functional/FR-044-link-from-a-typed-read-view-of-the-checked-package.md",
      "role": "examined",
      "excerpt": "No function of the crate constructs an `EdgeKind` or `CausalEdge` value or derives an edge from a node: the edge set comes only from `qsl_package::protocol_view::causal_relation`."
    },
    {
      "id": "FR-044-AC-3",
      "path": "spec/functional/FR-044-link-from-a-typed-read-view-of-the-checked-package.md",
      "role": "examined",
      "excerpt": "Over checked packages whose `protocol_view` identifies respectively a zero-progress `repeat` with maximum zero, two role nodes with the same authority, and two channel nodes with mutually conflicting premise terms, linking returns a consumer-owned `LinkRefusal` naming the offending node ids and retaining their available `locus` entries, with no representation. The linker obtains ids from `protocol_view`, reads content through Contract IR's checked node/term accessors, uses QSL owner accessors for content they publish, parses no bytes and maintains no second reader. Required temporal clauses re"
    },
    {
      "id": "FR-044-AC-4",
      "path": "spec/functional/FR-044-link-from-a-typed-read-view-of-the-checked-package.md",
      "role": "examined",
      "excerpt": "An absent node id, the id of a `protocol_reference` carrier and the id of a non-protocol node each return a typed refusal naming the cause and no partial representation."
    },
    {
      "id": "FR-044-AC-5",
      "path": "spec/functional/FR-044-link-from-a-typed-read-view-of-the-checked-package.md",
      "role": "examined",
      "excerpt": "The signature of the link entry names `CheckedPackageV2` and `CheckedNodeId`, returns `LinkedProtocol` or `LinkRefusal`, and names no `wire::Package`, `Handle` or `wire::Declaration`; compile-fail uses of each retired type through the crate's public surface fail to resolve."
    },
    {
      "id": "FR-044-AC-6",
      "path": "spec/functional/FR-044-link-from-a-typed-read-view-of-the-checked-package.md",
      "role": "examined",
      "excerpt": "The linker reads no window-binding node from `CheckedPackageV2` and imports no window view from QSL. A window value is accepted only at the evaluation input owned by FR-516; absence of a window member in the checked package has no effect on linking."
    },
    {
      "id": "FR-044-AC-7",
      "path": "spec/functional/FR-044-link-from-a-typed-read-view-of-the-checked-package.md",
      "role": "examined",
      "excerpt": "The criteria FR-002-AC-1 to FR-002-AC-5 hold over recipe **R**'s checked package: concurrent instances stay distinct; every permitted interleaving preserves the owner-supplied causal edges and joins; each consumer-owned refusal refuses; reordering equal-time input adds no edge; and changing the selected temporal profile or clock binding changes the linked result or returns its typed refusal."
    },
    {
      "id": "FR-043-AC-1",
      "path": "spec/functional/FR-043-admit-the-checked-package-handoff.md",
      "role": "examined",
      "excerpt": "Over the three packages of **R** (QSL FR-517: event position, fixed sample, timestamped event), the handoff is admitted with every selection identity unchanged and the admitted package is the value `read_checked_package` returned."
    },
    {
      "id": "FR-043-AC-2",
      "path": "spec/functional/FR-043-admit-the-checked-package-handoff.md",
      "role": "examined",
      "excerpt": "Each `reader` case of the handoff's standard mutation corpus (`quire.checked-handoff.mutations/1`, QSL FR-518) returns the code, loci, cause and node locus its manifest row names from `read_checked_package`, carried unchanged, and no linked representation is exposed."
    },
    {
      "id": "FR-043-AC-3",
      "path": "spec/functional/FR-043-admit-the-checked-package-handoff.md",
      "role": "examined",
      "excerpt": "Each `selection` case is admitted by `read_checked_package` and refused by `Selection::check` with the `SelectionAxis` its manifest row names, carried unchanged, and a changed caller selection identity over unchanged offered bytes returns a selection refusal and no admitted package."
    },
    {
      "id": "FR-043-AC-4",
      "path": "spec/functional/FR-043-admit-the-checked-package-handoff.md",
      "role": "examined",
      "excerpt": "No source file of the crate parses checked-package bytes or declares a refusal code or format string for the checked package: the only call that reads them is `read_checked_package`, and the refusal vocabulary is imported."
    },
    {
      "id": "FR-043-AC-5",
      "path": "spec/functional/FR-043-admit-the-checked-package-handoff.md",
      "role": "examined",
      "excerpt": "Bytes of `quire.compiled-protocol/1` or `/2` offered at the checked-package entry point return a typed refusal; no version is detected, upgraded or repaired. Checked-package bytes offered at a retired entry point return a typed refusal for as long as a retired entry point exists, which is until FR-047's end conditions are met."
    },
    {
      "id": "FR-043-AC-6",
      "path": "spec/functional/FR-043-admit-the-checked-package-handoff.md",
      "role": "examined",
      "excerpt": "A bare JSON number outside the safe-integer interval, a non-canonical rational and a signed-64 overflow in a package field each return the reader's refusal unchanged, and the library holds no numeric decoder."
    },
    {
      "id": "FR-043-AC-7",
      "path": "spec/functional/FR-043-admit-the-checked-package-handoff.md",
      "role": "examined",
      "excerpt": "A read whose limit, for a limit with nonzero measured usage, is one below the package's measured usage returns `CheckedPackageDispatchResult::Incomplete(CheckedPackageIncomplete)` naming the exhausted limit, carried unchanged; it is neither an admitted package nor a refusal, no linked representation is exposed, and the same read at the exact limit is admitted."
    },
    {
      "id": "FR-043-AC-8",
      "path": "spec/functional/FR-043-admit-the-checked-package-handoff.md",
      "role": "examined",
      "excerpt": "An `identities.json` or `equality.json` whose format string is another, or whose row names a node the package does not hold, or whose identity-row join exhausts the caller's native-join budget, returns QSL's file refusal unchanged and no admitted handoff; the retired identities formats `/1` and `/2` are refused as another format."
    },
    {
      "id": "FR-043-AC-9",
      "path": "spec/functional/FR-043-admit-the-checked-package-handoff.md",
      "role": "examined",
      "excerpt": "The production match over `CheckedPackageDispatchResult` has a `CatalogUnavailable` arm that returns a distinct non-admitted result carrying the imported `CatalogReadError` unchanged, including its path and message. That branch returns before selection, identity-row and equality-row checks, produces no admitted handoff, and maps to neither a package refusal nor an incomplete read; no local catalog error or panic substitutes for the owner's value."
    },
    {
      "id": "FR-046-AC-1",
      "path": "spec/functional/FR-046-consume-temporal-evidence-through-the-evidence-types.md",
      "role": "examined",
      "excerpt": "Over **R**'s four temporal clauses, the caller supplies an `EvidenceRequest` built from observation evidence; the assessment passes it to QSL and returns the `EvidenceResult`'s activation state, truth, settlement and decision support in its result record, equal to the values QSL's evidence call reports."
    },
    {
      "id": "FR-046-AC-2",
      "path": "spec/functional/FR-046-consume-temporal-evidence-through-the-evidence-types.md",
      "role": "examined",
      "excerpt": "`EvidenceRequest` and `EvidenceResult` cross the QSL call as in-memory values; the consumer neither serializes them as request/result documents nor assigns them document contract identifiers."
    },
    {
      "id": "FR-046-AC-3",
      "path": "spec/functional/FR-046-consume-temporal-evidence-through-the-evidence-types.md",
      "role": "examined",
      "excerpt": "The temporal evidence boundary accepts QSL's current `EvidenceRequest` and checked clause identity and returns QSL's `EvidenceResult` or typed call refusal."
    },
    {
      "id": "FR-046-AC-4",
      "path": "spec/functional/FR-046-consume-temporal-evidence-through-the-evidence-types.md",
      "role": "examined",
      "excerpt": "For the six call-refusal causes reachable from a caller-built request over an admitted package in **R**\u2014`trigger-required`, `trigger-unknown`, `occurrence-unknown`, `activation-mapping-missing`, `clause-root-unresolved`, and `binding-mismatch`\u2014the library returns the typed refusal naming the unchanged cause, charges nothing, and leaves caller ledgers unchanged. Causes requiring a package or request state that R cannot admit are outside this criterion."
    },
    {
      "id": "FR-046-AC-5",
      "path": "spec/functional/FR-046-consume-temporal-evidence-through-the-evidence-types.md",
      "role": "examined",
      "excerpt": "The bytes reader refuses duplicate temporal-clause name rows as `AmbiguousName`; because such rows cannot occur in an admitted package, no admitted **R** package reaches a `PackageMismatch` construction refusal. This criterion therefore has no post-admission test input; reader refusal is covered by FR-043."
    },
    {
      "id": "FR-046-AC-6",
      "path": "spec/functional/FR-046-consume-temporal-evidence-through-the-evidence-types.md",
      "role": "examined",
      "excerpt": "The IT-002 scenarios defined against recipe **R** pass on the evidence route, and an unavailable or withdrawn anchor settles as a missing non-value and never as a Boolean. The caller supplies the `EvidenceRequest`; IT-002 asserts the request passed to QSL and does not assert an observation-to-request mapping owned by this library."
    },
    {
      "id": "FR-046-AC-7",
      "path": "spec/functional/FR-046-consume-temporal-evidence-through-the-evidence-types.md",
      "role": "examined",
      "excerpt": "On the consumed QSL revision, the library builds against the retained `temporal::` surface and passes the `Position.instance_valuations` values that the evaluation reads. The integration verifies this positively; removal of the retained surface is not a supported build configuration."
    }
  ],
  "findings": [],
  "bindings": []
}
```

## New findings (post-review gate)

The lead's subsequent full test gate exposed a fixture path omitted by the confined review run. The initial clean conclusion above is historical and is superseded until this finding is dispositioned. fmt and Clippy passed; cargo test returned 101 with three TC-151 setup panics. Evidence: /tmp/qsl1802-final-gate-test.log. No implementation panic or incorrect product result is claimed: the concrete defect is that required regression assertions cannot execute against the refreshed owner graph.

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-002 | high | The refreshed owner uses operand-slot node paths, but TC-151 compensation_key passes package.graph() node ids to WireGraph::new, which requires resolved document indices. Three slot tests panic with OperandSlot { slot: 1 } before their outcome, unavailable-binder and binder-charge assertions; the required local test gate fails. | tests/tc_151_slot_outcomes.rs:61-77 |

Required correction: obtain the compensation identity from the owner's resolved document graph before WireGraph key conversion, retaining the existing behavioral assertions. Run all seven TC-151 cases and the affected neighboring tests; do not weaken the outcome or accounting oracles. No test-only product API, copied encoder or source shim is justified.

## Follow-up machine scope

```json
{
  "clean": false,
  "scope": [
    {
      "id": "FR-045-AC-1",
      "path": "spec/functional/FR-045-assess-compensation-through-the-checked-package-evaluation.md",
      "role": "examined",
      "excerpt": "For every outcome-map row that an admitted compensation in **R** can reach today, the test produces the successor outcome: true and false `trigger.guard` and `retry.relation`, an unavailable binder, a wrongly typed value and a meter one below measured usage. R has no admitted input that produces the unsupported-construct row; that row is excluded from this criterion until the checked-package owner admits such an input. Recovery-predicate rows remain excluded while the QSL bare projection refuses R's comprehension binder."
    }
  ],
  "findings": [
    {
      "fnd": "FND-002",
      "severity": "high",
      "confidence": "high",
      "method": "code-review",
      "check": "test-intent",
      "artifact_id": "FR-045-AC-1",
      "related_ids": [
        "FR-045-AC-3",
        "FR-045-AC-8",
        "TC-151"
      ],
      "path": "tests/tc_151_slot_outcomes.rs",
      "lines": "61-77",
      "excerpt": "For every outcome-map row that an admitted compensation in **R** can reach today, the test produces the successor outcome: true and false `trigger.guard` and `retry.relation`, an unavailable binder, a wrongly typed value and a meter one below measured usage. R has no admitted input that produces the unsupported-construct row; that row is excluded from this criterion until the checked-package owner admits such an input. Recovery-predicate rows remain excluded while the QSL bare projection refuses R's comprehension binder.",
      "finding": "The refreshed owner uses operand-slot node paths, but TC-151 compensation_key passes package.graph() node ids to WireGraph::new, which requires resolved document indices. Three slot tests panic with OperandSlot { slot: 1 } before their outcome, unavailable-binder and binder-charge assertions; the required local test gate fails."
    }
  ],
  "bindings": []
}
```
