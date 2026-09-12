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

**This repository is blocked by construction, and currently is.** FR-001's
Dependencies say it plainly: "The A-owned native compiler must publish an
accepted versioned compiled-protocol artifact contract; prose profile
definitions and native source are not that interface." AD-001 records the same
as a risk — the current composed proposal defines semantic profiles but no
accepted versioned compiled-protocol artifact interface, and admission cannot
parse prose or source. Under FR-009 A's `/1` and `/2` entries are both
`unresolved`. The cost of this decision is therefore not theoretical: it is the
reason IT-001-SC-06 through SC-09 assert nothing about A's behaviour today.
Parsing source would have unblocked them, and would have made what they proved
worthless.

**The positive proof is harder than the prohibition and is not yet delivered.**
IT-001-SC-05 requires evidence that "positively identifies the A-owned reader as
the only native compiler reader in the admitted path" and identifies no second
parser, formal frontend, or foreign runtime. IT-001's Notes record the state:
"The complete positive parser-set proof for SC-05 is still an open integration
obligation, not an absence-of-known bad-dependencies claim." An inventory that
finds no known-bad dependency is not the same statement, and IT-001 refuses to
let it stand in.

**Every upstream gap becomes a local gap, and none can be closed locally.**
Because vocabulary is adopted rather than restated, an upstream absence stops
this repository rather than being filled in. The specs are full of the
consequence: FR-006 records that the four progress/closure axes are unenumerated
upstream and that participation is unenumerated at the source; FR-008 records
that the diagnostic catalog itself is not adopted, and that FR-271 through
FR-274 each carry an open gap bearing on its criteria; FR-015 records that
FR-259, FR-261 and FR-263 select no digest domain. Each of those is a question
this repository is capable of answering and forbidden to answer. That is the
decision working as intended, and it is slow.

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

**The refusal path inherits producer authority it cannot interpret.** Because
this repository authors no diagnostic vocabulary, FR-008 owns only the refusal
*envelope* and treats the interior as upstream's, copied byte-exactly. A
producer's code, catalog and stage therefore pass through uninterpreted, and
where upstream contradicts itself the contradiction passes through too —
FR-273's producer-stage membership is contested between two sources, and
FR-008-AC-1 and AC-6 both rest on that axis.

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
beside the `/1` artifact, or any `/2` equivalent, is never B's selection source,
and SC-01 forbids it for `/1` and holds identically for `/2`.

**Wait for A and build nothing until the contract publishes.** Rejected as a
programme decision, not a technical one. The specification, the mutation corpus
and the fixture loader are authored against A's published surface names ahead of
A's final revision, with the parts that need A's boundary in one `#[ignore]`
lane that fails explicitly rather than passing vacuously. Building against names
is allowed; pinning an intermediate commit is not — B repins only to A's one
final immutable revision.
