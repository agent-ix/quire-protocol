---
id: SR-044
title: "EARS conformance review of the /2 version-explicit intake scenarios and their requirement statements"
type: SpecReview
analysis: ears-conformance
scope: "IT-001 scenarios SC-06..SC-09 and the requirement statements they lean on in FR-001, FR-002 and FR-008, as they stand in the agent-b/it001-v2-intake working tree (including the FR-001-AC-5 / FR-002-AC-5 amendments)"
review_set: all
relationships:
  - { target: ix://agent-ix/quire-protocol/IT-001, type: reviews }
  - { target: ix://agent-ix/quire-protocol/FR-001, type: references }
  - { target: ix://agent-ix/quire-protocol/FR-002, type: references }
  - { target: ix://agent-ix/quire-protocol/FR-008, type: references }
  - { target: ix://agent-ix/quire-protocol/NFR-002, type: references }
---
## Summary

The deterministic grammar check reports every document in the tree at review time clean (81 of 81) with zero
`[ears:*]` findings, so every finding below is semantic. Three are high: SC-09
asks for a resource refusal inside the range NFR-002 requires to be admitted,
and SC-06's "`admit_and_link` keeps its `/1` behaviour unchanged" clause — now
mirrored by a requirement statement in FR-001 and by FR-001-AC-5 — states an
outcome that names no baseline and cannot fail in any single run. The dominant
defect across the remaining rows is non-atomicity: one `SHALL`, or one scenario
id, carrying several triggers and several responses, so no statement maps 1:1
to an acceptance outcome.

## Method and Scope Limits

Two lenses are applied and they are not interchangeable.

FR-001, FR-002 and FR-008 carry **requirement statements**, so their Description
and Behavior prose is judged against EARS grammar: canonical event/state
trigger, one named subject belonging to this system, a single `SHALL`, and a
concrete verifiable response.

IT-001 is an integration test document. Its `IT-001-SC-NN` clauses are
**acceptance statements**, not requirement statements: they record an
observation the harness makes, not an obligation the library carries. EARS
templates do not apply to them, and no finding below flags an SC clause for
lacking a `When …`/`SHALL` form. SC-06 through SC-09 are judged only on whether
each is a single, unambiguous, verifiable trigger/response pair, and on whether
an owning requirement statement exists for what they assert.

FR-001 and FR-002 were amended during this review with the version-explicit
entry-point paragraph, the temporal-selection paragraph, and the FR-001-AC-5 /
FR-002-AC-5 rows. Those amendments close what would otherwise have been a
missing-requirement finding against SC-06 and SC-08; the findings below are
against the amended text, and FND-002, FND-010, FND-011, FND-016 and FND-017
concern the new statements specifically.

This repository consumes A's compiled-protocol artifacts and owns no parser or
grammar. `protocol_artifact::v2` is unpublished (`quire-spec-language#40`), so
no finding asserts what A's boundary does; the `/2` findings concern only what
this repository's own statements say and fail to say.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-001 | high | IT-001-SC-09 requires the exact-bound input to admit and the input *one unit short* of the limit to return the typed resource refusal, inverting NFR-002-AC-1, which requires the exact inclusive boundary to admit and only the smallest representable *excess* unit to refuse; as written the scenario demands a refusal from inside the admitted range | IT-001-SC-09, NFR-002-AC-1 | wrong-requirement |
| FND-002 | high | "When a later contract version is published, the library SHALL leave every already-published version's admission behaviour unchanged" states a response with no observable output: no baseline is named, no single execution can violate it, and FR-001-AC-5 inherits the same unfalsifiability by repeating it | FR-001, FR-001-AC-5 | wrong-requirement |
| FND-003 | high | SC-06 is not a single trigger/response pair: it carries `/2` byte and reference retention, cross-version refusal at both entry points, an eleven-axis mutation sweep, and the claim that `admit_and_link` "keeps its `/1` behaviour unchanged" — that last clause names no observable and no baseline, so the scenario cannot fail for a stated reason | IT-001-SC-06 | wrong-requirement |
| FND-004 | medium | FR-001 Description folds a state guard into an event-driven response ("When a protocol artifact is submitted, the library SHALL admit it **only when** … exactly match"), so one `SHALL` carries both trigger and qualifier and the mismatch outcome is left unstated; split into an event-driven admission statement plus an `If … then the library SHALL return a typed refusal` statement | FR-001 | wrong-requirement |
| FND-005 | medium | FR-001 Behavior places a `SHALL` on a subject this repository does not own — "the accepted compiled-artifact contract SHALL declare a typed encoding whose decode is injective" — an obligation on A that is unverifiable from here; restate as a consumer obligation (the library SHALL refuse a contract that does not declare such an encoding) | FR-001 | wrong-requirement |
| FND-006 | medium | FR-001 Description matches "artifact-contract version" and "contract version" as two separate identities with no definition distinguishing them, and the amended text adds a third spelling, "compiled-artifact contract version"; the `/2` intake routes on exactly one of these, so the ambiguity now changes what gets built | FR-001, FR-001-AC-5, IT-001-SC-06 | wrong-requirement |
| FND-007 | medium | FR-002 Description gives one `SHALL` a disjunctive response ("construct the exact finite typed representation … **or** return located refusals"), which is two obligations, and its trigger "When an admitted protocol is linked" names the response as its own precondition; use an event-driven statement on the link request plus an `If … then …` statement for the invalid-binding case | FR-002 | wrong-requirement |
| FND-008 | medium | SC-07 states two independent trigger/response pairs under one id (cross-domain digest substitution, and recanonicalized producer input), and its second half — "refuses rather than admitting an equivalent form" — names no refusal identity, so both can only be observed as one pass/fail | IT-001-SC-07 | wrong-requirement |
| FND-009 | medium | FR-002's amended statement and SC-08 both require a refusal "naming the offending binding axis" while covering six distinct malformations (missing, surplus, duplicated, reordered, foreign-owner, out-of-range); neither says whether those six must be distinguishable, so one shared code satisfies both as written — the same defect SR-027 already recorded for `Invalid::Selection` | FR-002-AC-5, IT-001-SC-08 | missing-requirement |
| FND-010 | medium | FR-002's amended "the linker SHALL treat the temporal profile, clock alternative, and clock parameters as independently retained selection identities rather than as offered values" has no observable response — "treat as" names no output that could differ — so the statement cannot be tested directly; state the observable instead (a selection that differs from the offered value SHALL refuse) | FR-002 | wrong-requirement |
| FND-011 | medium | FR-002's amended temporal-refusal statement puts six triggers and three conjoined responses under one `SHALL` (refuse it, name the offending axis, emit no linked representation), and uses the event keyword `When` for what is an unwanted condition | FR-002 | wrong-requirement |
| FND-012 | low | FR-001 Behavior's rejection sentence puts eight independent conditions under one `SHALL` ("unknown artifact encodings or contracts and unknown, draft-only, unaccepted, superseded, substituted, internally inconsistent, or feature-incomplete baselines"), so no acceptance criterion maps to a single condition | FR-001 | wrong-requirement |
| FND-013 | low | FR-002 names two different subjects for one system — "the library" in the Description, "the linker" in the Behavior and in both amended paragraphs — where EARS requires one stable named subject across a statement set | FR-002 | wrong-requirement |
| FND-014 | low | FR-008 Description states an unwanted condition ("When any public operation cannot return its declared success value") with the event-driven `When …` pattern; the canonical form for an unwanted condition is `If … then the library SHALL …` | FR-008 | wrong-requirement |
| FND-015 | low | FR-008 Behavior carries two obligations as subjectless assertions with no modal — "A wrapper cannot erase or replace that stage" and "an empty diagnostic-detail budget cannot erase their existence or establish success"; both are backed by FR-008-AC-6 and FR-008-AC-8, so the defect is grammatical only and should read `The library SHALL NOT …` | FR-008 | wrong-requirement |
| FND-016 | low | FR-001's amended "Where the native compiler publishes more than one compiled-artifact contract version, …" uses `Where`, the EARS optional-feature keyword, for a state of the producer environment rather than an optional feature of this library; the state-driven keyword `While` is the canonical form | FR-001 | wrong-requirement |
| FND-017 | low | FR-001's amended "When bytes are offered at an entry point other than their own contract version's, the library SHALL return a typed refusal" states an unwanted condition with the event keyword; canonical form is `If … then the library SHALL …` | FR-001 | wrong-requirement |

## Verdict

**CONDITIONAL** — the EARS grammar layer is clean, the amendments to FR-001 and
FR-002 supply the version-explicit and temporal-selection obligations SC-06 and
SC-08 previously leaned on with no owner, and the remaining grammar defects are
restatements that do not change intended behaviour. Two items must close before
the `/2` lane can execute: FND-001, which asks the implementation for a refusal
NFR-002 forbids, and the FND-002/FND-003 pair, which puts an unfalsifiable
"unchanged" clause into both a requirement statement and its acceptance
criterion and scenario.

## Recommendations

1. Correct SC-09 to "one unit *over* the declared limit", matching
   NFR-002-AC-1's inclusive-maximum semantics, or state explicitly that the
   `/2` limits are minimums and why they differ from every other limit here.
2. Replace the "unchanged by a later version" obligation in FR-001,
   FR-001-AC-5 and SC-06 with something a run can violate — for example, that
   each version's admission outcome is determined solely by its own contract
   and entry point, verified by re-running the SC-01..SC-05 `/1` lane unmodified
   after the `/2` entry point exists.
3. Split SC-06 into separate scenarios for `/2` retention, cross-version
   refusal, and the `/2` selection-axis mutation sweep; split SC-07 into its
   digest-domain and recanonicalization halves.
4. State whether the six temporal binding malformations must be distinguishable
   from one another, in FR-002's statement and in FR-002-AC-5; "naming the
   offending binding axis" currently admits one shared code.
5. Restate FND-004 through FND-008 and FND-010 through FND-017 as atomic,
   single-subject, single-`SHALL` statements with the canonical trigger keyword;
   none of these changes alters intended behaviour.
