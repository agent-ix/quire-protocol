---
id: ADR-007
title: "Consume compiled artifacts; author no parser and no second profile"
type: ADR
status: accepted
relationships:
  - target: ix://agent-ix/quire-protocol/FR-001
    type: relates_to
  - target: ix://agent-ix/quire-protocol/FR-010
    type: relates_to
  - target: ix://agent-ix/quire-protocol/NFR-003
    type: depends_on
  - target: ix://agent-ix/quire-protocol/AD-001
    type: depends_on
  - target: ix://agent-ix/quire-specification/FR-050
    type: depends_on
---
# ADR-007: Consume compiled artifacts; author no parser and no second profile

## Status

Accepted, and load-bearing for everything else in this repository. The positive
proof that no second parser is present is an open integration obligation (see
Consequences).

## Context

This repository evaluates protocols whose semantics are defined in
`agent-ix/quire-specification` and whose source is written in the native Quire
language, which A's compiler parses. The apparently direct route is to parse
that source here: it removes a dependency, removes a handoff, and removes the
wait for A to publish a compiled-artifact contract — which, at the time of
writing, is still the reason nothing in this repository executes.

That route was refused, and the refusal is not about effort. A parser is a
semantic authority. To parse native source you must decide what each construct
means, and once two implementations decide that independently they diverge —
silently, on the cases nobody wrote a test for. [FR-001](../functional/FR-001-admit-selected-protocol-profile.md)
names the failure mode in its Behavior: the library retains the accepted
baseline, artifact-contract, producer and dependency identities "without parsing
native source or copying upstream formal semantics into a second locally
authored profile".

"A second locally authored profile" is the phrase the whole repository is
organised around. [FR-010](../functional/FR-010-consumer-boundary-domain.md)
states the same rule for nouns rather than for grammar: this context "owns no
protocol, temporal or observation semantics", it "references that layer and
never restates it", and "A noun this context redefines locally would be the
\"second locally authored profile\" FR-001 exists to refuse."

The independence argument is the other half. AD-001's first Decision is to
create a separate private Rust repository "because every existing crate has a
narrower authority and combining producer with consumer would weaken independent
evidence." A consumer that parses is not an independent check on a producer that
parses; it is a second opinion from the same method. The evidence value of this
repository comes entirely from the fact that it could disagree with A about
whether an artifact is admissible, and it can only disagree usefully if it never
re-derives what A derived.

## Decision

This repository consumes compiled artifacts and parses no source.

- Admission takes protocol artifact bytes plus an accepted compiled-artifact
  contract at a declared version. A prose definition, a native source file, an
  emitted sidecar, hand-authored bytes, or a value from another contract version
  cannot substitute for the compiled interface.
- The A-owned public reader is the only native compiler reader in the admitted
  path. No second parser, alternate formal frontend, or foreign runtime appears
  in it.
- Semantic vocabulary is adopted by reference to
  `ix://agent-ix/quire-specification`, never restated. Where upstream records an
  open gap — an unenumerated axis, a contested membership, an unstated key — the
  gap is cited and no local member list is supplied.
- The boundary is enforced mechanically, not by intent.
  [NFR-003](../non-functional/NFR-003-preserve-rust-assurance-boundary.md)-AC-3
  requires the accepted shared checker to reject an unapproved tracked source,
  shebang, manifest, inline workflow command, generated executable template,
  runtime dependency, extensionless executable, rename, or Rust-wrapper bypass,
  and NFR-003-AC-4 requires the inventory to contain no local producer runner,
  stdout verdict parser, evidence store, method catalogue, sufficiency or
  human-disposition implementation.

AD-001's System Boundary states the corresponding exclusion: native
grammar/compiler, temporal evaluation, observation collection and correlation,
business execution, generic process execution, evidence retention, verification
assessment and human disposition all remain outside.

## Consequences

**The compiled boundary is delivered.** QSL `/1`, `/2` and native-temporal
contracts are pinned at `f1700a9`; the default `/2` handoff executes its owner
reader and mutation corpus. Task-009 still owes the complete `/1` regression
comparison, but no QSL contract publication blocker remains. Parsing source
would still create a second semantic authority and is still forbidden.

**The positive proof is harder than the prohibition and is not yet delivered.**
IT-001-SC-05 requires evidence that "positively identifies the A-owned reader as
the only native compiler reader in the admitted path" and identifies no second
parser, formal frontend, or foreign runtime. IT-001's Notes record the state:
"The complete positive parser-set proof for SC-05 is still an open integration
obligation, not an absence-of-known bad-dependencies claim." An inventory that
finds no known-bad dependency is not the same statement, and IT-001 refuses to
let it stand in.

**A real upstream gap becomes an explicit unsupported or blocked case.** QSpec
#28/#29/#31/#40 has since closed the progress, participation, identity, channel,
relationship, activation, completeness and temporal-boundary gaps this
repository previously listed. QSpec #57 accepted FR-236's per-attempt,
effect-only, and recovery-relation outcome partition at `478afa2`. Protocol
consumes that ruling as distinct domains and does not invent another partition.

**One local vocabulary was minted anyway, and had to be withdrawn.** FR-015
declared a closed four-member digest-domain set before discovering that upstream
`FR-201` publishes nine and says of itself that it *is* the vocabulary of digest
domains. Two of the four local members re-spelled upstream members, and because
spelling is load-bearing, this repository's own result digest would have compared
unequal to the same bytes under the ecosystem's own verification label. FR-015
names it: "That is the \"second locally authored profile\" FR-001 exists to
refuse, one level below protocol semantics." The prohibition is not
self-enforcing at the vocabulary level, and a review caught this one rather than
a checker.
[ADR-003](./ADR-003-compare-digest-domain-before-bytes.md) carries the detail.

**The refusal path preserves producer authority without guessing.** FR-008 owns
the local envelope, stages and causes. A known source response stays nested
under its exact selected source contract; an unknown contract remains opaque
under `unsupported-source-contract`. Local paths do not depend on a proposed
shared diagnostic catalog.

## Alternatives considered

**Parse native source directly.** Rejected. It creates a second semantic
authority, destroys the independence that AD-001's first Decision exists to
preserve, and is the failure mode FR-001's Behavior names by phrase.

**Parse native source, but only to cross-check A's compiled output.** Rejected,
and this is the alternative with real appeal: a disagreement between the two
would be a genuine finding. It fails because a cross-check requires a semantics
to check against, so the second profile exists whether or not it is the primary
path — and when the two disagree, this repository would have to decide which is
right, which is deciding the semantics. A disagreement is only useful if someone
is authorised to adjudicate it, and nobody here is.

**Consume A's emitted sidecar as the selection, rather than retaining a
selection independently.** Rejected, and refused explicitly across the specs
rather than merely declined. AD-001: "A producer-emitted sidecar is never an
intake source: an offer that supplies its own acceptance context authorizes
itself." IT-001's Dependencies apply it to the concrete file: `expected.json`
beside the version 1 artifact, or any version 2 equivalent, is never B's selection source,
and SC-01 forbids it for version 1 and holds identically for version 2.

**Wait for A and build nothing until the contract publishes.** Rejected as a
programme decision, not a technical one. The specification, the mutation corpus
and the fixture loader are authored against A's published surface names ahead of
A's final revision, with the parts that need A's boundary in one `#[ignore]`
lane that fails explicitly rather than passing vacuously. Building against names
is allowed; pinning an intermediate commit is not — B repins only to A's one
final immutable revision.
