---
id: SR-2127040
title: "TL-11 gap-analysis review"
type: SpecReview
analysis: gap-analysis
date: 2026-11-11
scope: "agent-ix/quire-protocol; TL-11; src/declarations.rs; src/dynamics.rs; src/disposition.rs; src/link.rs; tests/tc_050..tc_055; Cargo.toml"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-protocol/ADR-010
    type: references
---

## Summary

Ticket: TL-11; PR: quire-protocol#147; reviewed head b9c6e01b4112066a4db8f01c02f62c3d640e0a51; scope `git diff origin/main...HEAD`. Planless gap-analysis. quire matrix (quire 0.38.3) over the head: 45 of 46 FR-050..059 criteria tagged, FR-051-AC-3 untagged. Reverse gap: HistoryRefusal has no owning AC. Hollow evidence: three tests cannot fail, or pass over histories the owner semantics forbid.

## Verdict

**FAIL** (one untagged criterion, one high finding). The untagged and unbackable criteria are a spec defect, not missing tests: amend FR-050/FR-051/FR-053 in this PR. Semantic review was not run separately; intent checks on each bound test were made by reading the tests against their ACs.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | FR-051-AC-3 is untagged; it, FR-050-AC-2's absent half and FR-053-AC-4 are unbackable because QSL's owner types make max and overflow mandatory; amend the spec in this PR instead of a test-header note | spec/functional/FR-051-link-queue-declarations.md:69 |
| FND-002 | medium | FR-053-AC-4 is backed by spawning nine instances of the static role S, a history the owner semantics forbid | tests/tc_051_assess_role_lifetime_and_population.rs:134-145 |
| FND-003 | medium | FR-056-AC-3 test cannot fail: the closure states are built and discarded | tests/tc_054_assess_loop_bound_and_undischarged_proof.rs:92-102 |
| FND-004 | medium | FR-053-AC-5 permutes only slice order over an assessment that sorts by fixed caller positions; no permitted reordering of positions is tested | tests/tc_051_assess_role_lifetime_and_population.rs:147-172 |
| FND-005 | low | FR-054-AC-7 tail half is untested: the fixture has no unordered loss-tail channel | tests/tc_052_assess_queue_capacity_and_overflow.rs:281-304 |
| FND-006 | low | Undeclared-branch refusal test is tagged FR-055-AC-3, which is about quorum counting; HistoryRefusal has no owning requirement | tests/tc_053_assess_join_settlement.rs:246-258 |
| FND-007 | low | FR-050 asks for per-role activation and spawn/retire controls the owner types do not have; FR-053..056 Inputs do not name the Protocol-owned fact types | spec/functional/FR-050-link-role-declarations-without-minting-identity.md:6 |

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
- ADR-010; role: context_only; spec/adr/ADR-010-consume-qsl-protocol-analysis-own-assessment-and-dispatch.md; excerpt: The library owns exactly four things for these capabilities: linking, assessment, dispatch, carrying.

## Coverage

Matrix: quire matrix --scope <worktree> --format json; FR-050..059: 45 tagged, 1 untagged (FR-051-AC-3). Focused tests: tc_050..tc_055, 46 passed. Plan completion: not assessed.

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | Round 1, ecee04f: \| FR-051-AC-3 \| Over channels authoring `block`, `reject` and `loss` with each of `head`, `tail` and `new`, each linked channel's overflow policy is the authored one, and no channel is linked without one. \| Test (TC-050) \| (FR-050-AC-2 now: replicated roles of different `max` and a static role; FR-053-AC-4 now over a static role; ChannelDeclaration::overflow: ChannelOverflow) |
| FND-002 | fixed | Round 1, ecee04f: let act = \|position, who: &str\| fact(position, who, RoleEvent::Act, &dynamic, "S"); let assessment = assess(&dynamic, &[act(1, "s1"), act(2, "s1"), act(3, "s2")], "S"); ... Err(HistoryRefusal::StaticRoleLifecycle { role: dynamic.named("role", "S") }) |
| FND-003 | fixed | Round 1, ecee04f: let LoopAssessment { control: _, workflow: _, repeat_ordinals: _, iterations, maximum, within_maximum, exited, violation } = assessment.clone(); // Destructured with no rest pattern: a closure, progress or any other axis would have to be a new field |
| FND-004 | fixed | Round 1, ecee04f: for order in [[2, 1, 3, 4, 5, 6], [1, 2, 3, 6, 5, 4], [1, 3, 2, 4, 6, 5]] { ... } // A reordering that is not permitted does change the value let broken = assess(&dynamic, &at_positions([4, 2, 3, 1, 5, 6]), "R"); assert_eq!(broken.lifetime_violations.len(), 1); |
| FND-005 | fixed | Round 1, ecee04f: for dynamic in [Dynamic::with("workflow", 3), Dynamic::with_unordered_tail_loss()] { ... UnsupportedCause::Construct(Unsupported::UnorderedLossSelector) } |
| FND-006 | fixed | Round 1, ecee04f: #[trace("TC-053", "FR-055-AC-7")] \| FR-055-AC-7 \| A fact naming a branch label the join does not list refuses the history with the join and label named. \| Test (TC-053) \| |
| FND-007 | fixed | Round 1, ecee04f: FR-050: retain each authored role declaration (static, or replicated with its population, its `max` bound and its lifetime kind) as QSL's occurrence schema reads it ... the library carries no separate spawn or retire control and no per-role activation rule. FR-053..056 Inputs name RoleFact, ChannelFact, JoinFact, LoopFact. |
