---
id: ADR-009
title: "Two manifest statuses, and no partial pin"
type: ADR
status: accepted
relationships:
  - target: ix://agent-ix/quire-protocol/FR-009
    type: relates_to
  - target: ix://agent-ix/quire-protocol/FR-012
    type: relates_to
  - target: ix://agent-ix/quire-protocol/NFR-005
    type: depends_on
---
# ADR-009: Two manifest statuses, and no partial pin

## Status

Accepted. The model is specified and the schema is Protocol-owned and unblocked;
no resolver artifact exists in the tree today. Entry status is evaluated per
contract, so pinnable and unresolved entries coexist without a third status.

## Context

[FR-009](../functional/FR-009-lock-external-producer-revisions.md) resolves the
exact external contracts each scenario requires through one version-lock
manifest. QSL and Engineering Assurance contracts are pinnable; the current
QObs dependency is stale; qverify's cross-repository acceptance and Quoin's
Protocol scenario intake remain unresolved.

That state creates enormous pressure for a third status. Something that means
"this producer is real, we have looked at it, we are building against its names,
we just cannot pin it yet" — provisional, candidate, pending, best-effort. It
would be honest about the situation and it would let scenarios run.

It would also be the mechanism by which nothing ever gets pinned. A middle
status is consumable: a scenario would accept it, report a result, and that
result would be attributed to a producer revision nobody asserted. The value of
the manifest is that a result is attributable to an exact revision. A status that
permits consumption without that attribution removes the only thing the manifest
does.

The second pressure is toward partial pinning — resolve the entries that are
pinnable, run the parts of a scenario that depend only on those, and report
partial success. That converts a blocked integration into a partially green one,
which is precisely the flattering arithmetic
[ADR-005](./ADR-005-compose-evidence-never-retain-it.md) had to remove from
NFR-005's metrics after SR-045 FND-009 found a fully blocked integration scoring
100%.

## Decision

Each manifest entry carries exactly one status: `pinned`, or `unresolved` with
the reason it cannot be pinned. There is no third status.

The consequences of `unresolved` are fixed, and each closes a specific escape:

- **A scenario requiring an `unresolved` entry is reported unexecutable** — not
  passing, and not failing. It contributes neither a green nor a red, and under
  ADR-005's owed-scenario denominator it contributes a zero.
- **`unresolved` cannot be satisfied by substitution**, including by another
  contract version of the same owner. This is the rule that keeps version 2's
  unavailability from being papered over with version 1, and it is the manifest-level
  form of [ADR-002](./ADR-002-version-explicit-entry-points.md).
- **Resolution is all-or-nothing per scenario.** The library resolves every entry
  a scenario requires before it reads any producer value for that scenario. An
  absent, unpinned or digest-mismatched required entry returns a typed refusal
  naming that entry's (owner, contract version) key *before* any producer value
  for that scenario is read, and a differing digest is never written back.

Three structural rules make the statuses meaningful rather than decorative:

- **Entries have a unique key.** `(owner, contract version)`. Two entries sharing
  one key refuse every integration before resolution, because no refusal can
  name an ambiguous entry — SR-045 FND-003's point.
- **Entries are independent.** Changing one entry alters the resolved owner,
  revision, digest or status of no other; the manifest declares no dependency
  between entries, so resolution needs no order and can contain no cycle.
- **The manifest does not declare its own requiredness.** Each integration test
  declares the entries its scenarios require. SR-045 FND-004's reasoning, as
  SR-049 records it: "A document that says which scenarios need it can unblock a
  scenario by editing itself — the same self-authorization FR-009 forbids
  producers."

## Consequences

**Only scenarios requiring an unresolved entry are unexecutable.** FR-009's
per-contract table is the authority. It does not promote a repository merely
because code exists, and it does not demote a valid exact Git pin because a crate
sets `publish = false`.

**The two-status model makes the gate order visible, and it is mostly other
people's.** FR-009's gate order has five stages. Stage 1 — the manifest schema,
the two-status resolution model, digest recomputation over exact delivered
bytes, per-key refusal, entry independence, sidecar non-authority, and the
resolution budget — is Protocol's and owes no external party. Stages 2 through
5 apply independently to each scenario's required entries. TC-013 exists so
that stage 1 has a locally runnable evidence path against Protocol-owned fixtures, which SR-049 FND-002 had
recorded as missing.

**`unexecutable` has to survive contact with a test harness that knows only pass
and fail.** The status model states the requirement; nothing in it makes a Cargo
test runner report a third verdict. The current mechanism is IT-001's approach —
the parts needing an unpinned boundary live in one `#[ignore]` lane that fails
explicitly rather than passing vacuously — and that is a convention, not an
enforced property. An unexecutable scenario silently reported as an ignored test
is indistinguishable from a scenario nobody wrote.

AD-001 now uses the same two-status vocabulary. A provisional owner document is
the reason an entry is `unresolved`; `provisional` is not a third manifest
status.

**Resolution has its own budget, because it is otherwise the one unbounded
input.** SR-049 FND-003 recorded that manifest resolution was unbudgeted:
nothing stated a maximum entry count, a maximum resolved-artifact size, or a
deadline, and the integrations' own budgets start after resolution has already
happened. Resolution now consumes a caller-supplied budget over entry count,
total bytes digested, and a deadline, and exceeding any dimension returns that
dimension's typed resource-incomplete outcome with no producer value read — not
a refusal, which is the distinction ADR-008 and FR-014 make load-bearing.

**A delivery index is not a manifest, and the two had to be named apart.**
SR-047 FND-011 found that two different artifacts were both called "manifest",
and that as a result FR-009-AC-4 forbade the shape the version 2 loader already
implements. The producer-side file is a *delivery index*
(`b-v2-delivery-index.json`, `DeliveryIndex`, `Absence::MissingDeliveryIndex`);
FR-009 now states that a delivery index is evidence B reads about what was
delivered and can neither supply nor repair a version-lock entry.

## Alternatives considered

**A third `provisional` status.** Rejected. It would be consumable, and a
consumable middle status means a result attributed to a revision nobody
asserted — which removes the manifest's only function. It is also the shape
AD-001's Views clause still describes, so the alternative is not hypothetical:
it was the earlier position, and it was narrowed to two.

**Pin what is pinnable and run the reachable part of each scenario.** Rejected.
It reintroduces the arithmetic SR-045 FND-009 removed from NFR-005 — a blocked
integration reporting a high percentage — and it makes a scenario's verdict
depend on which subset of producers happened to be available, which is not
reproducible. FR-009 instead resolves every entry a scenario requires before
reading any producer value for it.

**Let a scenario substitute an adjacent contract version of the same owner when
its own is unresolved.** Rejected explicitly. It is the most tempting
substitution available today, since version 1 exists as a fixture while version 2 does
not, and it would make a version 2 scenario report a version 1 result.

**Let the manifest declare which scenarios require which entries.** Rejected. A
document that declares its own requiredness can unblock a scenario by editing
itself rather than by a producer change, which is the self-authorization shape
FR-009 forbids producers.
