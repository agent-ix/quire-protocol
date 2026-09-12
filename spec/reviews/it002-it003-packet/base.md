---
id: SR-049
title: "Base review of the IT-002/IT-003 integration packet"
type: SpecReview
analysis: base
scope: "FR-009, NFR-005, IT-002 (SC-05..SC-08), IT-003 (SC-06..SC-09), AD-001, TM-001, US-002, FR-004"
review_set: subset
---
## Summary

The base checklist was applied to the packet authored for quire-protocol #12 and
#13: two new requirements, four new scenarios on each integration, an owner
boundary allocation on each, and the qualification-evidence intake section in
AD-001. Identifiers, sequencing and uniqueness are clean, and each new scenario
names its own refusal axis rather than gesturing at a class of failures. The
substantive gaps are upstream and downstream of the new requirements: FR-009
descends from no user story, neither new requirement has a test case that B could
run without an external producer, and manifest resolution is itself unbudgeted.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-001 | medium | FR-009 descends from no user story. It declares `implements US-002`, but US-002's need is that canonical results and their E02 projections retain exact source identities for an evidence consumer; it never asks that producer revisions be pinned, and US-001 is about assessing a finite protocol. The version-lock manifest answers a real need — an outcome attributable to the wrong revision — but no stakeholder statement in this repository expresses it, so the FR was derived from the ticket rather than from a story. | FR-009; US-001; US-002; StR-001 | missing-requirement |
| FND-002 | medium | Neither new requirement has a B-runnable evidence path. All five FR-009 criteria and all five NFR-005 criteria verify through `Test (IT-002, IT-003)` with no TC companion, so both requirements are hard-gated on external producer acceptance. Manifest resolution itself — parsing entries, recomputing digests, refusing an absent or provisional entry, proving one entry's change does not move another — is B-owned logic that is testable today against B's own fixtures, exactly as the `/2` evidence loader already is. This repeats the gap SR-040 recorded against FR-001-AC-5. | FR-009-AC-1..AC-5; NFR-005-AC-1..AC-5; TM-001 | correct-requirement-no-evidence |
| FND-003 | medium | Manifest resolution is unbudgeted. FR-009 requires a digest recomputation per entry before any producer value is read, and NFR-002's bounds are authored for protocol evaluation, not for resolution. Nothing states a maximum entry count, a maximum resolved-artifact size, or a deadline for resolution, so a manifest is the one input to the system whose processing cost has no declared limit. IT-002-SC-06 and IT-003-SC-07 budget the integrations but start after resolution has already happened. | FR-009; NFR-002; IT-002-SC-06; IT-003-SC-07 | missing-requirement |
| FND-004 | medium | IT-002-SC-05 is the only new scenario with an unmeasurable clause. "Changing an authored premise changes both in the same way" states no comparison: two assessments could each change and still differ, and the clause would read as satisfied. FR-004-AC-4 states the underlying obligation precisely — batch replay and settled incremental assessment are semantically identical for every permutation of the same admitted facts — and SC-05 should discharge that wording rather than a looser paraphrase of it. | IT-002-SC-05; FR-004-AC-4 | wrong-requirement |
| FND-005 | low | NFR-005 carries both a Measurement table and an Acceptance Criteria table. The skeleton reserves the AC section for a policy NFR whose compliance does not reduce to a number; every metric here is a 100% target against a 100% threshold, which is a policy check written as a metric. One of the two should carry the obligation, not both, or the two can drift. | NFR-005 | wrong-requirement |
| FND-006 | low | Identifier hygiene is clean and worth recording as checked: FR-009 and NFR-005 are the next free identifiers in their classes, their criteria are `{PARENT}-AC-N` and sequential, IT-002-SC-01..SC-08 and IT-003-SC-01..SC-09 are sequential with no gap or duplicate, and every new `verifies` edge in both integrations resolves to an artifact that exists. | FR-009; NFR-005; IT-002; IT-003 | correct-requirement-no-evidence |

## Remediation

FND-001 needs either a user story expressing the attribution need the manifest
serves, or an explicit statement in FR-009 that it is a cross-cutting constraint
derived from the EPIC rather than from a story. FND-002 needs a test case owning
the B-side manifest logic so FR-009 is not wholly external. FND-003 needs
resolution bounds in NFR-002 or in FR-009 itself. FND-004 should restate
IT-002-SC-05 in FR-004-AC-4's terms. FND-005 should drop one of NFR-005's two
tables.

None of these is blocked on an external producer; all five are B-ownable now.

## Remediation applied

All five base findings and the high findings from SR-045, SR-046 and SR-048 were
remediated in the same change rather than carried. What changed:

- **FR-009 no longer asks B to do what B may not do.** Reachability of a revision
  is now an assertion the owner records in the entry, not a check this library
  performs, and the requirement says explicitly that resolution reads no
  version-control history, spawns no process and opens no network connection
  (SR-045 FND-001, SR-048 FND-001).
- **The digest preimage is stated.** Digests are computed over the exact
  delivered bytes with no canonicalization, re-encoding or normalization, the
  verified byte sequence is the one passed to the consumer, and an entry naming
  an interface rather than an artifact carries a revision and no digest
  (SR-048 FND-002, SR-045 FND-002, FND-012).
- **Entries have a key.** `(owner, contract version)` is unique; two entries
  under one key refuse every integration before resolution, because no refusal
  can name an ambiguous entry (SR-045 FND-003).
- **The manifest no longer declares its own requiredness.** Each integration
  test declares the entries its scenarios require. A document that says which
  scenarios need it can unblock a scenario by editing itself — the same
  self-authorization FR-009 forbids producers (SR-045 FND-004).
- **`unresolved` is a real status.** A scenario requiring one is reported
  unexecutable, not passing and not failing, and cannot be satisfied by another
  contract version of the same owner (SR-046 FND-005).
- **Resolution is budgeted** on entry count, bytes digested and a deadline, and
  exceeding any dimension returns resource-incomplete (SR-049 FND-003,
  SR-046 FND-009, SR-045 FND-016 in part).
- **PT02 is a producer** in FR-009's Inputs (SR-045 FND-005, SR-046 FND-002,
  SR-048 FND-005).
- **NFR-005 no longer disqualifies FR-009's own refusal scenarios.** A scenario
  refusing during resolution retains the partial state plus the causing entry;
  it is not required to hold a snapshot that does not exist (SR-046 FND-001).
- **NFR-005's metrics are denominated on owed scenarios**, so a blocked
  integration scores zero rather than a vacuous hundred per cent (SR-045 FND-009).
- **Distinctness is per refusal axis**, not per scenario: two scenarios on one
  axis may share an outcome identity, two axes may not (SR-045 FND-008,
  SR-046 FND-004).
- **IT-002-SC-05 is split into SC-05a..SC-05d** and restated in FR-004-AC-4's
  own terms, with SC-05b requiring the incremental path to settle a value
  strictly before the final fact — a buffering implementation now fails rather
  than passing vacuously (SR-049 FND-004, SR-045 FND-007, SR-046 FND-010).
- **The budget outcome is resource-incomplete, not refusal**, in both
  integrations, in the same packet that makes the distinction load-bearing
  (SR-046 FND-003), and IT-003-SC-06 now separates all three senses of
  "incomplete" (SR-045 FND-010).
- **IT-002's Objective calls the closure record PT02-owned**, matching its own
  owner table and issue #12 (SR-048 FND-003).
- **AD-001 reconciles its Views prohibition**: retaining evidence is forbidden;
  composing a record and handing it to Quoin is not retention (SR-048 FND-004).
- **NFR-003 now constrains FR-009 and NFR-005** (SR-048 FND-009).
- **TC-013 is new**, so FR-009 has a B-runnable evidence path that needs no
  external delivery (SR-049 FND-002).
- **TM-001** drops IT-003's unearned NFR-004 credit and reconciles its execution
  counts against four integration rows (SR-046 FND-012, FND-014).

Not closed, and carried deliberately: the owner-boundary tables still allocate
some responsibilities the eight AD-001 boundaries do not name individually, and
NFR-005's retention checks remain static inspections whose negative controls are
unwritten until the integrations are unblocked.

## Second remediation pass, after SR-047

SR-047 landed after the first pass and contradicted SR-048 on one point. It was
right and SR-048 was wrong, so a correction was reverted.

**PT02 is not an external owner.** SR-048 FND-003 read IT-002's Objective
("B-owned complete-global-conformance closure record") as contradicting the new
owner table, and the first pass resolved it toward the table by calling the
record PT02-owned, following issue #12's wording. SR-047 FND-005 checked the
artifact instead of the ticket: the record was specified in
`quire-specification` through PR #22 from `agent-b/global-conformance-closure`,
and `quire-research#48` is an open research ticket naming this repository as its
own implementation home. Confirmed here against that repository's history. The
Objective was the correct half; the owner table and FR-009's Inputs were wrong.
Both are corrected, and IT-002 now says plainly that the record takes no
version-lock entry because no external owner can assert anything about it.

Also applied:

- **Every entry is `unresolved`, not just A `/2`.** SR-047 FND-002 verified that
  A `/1`'s live pin is branch-only, Engineering Assurance's seam is in no
  release, E02 has no tags and self-declares provisional, Quoin exposes no Rust
  boundary at all, and no typed temporal producer exists in any state. FR-009
  now carries a per-entry status table naming each gate and why it cannot be
  pinned, rather than exempting one entry and implying the other six were fine.
- **The gate order is stated** (SR-047 FND-004). The revision's clause about
  entries carrying no dependency on each other describes resolution at run time
  and was not an ordering statement; the five-stage order is now explicit, and
  it separates the one stage B owns from the four that wait.
- **All seven gates appear in Dependencies** (SR-047 FND-003), where one did.
- **`depends_on FR-001` is removed** (SR-047 FND-009). Resolution precedes
  admission, so the edge closed a loop against AD-001 and against FR-009's own
  prose.
- **The two artifacts called "manifest" are now named apart** (SR-047 FND-011).
  The `/2` loader's producer-side file is a *delivery index*
  (`b-v2-delivery-index.json`, `DeliveryIndex`, `Absence::MissingDeliveryIndex`),
  and FR-009 states that a delivery index can neither supply nor repair a
  version-lock entry. Without this, AC-4 forbade the shape the loader already
  implements.
- **NFR-005's frontmatter carries the edges its prose relies on** (SR-047
  FND-012), and its sequencing behind FR-009 stage 5 is explicit.
- **IT-003's Preconditions no longer assert four acceptances that do not hold**
  (SR-047 FND-001). They are a gate table showing the actual state of each, plus
  A as a fifth gate neither Dependencies section had.
- **IT-002 no longer claims two accepted Rust producer interfaces** (SR-047
  FND-010). Observation is real but unpublished and unpinned; temporal has no
  Rust interface at any state.
- **IT-002's Notes no longer overstate the F revision's reach** (SR-047 FND-014):
  `d8d984b5` touches FR-116 and TC-146 and leaves FR-115 and TC-145 unchanged.
- **Task-013 owns the new scope** (SR-047 FND-006), split so that the B-owned
  resolution mechanism is scheduled as the enablement it is rather than behind
  seven external acceptances (SR-047 FND-013).

SR-047 FND-007 and FND-008 were already closed before it reported: TC-013 and
the revised `spec/tests.md` rows landed while that review was still reading, and
the "provisional" vocabulary it flagged had already been replaced. Verified
absent rather than assumed.
