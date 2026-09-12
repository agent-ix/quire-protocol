---
id: SR-046
title: "Integrity review of the IT-002/IT-003 version-lock and composed-evidence packet"
type: SpecReview
analysis: integrity
scope: "spec/functional/FR-009-lock-external-producer-revisions.md; spec/non-functional/NFR-005-retain-composed-acceptance-evidence.md; spec/integration/IT-002-consume-observation-temporal-facts.md SC-05..SC-08 and owner boundaries; spec/integration/IT-003-publish-assurance-and-e02-results.md SC-06..SC-09 and owner boundaries; spec/assurance/AD-001-protocol-library-boundaries.md; spec/tests.md TM-001; read against FR-003, FR-004, FR-006, FR-007, FR-008, NFR-002, NFR-003, NFR-004"
review_set: subset
relationships:
  - { target: ix://agent-ix/quire-protocol/FR-009, type: reviews }
  - { target: ix://agent-ix/quire-protocol/NFR-005, type: reviews }
  - { target: ix://agent-ix/quire-protocol/IT-002, type: reviews }
  - { target: ix://agent-ix/quire-protocol/IT-003, type: reviews }
  - { target: ix://agent-ix/quire-protocol/AD-001, type: reviews }
  - { target: ix://agent-ix/quire-protocol/TM-001, type: reviews }
  - { target: ix://agent-ix/quire-protocol/FR-008, type: reviews }
  - { target: ix://agent-ix/quire-protocol/NFR-002, type: reviews }
---

## Summary

Completeness, consistency and atomicity analysis of the newly authored
version-lock and composed-evidence packet: FR-009, NFR-005, the IT-002
SC-05..SC-08 and IT-003 SC-06..SC-09 scenarios, both new owner-boundaries
tables, AD-001's qualification-evidence intake section, and the TM-001 rows that
carry them. The packet's structure is sound — one manifest owns pinning, one NFR
owns per-scenario evidence content, retention itself stays external, the budget
probes correctly hold the input constant and vary the budget in the same shape
IT-001 version 2 established, and neither integration pins anything provisional. The
defects are at the joins: NFR-005's retention rule disqualifies exactly the
FR-009 refusal scenarios that prove the lock works; PT02 is an owner in IT-002's
table but appears in no producer enumeration FR-009 locks; the budget scenarios
call their outcome a refusal where NFR-002 calls it resource-incomplete, in the
same packet that newly makes that distinction load-bearing; and FR-009 supplies
no status term for the deliberately unresolved A version 2 entry its own Dependencies
carve out.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-001 | high | NFR-005 makes the FR-009 refusal scenarios permanently uncreditable: IT-002-SC-08 and IT-003-SC-09 refuse *before* the manifest resolves, so they retain no "resolved manifest snapshot", and NFR-005-AC-5 then requires them to report incomplete and leave their matrix row unexecuted — the only scenarios that prove the lock works are the ones the evidence rule disqualifies. | NFR-005-AC-1, NFR-005-AC-5, IT-002-SC-08, IT-003-SC-09, FR-009-AC-2, FR-009-AC-3 | wrong-requirement |
| FND-002 | high | PT02 supplies the complete-global-conformance closure record and its premises in IT-002's owner-boundaries table, but FR-009's Inputs enumerate the external producers to be locked (A, E, F, Engineering Assurance, E02, Quoin) and omit PT02, and AD-001 names no PT02 producer — so an externally owned value IT-002 consumes has no manifest entry while IT-002's Preconditions assert every external producer resolves through one. | IT-002 Owner boundaries, IT-002 Preconditions, FR-009 Inputs, FR-009 Behavior, AD-001 Views | missing-requirement |
| FND-003 | high | The budget scenarios specify the one-short outcome as "that dimension's typed resource refusal", while NFR-002's Scope and AC-1 specify a typed **resource-incomplete** result and FR-008-AC-5 keeps resource-incomplete structurally distinct from other outcomes — in the same packet where IT-003-SC-06 newly requires an incomplete outcome and a refusal never to be reported as each other. The same probe is specified as both. | IT-002-SC-06, IT-003-SC-07, IT-003-SC-06, NFR-002 Scope, NFR-002-AC-1, FR-008-AC-5 | wrong-requirement |
| FND-004 | high | NFR-005-AC-3's "typed outcome identity" is undefined and collides with FR-008: FR-008 binds refusal identity to a stable source-owned code plus affected identity, and FR-009-AC-2/AC-3 deliberately give several distinct probes one code differing only by the named entry. Read as code-only the criterion fails a correct library; read as code-plus-identity it is satisfied by construction. Two valid interpretations, opposite verdicts. | NFR-005-AC-3, FR-008-AC-1, FR-008-AC-5, FR-009-AC-2, FR-009-AC-3 | wrong-requirement |
| FND-005 | high | FR-009 supplies no status vocabulary for the deliberately unresolved A version 2 entry: Behavior says the library SHALL resolve *every* external producer before reading any producer value, AC-2 makes "unpinned" and "provisional" refusal triggers, and Dependencies then exempt an entry that is "explicitly unresolved" — a term that appears in no clause and is nowhere distinguished from "provisional". Whether the manifest admits or refuses with that entry present is unresolvable from the requirement. | FR-009 Behavior, FR-009-AC-2, FR-009 Dependencies, IT-002 Preconditions, IT-003 Preconditions | missing-requirement |
| FND-006 | medium | Both new artifacts claim scope over IT-001 that IT-001 neither declares nor can satisfy: FR-009's Description covers any value produced outside this repository (A's artifact included) and NFR-005's Scope names "every scenario of IT-001", yet IT-001 declares neither relationship, states no manifest precondition, has no retention scenario, and TM-001 lists both requirements' evidence as IT-002/IT-003 only. | FR-009 Description, NFR-005 Scope, IT-001 relationships, IT-001 Preconditions, TM-001 FR-009 row, TM-001 NFR-005 row | correct-requirement-no-evidence |
| FND-007 | medium | IT-003-SC-09 drops the sidecar probe that IT-002-SC-08 carries ("removing a producer-emitted sidecar changes no resolution outcome"), yet FR-009-AC-4 cites Test (IT-002, IT-003) and AD-001 states the sidecar rule as a boundary invariant — so the self-authorizing-offer rule is unverified on the Engineering Assurance/E02/Quoin side. | FR-009-AC-4, IT-003-SC-09, IT-002-SC-08, AD-001 Qualification-evidence intake | correct-requirement-no-evidence |
| FND-008 | medium | The two owner-boundaries tables disagree with each other and with AD-001: IT-002's omits Engineering Assurance and Quoin; IT-003's omits A even though its own Preconditions turn on version 1 versus version 2 provenance and its Expected Results demand attribution to one pinned producer revision; and IT-003 drops IT-002's closing owner-retention rule ("a value this repository derives is never attributed to a producer") that NFR-005-AC-2 verifies. | IT-002 Owner boundaries, IT-003 Owner boundaries, IT-003 Preconditions, IT-003 Expected Results, NFR-005-AC-2, AD-001 System Boundary | wrong-requirement |
| FND-009 | medium | FR-009 mandates manifest resolution and digest recomputation over every resolved artifact *before* any producer value is read, but NFR-002's Scope enumerates the bounded phases (strict reading, linking, compensation assessment, finite replay, result construction, support-lineage validation, E02 projection) and lists none of them. No clause says which budget covers digest recomputation or what a budget exhausted mid-verification returns, so "a budget exactly sufficient for one unchanged history/request" is undefined as to whether resolution cost is inside it. | FR-009 Behavior, NFR-002 Scope, NFR-002-AC-1, IT-002-SC-06, IT-003-SC-07 | missing-requirement |
| FND-010 | medium | IT-002-SC-05 is silently compound: one scenario id carries batch/incremental outcome agreement, settled-truth agreement, complete-global-conformance closure agreement, equal-time permutation invariance, authored-premise co-variance, and a disagreement-refuses rule — six independently falsifiable obligations against FR-004-AC-4, which owns only the correspondence. A single failing clause cannot be localized from the scenario id. | IT-002-SC-05, FR-004-AC-4, FR-004 Behavior, FR-006-AC-3 | wrong-requirement |
| FND-011 | medium | IT-002-SC-07 and IT-003-SC-08 fold a cross-scenario invariant into one scenario's evidence inspection: "no two refusing scenarios retain the same typed outcome identity" ranges over the whole integration and cannot be observed from the evidence of the scenario asserting it, so the clause is untestable at the granularity it is written. | IT-002-SC-07, IT-003-SC-08, NFR-005-AC-3, NFR-005 Verification | wrong-requirement |
| FND-012 | medium | TM-001 credits IT-003 with NFR-004 coverage in both the Integration Coverage row and NFR-004's evidence cell, but IT-003 declares no NFR-004 relationship, exercises the gate in no scenario, and omits NFR-004 from its Traceability section. The matrix claims integration evidence the integration does not offer. | TM-001 Integration Coverage IT-003, TM-001 NFR-004 row, IT-003 relationships, IT-003 Traceability, NFR-004-AC-1 | correct-requirement-no-evidence |
| FND-013 | low | NFR-005-AC-5 is compound and half-unobservable: "reports incomplete rather than passing" is library behaviour, "its matrix row stays unexecuted" is a TM-001 authoring obligation, and it is the one criterion verified by "Inspection" where its four siblings say Test (IT-002, IT-003) — so the two halves cannot be discharged by the same evidence. | NFR-005-AC-5, NFR-005 Measurement and Evaluation, TM-001 NFR-005 row | wrong-requirement |
| FND-014 | low | TM-001's Test Execution Summary still counts 3 integrations while Integration Coverage now lists four rows (IT-001 version 1, IT-001 version 2, IT-002, IT-003), so the planned totals cannot be reconciled with the table directly above them. | TM-001 Integration Coverage, TM-001 Test Execution Summary | wrong-requirement |

## Completeness Analysis

**US -> FR/StR.** FR-009 declares `implements US-002` and depends on FR-001 and
FR-008, matching the convention every other FR in this repository follows
(US-001/US-002 both descend from StR-001). NFR-005 declares `constrains` edges to
FR-009, FR-006 and FR-008. Both new artifacts are therefore reachable from
StR-001, and no orphan requirement was introduced.

**FR -> verification.** Every FR-009 and NFR-005 criterion names a verification
method, but two of those named paths do not exist: FR-009-AC-4's sidecar
obligation has no IT-003 clause (FND-007), and NFR-005-AC-5's matrix half has no
executor (FND-013). FR-009 is additionally the only FR in TM-001 with no TC
companion; the matrix discloses this, but it means every FR-009 criterion is
gated behind the same external producer acceptance both integrations wait on, so
FR-009 cannot reach any evidence independently of IT-002/IT-003 landing.

**NFR scoping.** NFR-002 is explicitly scoped and now referenced by IT-002-SC-06
and IT-003-SC-07, closing the gap where the two downstream integrations asserted
no budget behaviour. NFR-005 is explicitly scoped but over-scoped to IT-001
(FND-006). NFR-003's IT-003 edge is preserved and unchanged. NFR-004's IT-003
edge is asserted by the matrix and by nothing else (FND-012).

**Producer enumeration.** FR-009's Inputs are the closed list of what the lock
covers, and IT-002's owner table is the closed list of who supplies IT-002's
values. They differ by exactly one owner, PT02 (FND-002). Because FR-006 treats
the complete-global-conformance closure record as an input it must not synthesise
and IT-002 forbids substituting a scoped closure for it, the missing entry is
load-bearing rather than cosmetic.

**Hidden-assumption probes.** FR-009 performs a lookup over multiple producer
sources; a tie-breaking policy is not needed because the manifest is keyed by
producer and version, and the "separate entry per consumed version" rule plus
FR-009-AC-5's independence clause state the decision explicitly. FR-009 depends
on producers not yet published; the interim position is documented in its
Dependencies and in both Preconditions, but the *status term* for that interim
position is missing (FND-005). FR-009 delegates no external CLI, performs no
paginated enumeration, and invokes no authenticated API, so those probes do not
apply. The unbudgeted pre-read work phase is the one probe that fails outright
(FND-009).

## Consistency & Conflict Analysis

**FR-009 vs NFR-005.** They do not duplicate: FR-009 owns whether a producer may
be read at all, NFR-005 owns what an executed scenario must carry afterwards.
The division is stated in both directions (FR-009 supplies the snapshot; NFR-005
names FR-009 upstream) and AD-001 restates it as "indexing ... is inside the
boundary; producing, canonicalizing, storing, or assessing those artifacts is
outside it". The conflict is temporal rather than territorial: NFR-005-AC-1
presumes a resolved manifest for every executed scenario, and FR-009 exists
precisely to produce scenarios where no manifest resolves (FND-001).

**NFR-005 vs FR-008.** FR-008 owns the refusal envelope and the distinctness it
guarantees is between *cause classes*, not between scenarios. NFR-005-AC-3
imposes a stronger, differently-shaped uniqueness on the scenario set without
defining the identity it ranges over (FND-004). This is the packet's only
genuine contradiction between two requirements, as opposed to a gap.

**NFR-005 vs NFR-002.** No conflict in substance — NFR-005 consumes the budget
figures NFR-002 enforces, and correctly requires both granted and consumed so a
carried-over remainder is detectable. The conflict is lexical and is FND-003.

**FR-009 vs NFR-002.** No contradiction; a scope hole (FND-009).

**AD-001.** The new Qualification-evidence intake section is consistent with both
new requirements and states the self-authorizing-offer rule in the same terms
FR-009's Behavior uses. Its Views paragraph correctly places manifest resolution
before any producer value enters a boundary. Its producer enumeration is the one
that omits PT02 (FND-002); its risk list is unchanged and still accurate.

## Atomicity & Testability Analysis

Singular and testable as written: IT-002-SC-06, IT-003-SC-07 (both are one
three-step budget probe over one dimension at a time, with a single pass/fail
shape), IT-003-SC-06 (one distinction, four consequences of the same
distinction), FR-009-AC-1 through AC-3, NFR-005-AC-1, NFR-005-AC-2,
NFR-005-AC-4.

Compound: IT-002-SC-05 (FND-010), IT-002-SC-07 and IT-003-SC-08 (retention
inspection plus a global uniqueness property plus an incompleteness rule —
FND-011), IT-002-SC-08 and IT-003-SC-09 (four adverse manifest states plus, in
IT-002, the sidecar probe plus the independence probe; these are enumerable
one-axis mutations of one obligation and are more defensible than SC-05, but the
independence clause "repinning one entry changes the resolution of no other" is a
distinct obligation from "an adverse entry refuses"), FR-009-AC-5 (independence
plus multi-version resolution), NFR-005-AC-5 (FND-013).

Not externally observable as written: the matrix half of NFR-005-AC-5, and the
cross-scenario half of NFR-005-AC-3 / SC-07 / SC-08.

## Owner-boundary reconciliation

| Owner | AD-001 | IT-002 table | IT-003 table | FR-009 Inputs |
| --- | --- | --- | --- | --- |
| A (compiled protocol) | yes | yes | absent (FND-008) | yes |
| E (temporal) | yes | yes | n/a | yes |
| F (observation) | yes | yes | n/a | yes |
| PT02 | absent (FND-002) | yes | n/a | absent (FND-002) |
| Engineering Assurance | yes | absent | yes | yes |
| quire-verification (E02) | yes | n/a | yes | yes |
| Quoin | yes | absent | yes | yes |
| This repository | yes | yes | yes | n/a |

The two tables agree on the rows they share and on the shape of the "must not"
column; they disagree on membership, and only IT-002 states the value-level
owner-retention rule NFR-005-AC-2 verifies (FND-008).

## Terminology consistency with IT-001 version 2

The manifest language is consistent: "pinned", "immutable revision", "digest",
"provisional", "explicitly unresolved" and "an offer that supplies its own
acceptance context authorizes itself" appear identically in IT-001's Dependencies,
FR-009, AD-001 and both new Preconditions, and neither IT-002 nor IT-003 pins an
intermediate commit.

The budget-probe language is consistent in structure and correct in direction:
IT-002-SC-06 and IT-003-SC-07 hold the input constant and vary the budget through
exactly-sufficient / one-short / fresh-at-the-exact-value, matching IT-001-SC-09
clause for clause, including the "no budget is carried across a refused attempt"
conclusion. The one drift is the outcome noun (FND-003), which IT-002 and IT-003
inherited from IT-001-SC-09 and which only becomes a contradiction here, because
IT-003-SC-06 newly makes incomplete-versus-refusal a specified distinction.

The unresolved version 2 entry is handled consistently across all three integrations:
each states the entry is implemented but unpublished, each forbids substituting
one contract version for the other, and IT-003 additionally scopes SC-01..SC-09
to version 1-derived canonical results. No new scenario claims version 2 coverage.

## Recommendations

1. Carve manifest-refusal scenarios out of NFR-005-AC-1, or require the
   *attempted* manifest snapshot rather than the resolved one, so FR-009's
   refusal probes can be credited (FND-001).
2. Add PT02 to FR-009's Inputs and to AD-001's producer enumeration, or state in
   IT-002 why the closure record is exempt from the lock (FND-002).
3. Replace "typed resource refusal" with NFR-002's "typed resource-incomplete
   result" in IT-002-SC-06 and IT-003-SC-07 (FND-003).
4. Define "typed outcome identity" in NFR-005 by reference to FR-008's code plus
   affected identity, and restate AC-3 as a property over the scenario set with
   its own inspection step rather than a clause inside SC-07/SC-08 (FND-004,
   FND-011).
5. Give FR-009 an explicit entry-status enum that distinguishes *provisional*
   (refuses) from *declared-unresolved with no dependent scenario* (admits), and
   bind the A version 2 entry to the latter (FND-005).
6. Split IT-002-SC-05 into correspondence, permutation-invariance,
   premise-co-variance and disagreement-refuses clauses (FND-010).
7. Reconcile the two owner tables against AD-001 and repeat IT-002's
   owner-retention sentence in IT-003 (FND-007, FND-008).
8. Either add an NFR-004 scenario to IT-003 or drop the NFR-004 claim from
   TM-001's IT-003 rows, and correct the integration count (FND-012, FND-014).
