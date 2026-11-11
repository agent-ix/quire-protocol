---
id: SR-2127021
title: "Scope-boundary analysis of TL-11 FR-050..FR-059 against ADR-010"
type: SpecReview
analysis: scope-boundary
scope: "agent-ix/quire-protocol; spec/functional/FR-050..FR-059, spec/test-cases/TC-050..TC-055, spec/tests.md (PR #143, head 00836cf)"
review_set: subset
date: 2026-11-11
---

## Summary

Ticket: TL-11. Checked each FR against ADR-010's TL-11 row and Decisions 1-3. FR-050..FR-052 are linking, FR-053..FR-056 assessment of one observed history, FR-057/FR-058 carrying, FR-059 G5 carrying; none specifies exploration semantics, loop proof or a QSL-owned table, and G3, G4, G5 and G7 are each handled. QSL causes in FR-057 are described, not renamed. Two boundary defects: an unsupported disposition the library raises itself has no defined cause name, and FR-058 asserts requires-bound bound kinds QSL does not report.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-001 | medium | FR-054, FR-055 and FR-059 return 'the disposition of FR-057' during assessment of an observed history (symbolic capacity, predicate join, unruled join, undischarged loop proof), where QSL reports no `Unsupported` cause. FR-057's output 'carries the QSL cause name unchanged', so there is no name to carry, and the implementer must mint one, which FR-057 forbids ('define no second name'). State where the cause comes from in those paths: the QSL variant read for the construct (e.g. `SymbolicCapacity`, `JoinPredicate`, `UnruledOutstanding`) or QSL's existing `unsupported-requested-capability` for the FR-059 discharge request. | FR-057; FR-054-AC-5; FR-055-AC-5; FR-059-AC-2; spec/functional/FR-057-map-qsl-unsupported-outcomes-to-typed-dispositions.md:20 | wrong-requirement |
| FND-002 | medium | QSL's requires-bound outcome (`NotSimulated::RequiresBound { domains: UnboundedDomains }`, qsl-eval/src/simulation/not_simulated.rs) is keyed by domain and typed by `DomainKind` (`Collection`, `Population`, `Integer`, `Recursive`, `Loop`, temporal), read from the local QSL checkout of 2026-11-04. It has no queue-capacity kind: a non-literal capacity is `Unsupported::SymbolicCapacity` (FR-057), not requires-bound. FR-058's 'kind of bound missing (role population, queue capacity, loop maximum ...)' and AC-1's 'a channel without a capacity' invent a bound kind and imply the library maps domains to checked node ids itself. Carry QSL's `DomainKind` unchanged and drop the channel case, or cite the QSL source that reports it. | FR-058-AC-1; FR-058; FR-051; spec/functional/FR-058-map-requires-bound-outcomes-to-typed-dispositions.md:43 | wrong-requirement |

## Verdict

Findings above must be fixed or dispositioned before merge. Units examined:

- `FR-050` (examined)
- `FR-051` (examined)
- `FR-052` (examined)
- `FR-053` (examined)
- `FR-054` (examined)
- `FR-055` (examined)
- `FR-056` (examined)
- `FR-057` (examined)
- `FR-058` (examined)
- `FR-059` (examined)
- `ADR-010` (context_only)

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | Round 1, commit 5ef24e3: - When the assessment of an observed history meets a construct for which QSL would report an `Unsupported` cause, QSL having reported none for that history, the library shall return the same disposition carrying the name of that QSL `Unsupported` variant (`SymbolicCapacity`, `JoinPredicate`, `Unrule |
| FND-002 | fixed | Round 1, commit 5ef24e3: Given a requires-bound outcome naming a `Population`, a `Collection` and a `Loop` domain, each disposition is typed non-success and carries QSL's outcome unchanged, with each domain's kind and key equal to QSL's and no kind the library defined. |
