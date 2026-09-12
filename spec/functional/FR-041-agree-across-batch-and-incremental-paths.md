---
id: FR-041
title: "Agree across batch and incremental paths"
type: FR
relationships:
  - target: ix://agent-ix/quire-protocol/FR-004
    type: part_of
  - target: ix://agent-ix/quire-protocol/US-001
    type: implements
  - target: ix://agent-ix/quire-protocol/FR-014
    type: depends_on
  - target: ix://agent-ix/quire-specification/FR-059
    type: depends_on
  - target: ix://agent-ix/quire-specification/FR-240
    type: references
  - target: ix://agent-ix/quire-specification/FR-241
    type: references
  - target: ix://agent-ix/quire-specification/FR-242
    type: references
  - target: ix://agent-ix/quire-specification/FR-243
    type: references
  - target: ix://agent-ix/quire-specification/FR-247
    type: references
  - target: ix://agent-ix/quire-specification/FR-258
    type: references
  - target: ix://agent-ix/quire-specification/FR-260
    type: references
  - target: ix://agent-ix/quire-specification/FR-268
    type: references
---
# FR-041: Agree across batch and incremental paths

## Description

When one admitted fact set is assessed by batch replay and by settled incremental assessment, the evaluator SHALL produce for every assessed value the same settled-or-pending state and, where settled, the same settled value.

## Inputs

One admitted observation fact set, the selected assessment path (batch replay or
settled incremental), each admitted prefix the incremental path is advanced
through, the independently supplied decision-scope, surrounding-execution and
complete-global-conformance progress/closure facts, and the authored premises
the assessment is derived under.

## Outputs

Per-obligation and global assessment results from each path, comparable value by
value across the four progress/closure axes plus complete-global-conformance
closure, or one typed refusal identifying the divergence between the two paths.

## Behavior

The evaluator SHALL settle an assessed value at the earliest admitted prefix at which every fact that value requires to be decided is present.
When one authored premise changes, the evaluator SHALL change the same assessed values to the same new values on both paths.
When the batch path and the settled incremental path disagree on any assessed value, the evaluator SHALL return a typed refusal carrying its own outcome identity rather than reporting agreement.

Settling early is an obligation, not a permission. A path that buffers every
admitted fact and evaluates once at the end agrees with batch replay vacuously
and does not discharge this requirement; `IT-002-SC-05b` is the
integration-level check of exactly that.

Agreement is the positive obligation and refusal is the only admitted
alternative. The two paths never disagree quietly: where they differ, the
divergence itself is the reported outcome.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-041-AC-1 | Batch replay and settled incremental assessment agree, for each assessed value, on settled-or-pending and, where settled, on the settled value, the decision-scope and surrounding-execution progress/closure, and the complete-global-conformance closure. | Test (TC-004) |
| FR-041-AC-2 | The incremental path settles at least one value strictly before the final admitted fact arrives, on a history whose earlier prefix already contains every fact that value requires to be decided. | Test (TC-004) |
| FR-041-AC-3 | Changing one authored premise changes the same assessed values to the same new values on both paths, and a divergence between the paths returns a typed refusal rather than an agreement report. | Test (TC-004) |
| FR-041-AC-4 | The typed refusal a path divergence returns carries its own outcome identity, distinct from that of every other refusal axis in the same integration. | Analysis |

## Adopted vocabulary

The values compared across the two paths are upstream's result axes. This
requirement compares them and defines none of them. What it owns is the
agreement obligation itself, the earliest-prefix settlement rule and the
divergence refusal.

| Noun | Upstream object | Identity kind |
| --- | --- | --- |
| Admitted observation | `FR-260-admitted-observation` | not established upstream; see below |
| Decision scope and surrounding execution | `FR-258-temporal-decision-scope` | value, keyed (scope role, scope identity) |
| Assessment-execution disposition | `FR-241-assessment-execution-disposition` | value |
| Truth disposition | `FR-242-truth-disposition` | value |
| Settlement basis | `FR-243-settlement-basis` | value |
| Complete-global-conformance closure state | `FR-247-complete-global-conformance-closure-state` | value |
| Progress and late-data authority | `FR-268-progress-and-late-data-authority` | value, keyed (progress/assertion definition identity, revision, byte digest) |
| Canonical protocol result | `FR-240-canonical-protocol-result` | not established upstream; see below |

`FR-258` supplies the exclusion this whole requirement rests on: progress state,
closure state, input completeness and the exact decision-support set sit outside
the scope key, so one scope observed at two prefixes is one subject. Without that
exclusion "the same assessed value on both paths" would not be a comparison at
all, because the incremental path observes each scope at several prefixes.
`FR-268` is why settlement cannot be re-derived locally: a change to the
progress/assertion definition identity, revision or byte digest is a different
authority, so the incremental path consumes its settlement licence with that
authority retained.

FR-041-AC-4 adopts `FR-014`'s injectivity obligation, scoped per integration,
rather than restating it: the path-divergence refusal is one refusal axis among
this repository's others.

### Where the upstream vocabulary does not yet reach

- **"The same admitted fact set" has no upstream equality.**
  `FR-260-admitted-observation` states of the key that "the prose does not fix
  the key components", declares no key order, and does not place an observation
  in the *authored*/*derived*/*value* trichotomy. Every criterion above compares
  two assessments of *one* fact set; without a key, the two paths cannot be shown
  to have been given the same set. The owner is `agent-ix/quire-specification`.
- **The four progress/closure axes are declared.** `FR-284-scope-progress-and-closure-state`
  types them: each of decision-scope progress, decision-scope closure,
  surrounding-execution progress and surrounding-execution closure carries
  exactly one of `open` or `closed`, and `FR-284` "mints no new spelling".
  Adopted by reference; FR-041-AC-1 compares the two paths over those member sets. Input
  completeness remains outside that vocabulary with no member list, and is
  `agent-ix/quire-specification`'s (`FR-258` open gap).
- **Settlement-basis membership is not stated uniformly.** `FR-243`'s open gap:
  "FR-091 lists four members where FR-061, FR-094-AC-7 and FR-115 list five. No
  requirement states whether FR-091's evaluator may emit `unavailable`. Both
  lists are quoted above; this artifact does not merge them." This requirement
  compares the axis across paths and adopts no membership.
- **Canonical result identity is undeclared.** `FR-240`'s open gap records that
  no requirement states "whether the identity is authored by the producer or
  derived from named components", and none states "whether two results bearing
  the same result identity and different canonical bytes are admissible". This
  requirement compares the axes two results carry and asserts nothing about
  either result's own identity.

## Dependencies

- [FR-004](./FR-004-assess-finite-global-conformance.md) is the parent this
  requirement is part of.
- [FR-026](./FR-026-admit-every-permitted-observation-order.md) is the sibling
  that owns order invariance. The two are separate requirements because their
  failures are separate: an evaluator can be order-invariant and still buffer
  every fact before evaluating, and it can agree between paths while remaining
  order-dependent within one of them.
- [FR-014](./FR-014-typed-outcome-identity.md) supplies the typed outcome
  identity FR-041-AC-4 requires the divergence refusal to carry.
- `ix://agent-ix/quire-specification/FR-059` establishes that batch replay and
  settled incremental assessment over the same admitted facts produce the same
  semantic results.
- `IT-002` scenarios SC-05a, SC-05b and SC-05d verify this requirement at the
  integration boundary; SC-05c verifies FR-026.
