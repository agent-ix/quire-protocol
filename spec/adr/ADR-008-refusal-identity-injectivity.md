---
id: ADR-008
title: "One refusal axis, one typed identity"
type: ADR
status: accepted
relationships:
  - target: ix://agent-ix/quire-protocol/FR-014
    type: relates_to
  - target: ix://agent-ix/quire-protocol/FR-008
    type: relates_to
  - target: ix://agent-ix/quire-protocol/NFR-005
    type: depends_on
  - target: ix://agent-ix/quire-specification/FR-271
    type: depends_on
---
# ADR-008: One refusal axis, one typed identity

## Status

Accepted, and the reason the requirement exists is a defect that shipped.
FR-014 now records the resolved two-level operation/claim outcome algebra.

## Context

Almost every scenario in this repository is a negative scenario. IT-001's
mutation sweep alone covers offered wire/version, seal,
producer/source/dependency/baseline identity, feature, referenced binding,
numeric representation, numeric domain and digest domain, plus on the version 2 lane
temporal profile, clock alternative, clock parameter, six temporal-binding
malformations, recanonicalised inputs and budget exhaustion.

A negative scenario whose oracle is "it refused" is satisfied by any refusal.
That is the whole problem. If every mutation returns one shared code, the sweep
passes, coverage reports full, and the thing the sweep was built to detect — a
mutation that slips through under a neighbouring axis's code — is invisible.

### The defect, measured

SR-027 FND-002, high, shipped on `main` at `681e25a`: "Producer identity,
baseline selection state and baseline digest all asserted the single shared
`Invalid::Selection` code" (`tests/it001_native_handoff.rs:344`). SR-027's
Summary records that `tests/it001_native_handoff.rs` on `main` "asserted one
shared `Invalid::Selection` code for three different refusal axes", and its
verdict is FAIL.

The measurement taken during that review is the fact that makes this a design
decision rather than a test fix: A alone returns `Invalid(Selection)` for the
cross-domain offer, the same code as two other axes. The producer's own
vocabulary does not discriminate these axes. So discrimination is not something
this repository can obtain by asserting harder on A's output; it has to be a
B-owned layer, and SR-027 covers exactly that — "the B-owned discriminating
layer that remediates them."

### The defect, reintroduced structurally before it could recur

SR-038 FND-003, high, on the version 2 corpus
([spec/reviews/v2-intake/failure-domain.md](../reviews/v2-intake/failure-domain.md)):
"The lane's outcome oracle is two-valued, so any non-admission passes.
`Disposition` distinguishes only `Admits` from `RefusesBeforeLinking`;
SC-06..SC-09 likewise say \"returns a typed refusal\" without binding it to
FR-008's envelope. A panic, an untyped error, a refusal from the wrong stage, a
refusal carrying a shared code across three different axes, or a refusal raised
*after* a package was constructed all satisfy every negative row as written.
This is the exact defect SR-027 recorded on the version 1 lane — one shared
`Invalid::Selection` code standing for three axes — reintroduced structurally in
the version 2 corpus before A has delivered."

SR-044 FND-009 found the third instance, in a requirement statement rather than
in a corpus: FR-002's amended statement and SC-08 both require a refusal naming
the offending binding axis while covering six distinct malformations, and neither
says whether those six must be distinguishable, "so one shared code satisfies
both as written — the same defect SR-027 already recorded for
`Invalid::Selection`".

Three occurrences, in test code, in a test-support enum, and in a requirement
statement. The recurrence is what promoted this from a fix to a recorded
decision.

## Decision

The mapping from refusal axis to typed outcome identity is injective within one
integration. Distinct axes map to distinct identities, and the mapping onto a
producer's own typed refusal enum stays injective across the pin step.

Three directions are fixed deliberately:

- **Distinctness is scoped per integration**, matching NFR-005's scoping.
- **Sharing in the other direction is legitimate.** Many scenarios may exercise
  one axis and carry that axis's one identity; that is not a collision.
- **Nothing derives from display text.** A typed outcome identity derives from no
  display text, locale or cause rendering, and changing any of them changes no
  identity, no equality comparison and no retry decision.

The two-level outcome algebra is non-collapsing: an outer resource-incomplete
outcome ([NFR-002](../non-functional/NFR-002-enforce-resource-bounds.md)), an
outer or claim-local typed refusal
([FR-008](../functional/FR-008-return-typed-refusals.md)), every FR-241 semantic
claim disposition and a completed assessment whose finding set is legitimately
empty ([FR-006](../functional/FR-006-produce-canonical-protocol-results.md)) are
pairwise non-interchangeable. No non-success outcome yields an E02 success.

### The key cannot be the oracle

This is the part of the decision that is easy to get wrong, and
[FR-014](../functional/FR-014-typed-outcome-identity.md) states it explicitly.
The identity's key is (`integration`, `refusal_axis`, `outcome_kind`).
`refusal_axis` is itself a key component, so "two distinct axes carry distinct
identities" is true by construction: a check written over the key could never
fail, and the defect it exists to catch — one shared producer code across two
unrelated axes — would stay invisible behind it.

So the injectivity obligation is stated over the **observed interior** the
envelope carries, which the key does not determine. FR-014-AC-1 requires that
for every ordered pair of distinct axes within one integration, the
producer-supplied interior — its `FR-271` code together with its `FR-272` typed
cause — differs between the two, and that a failure is reported against the pair
rather than absorbed as a passing row.

Scenario identity is deliberately absent from the key. Including it would make
every scenario's outcome trivially distinct and destroy the collision check.

## Consequences

**Discrimination is B's work, because the producer's codes do not discriminate.**
The measured fact above is not incidental: A returns one code across three axes.
FR-008 therefore owns the envelope — its seven-member domain, the partial-output
state, the local `unsupported-source-contract` disposition — while treating the
interior as upstream's and copying it byte-exactly. SR-027's remediation is a
B-owned discriminating layer, and SR-027 FND-003 records that it is incomplete:
per-source, per-dependency and per-model digest slots are unguarded, and only
the three top-level selection references are covered.

**Local and source interiors remain separate.** Protocol's local refusal paths
use FR-008's accepted closed causes and stages. A known source response retains
the exact interior its selected contract publishes; an unknown contract remains
opaque under `unsupported-source-contract`. The proposed QSpec diagnostic
catalog is not required by local paths and does not block their injectivity
tests.

**A new axis is a breaking change to the identity space.** Adding a refusal axis
requires a distinct interior for it against every existing axis in the same
integration, pairwise. The obligation grows quadratically in axes, and the version 2
lane already enumerates seven in `ExpectedRefusal` — wrong contract version,
changed selection, changed offer, cross-domain digest, recanonicalized input,
temporal binding, exhausted budget. FR-014 is explicit that the enumeration is
the corpus's and the injectivity obligation is the requirement's.

**Compiler help is limited by `#[non_exhaustive]`.** SR-027 FND-007 records that
A's `Error`, `Invalid` and `Unsupported` are `#[non_exhaustive]`, "so a wildcard
arm is compulsory and the compiler cannot force a new variant to be classified"
(`src/inventory/map.rs:122`). Its disposition is "open and mitigated, not
closed": the compulsory arm produces the separately assertable
`AdmissionRefusal::UnclassifiedSource`, never a success and never a neighbouring
code, and a test fails if any A variant published at the pinned revision reaches
it — but a variant added in a future A revision is caught only after the pin
moves.

**The outcome distinction is explicit.** A pre-row resource-incomplete
operation outcome, a Protocol refusal, a semantic FR-241 non-success row and a
completed-empty row are pairwise non-interchangeable. FR-014 and FR-006 record
the two-level algebra; resource exhaustion is never a refusal cause.

## Alternatives considered

**Assert only that a refusal occurred, and rely on review to keep axes
distinct.** Rejected — it is the state that shipped at `681e25a` and that SR-027
failed. Review did not keep them distinct; four assertions passed for the wrong
reason and landed on `main`.

**Assert on the producer's code directly, per axis.** Rejected on evidence. A
returns one code across three axes, so per-axis assertions on A's code are
unwritable for those three, and any that were written would be assertions about
a `#[non_exhaustive]` enum that can gain variants without notice.

**Make the identity's key include the scenario, so every outcome is unique.**
Rejected, and worth recording because it is the intuitive move. It makes the
collision check vacuous: with scenario in the key no two outcomes ever collide,
and FR-014-AC-4's requirement that an integration whose every negative row
carries one shared identity must *fail* rather than report full coverage becomes
unsatisfiable.

**Author a local diagnostic code vocabulary so the interior is fully specified
and the criteria become runnable today.** Rejected. It would author a producer
diagnostic vocabulary this repository does not own — the prohibition
[ADR-007](./ADR-007-consume-compiled-artifacts-author-no-parser.md) records, and
the mistake FR-015 already had to reverse once. FR-014-AC-8 turns the refusal
into a checked criterion: no local member list, key or order is supplied for the
interior axes upstream records as open, and each such dependency cites the
upstream gap.
