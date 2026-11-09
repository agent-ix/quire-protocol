---
id: SR-2127009
title: "QSL-1802 implementation gap-analysis review"
type: SpecReview
analysis: gap-analysis
review_set: subset
scope: "agent-ix/quire-protocol; QSL-1802; PR #140; whole repository matrix/reverse inventory with PR-delta disposition"
relationships:
  - { target: ix://agent-ix/quire-protocol/FR-043, type: references }
---
## Summary

Independent gap-analysis review of Protocol PR #140 for QSL-1802. No introduced or in-scope defect found. Code-review includes the Rust language lane; plan analyses are direct analyses selected by the code lead.

## Assurance Context

AP-001 (spec/assurance/AP-001-protocol-implementation.md) was read as context. It is proposed, governs one exact private candidate, and has no governs edge to FR-043, FR-044 or FR-046. It is not an active applicable profile for this PR. No enforced profile-review acceptance is claimed. Architecture context is spec/spec.md and CLAUDE.md. The existing merged-spec readiness reviews remain the specification authority. No new formal requirement, vocabulary, schema or assurance control is introduced by the plan. The code-lead selected base, integrity and dependency lenses for the new plan files; no EARS lens is applicable to an unchanged requirement statement. These are direct analyses, not a claimed accepted enforced review workflow.

## Verdict

**FAIL — repository baseline assurance**: 200 pre-existing untagged criteria prevent a whole-repository PASS. **PASS — PR delta**: statuses are unchanged and there is no introduced/in-scope gap; this baseline is not a new QSL-1802 merge blocker. Final local gate remains pending.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Coverage

Plan completion: not assessed

The full repository matrix was computed, not authored: quoin matrix --repo <worktree> --json succeeded and covers 57 requirements / 388 criteria: 134 tagged, 54 method-without-symbol, 200 untagged. Every static (criterion ID, status) pair is byte-identical to the merged-spec baseline worktree. FR-043-AC-9 is correctly method-without-symbol (Inspection), not a missing runtime tag. None of the PR's relevant Test criteria loses a binding. No passing Quoin run evidence is claimed: the computed evidence axis reports no run evidence; the actual confined Cargo log is separate direct execution evidence.

The full quire coverage report also succeeded: 11 untracked symbols, 42 unmatched tags and five diagnostics. The normalized untracked-symbol identity list is byte-identical to baseline; all counts match. Diagnostics include the pre-existing spec/tests.md Status versus Coverage Status configuration disagreement and inspection archetype mismatch. The 200 untagged criteria include retired aggregate requirements, e.g. FR-001-AC-1 through AC-5; adding synthetic labels or reconstructing retired modules is not part of this PR. These are existing repository assurance debt, not defects introduced by QSL-1802. The precise criterion inventory and current/baseline comparisons are retained in /tmp/qsl1802-code-review/{matrix,baseline-matrix,current-status,baseline-status,coverage,baseline-coverage,current-untracked,baseline-untracked}.json for the lead to route as one owned coverage-baseline task if required; no per-criterion tickets are requested here.

Reverse inventory examined all 26 tracked Rust source files at module/public-surface level. Existing ownership is recorded by FR-009 version lock, FR-010/FR-011 identities, FR-014/FR-040 outcomes, FR-015 digests, FR-016/NFR-002 resource bounds, FR-018 delivery index, FR-008/FR-036..FR-042 refusal components, FR-043 admission, FR-044 linking, FR-045 compensation/slots, FR-046 evidence and NFR-001 reproduction. Closure axes reference their formal owner. No new unowned behavior, source stub, test stub or coverage inflation was found in the PR. Whole-repository semantic requalification is not implied by module-level inventory. The opted-in semantic review follows the changed FR-043/044/046 paths in the code-review artifact; unchanged behavior is not re-proved.

FR-043-AC-9 uses direct source Inspection; the live-catalog compatibility path remains genuine producer emission plus real reader admission. Relevant computed traces remain tagged, and all 30 affected integration tests pass independently. No Test Matrix, plan or source was edited during audit. This audit is not plan-assisted: the plan files are separately reviewed for definition quality, without checking task completion.

## Machine scope

```json
{
  "clean": true,
  "scope": [
    {
      "id": "FR-001-AC-1",
      "path": "spec/functional/FR-001-admit-selected-protocol-profile.md",
      "role": "examined",
      "excerpt": "Exact bytes under one accepted compiled-artifact contract, accepted frozen baseline, producer revision, and complete dependency closure are admitted with every identity unchanged."
    },
    {
      "id": "FR-001-AC-2",
      "path": "spec/functional/FR-001-admit-selected-protocol-profile.md",
      "role": "examined",
      "excerpt": "Mutating each artifact-contract, producer, accepted source-baseline, version, revision, dependency, profile, or required feature independently returns a typed refusal before semant"
    },
    {
      "id": "FR-001-AC-3",
      "path": "spec/functional/FR-001-admit-selected-protocol-profile.md",
      "role": "examined",
      "excerpt": "A pre-acceptance working draft, superseded or unknown baseline cannot be selected through display-name equality or a locally authored substitute."
    },
    {
      "id": "FR-001-AC-4",
      "path": "spec/functional/FR-001-admit-selected-protocol-profile.md",
      "role": "examined",
      "excerpt": "Every declared numeric minimum and maximum, the signed-64 extrema in fields whose source domains admit them, and every normalized exact-rational boundary round-trip without loss un"
    },
    {
      "id": "FR-001-AC-5",
      "path": "spec/functional/FR-001-admit-selected-protocol-profile.md",
      "role": "examined",
      "excerpt": "Each published compiled-artifact contract version is admitted only through its own explicit entry point: bytes of one version offered at another version's entry point return a type"
    },
    {
      "id": "FR-002-AC-1",
      "path": "spec/functional/FR-002-link-finite-protocol-representation.md",
      "role": "examined",
      "excerpt": "Concurrent workflow instances, static roles, role instances, participants, components, endpoints, channels, messages, sends, receives, duplicate deliveries, attempts, and business "
    },
    {
      "id": "FR-002-AC-2",
      "path": "spec/functional/FR-002-link-finite-protocol-representation.md",
      "role": "examined",
      "excerpt": "Every permitted interleaving of bounded parallel branches preserves authored causal edges and satisfies its declared join."
    },
    {
      "id": "FR-002-AC-3",
      "path": "spec/functional/FR-002-link-finite-protocol-representation.md",
      "role": "examined",
      "excerpt": "Each consumer-owned duplicate authority, missing/surplus/duplicate/wrong-kind/foreign/incompatible/slot-derived runtime binding, invalid internality, zero-progress iteration, FR-28"
    },
    {
      "id": "FR-002-AC-4",
      "path": "spec/functional/FR-002-link-finite-protocol-representation.md",
      "role": "examined",
      "excerpt": "Reordering equal-time input, replaying one exact relationship, changing a concrete runtime value or changing external presentation metadata cannot add a causal edge or independentl"
    },
    {
      "id": "FR-002-AC-5",
      "path": "spec/functional/FR-002-link-finite-protocol-representation.md",
      "role": "examined",
      "excerpt": "A changed temporal-profile, clock-alternative or clock-parameter selection, and a temporal binding set that is missing, surplus, duplicated, reordered, foreign-owned or out of the "
    },
    {
      "id": "FR-003-AC-1",
      "path": "spec/functional/FR-003-assess-compensation-state.md",
      "role": "examined",
      "excerpt": "Activation over the whole assessment is exactly what [FR-019](./FR-019-derive-obligation-activation-state.md) reports: triggered, inactive and activation-unknown for established, a"
    },
    {
      "id": "FR-003-AC-2",
      "path": "spec/functional/FR-003-assess-compensation-state.md",
      "role": "examined",
      "excerpt": "Registration over the whole assessment is exactly what [FR-020](./FR-020-register-compensation-once-per-effect.md) creates: one registration per paired successful effect, none from"
    },
    {
      "id": "FR-003-AC-3",
      "path": "spec/functional/FR-003-assess-compensation-state.md",
      "role": "examined",
      "excerpt": "The ordering violations of [FR-021](./FR-021-distinguish-commit-ordering-violations.md) and the retry-bound refusal of [FR-022](./FR-022-enforce-finite-compensation-retry-bound.md)"
    },
    {
      "id": "FR-003-AC-4",
      "path": "spec/functional/FR-003-assess-compensation-state.md",
      "role": "examined",
      "excerpt": "The attempt, effect and outcome facts of [FR-023](./FR-023-preserve-attempt-and-effect-distinctness.md) reach the aggregate result with their exact attempt, effect and registration"
    },
    {
      "id": "FR-003-AC-5",
      "path": "spec/functional/FR-003-assess-compensation-state.md",
      "role": "examined",
      "excerpt": "Recovery in the aggregate result is exactly what [FR-024](./FR-024-decide-recovery-only-by-authored-relation.md) decides, so a partial success alongside a full-restoration failure "
    },
    {
      "id": "FR-003-AC-6",
      "path": "spec/functional/FR-003-assess-compensation-state.md",
      "role": "examined",
      "excerpt": "A missing trigger, state, effect, commit, population, membership or completeness authority anywhere in the assessment leaves activation, registration and recovery without success: "
    },
    {
      "id": "FR-003-AC-7",
      "path": "spec/functional/FR-003-assess-compensation-state.md",
      "role": "examined",
      "excerpt": "The purity boundary of [FR-025](./FR-025-assess-compensation-without-io.md) holds over the whole assessment: the implementation performs no network, storage, telemetry, business-op"
    },
    {
      "id": "FR-004-AC-1",
      "path": "spec/functional/FR-004-assess-finite-global-conformance.md",
      "role": "examined",
      "excerpt": "Order invariance over the whole assessment is exactly what [FR-026](./FR-026-admit-every-permitted-observation-order.md) establishes: every permitted split-shipment and compensatio"
    },
    {
      "id": "FR-004-AC-2",
      "path": "spec/functional/FR-004-assess-finite-global-conformance.md",
      "role": "examined",
      "excerpt": "Adverse reporting over the whole assessment is exactly what [FR-027](./FR-027-report-adverse-observations-without-repair.md) produces: unexpected, missing, causally misordered, dup"
    },
    {
      "id": "FR-004-AC-3",
      "path": "spec/functional/FR-004-assess-finite-global-conformance.md",
      "role": "examined",
      "excerpt": "Axis independence over the whole assessment is exactly what [FR-028](./FR-028-keep-progress-and-closure-axes-independent.md) enforces: decision-scope progress/closure, surrounding-"
    },
    {
      "id": "FR-004-AC-4",
      "path": "spec/functional/FR-004-assess-finite-global-conformance.md",
      "role": "examined",
      "excerpt": "Path equivalence over the whole assessment is exactly what [FR-041](./FR-041-agree-across-batch-and-incremental-paths.md) establishes: batch replay and settled incremental assessme"
    },
    {
      "id": "FR-005-AC-1",
      "path": "spec/functional/FR-005-dispatch-independent-protocol-claims.md",
      "role": "examined",
      "excerpt": "Claim independence over the whole dispatch is exactly what [FR-029](./FR-029-return-one-disposition-per-requested-claim.md) establishes: a monitorable but unprojectable protocol re"
    },
    {
      "id": "FR-005-AC-2",
      "path": "spec/functional/FR-005-dispatch-independent-protocol-claims.md",
      "role": "examined",
      "excerpt": "Non-inference over the whole dispatch is exactly what [FR-029](./FR-029-return-one-disposition-per-requested-claim.md)-AC-2 enforces: satisfiable-but-unrealizable and circular-assu"
    },
    {
      "id": "FR-005-AC-3",
      "path": "spec/functional/FR-005-dispatch-independent-protocol-claims.md",
      "role": "examined",
      "excerpt": "Unsupported handling over the whole dispatch is exactly what [FR-029](./FR-029-return-one-disposition-per-requested-claim.md)-AC-3 enforces: unknown or unimplemented advanced claim"
    },
    {
      "id": "FR-005-AC-4",
      "path": "spec/functional/FR-005-dispatch-independent-protocol-claims.md",
      "role": "examined",
      "excerpt": "Backend failure over the whole dispatch is what [FR-029](./FR-029-return-one-disposition-per-requested-claim.md)-AC-4 and [FR-030](./FR-030-bind-each-disposition-to-its-exact-premi"
    },
    {
      "id": "FR-005-AC-5",
      "path": "spec/functional/FR-005-dispatch-independent-protocol-claims.md",
      "role": "examined",
      "excerpt": "Premise binding over the whole dispatch is exactly what [FR-030](./FR-030-bind-each-disposition-to-its-exact-premises.md) enforces: every claim disposition retains its exact admitt"
    },
    {
      "id": "FR-006-AC-1",
      "path": "spec/functional/FR-006-produce-canonical-protocol-results.md",
      "role": "examined",
      "excerpt": "Every valid FR-061 result-combination row derived from the typed assessment stages is accepted and every one-axis forbidden combination or caller-built substitute stage value is re"
    },
    {
      "id": "FR-006-AC-2",
      "path": "spec/functional/FR-006-produce-canonical-protocol-results.md",
      "role": "examined",
      "excerpt": "Settled truth on an open subject requires exact complete decisive support whose truth is preserved by every admitted continuation, which is exactly [FR-032](./FR-032-retain-orthogo"
    },
    {
      "id": "FR-006-AC-3",
      "path": "spec/functional/FR-006-produce-canonical-protocol-results.md",
      "role": "examined",
      "excerpt": "Assumption, observation-adequacy, protocol-adequacy, execution, truth, activation, participation, decision-scope progress, decision-scope closure, surrounding-execution progress, s"
    },
    {
      "id": "FR-006-AC-4",
      "path": "spec/functional/FR-006-produce-canonical-protocol-results.md",
      "role": "examined",
      "excerpt": "Every source, profile, model, protocol, request, binding, input, backend, decision-scope progress/closure, surrounding-execution progress/closure, complete-global-conformance closu"
    },
    {
      "id": "FR-006-AC-5",
      "path": "spec/functional/FR-006-produce-canonical-protocol-results.md",
      "role": "examined",
      "excerpt": "Support lineage rejects missing predecessors, cycles, self-reference, and invented independence while retaining shared and derived relations, which is exactly [FR-033](./FR-033-pre"
    },
    {
      "id": "FR-006-AC-6",
      "path": "spec/functional/FR-006-produce-canonical-protocol-results.md",
      "role": "examined",
      "excerpt": "A later contradiction emits a new linked result and cannot rewrite the earlier artifact bytes or identity, which is exactly [FR-033](./FR-033-preserve-result-lineage-integrity.md)-"
    },
    {
      "id": "FR-006-AC-7",
      "path": "spec/functional/FR-006-produce-canonical-protocol-results.md",
      "role": "examined",
      "excerpt": "Independently omitting, duplicating, cross-wiring, changing scope on, or changing the identity of every required execution, branch, and workflow global-closure premise refuses the "
    },
    {
      "id": "FR-006-AC-9",
      "path": "spec/functional/FR-006-produce-canonical-protocol-results.md",
      "role": "examined",
      "excerpt": "The public assessment operation returns exactly the three-member operation outcome; each completed bundle has one ordered canonical row or claim-local refusal per admitted claim, a"
    },
    {
      "id": "FR-007-AC-1",
      "path": "spec/functional/FR-007-project-results-to-e02.md",
      "role": "examined",
      "excerpt": "Every FR-062 source-state row maps to the exact E02 state and retains all simultaneous independent findings, which is exactly [FR-035](./FR-035-map-source-states-to-e02.md)-AC-1 an"
    },
    {
      "id": "FR-007-AC-2",
      "path": "spec/functional/FR-007-project-results-to-e02.md",
      "role": "examined",
      "excerpt": "The FR-062 projection identity has the required structure and reflects the source key and adapter identity/version, while target-binding equality is checked separately by [FR-034]("
    },
    {
      "id": "FR-007-AC-3",
      "path": "spec/functional/FR-007-project-results-to-e02.md",
      "role": "examined",
      "excerpt": "E02 adequacy derives only from observation adequacy; protocol adequacy and sufficiency cannot cross into it, which is exactly [FR-035](./FR-035-map-source-states-to-e02.md)-AC-3, w"
    },
    {
      "id": "FR-007-AC-4",
      "path": "spec/functional/FR-007-project-results-to-e02.md",
      "role": "examined",
      "excerpt": "Invalid source, unknown versions/features, identity mismatch, unrepresented source state, missing context, or finding loss returns the stable FR-062 refusal and emits no result, wh"
    },
    {
      "id": "FR-007-AC-5",
      "path": "spec/functional/FR-007-project-results-to-e02.md",
      "role": "examined",
      "excerpt": "`nativeArtifacts` retains exactly one unchanged canonical-result URI and media type, and provenance names both producer and adapter versions, which is exactly [FR-034](./FR-034-der"
    },
    {
      "id": "FR-007-AC-6",
      "path": "spec/functional/FR-007-project-results-to-e02.md",
      "role": "examined",
      "excerpt": "Every `satisfied` or `violated` projection preserves the source decision-support identities and dependence relation and invents no independent oracle relation, which is exactly [FR"
    },
    {
      "id": "FR-008-AC-1",
      "path": "spec/functional/FR-008-return-typed-refusals.md",
      "role": "examined",
      "excerpt": "Every public failure path returns the correct typed domain, actual originating stage and prerequisites, affected identity, available or explicitly unavailable location, ordered cau"
    },
    {
      "id": "FR-008-AC-2",
      "path": "spec/functional/FR-008-return-typed-refusals.md",
      "role": "examined",
      "excerpt": "Changing display text, locale, or cause rendering cannot change a refusal identity, code, equality comparison, or retry decision; stated atomically by FR-037-AC-1 and FR-037-AC-2."
    },
    {
      "id": "FR-008-AC-3",
      "path": "spec/functional/FR-008-return-typed-refusals.md",
      "role": "examined",
      "excerpt": "An unknown source contract, revision, or code becomes local `unsupported-source-contract`, retains the original contract and value uninterpreted, and cannot be interpreted through "
    },
    {
      "id": "FR-008-AC-4",
      "path": "spec/functional/FR-008-return-typed-refusals.md",
      "role": "examined",
      "excerpt": "Fatal resolution, admission, representation, whole-assessment, result, projection and evidence refusals emit no success value; only FR-005-authorized independent claim outcomes sur"
    },
    {
      "id": "FR-008-AC-5",
      "path": "spec/functional/FR-008-return-typed-refusals.md",
      "role": "examined",
      "excerpt": "Missing/conflicting definitions, wrong-anchor/missing-observation, unproved-definedness/logical-violation, identity-mismatch/incomplete-evidence, and unsupported-capability/project"
    },
    {
      "id": "FR-008-AC-6",
      "path": "spec/functional/FR-008-return-typed-refusals.md",
      "role": "examined",
      "excerpt": "Changing the selected source-contract identity/revision or erasing/replacing the originating stage refuses the response; stated atomically by FR-038-AC-3."
    },
    {
      "id": "FR-008-AC-7",
      "path": "spec/functional/FR-008-return-typed-refusals.md",
      "role": "examined",
      "excerpt": "Unavailable source context remains explicit and cannot become an invented location through repeated display names, paths, or message text; stated atomically by FR-037-AC-3 and FR-0"
    },
    {
      "id": "FR-008-AC-8",
      "path": "spec/functional/FR-008-return-typed-refusals.md",
      "role": "examined",
      "excerpt": "Simultaneous independent invalid and missing-input dispositions both remain represented under a zero-detail control and cannot produce overall success; stated atomically by FR-039-"
    },
    {
      "id": "FR-008-AC-9",
      "path": "spec/functional/FR-008-return-typed-refusals.md",
      "role": "examined",
      "excerpt": "Every obligation in the Behavior table is carried by exactly one child requirement, every child carries a `part_of` edge to this requirement and this requirement a `contains` edge "
    },
    {
      "id": "FR-009-AC-1",
      "path": "spec/functional/FR-009-lock-external-producer-revisions.md",
      "role": "examined",
      "excerpt": "A manifest whose required entries are each `pinned`, carry the owner's immutability assertion, and whose recorded digest equals the digest recomputed over the exact delivered bytes"
    },
    {
      "id": "FR-009-AC-2",
      "path": "spec/functional/FR-009-lock-external-producer-revisions.md",
      "role": "examined",
      "excerpt": "An absent, unpinned or digest-mismatched required entry returns a typed refusal naming that entry's (owner, contract version) key before the producer loader is invoked, and the dif"
    },
    {
      "id": "FR-009-AC-3",
      "path": "spec/functional/FR-009-lock-external-producer-revisions.md",
      "role": "examined",
      "excerpt": "A scenario requiring an entry whose status is `unresolved` is reported unexecutable, is not reported as passing or failing, and cannot be satisfied by another contract version of t"
    },
    {
      "id": "FR-009-AC-4",
      "path": "spec/functional/FR-009-lock-external-producer-revisions.md",
      "role": "examined",
      "excerpt": "A producer-emitted sidecar, display name, path, or message text cannot supply, select, or repair an entry; removing the sidecar changes no resolution outcome; and the manifest cann"
    },
    {
      "id": "FR-009-AC-5",
      "path": "spec/functional/FR-009-lock-external-producer-revisions.md",
      "role": "examined",
      "excerpt": "Two entries sharing one (owner, contract version) key refuse every integration before resolution; otherwise adding, removing or repinning one entry changes the resolved owner, revi"
    },
    {
      "id": "FR-009-AC-6",
      "path": "spec/functional/FR-009-lock-external-producer-revisions.md",
      "role": "examined",
      "excerpt": "Resolution at exactly the granted entry-count, byte and deadline budget completes; one unit short on entry or declared selection bytes, or a deadline expired before the selection u"
    },
    {
      "id": "FR-009-AC-7",
      "path": "spec/functional/FR-009-lock-external-producer-revisions.md",
      "role": "examined",
      "excerpt": "Neither resolution nor digest verification reads a version-control history, spawns a process, or accesses a network, and no entry's digest is computed over anything but the exact d"
    },
    {
      "id": "FR-009-AC-8",
      "path": "spec/functional/FR-009-lock-external-producer-revisions.md",
      "role": "examined",
      "excerpt": "Every interface entry matches an adapter-compiled accepted owner, contract version and revision before a private capability is minted; changing only the manifest revision or reusin"
    },
    {
      "id": "FR-009-AC-9",
      "path": "spec/functional/FR-009-lock-external-producer-revisions.md",
      "role": "examined",
      "excerpt": "Permuting the offered manifest changes neither the resolved set nor the first refusal/partial state: required keys are validated in FR-012 lexicographic order solely for determinis"
    },
    {
      "id": "FR-010-AC-1",
      "path": "spec/functional/FR-010-consumer-boundary-domain.md",
      "role": "examined",
      "excerpt": "Every object specified in this context describes a consumer-to-producer relationship, and none restates a protocol, temporal or observation subject that `quire-specification` types"
    },
    {
      "id": "FR-010-AC-2",
      "path": "spec/functional/FR-010-consumer-boundary-domain.md",
      "role": "examined",
      "excerpt": "In each requirement that has had an adoption pass, every protocol, temporal or observation noun resolves to an `ix://agent-ix/quire-specification` reference rather than a local def"
    },
    {
      "id": "FR-010-AC-3",
      "path": "spec/functional/FR-010-consumer-boundary-domain.md",
      "role": "examined",
      "excerpt": "Where an upstream artifact records an open gap, a requirement depending on it cites the gap rather than supplying the missing identity, key or order locally."
    },
    {
      "id": "FR-011-AC-1",
      "path": "spec/functional/FR-011-consumer-identity-discipline.md",
      "role": "examined",
      "excerpt": "Every object specified in this repository declares one of the three upstream kinds and one authority, and declares no fourth kind."
    },
    {
      "id": "FR-011-AC-2",
      "path": "spec/functional/FR-011-consumer-identity-discipline.md",
      "role": "examined",
      "excerpt": "Changing only a display name, source path, span, message text, locale or rendering changes no object's identity, equality, or ordering position."
    },
    {
      "id": "FR-011-AC-3",
      "path": "spec/functional/FR-011-consumer-identity-discipline.md",
      "role": "examined",
      "excerpt": "A producer-authored identity is byte-equal to the producer bytes it arrived as; deriving, repairing, normalizing or recomputing one refuses rather than producing a repaired identit"
    },
    {
      "id": "FR-011-AC-4",
      "path": "spec/functional/FR-011-consumer-identity-discipline.md",
      "role": "examined",
      "excerpt": "A consumer-authored identity appears in no canonical result as a producer reference, and is accepted by no producer as a selector."
    },
    {
      "id": "FR-011-AC-5",
      "path": "spec/functional/FR-011-consumer-identity-discipline.md",
      "role": "examined",
      "excerpt": "A key component from one digest domain is unequal to a component from another domain over identical bytes, in both directions, for every ordered pair of domains."
    },
    {
      "id": "FR-011-AC-6",
      "path": "spec/functional/FR-011-consumer-identity-discipline.md",
      "role": "examined",
      "excerpt": "An absent kind, authority or key component compares equal only to absence, never to a default, an empty value, or a present component."
    },
    {
      "id": "FR-011-AC-7",
      "path": "spec/functional/FR-011-consumer-identity-discipline.md",
      "role": "examined",
      "excerpt": "An object whose collections admit duplicate, surplus, missing or reordered defects either declares a unique key and a total order, or names the owner that has not declared one and "
    },
    {
      "id": "FR-012-AC-1",
      "path": "spec/functional/FR-012-version-lock-entry.md",
      "role": "examined",
      "excerpt": "Two entries whose owner and contract version are equal compare equal regardless of revision, digest, status or reason, and two entries differing in owner or in contract version com"
    },
    {
      "id": "FR-012-AC-2",
      "path": "spec/functional/FR-012-version-lock-entry.md",
      "role": "examined",
      "excerpt": "Two entries sharing one (owner, contract version) key refuse every integration before resolution, and the refusal names that key."
    },
    {
      "id": "FR-012-AC-3",
      "path": "spec/functional/FR-012-version-lock-entry.md",
      "role": "examined",
      "excerpt": "Adding, removing or repinning one entry changes the resolved owner, revision, digest and status of no other entry, and a manifest carrying two contract versions of one owner resolv"
    },
    {
      "id": "FR-012-AC-4",
      "path": "spec/functional/FR-012-version-lock-entry.md",
      "role": "examined",
      "excerpt": "An entry naming an interface rather than an artifact carries a revision and no digest, and synthesizing a digest for it refuses."
    },
    {
      "id": "FR-012-AC-5",
      "path": "spec/functional/FR-012-version-lock-entry.md",
      "role": "examined",
      "excerpt": "A producer-emitted sidecar, display name, path or message text supplies, selects or repairs no entry, and removing the sidecar changes no resolution outcome."
    },
    {
      "id": "FR-012-AC-6",
      "path": "spec/functional/FR-012-version-lock-entry.md",
      "role": "examined",
      "excerpt": "An entry carries exactly one status, an `unresolved` entry carries a reason, and an `unresolved` entry is satisfied by no other contract version of the same owner."
    },
    {
      "id": "FR-012-AC-7",
      "path": "spec/functional/FR-012-version-lock-entry.md",
      "role": "examined",
      "excerpt": "Entry keys admit a total order; duplicate detection and required-entry validation run over that order, so no duplicate escapes and manifest permutation changes neither the resolved"
    },
    {
      "id": "FR-012-AC-8",
      "path": "spec/functional/FR-012-version-lock-entry.md",
      "role": "examined",
      "excerpt": "The entry's revision and digest are byte-equal to the producer bytes they arrived as, and no resolution path derives, repairs or normalizes either."
    },
    {
      "id": "FR-013-AC-1",
      "path": "spec/functional/FR-013-offered-versus-selected.md",
      "role": "examined",
      "excerpt": "An offered value and a selected value that are byte-identical over the same subject compare unequal, in both directions."
    },
    {
      "id": "FR-013-AC-2",
      "path": "spec/functional/FR-013-offered-versus-selected.md",
      "role": "examined",
      "excerpt": "Mutating one selected-side component and mutating one offered-side component each refuse, and the two refusals carry different typed outcome identities."
    },
    {
      "id": "FR-013-AC-3",
      "path": "spec/functional/FR-013-offered-versus-selected.md",
      "role": "examined",
      "excerpt": "An offer supplying its own acceptance context \u2014 a selection, display name, path or message text carried in the offered bytes or in any member the offered package contains \u2014 is refu"
    },
    {
      "id": "FR-013-AC-4",
      "path": "spec/functional/FR-013-offered-versus-selected.md",
      "role": "examined",
      "excerpt": "A contract version is inferred from no offered bytes, and bytes of one contract version offered at another version's entry point refuse with no detection, upgrade, downgrade or rep"
    },
    {
      "id": "FR-013-AC-5",
      "path": "spec/functional/FR-013-offered-versus-selected.md",
      "role": "examined",
      "excerpt": "A value carrying no provenance is compared against no expectation and refuses, rather than being defaulted to either side."
    },
    {
      "id": "FR-013-AC-6",
      "path": "spec/functional/FR-013-offered-versus-selected.md",
      "role": "examined",
      "excerpt": "Changing a display name, path, span, message text or locale on either side changes no equality verdict and no refusal identity."
    },
    {
      "id": "FR-013-AC-7",
      "path": "spec/functional/FR-013-offered-versus-selected.md",
      "role": "examined",
      "excerpt": "Every selected component is retained before any offered byte for that integration is inspected."
    },
    {
      "id": "FR-013-AC-8",
      "path": "spec/functional/FR-013-offered-versus-selected.md",
      "role": "examined",
      "excerpt": "Adding or removing a member of either axis list changes no obligation this requirement states."
    },
    {
      "id": "FR-014-AC-1",
      "path": "spec/functional/FR-014-typed-outcome-identity.md",
      "role": "examined",
      "excerpt": "Within one integration, for every ordered pair of distinct refusal axes, the producer-supplied interior the envelope carries \u2014 its `FR-271` code together with its `FR-272` typed ca"
    },
    {
      "id": "FR-014-AC-2",
      "path": "spec/functional/FR-014-typed-outcome-identity.md",
      "role": "examined",
      "excerpt": "Two scenarios exercising one refusal axis carry one typed outcome identity, and that sharing fails no distinctness check."
    },
    {
      "id": "FR-014-AC-3",
      "path": "spec/functional/FR-014-typed-outcome-identity.md",
      "role": "examined",
      "excerpt": "The mapping from this repository's outcome identities onto a producer's typed refusal enum is injective, and a mapping collapsing two axes onto one producer code fails the integrat"
    },
    {
      "id": "FR-014-AC-4",
      "path": "spec/functional/FR-014-typed-outcome-identity.md",
      "role": "examined",
      "excerpt": "An integration in which every negative row carries one shared identity fails the distinctness check rather than reporting full coverage."
    },
    {
      "id": "FR-014-AC-5",
      "path": "spec/functional/FR-014-typed-outcome-identity.md",
      "role": "examined",
      "excerpt": "A resource-incomplete outcome, a typed refusal and a completed assessment with an empty finding set compare pairwise unequal, and none is reported as another."
    },
    {
      "id": "FR-014-AC-6",
      "path": "spec/functional/FR-014-typed-outcome-identity.md",
      "role": "examined",
      "excerpt": "No non-success outcome yields an E02 success, and each carries the reason it did not complete."
    },
    {
      "id": "FR-014-AC-7",
      "path": "spec/functional/FR-014-typed-outcome-identity.md",
      "role": "examined",
      "excerpt": "Changing display text, locale or cause rendering changes no typed outcome identity, no equality comparison and no retry decision."
    },
    {
      "id": "FR-014-AC-8",
      "path": "spec/functional/FR-014-typed-outcome-identity.md",
      "role": "examined",
      "excerpt": "Protocol-local causes use only FR-008's closed vocabulary; a source interior uses only its selected accepted contract, and an unknown contract is retained opaque rather than comple"
    },
    {
      "id": "FR-015-AC-1",
      "path": "spec/functional/FR-015-digest-domain-selection.md",
      "role": "examined",
      "excerpt": "Every digest domain label this repository uses is a member `FR-201` publishes, spelled exactly as published, and no label originates here."
    },
    {
      "id": "FR-015-AC-2",
      "path": "spec/functional/FR-015-digest-domain-selection.md",
      "role": "examined",
      "excerpt": "A producer reference arriving with an upstream domain label retains that label exactly through admission, linking and result production, and is never re-labelled."
    },
    {
      "id": "FR-015-AC-5",
      "path": "spec/functional/FR-015-digest-domain-selection.md",
      "role": "examined",
      "excerpt": "No requirement, test or implementation in this repository declares a digest domain vocabulary of its own."
    },
    {
      "id": "FR-016-AC-1",
      "path": "spec/functional/FR-016-resource-budget-dimension.md",
      "role": "examined",
      "excerpt": "Each member's value equals the exact label string in the table, and every dimension NFR-002 and FR-009 declare has exactly one member."
    },
    {
      "id": "FR-016-AC-2",
      "path": "spec/functional/FR-016-resource-budget-dimension.md",
      "role": "examined",
      "excerpt": "One unchanged input under a budget exactly sufficient on a dimension admits; the same input one unit short on that dimension returns that dimension's typed resource-incomplete outc"
    },
    {
      "id": "FR-016-AC-3",
      "path": "spec/functional/FR-016-resource-budget-dimension.md",
      "role": "examined",
      "excerpt": "A one-short outcome names exactly the dimension whose increment was refused and no other, and exhausting one dimension changes the granted or consumed figure of no other dimension."
    },
    {
      "id": "FR-016-AC-4",
      "path": "spec/functional/FR-016-resource-budget-dimension.md",
      "role": "examined",
      "excerpt": "No work is admitted after a dimension is exhausted, and no budget, partial consumption or accounted state carries from an exhausted attempt into the fresh-budget attempt."
    },
    {
      "id": "FR-016-AC-5",
      "path": "spec/functional/FR-016-resource-budget-dimension.md",
      "role": "examined",
      "excerpt": "A resource-incomplete outcome carries an identity distinct from every typed refusal identity, and is reported as neither a refusal of the input nor a pass."
    },
    {
      "id": "FR-016-AC-6",
      "path": "spec/functional/FR-016-resource-budget-dimension.md",
      "role": "examined",
      "excerpt": "The granted and consumed figures on `elapsed-time` and `resolution-deadline` are recorded on every scenario that declares them, and neither consumed figure participates in the repr"
    },
    {
      "id": "FR-016-AC-7",
      "path": "spec/functional/FR-016-resource-budget-dimension.md",
      "role": "examined",
      "excerpt": "A dimension with no caller-supplied limit compares only to absence, receives no default, and inherits no other dimension's grant."
    },
    {
      "id": "FR-016-AC-8",
      "path": "spec/functional/FR-016-resource-budget-dimension.md",
      "role": "examined",
      "excerpt": "Exceeding `resolution-entry-count` or pre-charged `resolution-bytes-digested`, or reaching `resolution-deadline` before loaded-selection verification, returns that dimension's reso"
    },
    {
      "id": "FR-017-AC-1",
      "path": "spec/functional/FR-017-scenario-evidence-record.md",
      "role": "examined",
      "excerpt": "Every owed scenario of IT-001, IT-002 and IT-003 composes exactly one record carrying its resolution state or declared legacy-compatibility selection, the owner of every consumed p"
    },
    {
      "id": "FR-017-AC-2",
      "path": "spec/functional/FR-017-scenario-evidence-record.md",
      "role": "examined",
      "excerpt": "A scenario that refused during resolution composes a record carrying the deterministic partial resolution state reached plus the first causing entry, and is not disqualified for la"
    },
    {
      "id": "FR-017-AC-3",
      "path": "spec/functional/FR-017-scenario-evidence-record.md",
      "role": "examined",
      "excerpt": "A scenario that cannot retain every required component is reported unexecuted rather than passing, and an owed-but-unexecuted scenario is scored as a zero rather than omitted from "
    },
    {
      "id": "FR-017-AC-4",
      "path": "spec/functional/FR-017-scenario-evidence-record.md",
      "role": "examined",
      "excerpt": "Across one integration, no two distinct refusal axes retain the same typed outcome interior, while two scenarios exercising one axis may share one."
    },
    {
      "id": "FR-017-AC-5",
      "path": "spec/functional/FR-017-scenario-evidence-record.md",
      "role": "examined",
      "excerpt": "Budget figures are retained per declared dimension and never as one scalar total, and the `elapsed-time` and `resolution-deadline` consumed figures participate in no reproducibilit"
    },
    {
      "id": "FR-017-AC-6",
      "path": "spec/functional/FR-017-scenario-evidence-record.md",
      "role": "examined",
      "excerpt": "Two records whose integration and scenario are equal compare equal regardless of resolution state, owners, outcome or budget figures, and two records differing in either component "
    },
    {
      "id": "FR-017-AC-7",
      "path": "spec/functional/FR-017-scenario-evidence-record.md",
      "role": "examined",
      "excerpt": "No record is written to a local evidence store, catalogue or disposition path, no record is read back after handoff, and no sufficiency decision is taken in this repository."
    },
    {
      "id": "FR-017-AC-8",
      "path": "spec/functional/FR-017-scenario-evidence-record.md",
      "role": "examined",
      "excerpt": "No record appears in a canonical result as a producer reference, and no producer accepts a record or any component this repository authored in it as a selector."
    },
    {
      "id": "FR-018-AC-1",
      "path": "spec/functional/FR-018-delivery-index.md",
      "role": "examined",
      "excerpt": "A delivery index supplies, selects or repairs no version-lock entry, and removing or editing an index changes no resolution outcome."
    },
    {
      "id": "FR-018-AC-2",
      "path": "spec/functional/FR-018-delivery-index.md",
      "role": "examined",
      "excerpt": "A delivery index reference does not satisfy a comparison expecting `ix://agent-ix/quire-specification/FR-224`, and no member of an index is a transport observation."
    },
    {
      "id": "FR-018-AC-3",
      "path": "spec/functional/FR-018-delivery-index.md",
      "role": "examined",
      "excerpt": "Two indexed bindings naming one `declaration` are reported, and an indexed binding whose `declaration` is lower than its predecessor's is reported, each naming the offending declar"
    },
    {
      "id": "FR-018-AC-4",
      "path": "spec/functional/FR-018-delivery-index.md",
      "role": "examined",
      "excerpt": "The binding cardinality, uniqueness and order this repository enforces are exactly the producer's quoted wire rule, and no stronger declared key or declared total order is asserted"
    },
    {
      "id": "FR-018-AC-5",
      "path": "spec/functional/FR-018-delivery-index.md",
      "role": "examined",
      "excerpt": "A selected delivery root that is missing, an absent index, a missing indexed member, an unreadable index, an ambiguous mutation, a repeated declaration and unsorted bindings are ea"
    },
    {
      "id": "FR-018-AC-6",
      "path": "spec/functional/FR-018-delivery-index.md",
      "role": "examined",
      "excerpt": "An unindexed input role and an unreadable indexed file are distinct outcomes, and neither is collapsed into the other."
    },
    {
      "id": "FR-018-AC-7",
      "path": "spec/functional/FR-018-delivery-index.md",
      "role": "examined",
      "excerpt": "Two indices whose delivery root and producer revision are equal compare equal regardless of offer, inputs, temporal records, limits or mutations, and two differing in either key co"
    },
    {
      "id": "FR-018-AC-8",
      "path": "spec/functional/FR-018-delivery-index.md",
      "role": "examined",
      "excerpt": "The indexed offer bytes are carried through untouched and are byte-equal to the bytes submitted, and no indexed producer input is recanonicalized."
    },
    {
      "id": "FR-018-AC-9",
      "path": "spec/functional/FR-018-delivery-index.md",
      "role": "examined",
      "excerpt": "Limit entries are keyed by accountant owner/contract version plus exact typed dimension; duplicate keys refuse, canonical ordering is deterministic and non-causal, and Protocol and"
    },
    {
      "id": "FR-019-AC-1",
      "path": "spec/functional/FR-019-derive-obligation-activation-state.md",
      "role": "examined",
      "excerpt": "An established admitted trigger reports `triggered`, an absent trigger under authoritatively closed and complete input reports `inactive`, and an unavailable trigger under open or "
    },
    {
      "id": "FR-019-AC-2",
      "path": "spec/functional/FR-019-derive-obligation-activation-state.md",
      "role": "examined",
      "excerpt": "Independently substituting the workflow, role, effect, predicate, capture or evaluation-anchor identity refuses the activation binding and names the substituted axis."
    },
    {
      "id": "FR-019-AC-3",
      "path": "spec/functional/FR-019-derive-obligation-activation-state.md",
      "role": "examined",
      "excerpt": "An `inactive` or `activation-unknown` activation is readable as neither satisfied nor demonstrated, and neither is evidence that recovery behavior was exercised."
    },
    {
      "id": "FR-019-AC-4",
      "path": "spec/functional/FR-019-derive-obligation-activation-state.md",
      "role": "examined",
      "excerpt": "An obligation with no reported activation compares only to absence and receives no default activation value."
    },
    {
      "id": "FR-019-AC-5",
      "path": "spec/functional/FR-019-derive-obligation-activation-state.md",
      "role": "examined",
      "excerpt": "No business-action invocation occurs on any activation path, including the refusing ones."
    },
    {
      "id": "FR-020-AC-1",
      "path": "spec/functional/FR-020-register-compensation-once-per-effect.md",
      "role": "examined",
      "excerpt": "One paired successful forward effect creates exactly one matching compensation registration, positioned after that effect."
    },
    {
      "id": "FR-020-AC-2",
      "path": "spec/functional/FR-020-register-compensation-once-per-effect.md",
      "role": "examined",
      "excerpt": "A send, a receive, a duplicate delivery and a failed attempt each create no compensation registration."
    },
    {
      "id": "FR-020-AC-3",
      "path": "spec/functional/FR-020-register-compensation-once-per-effect.md",
      "role": "examined",
      "excerpt": "Replaying one established forward effect identity yields the already-created registration and no second one."
    },
    {
      "id": "FR-020-AC-4",
      "path": "spec/functional/FR-020-register-compensation-once-per-effect.md",
      "role": "examined",
      "excerpt": "Each registration retains its source compensation definition identity and its forward effect identity, and a registration whose forward effect identity is absent compares only to a"
    },
    {
      "id": "FR-020-AC-5",
      "path": "spec/functional/FR-020-register-compensation-once-per-effect.md",
      "role": "examined",
      "excerpt": "No compensating-operation invocation occurs on any registration path."
    },
    {
      "id": "FR-021-AC-1",
      "path": "spec/functional/FR-021-distinguish-commit-ordering-violations.md",
      "role": "examined",
      "excerpt": "Registration after a forbidding commit, compensation before its paired forward effect, and compensation after a forbidding commit are reported as three mutually distinguishable typ"
    },
    {
      "id": "FR-021-AC-2",
      "path": "spec/functional/FR-021-distinguish-commit-ordering-violations.md",
      "role": "examined",
      "excerpt": "Compensation observed before its paired forward effect leaves the registration invalid and does not retroactively validate it."
    },
    {
      "id": "FR-021-AC-3",
      "path": "spec/functional/FR-021-distinguish-commit-ordering-violations.md",
      "role": "examined",
      "excerpt": "An absent commit observation yields incomplete or pending, compares only to absence, and is not read as a permitting commit."
    },
    {
      "id": "FR-021-AC-4",
      "path": "spec/functional/FR-021-distinguish-commit-ordering-violations.md",
      "role": "examined",
      "excerpt": "Reordering equal-time observations, changing a key order or changing a source position adds no ordering violation and removes none."
    },
    {
      "id": "FR-022-AC-1",
      "path": "spec/functional/FR-022-enforce-finite-compensation-retry-bound.md",
      "role": "examined",
      "excerpt": "An attempt sequence exactly at the authored finite bound is admitted, and the next attempt is refused as a bound violation distinct from every ordering violation."
    },
    {
      "id": "FR-022-AC-2",
      "path": "spec/functional/FR-022-enforce-finite-compensation-retry-bound.md",
      "role": "examined",
      "excerpt": "The bound counts the initial attempt, so a bound of n admits n attempts rather than n retries after a first."
    },
    {
      "id": "FR-022-AC-3",
      "path": "spec/functional/FR-022-enforce-finite-compensation-retry-bound.md",
      "role": "examined",
      "excerpt": "A refused over-bound attempt changes none of the attempts already retained for that registration."
    },
    {
      "id": "FR-022-AC-4",
      "path": "spec/functional/FR-022-enforce-finite-compensation-retry-bound.md",
      "role": "examined",
      "excerpt": "An attempt beyond the bound produces no recovery success on any path."
    },
    {
      "id": "FR-022-AC-5",
      "path": "spec/functional/FR-022-enforce-finite-compensation-retry-bound.md",
      "role": "examined",
      "excerpt": "A registration with no authored retry bound compares only to absence and is never evaluated against zero, an unbounded policy or a default."
    },
    {
      "id": "FR-023-AC-1",
      "path": "spec/functional/FR-023-preserve-attempt-and-effect-distinctness.md",
      "role": "examined",
      "excerpt": "A failed attempt, a timed-out attempt, a duplicate effect and a successful attempt remain four separately reported facts and none collapses into another."
    },
    {
      "id": "FR-023-AC-2",
      "path": "spec/functional/FR-023-preserve-attempt-and-effect-distinctness.md",
      "role": "examined",
      "excerpt": "A failed attempt followed by one successful retry retains two attempts and one established compensation effect, each with its own retained identity and its owning registration."
    },
    {
      "id": "FR-023-AC-3",
      "path": "spec/functional/FR-023-preserve-attempt-and-effect-distinctness.md",
      "role": "examined",
      "excerpt": "An attempt identity compares unequal to every effect identity, and a transport status, trace identifier, repeated delivery or equal payload establishes neither."
    },
    {
      "id": "FR-023-AC-4",
      "path": "spec/functional/FR-023-preserve-attempt-and-effect-distinctness.md",
      "role": "examined",
      "excerpt": "A successful send, receive, delivery, attempt or compensating-operation invocation establishes no compensation effect."
    },
    {
      "id": "FR-023-AC-5",
      "path": "spec/functional/FR-023-preserve-attempt-and-effect-distinctness.md",
      "role": "examined",
      "excerpt": "An attempt whose outcome is unestablished carries an absent outcome and receives no default member, and every reported outcome matches an exact `FR-236` label with no alias spellin"
    },
    {
      "id": "FR-024-AC-1",
      "path": "spec/functional/FR-024-decide-recovery-only-by-authored-relation.md",
      "role": "examined",
      "excerpt": "One achieved state satisfies an authored partial-recovery relation while failing an authored full-restoration relation over the same captured target, and neither relation ranks abo"
    },
    {
      "id": "FR-024-AC-2",
      "path": "spec/functional/FR-024-decide-recovery-only-by-authored-relation.md",
      "role": "examined",
      "excerpt": "A recovery success requires the exact authored relation, the exact captured target state and the exact achieved state; independently varying any of the three withdraws the success."
    },
    {
      "id": "FR-024-AC-3",
      "path": "spec/functional/FR-024-decide-recovery-only-by-authored-relation.md",
      "role": "examined",
      "excerpt": "An aggregating recovery relation reports success only under a complete declared population with its exact membership rule, exact arithmetic over the captured amount, and applicable"
    },
    {
      "id": "FR-024-AC-4",
      "path": "spec/functional/FR-024-decide-recovery-only-by-authored-relation.md",
      "role": "examined",
      "excerpt": "A missing target state, achieved state, commit, compensation effect, population member, membership authority or completeness authority yields incomplete or pending rather than reco"
    },
    {
      "id": "FR-024-AC-5",
      "path": "spec/functional/FR-024-decide-recovery-only-by-authored-relation.md",
      "role": "examined",
      "excerpt": "A successful compensating-operation invocation and a `successful` attempt outcome each produce no recovery success on their own."
    },
    {
      "id": "FR-024-AC-6",
      "path": "spec/functional/FR-024-decide-recovery-only-by-authored-relation.md",
      "role": "examined",
      "excerpt": "Production accepts no recovery callback; every supported relation is evaluated from its admitted typed QSL expression under the selected capability/accountant, and unsupported, sou"
    },
    {
      "id": "FR-025-AC-1",
      "path": "spec/functional/FR-025-assess-compensation-without-io.md",
      "role": "examined",
      "excerpt": "Every action-capable business-operation and compensation-operation double reports a zero call count after a full compensation assessment, including assessments that refuse or retur"
    },
    {
      "id": "FR-025-AC-2",
      "path": "spec/functional/FR-025-assess-compensation-without-io.md",
      "role": "examined",
      "excerpt": "Mutating the evaluator API to admit one network, storage, telemetry, business-operation or compensation-operation capability is rejected before assessment, and the rejection names "
    },
    {
      "id": "FR-025-AC-3",
      "path": "spec/functional/FR-025-assess-compensation-without-io.md",
      "role": "examined",
      "excerpt": "Mutating the declared dependency inventory to admit one such capability is rejected before assessment independently of the API mutation."
    },
    {
      "id": "FR-026-AC-1",
      "path": "spec/functional/FR-026-admit-every-permitted-observation-order.md",
      "role": "examined",
      "excerpt": "Every permitted split-shipment and compensation interleaving produces the same conformant obligation set."
    },
    {
      "id": "FR-026-AC-2",
      "path": "spec/functional/FR-026-admit-every-permitted-observation-order.md",
      "role": "examined",
      "excerpt": "Permuting equal-time arrivals under the time key the accepted temporal profile declares changes no assessed value, over a non-singleton permutation set."
    },
    {
      "id": "FR-026-AC-3",
      "path": "spec/functional/FR-026-admit-every-permitted-observation-order.md",
      "role": "examined",
      "excerpt": "No timestamp, arrival position, ingestion order, or key order over an adopted upstream object is cited as evidence of a causal edge."
    },
    {
      "id": "FR-027-AC-1",
      "path": "spec/functional/FR-027-report-adverse-observations-without-repair.md",
      "role": "examined",
      "excerpt": "Unexpected, missing, causally misordered, duplicate-effect, and cross-workflow observations identify the exact affected obligation."
    },
    {
      "id": "FR-027-AC-2",
      "path": "spec/functional/FR-027-report-adverse-observations-without-repair.md",
      "role": "examined",
      "excerpt": "Every reported adverse observation is present and unaltered in the admitted fact set the evaluator assessed, and no adverse report removes, reorders, merges or re-correlates a reco"
    },
    {
      "id": "FR-027-AC-3",
      "path": "spec/functional/FR-027-report-adverse-observations-without-repair.md",
      "role": "examined",
      "excerpt": "An adverse report names the affected obligation's exact identity, and changing only a display name, source path, span, message text, locale or rendering changes no report's subject"
    },
    {
      "id": "FR-027-AC-4",
      "path": "spec/functional/FR-027-report-adverse-observations-without-repair.md",
      "role": "examined",
      "excerpt": "A required absent valuation is reported as incomplete or refused and never as false or an implicit option-none."
    },
    {
      "id": "FR-027-AC-5",
      "path": "spec/functional/FR-027-report-adverse-observations-without-repair.md",
      "role": "examined",
      "excerpt": "Each adverse kind remains a distinct outcome; one adverse observation is never reclassified as another kind to make it reportable."
    },
    {
      "id": "FR-028-AC-1",
      "path": "spec/functional/FR-028-keep-progress-and-closure-axes-independent.md",
      "role": "examined",
      "excerpt": "Decision-scope progress/closure, surrounding-execution progress/closure and complete-global-conformance closure remain independent; changing or closing one cannot change or close a"
    },
    {
      "id": "FR-028-AC-2",
      "path": "spec/functional/FR-028-keep-progress-and-closure-axes-independent.md",
      "role": "examined",
      "excerpt": "A refused, unsupported, resource-incomplete or failed assessment execution cannot become complete global success, and an open or incomplete closure axis cannot either."
    },
    {
      "id": "FR-028-AC-3",
      "path": "spec/functional/FR-028-keep-progress-and-closure-axes-independent.md",
      "role": "examined",
      "excerpt": "Closing a decision scope neither closes the surrounding execution nor establishes complete global conformance."
    },
    {
      "id": "FR-028-AC-4",
      "path": "spec/functional/FR-028-keep-progress-and-closure-axes-independent.md",
      "role": "examined",
      "excerpt": "A claim that is not global conformance carries `not-required` explicitly with no global premise set; a global premise supplied under `not-required`, a global claim marked `not-requ"
    },
    {
      "id": "FR-028-AC-5",
      "path": "spec/functional/FR-028-keep-progress-and-closure-axes-independent.md",
      "role": "examined",
      "excerpt": "Omitting, duplicating, cross-wiring or scope-mismatching a required global closure premise withholds complete global success without changing an independently settled per-obligatio"
    },
    {
      "id": "FR-028-AC-6",
      "path": "spec/functional/FR-028-keep-progress-and-closure-axes-independent.md",
      "role": "examined",
      "excerpt": "An absent progress, closure or completeness fact compares only to absence and is never defaulted to open, closed, complete or `not-required`."
    },
    {
      "id": "FR-029-AC-1",
      "path": "spec/functional/FR-029-return-one-disposition-per-requested-claim.md",
      "role": "examined",
      "excerpt": "A monitorable but unprojectable protocol retains its finite conformance result and an independent projection refusal."
    },
    {
      "id": "FR-029-AC-2",
      "path": "spec/functional/FR-029-return-one-disposition-per-requested-claim.md",
      "role": "examined",
      "excerpt": "Satisfiable-but-unrealizable and circular-assumption cases cannot produce realizability or composition success."
    },
    {
      "id": "FR-029-AC-3",
      "path": "spec/functional/FR-029-return-one-disposition-per-requested-claim.md",
      "role": "examined",
      "excerpt": "Unknown or unimplemented advanced claims return unsupported with exact premises and cannot erase a supported result."
    },
    {
      "id": "FR-029-AC-4",
      "path": "spec/functional/FR-029-return-one-disposition-per-requested-claim.md",
      "role": "examined",
      "excerpt": "A selected backend failure returns failed for only that claim and leaves every other requested claim's disposition unchanged."
    },
    {
      "id": "FR-029-AC-5",
      "path": "spec/functional/FR-029-return-one-disposition-per-requested-claim.md",
      "role": "examined",
      "excerpt": "Requesting n claims returns exactly n dispositions, one per exact requested claim, with no claim answered twice and none answered by another claim's result."
    },
    {
      "id": "FR-029-AC-6",
      "path": "spec/functional/FR-029-return-one-disposition-per-requested-claim.md",
      "role": "examined",
      "excerpt": "No claim strength is promoted to another, and two claims of different strength remain distinct."
    },
    {
      "id": "FR-030-AC-1",
      "path": "spec/functional/FR-030-bind-each-disposition-to-its-exact-premises.md",
      "role": "examined",
      "excerpt": "Every claim disposition retains its exact admitted fragment, backend, assumptions, communication/visibility/environment premises and source/derived-artifact correspondence."
    },
    {
      "id": "FR-030-AC-2",
      "path": "spec/functional/FR-030-bind-each-disposition-to-its-exact-premises.md",
      "role": "examined",
      "excerpt": "Removing or substituting any one retained component refuses that disposition without changing independent results."
    },
    {
      "id": "FR-030-AC-3",
      "path": "spec/functional/FR-030-bind-each-disposition-to-its-exact-premises.md",
      "role": "examined",
      "excerpt": "A failed selected backend invokes no fallback, and no returned disposition is attributed to a backend other than the one selected for its claim."
    },
    {
      "id": "FR-030-AC-4",
      "path": "spec/functional/FR-030-bind-each-disposition-to-its-exact-premises.md",
      "role": "examined",
      "excerpt": "Each retained assumption carries its exact assumption identity and exactly one assumption state, and an assumption retained without a state compares only to absence rather than bei"
    },
    {
      "id": "FR-030-AC-5",
      "path": "spec/functional/FR-030-bind-each-disposition-to-its-exact-premises.md",
      "role": "examined",
      "excerpt": "A derived-artifact identity that disagrees with the identity derived from its exact selected tuple refuses that disposition."
    },
    {
      "id": "FR-030-AC-6",
      "path": "spec/functional/FR-030-bind-each-disposition-to-its-exact-premises.md",
      "role": "examined",
      "excerpt": "A producer-authored component is byte-equal to the bytes it arrived as; normalizing, repairing or recomputing one refuses rather than producing a repaired component."
    },
    {
      "id": "FR-031-AC-1",
      "path": "spec/functional/FR-031-canonical-result-identity-and-digest.md",
      "role": "examined",
      "excerpt": "Every emitted result carries the `FR-240` contract version and kind plus its FR-286 structural `resultKey`; a missing or incomplete key is refused by the strict reader."
    },
    {
      "id": "FR-031-AC-4",
      "path": "spec/functional/FR-031-canonical-result-identity-and-digest.md",
      "role": "examined",
      "excerpt": "The emitted `resultKey` is the complete FR-286 structure; the reader compares it exactly and does not derive a digest from it or from result bytes."
    },
    {
      "id": "FR-031-AC-5",
      "path": "spec/functional/FR-031-canonical-result-identity-and-digest.md",
      "role": "examined",
      "excerpt": "Every source, profile, model, protocol, request, binding, input, backend and producer identity plus the deterministic seed and exact toolchain and corpus identity bytes on the resu"
    },
    {
      "id": "FR-031-AC-6",
      "path": "spec/functional/FR-031-canonical-result-identity-and-digest.md",
      "role": "examined",
      "excerpt": "RFC 8785 canonicalization determines emitted result bytes only; it is not used to derive a result identity or digest."
    },
    {
      "id": "FR-031-AC-7",
      "path": "spec/functional/FR-031-canonical-result-identity-and-digest.md",
      "role": "examined",
      "excerpt": "Every opaque runtime identity carried in a string-only result field round-trips byte-exactly through `quire.protocol.runtime-reference/1`; wrong prefix, uppercase/odd/non-hex conte"
    },
    {
      "id": "FR-032-AC-1",
      "path": "spec/functional/FR-032-retain-orthogonal-result-axes.md",
      "role": "examined",
      "excerpt": "Every valid FR-061 result-combination row is accepted and every one-axis forbidden combination is refused."
    },
    {
      "id": "FR-032-AC-2",
      "path": "spec/functional/FR-032-retain-orthogonal-result-axes.md",
      "role": "examined",
      "excerpt": "Settled truth on an open subject requires exact complete decisive support whose truth is preserved by every admitted continuation."
    },
    {
      "id": "FR-032-AC-3",
      "path": "spec/functional/FR-032-retain-orthogonal-result-axes.md",
      "role": "examined",
      "excerpt": "Assumption, observation-adequacy, protocol-adequacy, execution, truth, activation, participation, decision-scope progress, decision-scope closure, surrounding-execution progress, s"
    },
    {
      "id": "FR-032-AC-4",
      "path": "spec/functional/FR-032-retain-orthogonal-result-axes.md",
      "role": "examined",
      "excerpt": "Neither adequacy domain establishes the other, and each remains separately readable after the other is mutated."
    },
    {
      "id": "FR-032-AC-5",
      "path": "spec/functional/FR-032-retain-orthogonal-result-axes.md",
      "role": "examined",
      "excerpt": "Independently omitting, duplicating, cross-wiring, changing scope on, or changing the identity of every required execution, branch, and workflow global-closure premise refuses the "
    },
    {
      "id": "FR-032-AC-6",
      "path": "spec/functional/FR-032-retain-orthogonal-result-axes.md",
      "role": "examined",
      "excerpt": "A non-global claim carries `not-required` explicitly with no global premise set, and a supplied global premise or another axis relabelled `not-required` is refused."
    },
    {
      "id": "FR-032-AC-7",
      "path": "spec/functional/FR-032-retain-orthogonal-result-axes.md",
      "role": "examined",
      "excerpt": "Each retained axis fact's exact scope, authority and boundary identities are strictly matched, and substituting one axis's identity for another's refuses the record."
    },
    {
      "id": "FR-033-AC-1",
      "path": "spec/functional/FR-033-preserve-result-lineage-integrity.md",
      "role": "examined",
      "excerpt": "Support lineage rejects missing predecessors, cycles, self-reference, and invented independence while retaining shared and derived relations."
    },
    {
      "id": "FR-033-AC-2",
      "path": "spec/functional/FR-033-preserve-result-lineage-integrity.md",
      "role": "examined",
      "excerpt": "A later contradiction emits a new linked result and cannot rewrite the earlier artifact bytes or identity."
    },
    {
      "id": "FR-033-AC-3",
      "path": "spec/functional/FR-033-preserve-result-lineage-integrity.md",
      "role": "examined",
      "excerpt": "Each retained decision-support record carries exactly one `FR-282` member plus its exact oracle and observation identities, and an absent relation is never read as `independent`."
    },
    {
      "id": "FR-033-AC-4",
      "path": "spec/functional/FR-033-preserve-result-lineage-integrity.md",
      "role": "examined",
      "excerpt": "The superseding or invalidating result strictly matches the prior-result identity, the contradicted premise identity and the corrected input identity."
    },
    {
      "id": "FR-033-AC-5",
      "path": "spec/functional/FR-033-preserve-result-lineage-integrity.md",
      "role": "examined",
      "excerpt": "A premise contradiction is not admitted as a continuation under the earlier input identity, and it makes dependent package success unavailable."
    },
    {
      "id": "FR-034-AC-1",
      "path": "spec/functional/FR-034-derive-e02-projection-identity.md",
      "role": "examined",
      "excerpt": "`TechniqueResult.resultId` has the exact FR-062/FR-249 structure and changes only when its source `resultKey` or adapter identity/version changes."
    },
    {
      "id": "FR-034-AC-2",
      "path": "spec/functional/FR-034-derive-e02-projection-identity.md",
      "role": "examined",
      "excerpt": "`nativeArtifacts` retains exactly one unchanged canonical-result URI and media type, and provenance names both producer and adapter versions."
    },
    {
      "id": "FR-034-AC-3",
      "path": "spec/functional/FR-034-derive-e02-projection-identity.md",
      "role": "examined",
      "excerpt": "The adapter compares `resultId` and its source key by exact structure and derives no identity hash."
    },
    {
      "id": "FR-034-AC-4",
      "path": "spec/functional/FR-034-derive-e02-projection-identity.md",
      "role": "examined",
      "excerpt": "Every E02 property, subject, profile, binding, dependency-closure, execution-scope, plan, task and run field equals its selected context and is never inferred or substituted from d"
    },
    {
      "id": "FR-034-AC-5",
      "path": "spec/functional/FR-034-derive-e02-projection-identity.md",
      "role": "examined",
      "excerpt": "`bindingRefs` retains the selected E02 semantic order and is not resorted by presentation text."
    },
    {
      "id": "FR-034-AC-6",
      "path": "spec/functional/FR-034-derive-e02-projection-identity.md",
      "role": "examined",
      "excerpt": "The source result key is the structural FR-286 key, and neither it nor the FR-062 projection identity is replaced by a digest."
    },
    {
      "id": "FR-034-AC-7",
      "path": "spec/functional/FR-034-derive-e02-projection-identity.md",
      "role": "examined",
      "excerpt": "A strict source-reader refusal emits `invalid-source-result` with the exact UTF-8 bytes of the selected source URI and no `TechniqueResult`."
    },
    {
      "id": "FR-035-AC-1",
      "path": "spec/functional/FR-035-map-source-states-to-e02.md",
      "role": "examined",
      "excerpt": "Every FR-062 source-state row maps to the exact E02 state and retains all simultaneous independent findings."
    },
    {
      "id": "FR-035-AC-2",
      "path": "spec/functional/FR-035-map-source-states-to-e02.md",
      "role": "examined",
      "excerpt": "Several findings implied by one source state are all emitted, regardless of the E02 compatibility summary status."
    },
    {
      "id": "FR-035-AC-3",
      "path": "spec/functional/FR-035-map-source-states-to-e02.md",
      "role": "examined",
      "excerpt": "E02 adequacy derives only from observation adequacy; protocol adequacy and sufficiency cannot cross into it."
    },
    {
      "id": "FR-035-AC-4",
      "path": "spec/functional/FR-035-map-source-states-to-e02.md",
      "role": "examined",
      "excerpt": "Protocol adequacy, completeness, the four progress/closure axes, complete-global-conformance closure, settlement basis and claim remain in the canonical artifact and never populate"
    },
    {
      "id": "FR-035-AC-5",
      "path": "spec/functional/FR-035-map-source-states-to-e02.md",
      "role": "examined",
      "excerpt": "Each `satisfied` or `violated` projection derives its oracle contributions from the exact decision-support identities and preserves the source `FR-282` relation and lineage, and no"
    },
    {
      "id": "FR-035-AC-6",
      "path": "spec/functional/FR-035-map-source-states-to-e02.md",
      "role": "examined",
      "excerpt": "Invalid source, unknown versions/features, identity mismatch, unrepresented source state, missing context, or finding loss returns the stable FR-062 refusal and emits no result."
    },
    {
      "id": "FR-035-AC-7",
      "path": "spec/functional/FR-035-map-source-states-to-e02.md",
      "role": "examined",
      "excerpt": "An unrepresented source state yields `unrepresented-source-state` and is not reclassified as `invalid-source-result`."
    },
    {
      "id": "FR-036-AC-1",
      "path": "spec/functional/FR-036-carry-complete-refusal-envelope.md",
      "role": "examined",
      "excerpt": "Every public failure path returns exactly one envelope carrying the typed domain, actual local originating stage and every available prerequisite, affected identity, an available o"
    },
    {
      "id": "FR-036-AC-2",
      "path": "spec/functional/FR-036-carry-complete-refusal-envelope.md",
      "role": "examined",
      "excerpt": "Every returned envelope records the refusal axis under which it was raised, and an envelope recording no axis is reported as an unassigned axis and satisfies no negative scenario r"
    },
    {
      "id": "FR-036-AC-3",
      "path": "spec/functional/FR-036-carry-complete-refusal-envelope.md",
      "role": "examined",
      "excerpt": "The recorded axis and the retained interior are together sufficient to evaluate FR-014-AC-1's pairwise comparison of distinct axes without reading any display text, and a corpus wh"
    },
    {
      "id": "FR-036-AC-4",
      "path": "spec/functional/FR-036-carry-complete-refusal-envelope.md",
      "role": "examined",
      "excerpt": "An envelope whose domain is absent, outside the seven declared values, or inconsistent with its closed stage-to-domain mapping is refused."
    },
    {
      "id": "FR-036-AC-5",
      "path": "spec/functional/FR-036-carry-complete-refusal-envelope.md",
      "role": "examined",
      "excerpt": "The partial-output state is explicit on every envelope and is inferred from neither an empty output nor the absence of a success value."
    },
    {
      "id": "FR-036-AC-6",
      "path": "spec/functional/FR-036-carry-complete-refusal-envelope.md",
      "role": "examined",
      "excerpt": "Every local cause belongs to `quire.protocol/refusal/1`; every retained source cause names the exact FR-009-resolved source contract, and an unknown contract is retained as `unsupp"
    },
    {
      "id": "FR-037-AC-1",
      "path": "spec/functional/FR-037-derive-no-refusal-fact-from-text.md",
      "role": "examined",
      "excerpt": "Changing display text, locale or cause rendering changes no refusal identity, no code, no equality comparison and no retry decision."
    },
    {
      "id": "FR-037-AC-2",
      "path": "spec/functional/FR-037-derive-no-refusal-fact-from-text.md",
      "role": "examined",
      "excerpt": "No code, originating stage, affected identity, location, equality comparison or retry decision is derived from human-readable text; removing every message string from an envelope c"
    },
    {
      "id": "FR-037-AC-3",
      "path": "spec/functional/FR-037-derive-no-refusal-fact-from-text.md",
      "role": "examined",
      "excerpt": "Unavailable source context remains explicitly unavailable, and neither byte zero, nor an empty path, nor a repeated display name, nor message text becomes an invented location."
    },
    {
      "id": "FR-037-AC-4",
      "path": "spec/functional/FR-037-derive-no-refusal-fact-from-text.md",
      "role": "examined",
      "excerpt": "Two distinct refusal axes whose envelopes carry one shared code and one shared typed cause fail FR-014-AC-1's pairwise check, and differing message text, locale or rendering betwee"
    },
    {
      "id": "FR-037-AC-5",
      "path": "spec/functional/FR-037-derive-no-refusal-fact-from-text.md",
      "role": "examined",
      "excerpt": "An unavailable location and an available location compare unequal, and an unavailable location compares only to absence rather than to any rendered spelling."
    },
    {
      "id": "FR-038-AC-1",
      "path": "spec/functional/FR-038-retain-unknown-source-contracts.md",
      "role": "examined",
      "excerpt": "A source refusal code is matched only after the exact FR-009-resolved source contract is selected, and interpreting a code before that contract is checked fails the path."
    },
    {
      "id": "FR-038-AC-2",
      "path": "spec/functional/FR-038-retain-unknown-source-contracts.md",
      "role": "examined",
      "excerpt": "An unknown source contract, revision or code becomes the local `unsupported-source-contract` disposition, retains the original contract and value uninterpreted and byte-exact, and "
    },
    {
      "id": "FR-038-AC-3",
      "path": "spec/functional/FR-038-retain-unknown-source-contracts.md",
      "role": "examined",
      "excerpt": "Changing the selected source-contract identity or revision refuses the response, and erasing or replacing the local originating stage refuses the response, including when a wrapper"
    },
    {
      "id": "FR-038-AC-4",
      "path": "spec/functional/FR-038-retain-unknown-source-contracts.md",
      "role": "examined",
      "excerpt": "Two distinct refusal axes both retained under `unsupported-source-contract` remain distinguishable by their retained original code and cause; two axes collapsing onto one retained "
    },
    {
      "id": "FR-038-AC-5",
      "path": "spec/functional/FR-038-retain-unknown-source-contracts.md",
      "role": "examined",
      "excerpt": "No local nearest-known-version mapping or source-owner producer-stage/code list is authored; supported source variants are matched exhaustively from their pinned Rust type."
    },
    {
      "id": "FR-038-AC-6",
      "path": "spec/functional/FR-038-retain-unknown-source-contracts.md",
      "role": "examined",
      "excerpt": "Local Protocol causes require no external catalog; a scenario requiring an unresolved source contract is unexecutable, while known pinned source contracts and unknown-contract rete"
    },
    {
      "id": "FR-039-AC-1",
      "path": "spec/functional/FR-039-preserve-typed-causes-distinctly.md",
      "role": "examined",
      "excerpt": "The missing/conflicting-definition, wrong-anchor/missing-observation, unproved-definedness/logical-violation, identity-mismatch/incomplete-evidence, and unsupported-capability/proj"
    },
    {
      "id": "FR-039-AC-2",
      "path": "spec/functional/FR-039-preserve-typed-causes-distinctly.md",
      "role": "examined",
      "excerpt": "A response declaring independent causes or dispositions yields an envelope retaining all of them in the response's declared semantic order; dropping one, reordering two, or merging"
    },
    {
      "id": "FR-039-AC-3",
      "path": "spec/functional/FR-039-preserve-typed-causes-distinctly.md",
      "role": "examined",
      "excerpt": "Simultaneous independent invalid-input and missing-input dispositions both remain represented under a zero diagnostic-detail control, and that control cannot produce overall succes"
    },
    {
      "id": "FR-039-AC-4",
      "path": "spec/functional/FR-039-preserve-typed-causes-distinctly.md",
      "role": "examined",
      "excerpt": "Two distinct refusal axes whose retained typed-cause sets are equal fail FR-014-AC-1's pairwise check, so collapsing any pair in FR-039-AC-1 into a single cause also fails the inje"
    },
    {
      "id": "FR-039-AC-5",
      "path": "spec/functional/FR-039-preserve-typed-causes-distinctly.md",
      "role": "examined",
      "excerpt": "A resource-incomplete outcome and an unsupported semantic disposition are never reported as each other or as a `ProtocolRefusal`, and neither is reported as a completed assessment "
    },
    {
      "id": "FR-039-AC-6",
      "path": "spec/functional/FR-039-preserve-typed-causes-distinctly.md",
      "role": "examined",
      "excerpt": "No local payload field name, total order or spelling convention is authored for a typed cause; the retained order is the response's declared semantic order and this requirement est"
    },
    {
      "id": "FR-040-AC-1",
      "path": "spec/functional/FR-040-emit-no-success-beside-fatal-refusal.md",
      "role": "examined",
      "excerpt": "A fatal resolution, admission, representation, whole-assessment, result, projection or evidence refusal emits no success value, and any co-emitted declared success value fails the "
    },
    {
      "id": "FR-040-AC-2",
      "path": "spec/functional/FR-040-emit-no-success-beside-fatal-refusal.md",
      "role": "examined",
      "excerpt": "One claim-local construction refusal emits no canonical row for that claim; only the independent sibling outcomes FR-005 authorizes survive, and any other retained result fails."
    },
    {
      "id": "FR-040-AC-3",
      "path": "spec/functional/FR-040-emit-no-success-beside-fatal-refusal.md",
      "role": "examined",
      "excerpt": "A refusal and a completed assessment whose finding set is legitimately empty compare unequal, and neither is reported as the other."
    },
    {
      "id": "FR-040-AC-4",
      "path": "spec/functional/FR-040-emit-no-success-beside-fatal-refusal.md",
      "role": "examined",
      "excerpt": "An integration whose negative rows are distinguished only by the absence of a success value fails the distinctness check, because absence of success is not a refusal identity and o"
    },
    {
      "id": "FR-041-AC-1",
      "path": "spec/functional/FR-041-agree-across-batch-and-incremental-paths.md",
      "role": "examined",
      "excerpt": "Batch replay and settled incremental assessment agree, for each assessed value, on settled-or-pending and, where settled, on the settled value, the decision-scope and surrounding-e"
    },
    {
      "id": "FR-041-AC-2",
      "path": "spec/functional/FR-041-agree-across-batch-and-incremental-paths.md",
      "role": "examined",
      "excerpt": "The incremental path settles at least one value strictly before the final admitted fact arrives, on a history whose earlier prefix already contains every fact that value requires t"
    },
    {
      "id": "FR-041-AC-3",
      "path": "spec/functional/FR-041-agree-across-batch-and-incremental-paths.md",
      "role": "examined",
      "excerpt": "Changing one authored premise changes the same assessed values to the same new values on both paths, and a divergence between the paths returns a typed refusal rather than an agree"
    },
    {
      "id": "FR-041-AC-4",
      "path": "spec/functional/FR-041-agree-across-batch-and-incremental-paths.md",
      "role": "examined",
      "excerpt": "The typed refusal a path divergence returns carries its own outcome identity, distinct from that of every other refusal axis in the same integration."
    },
    {
      "id": "FR-042-AC-1",
      "path": "spec/functional/FR-042-publish-result-reader-and-contract-ir-map.md",
      "role": "examined",
      "excerpt": "Every valid canonical-result combination produces deterministic FR-286 bytes/identity/digest and strict-reads to a constructor-private view; every one-axis forbidden combination re"
    },
    {
      "id": "FR-042-AC-2",
      "path": "spec/functional/FR-042-publish-result-reader-and-contract-ir-map.md",
      "role": "examined",
      "excerpt": "Omission, duplication, reordering, unknown fields/labels/contracts, trailing/noncanonical bytes and every identity, scope, decision-premise or lineage mutation refuse without parti"
    },
    {
      "id": "FR-042-AC-3",
      "path": "spec/functional/FR-042-publish-result-reader-and-contract-ir-map.md",
      "role": "examined",
      "excerpt": "Every independent combination of the four open/closed axes and FR-295 completeness remains representable and no axis supplies another."
    },
    {
      "id": "FR-042-AC-4",
      "path": "spec/functional/FR-042-publish-result-reader-and-contract-ir-map.md",
      "role": "examined",
      "excerpt": "The exact selected mapping derives true only from completed+satisfied and false only from completed+violated; all other valid rows remain typed non-values with complete source fact"
    },
    {
      "id": "FR-042-AC-5",
      "path": "spec/functional/FR-042-publish-result-reader-and-contract-ir-map.md",
      "role": "examined",
      "excerpt": "Mapping bytes/read views are fully re-derived from the validated result; every retained progress/closure owner-reference field is available through borrowed typed accessors without"
    },
    {
      "id": "FR-042-AC-6",
      "path": "spec/functional/FR-042-publish-result-reader-and-contract-ir-map.md",
      "role": "examined",
      "excerpt": "Exact and one-over limits cover every bounded dimension and refuse before excess retention; no public constructor, callback, parser, evaluator or trust flag bypasses admission. The"
    },
    {
      "id": "FR-043-AC-1",
      "path": "spec/functional/FR-043-admit-the-checked-package-handoff.md",
      "role": "examined",
      "excerpt": "Over the three packages of **R** (QSL FR-517: event position, fixed sample, timestamped event), the handoff is admitted with every selection identity unchanged and the admitted pac"
    },
    {
      "id": "FR-043-AC-2",
      "path": "spec/functional/FR-043-admit-the-checked-package-handoff.md",
      "role": "examined",
      "excerpt": "Each `reader` case of the handoff's standard mutation corpus (`quire.checked-handoff.mutations/1`, QSL FR-518) returns the code, loci, cause and node locus its manifest row names f"
    },
    {
      "id": "FR-043-AC-3",
      "path": "spec/functional/FR-043-admit-the-checked-package-handoff.md",
      "role": "examined",
      "excerpt": "Each `selection` case is admitted by `read_checked_package` and refused by `Selection::check` with the `SelectionAxis` its manifest row names, carried unchanged, and a changed call"
    },
    {
      "id": "FR-043-AC-4",
      "path": "spec/functional/FR-043-admit-the-checked-package-handoff.md",
      "role": "examined",
      "excerpt": "No source file of the crate parses checked-package bytes or declares a refusal code or format string for the checked package: the only call that reads them is `read_checked_package"
    },
    {
      "id": "FR-043-AC-5",
      "path": "spec/functional/FR-043-admit-the-checked-package-handoff.md",
      "role": "examined",
      "excerpt": "Bytes of `quire.compiled-protocol/1` or `/2` offered at the checked-package entry point return a typed refusal; no version is detected, upgraded or repaired. Checked-package bytes "
    },
    {
      "id": "FR-043-AC-6",
      "path": "spec/functional/FR-043-admit-the-checked-package-handoff.md",
      "role": "examined",
      "excerpt": "A bare JSON number outside the safe-integer interval, a non-canonical rational and a signed-64 overflow in a package field each return the reader's refusal unchanged, and the libra"
    },
    {
      "id": "FR-043-AC-7",
      "path": "spec/functional/FR-043-admit-the-checked-package-handoff.md",
      "role": "examined",
      "excerpt": "A read whose limit, for a limit with nonzero measured usage, is one below the package's measured usage returns `CheckedPackageDispatchResult::Incomplete(CheckedPackageIncomplete)` "
    },
    {
      "id": "FR-043-AC-8",
      "path": "spec/functional/FR-043-admit-the-checked-package-handoff.md",
      "role": "examined",
      "excerpt": "An `identities.json` or `equality.json` whose format string is another, or whose row names a node the package does not hold, or whose identity-row join exhausts the caller's native"
    },
    {
      "id": "FR-043-AC-9",
      "path": "spec/functional/FR-043-admit-the-checked-package-handoff.md",
      "role": "examined",
      "excerpt": "The production match over `CheckedPackageDispatchResult` has a `CatalogUnavailable` arm that returns a distinct non-admitted result carrying the imported `CatalogReadError` unchang"
    },
    {
      "id": "FR-044-AC-1",
      "path": "spec/functional/FR-044-link-from-a-typed-read-view-of-the-checked-package.md",
      "role": "examined",
      "excerpt": "Over **R**'s protocol, the linked representation lists the roles, channels, queues, controls, compensations, required temporal clause and activation rows equal to `protocol_view`'s"
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
      "excerpt": "Over checked packages whose `protocol_view` identifies respectively a zero-progress `repeat` with maximum zero, two role nodes with the same authority, and two channel nodes with m"
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
      "excerpt": "The signature of the link entry names `CheckedPackageV2` and `CheckedNodeId`, returns `LinkedProtocol` or `LinkRefusal`, and names no `wire::Package`, `Handle` or `wire::Declaratio"
    },
    {
      "id": "FR-044-AC-6",
      "path": "spec/functional/FR-044-link-from-a-typed-read-view-of-the-checked-package.md",
      "role": "examined",
      "excerpt": "The linker reads no window-binding node from `CheckedPackageV2` and imports no window view from QSL. A window value is accepted only at the evaluation input owned by FR-516; absenc"
    },
    {
      "id": "FR-044-AC-7",
      "path": "spec/functional/FR-044-link-from-a-typed-read-view-of-the-checked-package.md",
      "role": "examined",
      "excerpt": "The criteria FR-002-AC-1 to FR-002-AC-5 hold over recipe **R**'s checked package: concurrent instances stay distinct; every permitted interleaving preserves the owner-supplied caus"
    },
    {
      "id": "FR-045-AC-1",
      "path": "spec/functional/FR-045-assess-compensation-through-the-checked-package-evaluation.md",
      "role": "examined",
      "excerpt": "For every outcome-map row that an admitted compensation in **R** can reach today, the test produces the successor outcome: true and false `trigger.guard` and `retry.relation`, an u"
    },
    {
      "id": "FR-045-AC-2",
      "path": "spec/functional/FR-045-assess-compensation-through-the-checked-package-evaluation.md",
      "role": "examined",
      "excerpt": "The behaviors of FR-020-AC-1 to AC-5, FR-021, FR-022, FR-023 and FR-024-AC-1 to AC-6 asserted by `tc_003` hold with the same assertions, the report-based ones restated on the outco"
    },
    {
      "id": "FR-045-AC-3",
      "path": "spec/functional/FR-045-assess-compensation-through-the-checked-package-evaluation.md",
      "role": "examined",
      "excerpt": "A recovery supply with an incomplete population, a missing target state, a missing achieved state or an unavailable binder never yields a true recovery, and no engine function conv"
    },
    {
      "id": "FR-045-AC-4",
      "path": "spec/functional/FR-045-assess-compensation-through-the-checked-package-evaluation.md",
      "role": "examined",
      "excerpt": "A retry bound of `n` read from `attempts maximum` admits exactly `n` attempts, counting the initial attempt, and `n + 1` is the retry-bound refusal."
    },
    {
      "id": "FR-045-AC-5",
      "path": "spec/functional/FR-045-assess-compensation-through-the-checked-package-evaluation.md",
      "role": "examined",
      "excerpt": "The compensation assessment performs no I/O: execute it under a denied-syscall harness and inspect the compensation module's dependency tree for I/O crates; the assessment complete"
    },
    {
      "id": "FR-045-AC-6",
      "path": "spec/functional/FR-045-assess-compensation-through-the-checked-package-evaluation.md",
      "role": "examined",
      "excerpt": "Compensation assessment consumes QSL's package-only checked-slot result as the typed slot outcome through the public evaluation boundary."
    },
    {
      "id": "FR-045-AC-7",
      "path": "spec/functional/FR-045-assess-compensation-through-the-checked-package-evaluation.md",
      "role": "examined",
      "excerpt": "Once AC-8's per-binder charge-log gate is met, each compensation assessment reports attribution as supported; it returns no `unsupported` attribution result and substitutes no zero"
    },
    {
      "id": "FR-045-AC-8",
      "path": "spec/functional/FR-045-assess-compensation-through-the-checked-package-evaluation.md",
      "role": "examined",
      "excerpt": "Once QSL's per-binder charge log with its accounting version exists, as QSL FR-520-AC-11 states it and where that criterion puts it, the assessment consumes it: each binder read ha"
    },
    {
      "id": "FR-046-AC-1",
      "path": "spec/functional/FR-046-consume-temporal-evidence-through-the-evidence-types.md",
      "role": "examined",
      "excerpt": "Over **R**'s four temporal clauses, the caller supplies an `EvidenceRequest` built from observation evidence; the assessment passes it to QSL and returns the `EvidenceResult`'s act"
    },
    {
      "id": "FR-046-AC-2",
      "path": "spec/functional/FR-046-consume-temporal-evidence-through-the-evidence-types.md",
      "role": "examined",
      "excerpt": "`EvidenceRequest` and `EvidenceResult` cross the QSL call as in-memory values; the consumer neither serializes them as request/result documents nor assigns them document contract i"
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
      "excerpt": "For the six call-refusal causes reachable from a caller-built request over an admitted package in **R**\u2014`trigger-required`, `trigger-unknown`, `occurrence-unknown`, `activation-map"
    },
    {
      "id": "FR-046-AC-5",
      "path": "spec/functional/FR-046-consume-temporal-evidence-through-the-evidence-types.md",
      "role": "examined",
      "excerpt": "The bytes reader refuses duplicate temporal-clause name rows as `AmbiguousName`; because such rows cannot occur in an admitted package, no admitted **R** package reaches a `Package"
    },
    {
      "id": "FR-046-AC-6",
      "path": "spec/functional/FR-046-consume-temporal-evidence-through-the-evidence-types.md",
      "role": "examined",
      "excerpt": "The IT-002 scenarios defined against recipe **R** pass on the evidence route, and an unavailable or withdrawn anchor settles as a missing non-value and never as a Boolean. The call"
    },
    {
      "id": "FR-046-AC-7",
      "path": "spec/functional/FR-046-consume-temporal-evidence-through-the-evidence-types.md",
      "role": "examined",
      "excerpt": "On the consumed QSL revision, the library builds against the retained `temporal::` surface and passes the `Position.instance_valuations` values that the evaluation reads. The integ"
    },
    {
      "id": "FR-047-AC-1",
      "path": "spec/functional/FR-047-end-the-retired-compiler-surfaces-and-the-integrity-lanes.md",
      "role": "examined",
      "excerpt": "After the repoint, no source file or manifest names `quire_spec_language::number`; before it, the import exists and a build with it removed fails."
    },
    {
      "id": "FR-047-AC-2",
      "path": "spec/functional/FR-047-end-the-retired-compiler-surfaces-and-the-integrity-lanes.md",
      "role": "examined",
      "excerpt": "The crate's `Cargo.toml` has no `handoff-writer` feature, and no production source or public API names `quire.compiled-protocol/1`, `quire.compiled-protocol/2` or `quire.protocol.v"
    },
    {
      "id": "FR-047-AC-3",
      "path": "spec/functional/FR-047-end-the-retired-compiler-surfaces-and-the-integrity-lanes.md",
      "role": "examined",
      "excerpt": "The IT-001 and IT-002 tests run against **R** through the checked-package route and every scenario of the retired recipe has a counterpart row or a stated reason it ends."
    },
    {
      "id": "FR-047-AC-4",
      "path": "spec/functional/FR-047-end-the-retired-compiler-surfaces-and-the-integrity-lanes.md",
      "role": "examined",
      "excerpt": "No checksum or seal lane, no `ArtifactRef.digest` or `revision` and no integrity digest of an admitted checked package exists in the crate, and an admitted package is identified by"
    },
    {
      "id": "FR-301-AC-1",
      "path": "spec/functional/FR-301-admit-protocol-refinement-dispatch.md",
      "role": "examined",
      "excerpt": "A refinement disposition never carries the complete-global-conformance premise set as evidence for its own success, exactly as `ix://agent-ix/quire-specification/FR-290`-AC-3 requi"
    },
    {
      "id": "FR-301-AC-2",
      "path": "spec/functional/FR-301-admit-protocol-refinement-dispatch.md",
      "role": "examined",
      "excerpt": "A monitorable protocol whose refinement backend is absent or unaccepted retains its independent global conformance result and returns an explicit `unsupported` refinement dispositi"
    },
    {
      "id": "FR-301-AC-3",
      "path": "spec/functional/FR-301-admit-protocol-refinement-dispatch.md",
      "role": "examined",
      "excerpt": "Once FR-176 and FR-177 are admitted, a requested refinement claim is checked by one explicit-state product construction over the FR-002 linked graph, using paired correspondence, t"
    },
    {
      "id": "FR-301-AC-4",
      "path": "spec/functional/FR-301-admit-protocol-refinement-dispatch.md",
      "role": "examined",
      "excerpt": "A refinement request without an accepted FR-292 analysis backend settles `unsupported` with a warning naming the required capability, and this settlement is never a hold, blocker o"
    },
    {
      "id": "FR-301-AC-5",
      "path": "spec/functional/FR-301-admit-protocol-refinement-dispatch.md",
      "role": "examined",
      "excerpt": "A refinement disposition retains its exact admitted fragment, backend, assumptions, communication/visibility/environment premises and source/derived-artifact correspondence exactly"
    },
    {
      "id": "FR-302-AC-1",
      "path": "spec/functional/FR-302-carry-trace-position-witness.md",
      "role": "examined",
      "excerpt": "Every `realizability` or `refinement` claim settled `decisive-witness` or `decisive-counterexample` carries exactly one trace-position witness whose deciding-element identity, valu"
    },
    {
      "id": "FR-302-AC-2",
      "path": "spec/functional/FR-302-carry-trace-position-witness.md",
      "role": "examined",
      "excerpt": "A settlement basis outside `decisive-witness`/`decisive-counterexample` carries no witness, and a decisive basis missing its witness refuses the result."
    },
    {
      "id": "FR-302-AC-3",
      "path": "spec/functional/FR-302-carry-trace-position-witness.md",
      "role": "examined",
      "excerpt": "Each carried component is copied byte-exactly from the admitted record and an absent component is carried as absence; normalizing, recomputing, reordering, substituting a default f"
    },
    {
      "id": "FR-302-AC-4",
      "path": "spec/functional/FR-302-carry-trace-position-witness.md",
      "role": "examined",
      "excerpt": "A settlement whose analysis backend cannot produce a `native-run-result/2` record settles `unsupported` with a warning naming the required capability."
    },
    {
      "id": "FR-302-AC-5",
      "path": "spec/functional/FR-302-carry-trace-position-witness.md",
      "role": "examined",
      "excerpt": "The witness-bearing result carries contract version `quire.protocol.result/v1-draft.2`, selected explicitly at production and fixed in the artifact, and a consumer requiring the wi"
    },
    {
      "id": "NFR-001-AC-1",
      "path": "spec/non-functional/NFR-001-reproduce-semantic-results.md",
      "role": "examined",
      "excerpt": "Every permitted map-order, locale, allocation-layout and causally independent arrival-order permutation that preserves all admitted identities produces byte-identical canonical JSO"
    },
    {
      "id": "NFR-001-AC-2",
      "path": "spec/non-functional/NFR-001-reproduce-semantic-results.md",
      "role": "examined",
      "excerpt": "Independently changing each named source, accepted source-baseline/profile, model, compiled-protocol, request, binding, input, observation, decision-scope progress/closure, surroun"
    },
    {
      "id": "NFR-001-AC-3",
      "path": "spec/non-functional/NFR-001-reproduce-semantic-results.md",
      "role": "examined",
      "excerpt": "Unordered semantic sets use their declared canonical order, while authored sequences and causal order remain unchanged and an identity-changing source presentation is never treated"
    },
    {
      "id": "NFR-002-AC-1",
      "path": "spec/non-functional/NFR-002-enforce-resource-bounds.md",
      "role": "examined",
      "excerpt": "Every authored maximum and caller-supplied static limit admits its exact inclusive boundary and returns a typed resource-incomplete result before starting the smallest representabl"
    },
    {
      "id": "NFR-002-AC-2",
      "path": "spec/non-functional/NFR-002-enforce-resource-bounds.md",
      "role": "examined",
      "excerpt": "Every declared indivisible unit checks the monotonic deadline before and after the unit, starts no later work after expiry, and reports the observed elapsed time and any indivisibl"
    },
    {
      "id": "NFR-002-AC-3",
      "path": "spec/non-functional/NFR-002-enforce-resource-bounds.md",
      "role": "examined",
      "excerpt": "Every caller-controlled allocation, retention, queue, iteration and traversal source site has exactly one complete inventory entry and discriminating boundary case, and every delet"
    },
    {
      "id": "NFR-002-AC-4",
      "path": "spec/non-functional/NFR-002-enforce-resource-bounds.md",
      "role": "examined",
      "excerpt": "An action-capable double records zero business-action and compensation calls during every assessment boundary case."
    },
    {
      "id": "NFR-003-AC-1",
      "path": "spec/non-functional/NFR-003-preserve-rust-assurance-boundary.md",
      "role": "examined",
      "excerpt": "Build and qualification workflows have only `workflow_dispatch`. No hosted run is used as evidence for the local qualification gate; the existing CLA administration workflow keeps "
    },
    {
      "id": "NFR-003-AC-2",
      "path": "spec/non-functional/NFR-003-preserve-rust-assurance-boundary.md",
      "role": "examined",
      "excerpt": "An unsafe Rust block is rejected unless a later reviewed requirement and owner disposition explicitly authorize that exact block."
    },
    {
      "id": "NFR-003-AC-3",
      "path": "spec/non-functional/NFR-003-preserve-rust-assurance-boundary.md",
      "role": "examined",
      "excerpt": "Every first-party production, qualification, generator, validator, canonicalizer, adapter, fixture-audit, test-assertion and command path is Rust."
    },
    {
      "id": "NFR-003-AC-4",
      "path": "spec/non-functional/NFR-003-preserve-rust-assurance-boundary.md",
      "role": "examined",
      "excerpt": "The dependency and source inventory contains no local producer runner, stdout verdict parser, evidence store, method catalogue, sufficiency or human-disposition implementation; acc"
    },
    {
      "id": "NFR-003-AC-5",
      "path": "spec/non-functional/NFR-003-preserve-rust-assurance-boundary.md",
      "role": "examined",
      "excerpt": "Every automated acceptance test carries a bare compiler-checked `ix_trace_rs::trace` binding to each obligation it claims, and removing or misspelling one binding fails compilation"
    },
    {
      "id": "NFR-003-AC-6",
      "path": "spec/non-functional/NFR-003-preserve-rust-assurance-boundary.md",
      "role": "examined",
      "excerpt": "Existing Quoin execution remains external behind its accepted structured interface, no Quoin implementation is copied or added as a runtime dependency, and no new JavaScript or Typ"
    },
    {
      "id": "NFR-004-AC-1",
      "path": "spec/non-functional/NFR-004-define-local-verification-gate.md",
      "role": "examined",
      "excerpt": "The local gate is exactly four commands: `cargo fmt --all -- --check`, `cargo clippy --locked --all-targets --target-dir target-codex-backends -- -D warnings`, `cargo test --locked"
    },
    {
      "id": "NFR-004-AC-2",
      "path": "spec/non-functional/NFR-004-define-local-verification-gate.md",
      "role": "examined",
      "excerpt": "Each enumerated command is recorded with the risk it detects: formatting drift for `fmt`; lint violations and, because `--all-targets` covers tests, fixtures and examples as well a"
    },
    {
      "id": "NFR-004-AC-3",
      "path": "spec/non-functional/NFR-004-define-local-verification-gate.md",
      "role": "examined",
      "excerpt": "The gate runs serially with `CARGO_BUILD_JOBS=1` and under `flock /tmp/quire-heavy-check.lock`, so that concurrent sessions on the shared workstation cannot interleave heavy Cargo "
    },
    {
      "id": "NFR-004-AC-4",
      "path": "spec/non-functional/NFR-004-define-local-verification-gate.md",
      "role": "examined",
      "excerpt": "Warnings are denied on two independent channels \u2014 the crate's `[lints.rust]` and `[lints.clippy]` tables and the `-D warnings` command-line argument \u2014 and removing either channel a"
    },
    {
      "id": "NFR-004-AC-5",
      "path": "spec/non-functional/NFR-004-define-local-verification-gate.md",
      "role": "examined",
      "excerpt": "While `Cargo.toml` declares no `[features]` table, no `optional = true` dependency and no tracked `cfg(feature = ...)` condition, the gate requires exactly one Cargo-feature lane, "
    },
    {
      "id": "NFR-004-AC-6",
      "path": "spec/non-functional/NFR-004-define-local-verification-gate.md",
      "role": "examined",
      "excerpt": "A `--features <name>` argument naming a feature this crate does not declare is a gate definition defect, not a lane to be satisfied; the gate refuses such an instruction rather tha"
    },
    {
      "id": "NFR-004-AC-7",
      "path": "spec/non-functional/NFR-004-define-local-verification-gate.md",
      "role": "examined",
      "excerpt": "Introducing a `[features]` entry, an `optional = true` dependency or a tracked `cfg(feature = ...)` condition obligates opposing Cargo-feature lanes that build, lint and test each "
    },
    {
      "id": "NFR-004-AC-8",
      "path": "spec/non-functional/NFR-004-define-local-verification-gate.md",
      "role": "examined",
      "excerpt": "Every tracked conditional-compilation site names how the gate selects it: `cfg(test)` is selected by `clippy --all-targets` and `test`; a target-platform predicate is selected by a"
    },
    {
      "id": "NFR-004-AC-9",
      "path": "spec/non-functional/NFR-004-define-local-verification-gate.md",
      "role": "examined",
      "excerpt": "Every compiling gate command uses the running worktree's literal `target-codex-backends` directory, and a result served through a shared `CARGO_TARGET_DIR` or another worktree's ta"
    },
    {
      "id": "NFR-004-AC-10",
      "path": "spec/non-functional/NFR-004-define-local-verification-gate.md",
      "role": "examined",
      "excerpt": "Every gate result is bound to one unchanged commit and clean tracked worktree state, and a result aggregated from separate worktrees, a moving head or different tracked bytes is no"
    },
    {
      "id": "NFR-005-AC-1",
      "path": "spec/non-functional/NFR-005-retain-composed-acceptance-evidence.md",
      "role": "examined",
      "excerpt": "Every owed IT-001, IT-002 and IT-003 scenario carries its complete resolution state, or the deterministic partial state and first causing entry when resolution does not complete; a"
    },
    {
      "id": "NFR-005-AC-2",
      "path": "spec/non-functional/NFR-005-retain-composed-acceptance-evidence.md",
      "role": "examined",
      "excerpt": "Every consumed producer value names its exact owner, contract version and immutable revision, and substituting any component refuses evidence composition."
    },
    {
      "id": "NFR-005-AC-3",
      "path": "spec/non-functional/NFR-005-retain-composed-acceptance-evidence.md",
      "role": "examined",
      "excerpt": "Every scenario carries its exact typed operation/claim outcome and distinct refusal axes within one integration have distinct retained interiors; a shared catch-all outcome fails t"
    },
    {
      "id": "NFR-005-AC-4",
      "path": "spec/non-functional/NFR-005-retain-composed-acceptance-evidence.md",
      "role": "examined",
      "excerpt": "Every declared resource dimension carries its granted and consumed values under the correct accountant owner/version; missing, duplicated, cross-wired or merged dimensions refuse c"
    },
    {
      "id": "NFR-005-AC-5",
      "path": "spec/non-functional/NFR-005-retain-composed-acceptance-evidence.md",
      "role": "examined",
      "excerpt": "Every scenario carries a compiler-checked acceptance-criterion trace binding, and a runtime string without that binding does not satisfy it."
    },
    {
      "id": "NFR-005-AC-6",
      "path": "spec/non-functional/NFR-005-retain-composed-acceptance-evidence.md",
      "role": "examined",
      "excerpt": "The composed value is passed to the selected Quoin intake and is never stored, read back, indexed, scored or disposed by Protocol; before that intake is accepted, the handoff retur"
    },
    {
      "id": "NFR-006-AC-1",
      "path": "spec/non-functional/NFR-006-fix-the-consumed-dependency-set.md",
      "role": "examined",
      "excerpt": "Every declared dependency and dev-dependency resolves through an exact `=` version requirement or an exact 40-hex git `rev`; introducing a caret or range requirement, a `branch`, a"
    },
    {
      "id": "NFR-006-AC-2",
      "path": "spec/non-functional/NFR-006-fix-the-consumed-dependency-set.md",
      "role": "examined",
      "excerpt": "Every lockfile package other than this crate carries either a `checksum` or a git `rev` byte-equal to the revision its manifest entry pins; deleting one `checksum` line, or changin"
    },
    {
      "id": "NFR-006-AC-3",
      "path": "spec/non-functional/NFR-006-fix-the-consumed-dependency-set.md",
      "role": "examined",
      "excerpt": "The build-execution enumeration and the lockfile agree in both directions; adding a closure package that runs a build script or is a procedural macro without enumerating it, and le"
    },
    {
      "id": "NFR-006-AC-4",
      "path": "spec/non-functional/NFR-006-fix-the-consumed-dependency-set.md",
      "role": "examined",
      "excerpt": "Every git-sourced dependency's revision in the rights record equals the revision the resolved lockfile carries, and no git source in the lockfile is absent from the record; a diver"
    },
    {
      "id": "NFR-006-AC-5",
      "path": "spec/non-functional/NFR-006-fix-the-consumed-dependency-set.md",
      "role": "examined",
      "excerpt": "A verification run whose lockfile is not byte-identical before and after it is not accepted as evidence of the closure; introducing a dependency change that the run would resolve r"
    },
    {
      "id": "NFR-006-AC-6",
      "path": "spec/non-functional/NFR-006-fix-the-consumed-dependency-set.md",
      "role": "examined",
      "excerpt": "A byte sequence in a pinned delivery root that no `pinned` entry's recorded digest covers does not reach a consumer path, and adding one unenumerated file to an otherwise complete "
    },
    {
      "id": "NFR-006-AC-7",
      "path": "spec/non-functional/NFR-006-fix-the-consumed-dependency-set.md",
      "role": "examined",
      "excerpt": "Swapping one delivery member's bytes for different bytes at the same path returns a typed refusal, and the substituted bytes reach no consumer path; the refusal is distinct from th"
    },
    {
      "id": "NFR-006-AC-8",
      "path": "spec/non-functional/NFR-006-fix-the-consumed-dependency-set.md",
      "role": "examined",
      "excerpt": "A delivery index supplies, selects or repairs no version-lock entry and widens no accepted byte set; editing an index to name a file the manifest does not cover changes no acceptan"
    },
    {
      "id": "NFR-007-AC-1",
      "path": "spec/non-functional/NFR-007-hold-each-contract-version-to-its-own-path.md",
      "role": "examined",
      "excerpt": "Each published contract version is reachable through only its own version-explicit integration path; the sole version-neutral root item is the frozen `/1`-only compatibility facade"
    },
    {
      "id": "NFR-007-AC-2",
      "path": "spec/non-functional/NFR-007-hold-each-contract-version-to-its-own-path.md",
      "role": "examined",
      "excerpt": "No path selects a contract version from the offered bytes; introducing a dispatcher that reads a media type, schema label or version field out of the offer and chooses a reader fro"
    },
    {
      "id": "NFR-007-AC-3",
      "path": "spec/non-functional/NFR-007-hold-each-contract-version-to-its-own-path.md",
      "role": "examined",
      "excerpt": "Bytes of one published contract version submitted at another version's entry point return a typed refusal before a linked package is exposed, in both ordered directions, with no up"
    },
    {
      "id": "NFR-007-AC-4",
      "path": "spec/non-functional/NFR-007-hold-each-contract-version-to-its-own-path.md",
      "role": "examined",
      "excerpt": "The two cross-version refusals carry typed outcome identities distinct from each other and from every other refusal axis of the integration; two directions returning one shared cod"
    },
    {
      "id": "NFR-007-AC-5",
      "path": "spec/non-functional/NFR-007-hold-each-contract-version-to-its-own-path.md",
      "role": "examined",
      "excerpt": "Re-running the earlier version's owed scenarios on the same fixture at the revision that adds a later version's path yields the same admitted identities and the same typed refusal "
    },
    {
      "id": "NFR-007-AC-6",
      "path": "spec/non-functional/NFR-007-hold-each-contract-version-to-its-own-path.md",
      "role": "examined",
      "excerpt": "The earlier path's public signature and refusal set are unchanged across that revision pair; adding a parameter, widening or narrowing a type, adding a refusal variant, removing on"
    },
    {
      "id": "NFR-007-AC-7",
      "path": "spec/non-functional/NFR-007-hold-each-contract-version-to-its-own-path.md",
      "role": "examined",
      "excerpt": "Every inherited selection axis the earlier path refuses, the later path also refuses; a later path that admits an inherited-axis mutation the earlier path refuses fails the criteri"
    },
    {
      "id": "NFR-007-AC-8",
      "path": "spec/non-functional/NFR-007-hold-each-contract-version-to-its-own-path.md",
      "role": "examined",
      "excerpt": "An owed cross-version or regression scenario that cannot run because its contract version is `unresolved` under [FR-009](../functional/FR-009-lock-external-producer-revisions.md) i"
    },
    {
      "id": "NFR-008-AC-1",
      "path": "spec/non-functional/NFR-008-carry-every-refusal-in-its-return-value.md",
      "role": "examined",
      "excerpt": "Every owed refusal axis is determined by a total function over public typed accessors on the returned value, taking no input but that value; an axis whose determination needs a fil"
    },
    {
      "id": "NFR-008-AC-2",
      "path": "spec/non-functional/NFR-008-carry-every-refusal-in-its-return-value.md",
      "role": "examined",
      "excerpt": "Replacing every `Display`, `Debug` and serialization impl reachable from a returned non-success value with a constant changes no axis determination; an axis that stops being determ"
    },
    {
      "id": "NFR-008-AC-3",
      "path": "spec/non-functional/NFR-008-carry-every-refusal-in-its-return-value.md",
      "role": "examined",
      "excerpt": "No refusal path writes to an output stream, a file, a process global, a thread-local or a logging facade; introducing one such write is refused by the static site count, and the ne"
    },
    {
      "id": "NFR-008-AC-4",
      "path": "spec/non-functional/NFR-008-carry-every-refusal-in-its-return-value.md",
      "role": "examined",
      "excerpt": "No refusal path panics, aborts, unwraps or expects in place of returning a value; replacing one refusal return with a panic is refused and names that site. The three occurrences in"
    },
    {
      "id": "NFR-008-AC-5",
      "path": "spec/non-functional/NFR-008-carry-every-refusal-in-its-return-value.md",
      "role": "examined",
      "excerpt": "Every envelope component any acceptance criterion relies on is reachable through a public typed accessor on the returned value; making one relied-on accessor private, or moving one"
    },
    {
      "id": "NFR-008-AC-6",
      "path": "spec/non-functional/NFR-008-carry-every-refusal-in-its-return-value.md",
      "role": "examined",
      "excerpt": "An absent location, stage, prerequisite or cause chain compares unequal to a present component of that kind whose content is empty, and is never defaulted to it; a fixture pair dif"
    },
    {
      "id": "NFR-008-AC-7",
      "path": "spec/non-functional/NFR-008-carry-every-refusal-in-its-return-value.md",
      "role": "examined",
      "excerpt": "Two runs on identical inputs return non-success values that compare equal component by component, with the observed elapsed time of a resource-incomplete outcome excluded; a value "
    },
    {
      "id": "NFR-008-AC-8",
      "path": "spec/non-functional/NFR-008-carry-every-refusal-in-its-return-value.md",
      "role": "examined",
      "excerpt": "An owed source-refusal axis that cannot be observed because its selected source contract is `unresolved` under [FR-009](../functional/FR-009-lock-external-producer-revisions.md) is"
    }
  ],
  "findings": [],
  "bindings": [],
  "repository_baseline_status": "fail",
  "pr_delta_status": "pass",
  "baseline_untagged_criteria": 200
}
```
