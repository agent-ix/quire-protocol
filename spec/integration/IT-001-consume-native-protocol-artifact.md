---
id: IT-001
title: "Consume the native compiler protocol artifact"
type: IT
relationships:
  - target: ix://agent-ix/quire-protocol/ADR-002
    type: references
  - target: ix://agent-ix/quire-protocol/ADR-003
    type: references
  - target: ix://agent-ix/quire-protocol/FR-001
    type: verifies
  - target: ix://agent-ix/quire-protocol/FR-002
    type: verifies
  - target: ix://agent-ix/quire-protocol/FR-009
    type: verifies
  - target: ix://agent-ix/quire-protocol/NFR-002
    type: verifies
  - target: ix://agent-ix/quire-protocol/NFR-007
    type: verifies
  - target: ix://agent-ix/quire-protocol/NFR-005
    type: verifies
---
# IT-001: Consume the native compiler protocol artifact

## Objective

Verify that the native Quire compiler's selected protocol artifact is admitted
and linked through the public Rust boundary without a copied parser, alternate
formal frontend, or identity reinterpretation, and that each published contract
version is consumed only through its own version-explicit entry point.

## Target Integration

The producer is the A-owned native Quire compiler artifact interface. The
consumer is this library's strict protocol admission and linking API. The version 1
and version 2 compiled-protocol contracts are two separate integrations that share
this test; neither may stand in for the other.

## Preconditions

Normal full-assessment integrations resolve the exact QSL compiled-protocol
contract capability through FR-009. The version-1 lane here is the explicit
legacy compatibility exception: it mints no capability, independently selects
and retains the compiler contract, baseline, producer, native-source,
model-source and dependency bytes before inspecting the offer, and cannot read
any full-assessment input. The native compiler publishes both versioned
compiled-protocol contracts and committed owner-addressed handoffs at current
exact pin `2736b62b2f741496590d099d71478b3044d638d6`. Version 1 retains its
`quire.protocol.numeric/1` wire profile.

The version 2 intake is a separate contract with its own published surface:

- `protocol_artifact::v2::{Expected, ExpectedDefinition, ExpectedTemporal,
  AdmittedPackage, read, encode_candidate}`
- `protocol_artifact::v2::wire::{Package, TemporalBinding, ClockConfiguration}`
- `protocol_artifact::native::{TemporalSelection, AdmissionV2, admit_v2}`
- `temporal::{evaluate_v2, evaluate_with_progress_v2, mapping_support_v2}`
- the existing shared `protocol_artifact::{Limits, Report, Error, Invalid,
  Unsupported}`
- `protocol_artifact::handoff::{PUBLISHED_HANDOFF, PUBLISHED_OFFER_FILE,
  PUBLISHED_ARTIFACT_REFERENCE_FILE, PUBLISHED_SELECTION_FILE,
  PUBLISHED_MUTATION_MANIFEST_FILE, PUBLISHED_CHECKSUMS_FILE,
  MUTATION_MANIFEST_FORMAT}`

B constructs `Expected`, `ExpectedDefinition` and `ExpectedTemporal`
independently from its own retained inputs and submits the exact offered bytes
through `v2::read`. Temporal selection is therefore an admission-time input on
this contract; `temporal::evaluate_v2` and its companions remain a separate
post-admission concern that IT-001 does not exercise. Neither version may
select, repair, upgrade, or downgrade the other, neither may be detected from
the offered bytes, and the two mutually refuse.

The published handoff is integration evidence owned by the producer and reached
through `PUBLISHED_HANDOFF` in the pinned dependency source. No environment
variable, copied path, or vendored mirror selects it.

## Inputs

For version 1: the retained four-unit compiled package; independently selected
original native sources, model source, admitted model, and dependency closure.

For version 2: A's raw emitted package bytes and external artifact reference;
B-retained original, unrecanonicalized source, model, definition, profile,
configuration, dependency, producer, artifact, and result selections; the
B-retained temporal profile, clock alternative, and clock parameter selections
bound by that package, carried as `v2::ExpectedTemporal` over
`v2::wire::{TemporalBinding, ClockConfiguration}`; and A's typed
`v2::AdmittedPackage` report and its `Report`/`Error`/`Invalid`/`Unsupported`
refusal boundary.

Temporal binding cardinality, uniqueness and order are the version 2 wire contract,
quoted exactly: "There is exactly one binding for every temporal declaration and
none for other declaration families. Bindings sort by `declaration` and cannot
repeat."

That sentence, and nothing stronger, is what B relies on. It settles `missing`
and `surplus` through the cardinality rule, `duplicate` through the no-repeat
rule, and `reordered` through the sort rule — all keyed on A's `declaration`
field. B does not restate it as a "declared key" or a "declared total order":
A uses neither phrase, and the inflation would attribute to A a declaration it
did not make.

`quire-specification` settles none of this. `FR-252-temporal-clock-binding`
gives a clock binding identity kind *value* with a key that contains no
declaration, owner or positional component, and a key order that is
"lexicographic over the key components, for collection comparison only … it
carries no causal meaning". Two bindings on two different declarations compare
equal under it. The association between a binding and the declaration it binds
is therefore A's wire concern, not an upstream semantic one.

The sentence above lives in A's `docs/compiled-protocol-v2.md`, published on
A's `main` and citable at `93c411b287ccf4789879ac4ef3094fc921dada58`. The four
mutations it grounds are in A's published manifest and each refuses with the
identity A declares.

Both lanes use one-axis mutations of offered wire/version, seal,
producer/source/dependency/baseline identity, feature, a binding actually
referenced by the package, numeric representation, numeric domain, and digest
domain. The version 2 lane additionally uses one-axis mutations of the temporal
profile, clock alternative, and clock parameter selections; temporal binding
sets that are missing, surplus, duplicated, reordered, owned by a foreign
owner, or out of the declared range; recanonicalized source, model, dependency
and configuration inputs; and digest-domain substitution across the domains
`FR-015-digest-domain-selection` selects from `FR-201`'s published vocabulary --
`raw-artifact-digest` for offered producer bytes and `quire.verification.jcs`
for results this repository produces. An earlier revision of this paragraph
named four local labels (`producer-canonical`, `native-raw`,
`compiled-artifact`, `protocol-result`) that corresponded to nothing specified
once FR-015 withdrew its minted vocabulary; the substitution substance is
unchanged and the labels are now the upstream spellings.
Numeric inputs include fields whose declared domains admit `i64::MIN`,
`i64::MAX`, and reduced exact rationals `(i64::MAX, i64::MAX - 1)` and
`(i64::MIN, i64::MAX)`. Resource inputs hold one version 2 input constant and
vary the budget: exactly the amount that input consumes on each declared limit
dimension, one unit short of it, and a fresh budget at the exact value.

## Test Procedure

1. Construct `Expected` only from B-selected retained inputs, resolve
   `quire.compiled-protocol/1`, then submit the exact producer package through
   `quire_protocol::intake::v1::admit` and
   `quire_protocol::model::v1::link_static`.
   - IT-001-SC-01: admission retains artifact, contract, baseline, producer,
     native-source, model-source, and dependency identities unchanged; the
     producer's `expected.json` is not read as authorization.
2. Inspect the public admitted package and its source/declaration links.
   - IT-001-SC-02: all four source units, every declaration/reference, and the
     actual `Workflow.wide`/`Workflow.exact` binding path agree with the selected
     producer inputs.
3. Reseal each single-axis mutation and submit it to the same public API.
   - IT-001-SC-03: a changed wire/version, artifact seal, producer/source/
     dependency/baseline identity or digest, unknown feature, or referenced
     binding returns a typed refusal before a linked package is exposed.
4. Submit signed-64 and exact-rational endpoint values and invalid or lossy
   representations through their declared fields.
   - IT-001-SC-04: admitted endpoint values canonically round-trip byte for
     byte; bare JSON numbers, non-canonical forms, signed-64 or field-domain
     overflow, non-positive or unreduced denominators, wrong numeric kinds, and
     producer/result digest-domain substitution refuse before linking.
5. Inspect the resolved direct and transitive consumer path and the public API
   call used by the integration harness.
   - IT-001-SC-05: the evidence positively identifies the A-owned reader as the
     only native compiler reader and identifies no second parser, formal
     frontend, or foreign runtime in the admitted path.
6. Resolve `quire.compiled-protocol/2`, then submit the exact version 2 bytes and an independently retained version 2 selection,
   built as `v2::Expected` and `v2::ExpectedTemporal` without reading A's
   offered bytes, through `quire_protocol::intake::v2::admit` and
   `quire_protocol::model::v2::link`. Load the bundle
   from A's `protocol_artifact::handoff::PUBLISHED_HANDOFF` and use A's exported
   member-name and manifest-format constants rather than B-owned literals. Submit each
   version's bytes at the other version's entry point; and reseal one changed
   version 2 selection axis at a time.
   - IT-001-SC-06: version 2 retains the exact A-provided artifact bytes and external
     artifact reference through linking and returns A's `v2::AdmittedPackage`
     only for the independently selected version 2 inputs. Version 1 bytes at
     `intake::v2::admit` and version 2 bytes at `intake::v1::admit` each return a typed
     refusal with no auto-detection, upgrade, or downgrade. A changed artifact,
     contract, baseline, producer, source, model, dependency, definition,
     configuration, result, temporal-profile, clock-alternative, or
     clock-parameter selection refuses before a package is exposed, as does a
     resealed change to the offered wire/version, seal, feature, referenced
     binding, numeric representation, or numeric domain. Each refusal is
     distinguishable from the others by its typed identity; one shared cause
     across two axes does not discharge this scenario.
   - IT-001-SC-06a: the explicit legacy `/1` facade `admit_and_link` is unchanged
     by the addition of version 2,
     shown by re-running SC-01 through SC-05 on the same version 1 fixture at the
     revision that adds version 2 and obtaining the same admitted identities and the
     same typed refusal per axis as at the revision before it, with
     `admit_and_link`'s signature and refusal set unchanged.
7. Substitute each digest domain for another across the same version 2 selection, and
   submit recanonicalized source, model, dependency and configuration inputs.
   - IT-001-SC-07: a `raw-artifact-digest` presented in the
     `quire.verification.jcs` slot, and a `quire.verification.jcs` digest
     presented in the `raw-artifact-digest` slot, each return a typed refusal --
     both ordered directions, so that one domain silently absorbing the other
     cannot pass. Producer source, binding-definition, Producer-interface,
     configuration and closure definition bytes use the accepted owner
     selection `raw-artifact-digest`; derived population and membership-rule
     identities use `quire.verification.jcs`. The scenario crosses those
     owner-selected labels without introducing a per-slot domain vocabulary. A
     producer input that B recanonicalized instead of copying
     byte for byte refuses rather than admitting an equivalent form. The
     recanonicalization probe uses a form that is byte-distinguishable from the
     as-delivered form — a pretty-printed JSON re-encoding of the original model
     bytes — so a producer input that is already canonical cannot make the
     scenario vacuous. Digest-domain crossing is exercised over raw bytes.
8. Submit version 2 temporal binding sets that are missing, surplus, duplicated,
   reordered, owned by a foreign owner, or out of the declared range.
   - IT-001-SC-08: each returns a typed refusal that names the offending
     temporal binding axis before a linked package is exposed; a correctly
     ordered, singly owned, in-range binding set admits. Cardinality, duplication
     and ordering are judged against the version 2 wire rule quoted in Preconditions
     — one binding per temporal declaration, sorted by `declaration`, no repeats
     — and `out-of-range` against upstream `FR-255-bounded-temporal-interval`.
     `foreign-owner` is grounded in the same version 2 wire rule: the producer's
     contract states "Missing, surplus, duplicate, out-of-order, foreign-owner or
     out-of-range bindings refuse", so the axis is the producer's and the reader
     refuses it. An earlier revision of this scenario claimed `foreign-owner`
     "is grounded in neither: no owner component appears in A's binding record or
     in any upstream key" — false against the same contract file this scenario
     quotes for cardinality and ordering. What remains true is narrower: no
     *owner component* appears in `FR-252`'s clock-binding key, so the axis is
     grounded in the producer's wire contract and not in an upstream key.
9. Submit one unchanged version 2 input under a budget set exactly to what that
   input consumes on each member of `protocol_artifact::Dimension::ALL`, then
   under a budget one unit short of that member's usage, then resubmit the same
   input under a fresh budget at the exact value. Hold the input bytes and all
   other dimensions constant while changing only the selected dimension.
   - IT-001-SC-09: the exactly sufficient budget admits, the one-short budget
     returns that dimension's typed resource refusal before a linked package is
     exposed, and the fresh-budget resubmission of the same bytes admits, so no
     budget is consumed or retained across a refused attempt.

## Expected Results

Only the exact independently selected package for its explicit contract version
is admitted and linked, through that version's own entry point. Every mismatch
is a typed refusal before a linked package is exposed, and every admitted
numeric endpoint re-encodes to the offered canonical bytes. The integration uses
the A-owned public reader directly; it does not make the candidate producer
revision, display names, or fixture sidecar self-authorizing. Admission and
linking establish no PT02, observation-adequacy, replay-agreement, evaluation,
or campaign-completion result.

## Metadata

- Priority: P0
- Target Integration: native compiler artifact to protocol library
- Automation: Automated local fixture lane reached through the pinned producer
  dependency; no hosted execution

## Dependencies

The A-owned frozen profile and each versioned compiled-protocol artifact contract
are accepted. `quire-spec-language` release `v0.2.0` is at
`6eccb05adf58cb2ed0c6092f5ef8e23d55030028`. Its post-release interface PR #92
exports the handoff paths, all member names, and the mutation-manifest format on
`main`; this repository pins
`2736b62b2f741496590d099d71478b3044d638d6`. The version 2 reader, its typed per-axis refusals, the independent
selection records, the original inputs, the 28-case mutation corpus, and the
published handoff are all present under that dependency source. No environment
configuration or copied artifact tree is part of the gate.

`v2::Expected` **composes** the version 1 expectation rather than replacing it:
`{ inherited: protocol_artifact::Expected, temporal: &[ExpectedTemporal] }`, and
each `ExpectedTemporal` carries `{ source, declaration, definition, clock }`
where `clock` is a `v2::wire::ClockConfiguration`. The clock alternative is that
enum's variant and the clock parameters are its fields. Changed-selection axes
therefore divide into inherited axes, which version 1 already discriminates, and the
temporal axes this contract adds.

B consumes only A's exported surface. The producer sidecar remains inert as
authorization: B reconstructs the expected values from its independently
retained inputs, and the loader checks the original bytes against those selected
identities before admission.

The recipe is a derivation B implements, not a file B reads. A producer-emitted
sidecar carrying the expected selection — `expected.json` beside the version 1
artifact, or any version 2 equivalent — is never B's selection source, because an
offer that ships its own acceptance context authorizes itself. SC-01 forbids
this for version 1 and it holds identically for version 2.

The version 1 numeric and handoff commits that were previously reachable only from
`agent-a/handoff-numeric-domains` are contained in A's `main`. QSL #112 added
the committed version-1 address, and QSL #114 corrected the published package's
Service/Provider authority collision; both are included by the same `6c8c401`
pin.

A prose definition, native source file, emitted sidecar, hand-authored bytes, or
a version 1 value cannot substitute for either compiled interface because this
repository owns no parser.

## Notes

The version-1 lane loads A's committed handoff through
`PUBLISHED_V1_HANDOFF` and A's published offer-file constant. B's independently
retained selection snapshot is loaded before the offer and every selected source,
model and dependency byte is checked against it. The lane runs by default,
executes the positive parser/reader inventory for SC-05, and reaches the local
version-1 static linker.

The version 2 lane submits to A's published `/2` reader at the pinned revision.
It adds no `[features]` entry and no `cfg` attribute, so the crate stays
single-lane under NFR-004-AC-7. Its integrity mechanism — `SHA256SUMS`
verification over every published member, and refusal of an unlisted file, an
escaping path, a symbolic link, a malformed checksum line and a ceiling above
A's hard maximum — builds synthetic handoffs under `CARGO_TARGET_TMPDIR` and is
checked on every gate run.

The version 2 lane runs on every local gate from A's exported
`PUBLISHED_HANDOFF`. Its loader verifies `PUBLISHED_CHECKSUMS_FILE` over every
published member before decoding, and the lane admits the base offer, checks
both cross-version submissions, runs all 28 manifest mutations, and probes every
member of `Dimension::ALL` at exact usage, one-short, and fresh exact usage.
The superseded B-authored pre-delivery corpus is not retained as a second source
of protocol vocabulary. The version 1 SC-01 through SC-05 lane runs 24 default
tests with no ignores. SC-06a additionally ran the identical historical 23-case
corpus against Protocol `0e5ec292` and `5ce9070e`, using their recorded QSL
revisions `5690c2b2` and `d03e4ca5`; both runs passed and the fixture/test blobs,
root facade signature/body and `/1` refusal definitions were unchanged.
The current paired-reader lane also mutates all eight inherited `Expected`
fields and requires both explicit readers to refuse each one; temporal-only
axes remain a separate `/2` corpus.

## Traceability

Verifies FR-001 and FR-002. SC-07 and SC-09 additionally verify NFR-002: the
digest domains that must stay separate, and the declared version 2 resource limits at
their exact bound, one unit short, and on fresh retry.

SC-06 and SC-06a verify NFR-007, and SC-01 through SC-05 are the regression
baseline SC-06a re-runs.
