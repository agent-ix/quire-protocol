---
id: ADR-005
title: "Compose acceptance evidence, never retain it"
type: ADR
status: accepted
relationships:
  - target: ix://agent-ix/quire-protocol/NFR-005
    type: relates_to
  - target: ix://agent-ix/quire-protocol/FR-017
    type: relates_to
  - target: ix://agent-ix/quire-protocol/NFR-003
    type: depends_on
  - target: ix://agent-ix/quire-protocol/AD-001
    type: depends_on
---
# ADR-005: Compose acceptance evidence, never retain it

## Status

Accepted. The boundary is stated; the retention checks behind it are static
inspections whose negative controls are unwritten, and every scenario they would
run on is blocked.

## Context

An integration that composes several externally owned producers can pass for the
wrong reason. NFR-005's Rationale names the three ways: the right outcome from
the wrong revision, a refusal sharing one code with an unrelated axis, or a
budget silently carried over from a previous attempt. Evidence that does not
name the resolution state, the owner, the distinct typed outcome and the
per-dimension budget cannot separate those cases after the fact, and a green
scenario then proves less than it appears to.

So a scenario has to carry a record. But this repository is forbidden to keep
one. AD-001's Views paragraph states that no boundary "reads the filesystem,
spawns a process, accesses a network, retains evidence, or invokes a business
action", and NFR-003-AC-4 requires the inventory to contain no evidence store,
method catalogue, sufficiency or human-disposition implementation. Quoin owns
retained evidence.

Read carelessly, those two facts contradict each other, and for one revision
they did. SR-048 FND-004 recorded it: "AD-001 now contradicts itself: the Views
paragraph states no boundary \"records evidence\", while the new intake section
gives this repository a per-scenario evidence record; the intended
reconciliation (lanes record, boundaries do not) is nowhere written".
SR-048 FND-010 recorded the same gap from the other side: NFR-005 declined to
say where a scenario record lives between composition and Quoin handoff, so only
NFR-003-AC-4's inventory ban prevented it from being materialised as a
repository-local evidence store — and NFR-005 did not cite it.

## Decision

Composing a record and handing it to Quoin is not retention. Retention is the
prohibition, and it is defined by what a boundary can *do with* a record, not by
whether a record exists.

AD-001 now states the reconciliation: a boundary that stored such a record,
indexed it, or answered questions from it would be retaining evidence and is
forbidden. The record is produced, passed, and not kept.

NFR-005 fixes what a scenario must carry to its handoff — the resolution state
it ran against, the owner of each consumed producer value, its typed outcome,
the budget granted and consumed on each declared dimension, and the trace
binding attributing it to its acceptance criterion — and explicitly not where it
is kept afterwards or whether it suffices. Retention, storage, sufficiency
assessment and human disposition are Quoin's and Engineering Assurance's.

A scenario that refuses *during* resolution carries the partial resolution state
it reached plus the entry that caused the refusal. It has no full snapshot and is
not disqualified for lacking one.

## The metric denominator, and the defect that set it

Every NFR-005 metric is denominated on the scenarios each integration **owes**,
not on those it executed.

That is not a stylistic preference. SR-045 FND-009 recorded the defect it fixes,
at severity high: "Every NFR-005 metric is denominated on executed scenarios,
and no scenario executes. All five rows of the Measurement table are percentages
of \"executed scenarios\" or of \"refusing scenarios\"; TM-001 records IT-002 and
IT-003 as blocked on external producer acceptance, so the denominator is
presently zero and all five metrics read 100% against a 100% threshold while
nothing has been observed. A blocked integration therefore scores a perfect
NFR-005, and will keep scoring one as scenarios are added but not run. No metric
counts planned-but-unexecuted scenarios, and no threshold constrains that count."

A fully blocked integration scored 100%. Denominating on owed scenarios makes it
score zero, and makes adding an unrun scenario lower the number rather than
leave it flat. The verification section carries the same rule operationally: an
owed-but-unexecuted scenario is reported as a zero rather than omitted from the
denominator.

Two further denominator rules follow from the same principle that a measurement
must be able to fail:

- **Distinctness is per refusal axis, not per scenario.** Two scenarios
  exercising one axis may legitimately share a typed outcome; two distinct axes
  sharing one may not. The check fails the integration when two axes collide.
  [ADR-008](./ADR-008-refusal-identity-injectivity.md) carries the obligation.
- **Elapsed time is recorded but excluded from the reproducibility comparison**,
  because it is not reproducible and NFR-001 requires the retained evidence to
  be.

## Consequences

**A scenario's record is unavailable the moment it is handed off.** Nothing in
this repository can re-read it, re-derive a verdict from it, or answer a
question about a previous run. Any such query goes to Quoin. That forecloses a
whole class of convenient local tooling — a "why did this scenario refuse last
time" command cannot exist here — and it is the intended consequence.

**"Not retention" is a rule about code, not about data, so it is only as strong
as the inventory check behind it.** The distinction between composing and
retaining is invisible in the record itself; it lives entirely in whether some
boundary holds a reference to it afterwards. NFR-003-AC-4's inventory ban is the
enforcement, and NFR-005's Downstream section now cites it rather than leaving
it as the unnamed backstop SR-048 FND-010 found.

**NFR-005 carries both a Measurement table and an Acceptance Criteria table,
which SR-049 FND-005 flagged and which was not fixed.** The finding: "The
skeleton reserves the AC section for a policy NFR whose compliance does not
reduce to a number; every metric here is a 100% target against a 100% threshold,
which is a policy check written as a metric. One of the two should carry the
obligation, not both, or the two can drift." SR-049's Remediation says FND-005
should drop one of NFR-005's two tables, and SR-049's own record of what was
remediated does not list it. This is an open, B-ownable defect in the artifact
this ADR governs, and this ADR does not close it because NFR-005 is not edited
here.

**The checks are static inspections with unwritten negative controls.** SR-049
records this as carried deliberately: "NFR-005's retention checks remain static
inspections whose negative controls are unwritten until the integrations are
unblocked." A static inspection that has never been shown to fail is not yet
evidence.

**Nothing is measured.** NFR-005 scopes IT-002 and IT-003. Under FR-009 every
producer entry is `unresolved`, so both integrations are unexecutable, and under
this ADR's own denominator rule that is a score of zero — not a pending
hundred.

**IT-001 is deliberately out of scope, which SR-048 had to correct.** FND-008
recorded that NFR-005's Scope once claimed every scenario of IT-001, IT-002 and
IT-003, while IT-001 verifies neither NFR-005 nor FR-009, has no
manifest-snapshot or retention scenario, and `spec/tests.md` maps both
requirements to IT-002/IT-003 only. NFR-005's Scope now says so explicitly.

## Alternatives considered

**Keep a local evidence store and hand Quoin a reference.** Rejected. It is the
thing NFR-003-AC-4 bans by name, and it would give this repository two
authorities it has no claim to: deciding what an evidence record means over
time, and deciding when one supersedes another. It would also duplicate
Quoin's model, which NFR-003's Rationale names as one of the drifts the Rust
boundary exists to prevent.

**Emit nothing and let Quoin reconstruct the context from the typed outcome.**
Rejected. The typed outcome carries the refusal identity and nothing about the
resolution state or the budget, so Quoin would have to infer which revision and
which budget produced it. That inference is exactly the ambiguity NFR-005's
Rationale exists to remove.

**Denominate the metrics on executed scenarios and add a separate
coverage-of-owed metric.** Rejected. It was the shape the defective revision
would have grown into, and it keeps the failure mode: the five substantive
metrics still read 100% on a blocked integration, and a reader who sees five
greens and one amber draws the wrong conclusion. One denominator that cannot
flatter a blocked lane is better than two that can be read selectively.
