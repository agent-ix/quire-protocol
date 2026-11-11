---
id: SR-2127039
title: "TL-11 code-review review"
type: SpecReview
analysis: code-review
date: 2026-11-11
scope: "agent-ix/quire-protocol; TL-11; src/declarations.rs; src/dynamics.rs; src/disposition.rs; src/link.rs; tests/tc_050..tc_055; Cargo.toml"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-protocol/ADR-010
    type: references
---

## Summary

Ticket: TL-11; PR: quire-protocol#147; reviewed head b9c6e01b4112066a4db8f01c02f62c3d640e0a51; scope `git diff origin/main...HEAD`. Code-review with the Rust lane (rust-style loaded) over the FR-050..059 declarations, dynamics and disposition modules. The declaration reader reads channel delivery and capacity through QSL's accessors, keeps symbolic capacity unresolved, carries predicate joins and loop proofs by node, and mints no instance identity. Two defects are high: static-role activity is always a lifetime violation, and role replication is re-parsed although QSL publishes an owner reader.

## Verdict

**FAIL (changes required)** on FND-001 and FND-002. No vendored QSL code, no compatibility shim, no new pin or SHA (qsl-semantics moved to a normal dependency on branch main, no rev), trace tags are ix-trace-rs #[trace] with no /// Trace: lines, and commit 1 (366bc79) not compiling alone is acceptable because the repo squash-merges. Accepted as scoped, not defects: library-defined RoleFact/ChannelFact/JoinFact/LoopFact (ADR-010 Decision 2 makes assessment Protocol-owned, quire-observation publishes no protocol-history type, and compensation's Fact types are the precedent); clause-level activation (the owner row is clause-level); FR-054 reordering covered by the key-order check (delivery premises in QSL say nothing about order); no resource accountant or PureAssessment token (no FR-05x requires one); per-assessment violation types instead of one FR-027 enum (FR-027-AC-5 asks only that kinds stay distinct). DuplicateRoleAuthority refusing two roles over one object type is pre-existing FR-044-AC-3 behavior, not introduced here.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | Static-role activity is always reported BeforeSpawn: a static role is never spawned, so every Act by it is a false lifetime violation | src/dynamics.rs:186-192 |
| FND-002 | high | Role lifetime, population and max are re-parsed locally and Lifetime mirrors QSL's RoleLifetime, although QSL publishes them via protocol_occurrence_schemas_from_read (RoleReplication) | src/declarations.rs:23-47 |
| FND-003 | medium | Join and loop occurrences are keyed by control and workflow only, so a nested repeat or a join inside a repeat merges occurrences across enclosing iterations | src/dynamics.rs:746 |
| FND-004 | medium | Loss overflow is judged on a caller-asserted selector label; the dropped message is never checked against the queue head, tail or new message | src/dynamics.rs:555-571 |
| FND-005 | medium | UnsupportedCause keeps a local table of QSL variant names and a local 'unsupported-requested-capability' string instead of QSL's CatalogCode; from_name is unexhaustive and has no production caller | src/disposition.rs:47-126 |
| FND-006 | low | QslBoundary/settle_claims seam lacks a refusal arm, an optional construct and a warning slot that TL-9/TL-10 dispatch will need; no production caller yet | src/disposition.rs:177-249 |
| FND-007 | low | unreachable! after the symbolic-capacity check and a debug_assert-only guard in request_discharge; use let-else / a typed check | src/dynamics.rs:501-503 |
| FND-008 | low | assess_joins doc says an unsupported join yields one outcome; it yields one per workflow occurrence | src/dynamics.rs:733-734 |

## Scope

- FR-050-AC-1; role: examined; spec/functional/FR-050-link-role-declarations-without-minting-identity.md; excerpt: Over a checked package declaring one role per lifetime kind (`workflow`, `scope`, `until(p)`), the linked representation lists each role with the checked node id and the lifetime kind that node carries.
- FR-050-AC-2; role: examined; spec/functional/FR-050-link-role-declarations-without-minting-identity.md; excerpt: Over a replicated role with a `max` bound and one with none, the linked representation carries the bound exactly and carries absence as absence, with no default.
- FR-050-AC-3; role: examined; spec/functional/FR-050-link-role-declarations-without-minting-identity.md; excerpt: Linking the same checked package twice yields equal representations, and the representation holds no role instance identifier.
- FR-050-AC-4; role: examined; spec/functional/FR-050-link-role-declarations-without-minting-identity.md; excerpt: A replicated role is linked as one declaration; the representation holds no per-instance expansion of it.
- FR-051-AC-1; role: examined; spec/functional/FR-051-link-queue-declarations.md; excerpt: Over channels with a literal capacity, a symbolic capacity, each overflow policy and each ordering, the linked representation carries each field equal to the value the checked node and QSL's accessors report.
- FR-051-AC-2; role: examined; spec/functional/FR-051-link-queue-declarations.md; excerpt: A symbolic capacity is carried as its authored expression, and no linked field holds a number derived from it.
- FR-051-AC-3; role: examined; spec/functional/FR-051-link-queue-declarations.md; excerpt: A channel with no authored overflow policy is linked with no overflow policy and no default.
- FR-052-AC-1; role: examined; spec/functional/FR-052-link-join-and-loop-declarations.md; excerpt: Over joins of each policy and a quorum threshold, the linked representation carries the control identity, policy and threshold equal to the checked package's.
- FR-052-AC-2; role: examined; spec/functional/FR-052-link-join-and-loop-declarations.md; excerpt: A `predicate` join and a join with no `outstanding` rule link successfully and are carried as declared, without evaluation and without a default rule.
- FR-052-AC-3; role: examined; spec/functional/FR-052-link-join-and-loop-declarations.md; excerpt: A loop with an invariant and a lexicographic variant links with both declarations referenced by checked node id, and linking performs no check of either.
- FR-053-AC-1; role: examined; spec/functional/FR-053-assess-role-lifetime-and-population.md; excerpt: Over a history whose instances all act inside their lifetimes and whose live count never exceeds the bound, the role assessment reports no violation.
- FR-053-AC-2; role: examined; spec/functional/FR-053-assess-role-lifetime-and-population.md; excerpt: Over histories with activity before spawn, activity after retire, and a retire that precedes its spawn, the assessment reports a lifetime violation per case naming the instance and observation.
- FR-053-AC-3; role: examined; spec/functional/FR-053-assess-role-lifetime-and-population.md; excerpt: Over a history reaching exactly the bound and one reaching the bound plus one live instance, the assessment reports no violation for the first and a population violation naming role, bound and count for the second.
- FR-053-AC-4; role: examined; spec/functional/FR-053-assess-role-lifetime-and-population.md; excerpt: Over a role with no authored population bound, the assessment reports no population violation and reports the bound as absent; it does not infer a bound from the history.
- FR-053-AC-5; role: examined; spec/functional/FR-053-assess-role-lifetime-and-population.md; excerpt: Permuting the presentation order of the history within the permitted orders changes no role assessment value.
- FR-053-AC-6; role: examined; spec/functional/FR-053-assess-role-lifetime-and-population.md; excerpt: Over one role per lifetime kind, activity after the observation that ends the workflow, after the observed end of a `scope` lifetime and after the observed `retire` of an `until(p)` instance is reported as a lifetime violation for `workflow`, `scope` and `until(p)` respectively, and activity before those observations is not.
- FR-053-AC-7; role: examined; spec/functional/FR-053-assess-role-lifetime-and-population.md; excerpt: Over a `scope` role that owns a compensation template, a compensation attempt observed while a registration of that template is open, or while a running branch can still reach its effect, is not reported as a lifetime violation.
- FR-053-AC-8; role: examined; spec/functional/FR-053-assess-role-lifetime-and-population.md; excerpt: Over an `until(p)` role and a history in which `p` is observed to hold and the instance acts before its observed `retire`, the assessment reports no lifetime violation and does not evaluate `p`.
- FR-054-AC-1; role: examined; spec/functional/FR-054-assess-queue-capacity-and-overflow-outcome.md; excerpt: Over a literal-capacity channel whose history stays within capacity, the assessment reports no capacity violation and the observed maximum occupancy.
- FR-054-AC-2; role: examined; spec/functional/FR-054-assess-queue-capacity-and-overflow-outcome.md; excerpt: Over histories whose occupancy exceeds capacity with and without an authored overflow event, the assessment reports a capacity violation only for the one without.
- FR-054-AC-3; role: examined; spec/functional/FR-054-assess-queue-capacity-and-overflow-outcome.md; excerpt: Over one history per authored policy (`block`, `reject`, `loss head`, `loss tail` and `loss new` on `fifo` channels, `loss new` on an `unordered` channel) whose observed outcome matches, and one whose outcome differs, the assessment reports no violation for each match and an overflow violation naming authored policy and observed outcome for the mismatch.
- FR-054-AC-4; role: examined; spec/functional/FR-054-assess-queue-capacity-and-overflow-outcome.md; excerpt: Over a `fifo` channel keyed by a field, a history delivering two messages of one key out of send order yields an order violation, and a history interleaving different keys does not.
- FR-054-AC-5; role: examined; spec/functional/FR-054-assess-queue-capacity-and-overflow-outcome.md; excerpt: Over a channel with a symbolic capacity, the assessment returns the FR-057 disposition and reports neither a capacity violation nor its absence.
- FR-054-AC-6; role: examined; spec/functional/FR-054-assess-queue-capacity-and-overflow-outcome.md; excerpt: Over a channel whose delivery premise forbids loss and a history with a lost message, and over one whose premise permits it, the assessment reports a premise violation naming channel and premise only for the first.
- FR-054-AC-7; role: examined; spec/functional/FR-054-assess-queue-capacity-and-overflow-outcome.md; excerpt: Over an `unordered` channel, any delivery order of the same messages reports no order violation; over an `unordered` channel whose overflow loses `head` or `tail`, the assessment returns the FR-057 disposition for that channel.
- FR-055-AC-1; role: examined; spec/functional/FR-055-assess-join-settlement-per-policy.md; excerpt: Over an `all` join, a history settling with every branch complete reports no violation, and a history settling with one branch missing reports a join violation.
- FR-055-AC-2; role: examined; spec/functional/FR-055-assess-join-settlement-per-policy.md; excerpt: Over an `any` join, settlement after the first completed branch reports no violation, and settlement with none completed reports a join violation.
- FR-055-AC-3; role: examined; spec/functional/FR-055-assess-join-settlement-per-policy.md; excerpt: Over a `quorum(n)` join, histories with n-1, n and n+1 distinct completed branches report a violation only for n-1; a branch counted twice in the history is counted once.
- FR-055-AC-4; role: examined; spec/functional/FR-055-assess-join-settlement-per-policy.md; excerpt: Over joins authored `outstanding continue` and `outstanding cancel`, a history whose outstanding branches behave as authored reports none, and one whose branches behave otherwise reports an outstanding violation.
- FR-055-AC-5; role: examined; spec/functional/FR-055-assess-join-settlement-per-policy.md; excerpt: Over a `predicate` join and over an `any` and a `quorum(n)` join with no `outstanding` rule, the assessment returns the FR-057 disposition and reports neither a settled nor an unsettled join.
- FR-055-AC-6; role: examined; spec/functional/FR-055-assess-join-settlement-per-policy.md; excerpt: Over an `all` join with no `outstanding` rule, a history settling with every branch complete reports no violation and no outstanding branch, and a history settling with a branch missing reports a join violation.
- FR-056-AC-1; role: examined; spec/functional/FR-056-assess-loop-iteration-bound.md; excerpt: Over a loop with maximum m, histories with m-1, m and m+1 iterations report a loop-bound violation only for m+1.
- FR-056-AC-2; role: examined; spec/functional/FR-056-assess-loop-iteration-bound.md; excerpt: A history that ends inside a loop reports the loop as not exited and reports no violation by that fact alone.
- FR-056-AC-3; role: examined; spec/functional/FR-056-assess-loop-iteration-bound.md; excerpt: The loop assessment of a history is unchanged by the closure axis value assessed for the same history.
- FR-056-AC-4; role: examined; spec/functional/FR-056-assess-loop-iteration-bound.md; excerpt: Over a loop with no authored maximum and a history of any iteration count, the assessment reports the observed count, reports the maximum as absent and reports no loop-bound violation.
- FR-057-AC-1; role: examined; spec/functional/FR-057-map-qsl-unsupported-outcomes-to-typed-dispositions.md; excerpt: Given each QSL cause ADR-010 lists, the disposition is `unsupported`, carries that cause name unchanged and the construct's checked node id, and is bound to the request's exact premises.
- FR-057-AC-2; role: examined; spec/functional/FR-057-map-qsl-unsupported-outcomes-to-typed-dispositions.md; excerpt: Given an `unsupported` cause on one claim of a request of n claims, the other n-1 dispositions equal those returned without it, and the unsupported claim is neither success nor failure.
- FR-057-AC-3; role: examined; spec/functional/FR-057-map-qsl-unsupported-outcomes-to-typed-dispositions.md; excerpt: Given a symbolic-capacity channel and a `predicate` join, no disposition or assessment field holds a number or truth value derived from evaluating the capacity or predicate.
- FR-057-AC-4; role: examined; spec/functional/FR-057-map-qsl-unsupported-outcomes-to-typed-dispositions.md; excerpt: Given a QSL cause name the library does not recognise, the disposition retains it unchanged as an unknown source cause and is not mapped to a known cause.
- FR-058-AC-1; role: examined; spec/functional/FR-058-map-requires-bound-outcomes-to-typed-dispositions.md; excerpt: Given a requires-bound outcome naming a `Population`, a `Collection` and a `Loop` domain, each disposition is typed non-success and carries QSL's outcome unchanged, with each domain's kind and key equal to QSL's and no kind the library defined.
- FR-058-AC-2; role: examined; spec/functional/FR-058-map-requires-bound-outcomes-to-typed-dispositions.md; excerpt: The library calls no QSL analysis with a bound the library derived, and no disposition field holds a bound the library chose.
- FR-058-AC-3; role: examined; spec/functional/FR-058-map-requires-bound-outcomes-to-typed-dispositions.md; excerpt: A requires-bound disposition on one claim leaves every other requested claim's disposition unchanged and changes none of the conformance, progress or closure axis values.
- FR-059-AC-1; role: examined; spec/functional/FR-059-keep-loop-invariant-and-variant-proof-outside-the-library.md; excerpt: Over a loop with an invariant and a lexicographic variant, the result carries both by checked node id with discharge status `undischarged`.
- FR-059-AC-2; role: examined; spec/functional/FR-059-keep-loop-invariant-and-variant-proof-outside-the-library.md; excerpt: A request for the discharge of that loop returns an `unsupported` disposition naming the missing capability, bound to the request's premises, and no proved status.
- FR-059-AC-3; role: examined; spec/functional/FR-059-keep-loop-invariant-and-variant-proof-outside-the-library.md; excerpt: Over a history in which the loop ends within its maximum and exits, the loop's discharge status remains `undischarged` and the loop bound assessment (FR-056) reports no proof.
- FR-059-AC-4; role: examined; spec/functional/FR-059-keep-loop-invariant-and-variant-proof-outside-the-library.md; excerpt: Over a loop with an invariant and a variant, no result or disposition field reports an invariant checked, a variant rank or a termination verdict, and the only discharge status produced is `undischarged`.
- src/declarations.rs; role: examined; src/declarations.rs; excerpt: FR-050..052 declaration reader: Lifetime, Replication, RoleDeclaration, Ordering, Capacity, ChannelDeclaration, JoinPolicy, JoinDeclaration, LoopDeclaration, Declarations::read
- src/dynamics.rs; role: examined; src/dynamics.rs; excerpt: FR-053..056 assessments: assess_roles, assess_channels, assess_joins, assess_loops over RoleFact/ChannelFact/JoinFact/LoopFact keyed by ObservationRef
- src/disposition.rs; role: examined; src/disposition.rs; excerpt: FR-057..059: UnsupportedCause, UnsupportedDisposition, RequiresBoundDisposition, QslReport, QslBoundary, settle_claims, LoopProof, request_discharge
- src/link.rs; role: examined; src/link.rs; excerpt: LinkedProtocol::declarations(); LinkRefusal::MalformedMember; pub(crate) node/argument/reference/source_locus
- Cargo.toml; role: examined; Cargo.toml; excerpt: qsl-semantics moved from dev-dependency to dependency, branch main, no rev pin
- ADR-010; role: context_only; spec/adr/ADR-010-consume-qsl-protocol-analysis-own-assessment-and-dispatch.md; excerpt: The library owns exactly four things for these capabilities: linking, assessment, dispatch, carrying.

## Coverage

Focused checks on the head: cargo clippy --locked --all-targets -D warnings clean; cargo test for tc_050..tc_055: 46 passed, 0 failed. The full aggregate gate was not run (left to the lead). Owner types read in QSL main 91c6158: qsl-package protocol_occurrence.rs (RoleReplication, RoleLifetime, RepeatDimension, protocol_occurrence_schemas_from_read), protocol_occurrence_key.rs (ProtocolOccurrenceKey), qsl-semantics protocol_slots.rs (CheckedReplicatedRole.max: u64), protocol_controls.rs (CheckedChannel.overflow non-optional), qsl-eval simulation/protocol.rs (Unsupported, construct() only), qsl-route model_request.rs:449 (CatalogCode for unsupported-requested-capability). Plan completion: not assessed.

## New findings (disposition pass 1)

Reviewed head ecee04fb29b155ea52a4c4f576882992d03f85da (`git diff b9c6e01..ecee04f`).

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-009 | low | request_discharge keeps a caller-supplied locus under the loop's node; derive the whole Construct from the package | src/disposition.rs:274-288 |
| FND-010 | medium | RoleFact has no workflow instance, so population is counted across workflow instances against a per-instance max | src/dynamics.rs:146-155, 220-250 |

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | Round 1, ecee04f: RoleEvent::Act if lifetime.is_none() => {} ... if is_static && fact.event != RoleEvent::Act {     return Err(HistoryRefusal::StaticRoleLifecycle { role: fact.role.clone() }); } |
| FND-002 | fixed | Round 1, ecee04f: let schema = protocol_occurrence_schemas_from_read(package)     .map_err(LinkRefusal::Occurrence)?     ...     .find(\|schema\| schema.workflow == clause_key) ... replication: slot.replication.clone(),   // RoleDeclaration { node, key, replication: RoleReplication } |
| FND-003 | fixed | Round 1, ecee04f: pub repeat_ordinals: Vec<Integer>,   // on JoinFact and LoopFact ... schema.bind(identity, control, &owner, repeat_ordinals) ... type Key<'a> = (usize, &'a WorkflowInstanceId, Vec<String>); |
| FND-004 | fixed | Round 1, ecee04f: let observed_matches_queue = match outcome {     OverflowOutcome::Blocked \| OverflowOutcome::Rejected => true,     OverflowOutcome::Lost { selector, dropped } => match selector {         LossSelector::Head => others.first().copied() == Some(dropped),         LossSelector::Tail => others.last().copied() == Some(dropped),         LossSelector::New => dropped == message,     }, }; |
| FND-005 | fixed | Round 1, ecee04f: RequestedCapability(CatalogCode), ... pub const fn requested_capability() -> Self {     Self::RequestedCapability(CatalogCode::new(         "unsupported_projection",         "unsupported-requested-capability",     )) } |
| FND-006 | deferred | Seam shape is a forward concern, not a defect for FR-057/058; the QslBoundary doc now records that TL-9 (FR-200..204) and TL-10 (FR-150..153) extend it with a refusal arm, an optional construct and a warning slot. Built by those tickets. |
| FND-007 | fixed | Round 1, ecee04f: let capacity = match steppable_capacity(channel) {     Ok(capacity) => capacity,     Err(cause) => { return ChannelOutcome::Unsupported { .. } } }; ... let construct = Construct { node: declaration.control.clone(), locus: construct.locus }; |
| FND-008 | fixed | Round 1, ecee04f: /// A join declaration with no fact yields no occurrence. A join QSL would /// report `Unsupported` for yields one [`JoinOutcome::Unsupported`] for each /// occurrence (workflow instance and enclosing repeat ordinals) the history /// names. |
| FND-009 | fixed | Round 2, df16bc9: pub fn request_discharge<P>(     package: &CheckedPackageV2,     declaration: &LoopDeclaration,     premises: P, ) -> UnsupportedDisposition<P> {     let construct = Construct::of(package, &declaration.control); |
| FND-010 | fixed | Round 2, df16bc9: /// The workflow instance the role instance belongs to. A role's `max` /// bounds the live instances of one workflow instance, and a workflow /// end ends only that workflow's instances. pub workflow: WorkflowInstanceId, ... let mut by_role: BTreeMap<(usize, &WorkflowInstanceId), Vec<&RoleFact>> = BTreeMap::new(); |

## New findings (disposition pass 2)

Reviewed head df16bc9a2231d7c2953a4efbb924a883a937761e (`git diff ecee04f..df16bc9`).

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-011 | medium | The PR deletes reviews/tl-11-spec-review-base.md (SR-2127018, merged with #143); restore it from main | reviews/tl-11-spec-review-base.md:1-85 |
