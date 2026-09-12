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

Accepted. The model is specified and the schema is B-owned and unblocked; no
manifest artifact exists in the tree today, and every entry the specification
enumerates is `unresolved`.

## Context

[FR-009](../functional/FR-009-lock-external-producer-revisions.md) resolves eight
external producers through one version-lock manifest. Seven of the eight cannot
currently be pinned at all. Their gates are open, their code is unpublished or
`publish = false`, their interfaces self-declare provisional, or in two cases no
Rust-callable boundary exists in any state.

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
  contract version of the same owner. This is the rule that keeps `/2`'s
  unavailability from being papered over with `/1`, and it is the manifest-level
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

**Nothing runs, and the specification says so in a table rather than in a
hedge.** FR-009 carries a per-entry status table naming each of the eight
entries, its gate, and why it cannot be pinned. That table exists because of
SR-047 FND-002, which verified the claim entry by entry and found that an
earlier revision exempted one entry and implied the other six were fine. The
honest form is worse-looking and correct: A `/1` pinned to a commit that is not
an ancestor of A's `origin/main`, A `/2` in an unpushed worktree, E with no Rust
interface at any state, F's crate `publish = false` and undeclared as a
dependency here, Engineering Assurance's seam in no release with its cited
acceptance revision unreachable after a squash, E02 with zero tags and zero
releases, the only offered diagnostic catalog stating of itself that it is
proposed and not adopted, and Quoin exposing no Rust-callable boundary and
containing no `Cargo.toml`.

**The two-status model makes the gate order visible, and it is mostly other
people's.** FR-009's gate order has five stages. Stage 1 — the manifest schema,
the two-status resolution model, digest recomputation over exact delivered
bytes, per-key refusal, entry independence, sidecar non-authority, and the
resolution budget — is B's and owes no external party. Stages 2 through 5 are
blocked on all seven external gates. TC-013 exists so that stage 1 has a
B-runnable evidence path against B's own fixtures, which SR-049 FND-002 had
recorded as missing.

**`unexecutable` has to survive contact with a test harness that knows only pass
and fail.** The status model states the requirement; nothing in it makes a Cargo
test runner report a third verdict. The current mechanism is IT-001's approach —
the parts needing an unpinned boundary live in one `#[ignore]` lane that fails
explicitly rather than passing vacuously — and that is a convention, not an
enforced property. An unexecutable scenario silently reported as an ignored test
is indistinguishable from a scenario nobody wrote.

**AD-001 still names a status this decision removed.** Its Views section states
that "no producer value enters a boundary whose entry is absent, unpinned,
provisional, or digest-mismatched", while FR-009 states that each entry carries
exactly one status, `pinned` or `unresolved`. SR-047 FND-008 was closed on the
ground that the "provisional" vocabulary "had already been replaced", and it had
been in FR-009 — but not in AD-001, where the word survives. **Unresolved.**
Both readings stand as quoted; AD-001's owner decides whether that clause becomes
"unresolved" or whether a third status is intended after all. AD-001 is not
edited here.

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
and that as a result FR-009-AC-4 forbade the shape the `/2` loader already
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
substitution available today, since A's `/1` exists as a fixture while `/2` does
not, and it would make a `/2` scenario report a `/1` result.

**Let the manifest declare which scenarios require which entries.** Rejected. A
document that declares its own requiredness can unblock a scenario by editing
itself rather than by a producer change, which is the self-authorization shape
FR-009 forbids producers.
